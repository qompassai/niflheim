//! The FOE gate runner: assembles the player's boss crate —
//! Floor 1's Interner's Hoard, Floor 2's Admission Desk — into a
//! scratch cargo project, runs the floor's gate (`cargo test`
//! with the hidden suite, half validation and half adversarial,
//! then `cargo clippy --all-targets -- -D warnings`), and reads
//! the verdict.
//!
//! OS-boundary discipline (Tiger Style): subprocesses are spawned
//! with separate argv values, never through a shell; stdout and
//! stderr are drained on their own threads into capped buffers;
//! every run has a deadline, and a run that passes it is killed by
//! process group so a hostile test binary cannot outlive its
//! cargo. Player code is data here — the worst it can do is fail,
//! hang to the deadline, or shout past the output cap, and all
//! three come back as a verdict, never a crash.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::content::{GATE_CARGO_TOML, GATE_TESTS, GATE2_CARGO_TOML, GATE2_TESTS};

/// Most bytes of player code the gate will assemble.
pub const MAX_PLAYER_CODE_BYTES: usize = 64 * 1024;
/// Most characters of failure detail kept for the gate record.
const DETAIL_CHARS_MAX: usize = 6_000;
/// Bounded attempts to claim a scratch project directory.
const SCRATCH_ATTEMPTS: usize = 100;

/// The budget for one bounded subprocess run.
#[derive(Clone, Copy, Debug)]
pub struct GateLimits {
    /// The deadline; a run past it is killed and reported.
    pub timeout: Duration,
    /// Most bytes kept from each output stream.
    pub output_bytes_max: usize,
}

impl Default for GateLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(180),
            output_bytes_max: 64 * 1024,
        }
    }
}

/// The outcome of one bounded subprocess run.
#[derive(Clone, Debug)]
pub struct BoundedOutput {
    /// The exit code, or `None` when the run was killed.
    pub exit_code: Option<i32>,
    /// Captured stdout, capped (lossy UTF-8).
    pub stdout: String,
    /// Captured stderr, capped (lossy UTF-8).
    pub stderr: String,
    /// The deadline killed this run.
    pub timed_out: bool,
    /// Output was produced past the cap and dropped.
    pub truncated: bool,
}

/// Run one subprocess under the budget: separate argv, piped and
/// drained output, deadline enforced by process-group kill.
pub fn run_bounded(
    program: &str,
    args: &[&str],
    cwd: &Path,
    limits: &GateLimits,
) -> std::io::Result<BoundedOutput> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Its own process group, so the deadline kill reaches the
        // whole tree (cargo and the test binary it spawned).
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let out_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    let err_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    let out_total = Arc::new(Mutex::new(0usize));
    let err_total = Arc::new(Mutex::new(0usize));
    let cap = limits.output_bytes_max;
    let drain = |pipe: Option<std::process::ChildStdout>| -> Option<std::thread::JoinHandle<()>> {
        pipe.map(|mut pipe| {
            let buffer = Arc::clone(&out_buffer);
            let total = Arc::clone(&out_total);
            std::thread::spawn(move || {
                drain_pipe(&mut pipe, &buffer, &total, cap);
            })
        })
    };
    let out_thread = drain(stdout_pipe);
    let err_thread = stderr_pipe.map(|mut pipe| {
        let buffer = Arc::clone(&err_buffer);
        let total = Arc::clone(&err_total);
        std::thread::spawn(move || drain_pipe(&mut pipe, &buffer, &total, cap))
    });

    let deadline = Instant::now() + limits.timeout;
    let mut timed_out = false;
    let exit_code = loop {
        match child.try_wait()? {
            Some(status) => break Some(status.code().unwrap_or(-1)),
            None => {
                if Instant::now() >= deadline {
                    timed_out = true;
                    kill_tree(&mut child);
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    };
    if let Some(thread) = out_thread {
        let _ = thread.join();
    }
    if let Some(thread) = err_thread {
        let _ = thread.join();
    }
    let stdout_bytes = out_buffer.lock().map(|b| b.clone()).unwrap_or_default();
    let stderr_bytes = err_buffer.lock().map(|b| b.clone()).unwrap_or_default();
    let truncated = out_total.lock().map(|t| *t > cap).unwrap_or(false)
        || err_total.lock().map(|t| *t > cap).unwrap_or(false);
    Ok(BoundedOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout_bytes).into_owned(),
        stderr: String::from_utf8_lossy(&stderr_bytes).into_owned(),
        timed_out,
        truncated,
    })
}

