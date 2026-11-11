# FLOOR 8 BRIEF — The Suspended Hour (Niflheim completion programme)

Floor 7 completes at commit 7e1ff4e on branch
`pax/floor2-build-20261109` (suite 229 passed / 0 failed,
lane-reported; Pax's boundary gate is being run and a
docs-only amendment commit by Pax may follow).

RE-READ the live branch tip before basing any work on it;
record the actual base and any boundary amendment. The branch
continues — do not fork without Pax's word. Local commits
only — NEVER push. Per-rung gates: fmt, cargo test (counts
grow from 229, 0 failures), clippy `-D warnings`, theodolite
verified zero errors on touched files, two byte-identical
state-level determinism saves, pinned-toolchain Android lib
build, Xvfb launch smoke, and `#![forbid(unsafe_code)]` intact.
File evidence and append FLOOR8-STEP-REPORT.md in-tree as you go.
Method (D-1189): coordinator-sequenced Part A lane build,
coordinator-authored blocks + splicer, per SPEC §3; anchors
asserted exactly once against the CURRENT tree before any write.
Build contract of record: SPEC.md §3 standing pattern + §4
Floor 8; §5 interface freeze and §9 DS-1…DS-15 bind. This brief
is authoring, not evidence that any build gate has passed.

## The floor (design of record: FULL-DESIGN.md, Floor 8 — The Stillroom of Hours + §5.2/§5.4/§5.6)

- Domain 8 — Async & Pin. Class and keeper: **Pinwright**.
  Stratum II — **The Stillroom of Hours**, a hushed warren of
  suspended bells and sleeping messengers, time kept in jars.
- Learning objective: a future is a state machine you could
  have written by hand; Pin is a promise about not moving;
  a wakeup is a signal with provenance, and a stale one is
  a wrong-room herald.
- **Primary failure class: #2, The Herald in the Wrong Hall.**
  Secondary **#6, The Unsequenced Bell**, lives inside the
  FOE gate's roster suite, per SPEC. FULL-DESIGN instead
  places a #6 Blind Eye miniature among the chamber notes;
  flag this variance in the step report, never harmonize it.
- Prerequisites (design §5.2): Floors 2 + 6.
  `floor8_reachable()` = FOE 2 fallen AND `floor6_complete()`.
  This is not a previous-floor chain: Floor 7 is not a gate.
  Refusal names each unmet condition, never a buzz.
- Navigation (standing coordinator ruling): door on **Floor
  6's hub**, deepest prerequisite's hub, on open ground,
  never occluding text. `geom::door_floor8()`, stairs-stone
  size 124×124; `El::DoorFloor8`, with the floor's name and
  OPEN/BARRED tag following the current hub convention.
  The exact spot is a Step 1 design task: inspect the live
  hub's existing furniture, chamber row, camp, FOE door,
  panels and any other doors; verify the chosen rectangle
  against them and record the spot and clearances in the
  step report, as Floors 6 and 7 did. No coordinates are
  supplied by the sources.
- **SYSTEM LANDED HERE (frozen for Part B, §5): the executor
  simulation** — `puzzles::executor`: errands as authored
  state machines (states, poll outcomes); a waker model
  with per-errand generation counters and a wake log;
  `run(roster) -> Vec<Event>` producing the deterministic
  event roster. Consumed by this floor's C4/FOE; traced
  onto by F14 C5. The executor module skeleton + its first
  unit tests land in Step 1, not Step 4. At the Floor 9
  Step 5 freeze, `puzzles::executor::{run, Event}` is a
  Part B reliance; no Part B floor extends it. Do not defer
  required behaviour to a later floor.
- The Lockheart Pair, Floor 6's borrow-flag and spin hearts,
  is this floor's shared-state furniture: errands borrow
  the hearts under Floor 6's rules. Earlier records carry
  forward without re-judging F6's provisional annotations
  (DS-7). FULL-DESIGN also names F7's Arc & Queue as this
  floor's task queue; SPEC does not prescribe that use and
  F7 is not required for entry. Record REVIEW-F8-ARTIFACTS
  before adding any artifact gate or extra mechanic.