/// Drain one pipe into a capped buffer, counting everything.
fn drain_pipe(
    pipe: &mut impl Read,
    buffer: &Arc<Mutex<Vec<u8>>>,
    total: &Arc<Mutex<usize>>,
    cap: usize,
) {
    let mut chunk = [0u8; 4096];
    loop {
        match pipe.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                if let Ok(mut count) = total.lock() {
                    *count += read;
                }
                if let Ok(mut stored) = buffer.lock() {
                    let room = cap.saturating_sub(stored.len());
                    let keep = room.min(read);
                    stored.extend_from_slice(&chunk[..keep]);
                }
            }
        }
    }
}

/// Kill the child's whole process group (Unix), falling back to
/// the child alone.
fn kill_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let pgid = child.id();
        let killed = Command::new("kill")
            .args(["-KILL", &format!("-{pgid}")])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if killed.is_ok_and(|status| status.success()) {
            return;
        }
    }
    let _ = child.kill();
}

/// The gate's verdict on one submission.
#[derive(Clone, Debug)]
pub struct GateReport {
    /// `cargo test` exited clean.
    pub tests_passed: bool,
    /// Tests the suite reported passed.
    pub tests_run: u32,
    /// Tests the suite reported failed.
    pub tests_failed: u32,
    /// `cargo clippy --all-targets -- -D warnings` exited clean.
    pub clippy_passed: bool,
    /// A deadline killed one of the runs.
    pub timed_out: bool,
    /// The one-line verdict, for the status line and the save.
    pub summary: String,
    /// Bounded failure detail, for the gate record reading view.
    pub detail: String,
}

impl GateReport {
    /// Both halves of the gate stand.
    pub fn passed(&self) -> bool {
        self.tests_passed && self.clippy_passed
    }

    /// A report for a run that never reached cargo.
    fn refused(summary: String) -> Self {
        Self {
            tests_passed: false,
            tests_run: 0,
            tests_failed: 0,
            clippy_passed: false,
            timed_out: false,
            summary,
            detail: String::new(),
        }
    }
}

/// One gate's furniture: the scratch manifest, the hidden
/// suite, and the artifact's name as the fiction speaks it. The
/// runner is shared; only the furniture differs by floor.
struct GateSpec {
    /// The scratch project's manifest.
    cargo_toml: &'static str,
    /// The hidden test suite's source.
    tests: &'static str,
    /// The artifact under judgement ("Hoard", "Desk").
    artifact: &'static str,
}

/// Floor 1's gate: the Interner's Hoard.
const HOARD_SPEC: GateSpec = GateSpec {
    cargo_toml: GATE_CARGO_TOML,
    tests: GATE_TESTS,
    artifact: "Hoard",
};

/// Floor 2's gate: the Admission Desk.
const DESK_SPEC: GateSpec = GateSpec {
    cargo_toml: GATE2_CARGO_TOML,
    tests: GATE2_TESTS,
    artifact: "Desk",
};

/// Assemble `player_code` into a scratch project under
/// `scratch_root` and run Floor 1's gate. The project directory
/// is removed when the run ends, whatever the verdict.
pub fn run_gate(scratch_root: &Path, player_code: &str, limits: &GateLimits) -> GateReport {
    run_gate_spec(scratch_root, player_code, limits, &HOARD_SPEC)
}

/// Assemble `player_code` into a scratch project under
/// `scratch_root` and run Floor 2's gate: the Admission Desk
/// against its own hidden suite, under the same bounds.
pub fn run_gate2(scratch_root: &Path, player_code: &str, limits: &GateLimits) -> GateReport {
    run_gate_spec(scratch_root, player_code, limits, &DESK_SPEC)
}

/// The shared gate: bound the draft, claim a scratch project,
/// run the floor's suite and lanterns, clean up.
fn run_gate_spec(
    scratch_root: &Path,
    player_code: &str,
    limits: &GateLimits,
    spec: &GateSpec,
) -> GateReport {
    if player_code.len() > MAX_PLAYER_CODE_BYTES {
        return GateReport::refused(format!(
            "The gate refuses the draft: {} bytes is past the {MAX_PLAYER_CODE_BYTES}-byte \
             bound. Shorten the {} and submit again.",
            player_code.len(),
            spec.artifact
        ));
    }
    let project = match claim_scratch(scratch_root) {
        Ok(path) => path,
        Err(error) => {
            return GateReport::refused(format!("The vault could not assemble the proof: {error}"));
        }
    };
    let report = run_gate_in(&project, player_code, limits, spec);
    let _ = std::fs::remove_dir_all(&project);
    report
}

/// Claim a fresh scratch project directory, with bounded attempts.
fn claim_scratch(scratch_root: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(scratch_root)?;
    for attempt in 0..SCRATCH_ATTEMPTS {
        let candidate = scratch_root.join(format!("gate-run-{attempt}"));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "every scratch directory is claimed",
    ))
}

/// The gate itself, inside a claimed project directory.
fn run_gate_in(
    project: &Path,
    player_code: &str,
    limits: &GateLimits,
    spec: &GateSpec,
) -> GateReport {
    let assembled = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(project.join("src"))?;
        std::fs::create_dir_all(project.join("tests"))?;
        std::fs::write(project.join("Cargo.toml"), spec.cargo_toml)?;
        let lib = format!("#![forbid(unsafe_code)]\n{player_code}");
        std::fs::write(project.join("src/lib.rs"), lib)?;
        std::fs::write(project.join("tests/gate_tests.rs"), spec.tests)?;
        Ok(())
    })();
    if let Err(error) = assembled {
        return GateReport::refused(format!("The vault could not write the proof: {error}"));
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let test_run = match run_bounded(&cargo, &["test", "--offline"], project, limits) {
        Ok(output) => output,
        Err(error) => {
            return GateReport::refused(format!("The gate could not run cargo: {error}"));
        }
    };
    let (passed_count, failed_count) = parse_test_counts(&test_run.stdout);
    let tests_passed = test_run.exit_code == Some(0) && !test_run.timed_out;
    if test_run.timed_out {
        return GateReport {
            tests_passed: false,
            tests_run: passed_count,
            tests_failed: failed_count,
            clippy_passed: false,
            timed_out: true,
            summary: format!(
                "The gate's deadline fell: your {} ran past the bound and was \
                 stopped. Somewhere in it, something never finishes.",
                spec.artifact
            ),
            detail: tail_text(&test_run.stdout, &test_run.stderr),
        };
    }

    let clippy_run = run_bounded(
        &cargo,
        &[
            "clippy",
            "--all-targets",
            "--offline",
            "--",
            "-D",
            "warnings",
        ],
        project,
        limits,
    );
    let (clippy_passed, clippy_timed_out, clippy_detail) = match clippy_run {
        Ok(output) => (
            output.exit_code == Some(0) && !output.timed_out,
            output.timed_out,
            tail_text(&output.stdout, &output.stderr),
        ),
        Err(error) => (false, false, format!("clippy could not run: {error}")),
    };

    let mut detail = String::new();
    if !tests_passed {
        detail.push_str(&tail_text(&test_run.stdout, &test_run.stderr));
    }
    if !clippy_passed {
        if !detail.is_empty() {
            detail.push_str("\n— the clippy lanterns —\n");
        }
        detail.push_str(&clippy_detail);
    }
    let summary = if tests_passed && clippy_passed {
        format!(
            "The gate stands open: cargo test {passed_count} passed, 0 failed; the \
             clippy lanterns are lit. The {} is real.",
            spec.artifact
        )
    } else if !tests_passed && passed_count == 0 && failed_count == 0 {
        format!(
            "The gate refuses: the {} did not build, so no test could run. The gate \
             record holds the compiler's own words.",
            spec.artifact
        )
    } else if !tests_passed {
        format!(
            "The gate refuses: cargo test {passed_count} passed, {failed_count} failed. \
             The gate record holds which ones, and why."
        )
    } else {
        "The gate refuses on the lanterns: the tests pass, but clippy burns red at \
         -D warnings. The gate record holds its words."
            .to_string()
    };
    GateReport {
        tests_passed,
        tests_run: passed_count,
        tests_failed: failed_count,
        clippy_passed,
        timed_out: clippy_timed_out,
        summary,
        detail,
    }
}