- Save format **v8**, one bump at Step 1 (DS-2); work record
  **`HourWork`**. All added fields serde-defaulted; existing
  fields never renamed, removed or re-typed. Version-load
  fixtures and the future-version forge test ride the bump.
- No cargo gate: `gate.rs` remains untouched (DS-4). The
  executor is a real safe simulation, not an unsafe runtime.
  `#![forbid(unsafe_code)]` stands. **Real facts only
  (DS-13): every roster/count the game asserts comes from
  the REAL executor simulation, never a hand-typed answer
  table.** Authored inputs and policies are content; outputs
  are computed. The spinning twin's authored large count
  must be backed by its simulated drive, not a claimed
  measurement of the host.
- Mastery note (FULL-DESIGN): from here down, a suspended
  task is a state machine the player could in principle
  have written by hand, and Pin is a promise they can state.
- Platform: roster = event list, fully operable by keyboard
  and touch with identical ordering (§7.7); sleep uses
  dimming, never flashing (§6.7). No live-timing inputs,
  host timing gates or RNG: all clocks turn-stepped, all
  pressure authored and bounded. Preserve reduced motion,
  labelled non-colour signals and phone-width readability.
  Floor 8 counts toward Stratum II stairs' “any two more”;
  it does not itself land the later stairs predicate.

### Chambers (ordinary encounters, in spec order)

All five follow the standing resolution model (design §5.4):
UNWORKED → DIAGNOSED → REPAIRED, work fields on the chamber's
progress record, pure judges in puzzles/state over authored
content.rs inputs, proof = the specified proof, repair via
the shared seal. Work record: `HourWork`. Four-rung hint
ladders each, fourth naming the seam; keeper **Pinwright**.
Chambers open strictly in order after the prior repair.
Proof slots labelled by KIND, never answer content; wrong
proof names its mismatch. Facet = repair at hints_used ≤ 3,
with facet text naming **The Pinstone**. Extend only this
floor's invariant section; recompute proofs, five chambers.

1. **The Countdown Bell.** A five-state future. Player polls
   and chooses Pending / Ready per the authored drive (a
   load completes at poll 3). Wrong poll refused with the
   state named; no wrong state transition. Judge: all five
   polls right + bell rung exactly at completion. Fields:
   `polls: [u8; 5]`, `rung: bool`. The precise five-state
   drive is REVIEW-F8-CONTENT, not an invented answer key.
2. **The Pinned Tapestry.** Self-referential hanging. FIRST
   move unpinned: it tears (the tear is the demonstration);
   then pin and move soundly. Judge: tear exhibited AND
   pinned move completed. Fields: `torn: bool`,
   `pinned: bool`, `moved: bool`. Preserve the sequence;
   consult REVIEW-F8-PIN for what the sound move denotes
   without moving the pinned referent or using unsafe code.
3. **The Hand-Built Waker.** Assemble four parts in order:
   counter, errand name, generation stamp, wake face. Then
   walk the sleep trial: errand sleeps and is woken exactly
   twice — wake log shows 2 wakes for 2 sleeps, sleep not
   spin. Show the spinning twin's authored large counter
   for contrast, with real simulated provenance. Judge:
   parts in true order + trial walked. Fields:
   `parts: [u8; 4]`, `trial_walked: bool`.
4. **The Single-Attendant Executor.** Four errands run ON
   the Lockheart hearts, borrowed under F6's rules. Player
   predicts all twelve events in order, steppers per event
   slot over event kinds. Referee roster from the REAL
   executor simulation; judge matches event-for-event.
   Fields: `roster: [u8; 12]`, `judged: bool`.
5. **The Cancellation Ledger.** Six started errands,
   variously completed / cancelled mid-flight / still
   running. Assign every line its terminal state:
   completed / cancelled-with-cleanup / running-by-name.
   Reconcile every started errand exactly once. Judge: all
   six lines right. Field: `ledger: [u8; 6]`. The design's
   timed errand and returning Floor 1 scope-guard witness
   are recorded as REVIEW-F8-CANCELLATION; no new real-time
   gate or prerequisite is licensed.

### The FOE — The Suspended Hour