/// Sum the `test result:` lines of a cargo test run.
fn parse_test_counts(stdout: &str) -> (u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    for line in stdout.lines() {
        if !line.contains("test result:") {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        for (index, word) in words.iter().enumerate() {
            if index == 0 {
                continue;
            }
            let count = words[index - 1].parse::<u32>().unwrap_or(0);
            match word.trim_end_matches([';', ',']) {
                "passed" => passed += count,
                "failed" => failed += count,
                _ => {}
            }
        }
    }
    (passed, failed)
}

/// The last stretch of a run's output, bounded for the record.
fn tail_text(stdout: &str, stderr: &str) -> String {
    let combined = format!("{stdout}\n{stderr}");
    let total = combined.chars().count();
    if total <= DETAIL_CHARS_MAX {
        combined.trim().to_string()
    } else {
        combined
            .chars()
            .skip(total - DETAIL_CHARS_MAX)
            .collect::<String>()
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::GATE_SCAFFOLD;

    /// The reference Hoard: what a correct submission looks like.
    /// Kept in the tests so the gate is proven against a known pass.
    const REFERENCE_SOLUTION: &str = r"
use std::collections::HashMap;

pub struct Interner {
    entries: Vec<Box<str>>,
    by_text: HashMap<Box<str>, u32>,
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

impl Interner {
    pub fn new() -> Self {
        Self { entries: Vec::new(), by_text: HashMap::new() }
    }
    pub fn intern(&mut self, text: &str) -> u32 {
        if let Some(&symbol) = self.by_text.get(text) {
            return symbol;
        }
        let symbol = self.entries.len() as u32;
        let owned: Box<str> = Box::from(text);
        self.entries.push(owned.clone());
        self.by_text.insert(owned, symbol);
        symbol
    }
    pub fn lookup(&self, text: &str) -> Option<u32> {
        self.by_text.get(text).copied()
    }
    pub fn text(&self, symbol: u32) -> Option<&str> {
        self.entries.get(symbol as usize).map(|e| &**e)
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub struct LineIndex<'a> {
    lines: Vec<&'a str>,
}

impl<'a> LineIndex<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { lines: source.lines().collect() }
    }
    pub fn get(&self, line: usize) -> Option<&'a str> {
        self.lines.get(line).copied()
    }
    pub fn len(&self) -> usize {
        self.lines.len()
    }
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}
";

    fn scratch(tag: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("niflheim-gate-test-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn fast_limits() -> GateLimits {
        GateLimits {
            timeout: Duration::from_millis(300),
            output_bytes_max: 1024,
        }
    }

    /// Validation: a plain run captures output and exit code.
    #[test]
    fn bounded_captures_output_and_exit_code() {
        let root = scratch("capture");
        std::fs::create_dir_all(&root).expect("scratch");
        let output = run_bounded(
            "sh",
            &["-c", "echo hello vault; exit 3"],
            &root,
            &GateLimits::default(),
        )
        .expect("run");
        assert_eq!(output.exit_code, Some(3));
        assert!(output.stdout.contains("hello vault"));
        assert!(!output.timed_out);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Validation: test counts parse from cargo's own lines.
    #[test]
    fn test_counts_parse() {
        let text = "test result: ok. 14 passed; 0 failed; 0 ignored\n\
                    test result: ok. 0 passed; 0 failed; 0 ignored\n";
        assert_eq!(parse_test_counts(text), (14, 0));
        let failing = "test result: FAILED. 9 passed; 5 failed; 0 ignored";
        assert_eq!(parse_test_counts(failing), (9, 5));
    }

    /// Adversarial: a run past its deadline is killed and reported.
    #[test]
    fn bounded_kills_on_timeout() {
        let root = scratch("timeout");
        std::fs::create_dir_all(&root).expect("scratch");
        let started = Instant::now();
        let output = run_bounded("sh", &["-c", "sleep 30"], &root, &fast_limits()).expect("run");
        assert!(output.timed_out);
        assert_eq!(output.exit_code, None);
        assert!(started.elapsed() < Duration::from_secs(10));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Adversarial: a shouting run is capped, not obeyed.
    #[test]
    fn bounded_caps_output() {
        let root = scratch("cap");
        std::fs::create_dir_all(&root).expect("scratch");
        let output = run_bounded(
            "sh",
            &["-c", "while :; do echo 0123456789; done"],
            &root,
            &fast_limits(),
        )
        .expect("run");
        assert!(output.truncated);
        assert!(output.stdout.len() <= 1024);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Validation (integration): the reference Hoard passes the
    /// whole gate — tests and clippy lanterns.
    #[test]
    fn gate_passes_the_reference_hoard() {
        let root = scratch("reference");
        let report = run_gate(&root, REFERENCE_SOLUTION, &GateLimits::default());
        assert!(
            report.passed(),
            "report: {}\ndetail:\n{}",
            report.summary,
            report.detail
        );
        assert_eq!(report.tests_failed, 0);
        assert!(report.tests_run >= 14);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Adversarial (integration): the unfinished scaffold fails
    /// the gate — its tests cannot pass on `todo!()` bodies.
    #[test]
    fn gate_refuses_the_scaffold() {
        let root = scratch("scaffold");
        let report = run_gate(&root, GATE_SCAFFOLD, &GateLimits::default());
        assert!(!report.passed());
        assert!(!report.tests_passed);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The reference Desk: what a correct Floor 2 submission
    /// looks like. Kept beside the reference Hoard so the second
    /// gate is proven against a known pass.
    const REFERENCE_DESK: &str = r"
pub trait FireWard {}
pub trait MistSense {}
pub trait Swift {}

pub struct AshHound {
    pub name: &'static str,
}

pub struct MistHound {
    pub name: &'static str,
}

impl FireWard for AshHound {}

impl MistSense for MistHound {}

pub fn staff_fire_watch<B: FireWard>(beast: B) -> B {
    beast
}

pub fn staff_mist_watch<B: MistSense>(beast: B) -> B {
    beast
}

pub fn admitted(sigils: &[&str], bound: &[&str]) -> bool {
    !bound.is_empty() && bound.iter().all(|sigil| sigils.contains(sigil))
}
";

    /// Validation (integration): the reference Desk passes the
    /// whole second gate — tests and clippy lanterns.
    #[test]
    fn gate2_passes_the_reference_desk() {
        let root = scratch("desk-reference");
        let report = run_gate2(&root, REFERENCE_DESK, &GateLimits::default());
        assert!(
            report.passed(),
            "report: {}\ndetail:\n{}",
            report.summary,
            report.detail
        );
        assert_eq!(report.tests_failed, 0);
        assert!(report.tests_run >= 9);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Adversarial (integration): the unfinished Desk scaffold
    /// fails the second gate — its staffing and predicate cannot
    /// pass on `todo!()` bodies.
    #[test]
    fn gate2_refuses_the_scaffold() {
        let root = scratch("desk-scaffold");
        let report = run_gate2(
            &root,
            crate::content::GATE2_SCAFFOLD,
            &GateLimits::default(),
        );
        assert!(!report.passed());
        assert!(!report.tests_passed);
        let _ = std::fs::remove_dir_all(&root);
    }
    /// Adversarial (integration): hostile code that loops forever
    /// inside `intern` is killed at the deadline and reported —
    /// the gate returns a verdict instead of hanging.
    #[test]
    fn gate_kills_hostile_infinite_code() {
        let hostile = REFERENCE_SOLUTION.replace(
            "if let Some(&symbol) = self.by_text.get(text) {",
            "loop { std::hint::spin_loop(); }\n        if let Some(&symbol) = \
             self.by_text.get(text) {",
        );
        assert_ne!(hostile, REFERENCE_SOLUTION);
        let root = scratch("hostile");
        let limits = GateLimits {
            timeout: Duration::from_secs(20),
            output_bytes_max: 64 * 1024,
        };
        let started = Instant::now();
        let report = run_gate(&root, &hostile, &limits);
        assert!(!report.passed());
        assert!(report.timed_out, "report: {}", report.summary);
        assert!(started.elapsed() < Duration::from_secs(60));
        let _ = std::fs::remove_dir_all(&root);
    }
}