Errands complete before they are run: a stale waker from the
previous watch rings a sound-alike bell indistinguishable
from true completion. A completion believed in the wrong room.

- **Diagnosis (exhibits, IN SEQUENCE):** (1) the state box
  that never left state 1, the stuck errand's trace;
  (2) the wake log exhibiting the phantom wake with its
  stale generation; (3) the generation mismatch itself,
  signal's generation ≠ errand's current generation.
  Out-of-sequence Exhibit refused with the missing prior
  exhibit named.
- **Pressure (DS-3; FULL-DESIGN Stratum II rule):** stale
  bells keep ringing during diagnosis: bounded authored
  misleading evidence. Never damage, never destroy proof,
  never impose a real-time limit. Preserve the provenance
  that lets the player discriminate stale signals.
- **The gate:** deterministic interleaving suite — four
  named rosters actually run on the executor; each matches
  its predicted roster event-for-event. Player walks all
  four. Secondary **#6 The Unsequenced Bell**: one roster's
  events arrive unsequenced, and the player names it.
  Also demonstrate shutdown/panic policy: one errand panics,
  executor survives and policy line is shown; a never-waking
  errand reaches the documented shutdown behaviour. Carry
  FULL-DESIGN's timers-in-order and wake-counter/no-busy-wait
  proof, using computed event evidence. Exact rosters,
  policies and the unsequenced/timer relationship require
  REVIEW-F8-CONTENT and REVIEW-F8-POLICY, not guesses.
- **Repair (strike's substance):** typed completion signals:
  rewrite the signal card to three checkable truths — every
  signal carries its errand seal; every signal carries its
  generation; a signal whose generation is stale is refused
  at the face. Match the living errand's seal and generation;
  a stale ring cannot publish completion. Shared two-step
  seal strike arms only with proof complete, gate walked,
  and rewrite accepted. STANDING → EXPOSED → FALLEN; no
  strike can bypass proof, and proven repair cannot fumble.
- **Codex stone:** **The Pinstone** — “Let the type carry
  the errand.” SPEC names it so; FULL-DESIGN calls it
  **The Herald Stone** with the same text. Build to SPEC,
  flag the name variance in the step report, do not silently
  harmonize it (Floor 7's Loomstone precedent). Preserve
  the design's mechanism explanation: distinct message
  kinds prevent wrong-meaning handling; generation tokens
  prevent stale completions publishing.
- **Loot:** **The Pinwright's Executor** — the attendant,
  the roster, the timer jars, the typed bell system.
  Wielded later: Floor 13 (profiling specimen), Floor 15
  (async desugaring). F14 C5's executor trace is additionally
  required by SPEC; record the destination-list variance.

## Steps

1. **Scaffolding.** Re-read live tip and boundary report.
   Save v8; `Floor8Progress` + `HourWork` (empty at S1,
   standing watched flag as the current precedent requires)
   + `Floor8FoeProgress`; serde-defaulted expedition/save
   field and mapping touch-points. `Screen::Floor8Hub`,
   `El::DoorFloor8` / `Floor8Door(u8)`;
   `geom::door_floor8()` on F6 hub on verified open ground,
   spot/clearances in report; `build_floor8_hub` in shell.rs;
   map's Floor 8 square; `floor8_reachable()` gates door.
   All five ChamberMeta, intros, examine texts and general
   hint ladder. Executor module skeleton + first unit tests
   land HERE. Update v8 fixtures, preserve old records.
   Resolve executor-shape/content REVIEWs with Pax before
   turning unsettled choices into a frozen API.
2. **Chambers 1–2** — Countdown Bell and Pinned Tapestry.
3. **Chambers 3–4** — Hand-Built Waker and Single-Attendant
   Executor, including real simulated roster comparison.
4. **Chamber 5 + FOE** — Cancellation Ledger, sequenced
   exhibits, bounded stale-bell pressure, all four roster
   walks and secondary identification, shutdown/panic
   demonstrations, typed-signal rewrite and strike.
5. **Integration** — shipped whole-floor GUI playthrough:
   a `#[test]` driving real press handlers through all five
   chambers and the FOE, not setters substituting for play;
   full gates, close-out. Floor 8 completes here.

Per-step gates (Floor 6/7 set; all on primo; evidence in
`~/.local/share/pax/niflheim-floor8-step<N>-build-20261110/`):

- `cargo fmt` apply on primo then `cargo fmt --check` clean.
- `cargo test`: **counts grow from 229, 0 failures**; no test
  deleted, skipped or weakened. Roughly half validation,
  half adversarial: successful drives paired with wrong
  polls, order refusals, stale generations, mismatched
  predictions, reconciliation faults and premature strikes.
  Version fixtures run every rung. Record actual counts.
- `cargo clippy --all-targets -- -D warnings`: exit 0.
- Theodolite: cured protocol, producer first; planted control
  seen in-session, restore sha256-verified, clean count=0
  on all touched files with a supported route. See cautions.
- Determinism: state-level playthrough probe run twice with
  `NIFLHEIM_SAVE_OUT`, saves byte-identical, sha256 recorded.
  Transient per DS-9: copy into `tests/` for gate, remove
  afterward, never commit it. Cover earned current-step
  state and load-back, culminating in completed v8 save.
- Android `cargo build --target aarch64-linux-android --lib`
  exit 0; record library size. Pinned rustc 1.99.0 from
  repo rust-toolchain.toml, `~/.cargo/bin` first on PATH
  before every cargo call.
- Xvfb smoke: `cargo build`, fresh `NIFLHEIM_SAVE_PATH`,
  `timeout 25 xvfb-run -a ./target/debug/niflheim`;
  `cold_start_ms=` recorded, zero panics. Expected timeout
  is distinguished from a crash. Unsafe forbid intact.

Per-step local commit on continuing branch; brief committed
with Step 1, as Floors 6 and 7 were. This is a builder's
future procedure, not an instruction to commit this authoring
workspace. No pushes. Reports follow FLOOR7-STEP-REPORT.md:
actual base, built delta, geometry/API decisions, named
variances and REVIEWs, per-gate evidence paths/hashes,
counts/exits, incidents and fixes, commit hash; close-out
lists all five commits, final count, earned save v8,
determinism hash, Android size, GUI evidence and open items.
No artifact means REVIEW, not PASS; Pax verifies boundary.

## Standing cautions (from the earlier lanes' record)

- Splicer anchors assert exactly once against CURRENT tree
  before any write. Closing-brace anchors may put code
  outside the item; insert-before at an arm boundary can
  land between arms. New `Screen` variants have more match
  sites than the obvious ones; exhaustiveness errors are
  the census, including layout's tab-bar match.
- Local splice mirrors are unformatted: apply `cargo fmt`
  on primo after shipping, then check. Primo's committed
  tree is the fmt-clean source of truth. Inspect control
  geometry and hit testing: F7 integration found a real
  order-button/seal overlap missed by the prior gate path.
- Validate every touched file through primo
  `~/.local/bin/theodolite`. Zero error diagnostics before
  reporting done; cite diagnostics per file type. Name any
  type without a route, or file without an arriving payload,
  **not-LSP-validated**, never silently skip it. For Rust,
  locations file exists and is current WHEN SESSION STARTS:
  `bacon --headless -j bacon-ls` producer FIRST, bridge
  afterward. Producer session `niflheim-bacon`: restart if
  gone before the first session; stop at floor boundary.
  Same-session-shape planted `theodolite_route_control`
  must yield count=3 E0308 while clean files yield 0;
  restore with sha256 verification, rerun clean on every
  touched Rust file. Warm-up calls are not evidence.
- Record outcomes for any custom skill materially used via
  `~/workspace/bin/skill-trust.py record`. This is Pax's
  standing recorder path; on primo the recorder lives in
  the lane workspace if present. Coordinator resolves the
  dispatch path; do not invent one or claim an unrun record.
- Real facts only: executor rosters, wake/sleep counters,
  timer order and cancellation accounting derive from real
  simulation over authored inputs, never answer tables.
- Primo good-neighbour rules: one heavy job at a time,
  `nice -n 10`, Owner's interactive work wins.
- Anything SPEC/FULL-DESIGN does not settle is a named
  REVIEW for Pax (see REPORT.md), not permission to invent
  constants, event types, policies or new mechanics.
