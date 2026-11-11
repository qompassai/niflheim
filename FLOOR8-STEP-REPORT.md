# Floor 8 Step Report — The Suspended Hour

## Step 1 — Scaffolding and the frozen executor

Base: `75d66223ddef5a5a3440dc4ea84423de23dc65a2`, also the re-read live
`pax/floor2-build-20261109` tip. Pax's F7 boundary verdict is PASS (229/0);
75d6622 is its docs-only amendment. Lane: `pax/floor8-build-20261110`.

Built save v8 (one bump), serde-defaulted Floor8Progress / HourWork / FOE
records and save mappings, old-version fixtures plus a v7 migration and
future-version forgery check. HourWork carries only watched at this rung.
Five chamber metadata sets, intros, examine texts and the general four-rung
hint ladder stand; no chamber judge passes yet. The hub, map square,
ordered chamber doors, watch and shared seal scaffolding are wired.
Reachability requires FOE 2 fallen AND Floor 6 complete, never Floor 7;
the barred door names each unmet condition. Floor 6 annotations are carried
forward untouched. The new floor's own invariants check chamber count,
recompute proofs and reject unsupported facets.

REVIEW-F8-DOOR derived and settled here: Floor 6 hub rectangle
(640,150,124,124). The chamber row ends at y=136 (14 clear); FOE door
starts y=292 (18 clear); side panel starts x=792 (28 clear); camp ends
x=396 and starts y=332 (244 horizontal / 58 vertical clear). Inspected
live hub furniture; the door is both drawn with OPEN/BARRED and hit-tested.
New hub cards separate title and state text. The brief copy is byte-identical
to staging (sha256 341e6d7a90705b3556fb43609b8d43745c6027e3d3110efa7302f7d1ab22d56d).

D-1262 / REVIEW-F8-POLICY and EXECUTOR: puzzles::executor::run accepts
&[ErrandSpec] and returns Vec<Event>. Authored data comprises seal, State
name/outcome/heart, poll drive indices and timer schedule. No callbacks
cross the surface. Outcomes include Pending, Ready, authored Panic(code),
Cancel and a finite runnable Continue drive for the spinning contrast.
Typed Events cover Poll, Sleep, Wake, TimerFired, Completed,
CancelledWithCleanup, Panicked, StaleRefused and Accounting. All carry
seal and generation. Accounting contains final state; every roster entry
is accounted exactly once, including Running(state-name). Quiescence has
no fixed turn cap and no implicit cancellation. Timer order is
(tick, roster index, authored timer index); every re-arming increments the
errand generation, and mismatches emit refusal evidence. Panic is distinct
from cancellation, contains the authored code, releases loans through the
same cleanup function, and permits neighbors to finish. Exclusive heart
acquisition/release uses Floor 6's actual flag_step transitions. All
required executor behavior lands here, before chamber consumers.

The 14 added shipped tests are balanced validation/adversarial: eight
executor tests, three state tests and three GUI/routing tests (one mixed
entry test). They cover timer ties, stale re-arming, never-waking shutdown,
panic survival/cleanup, completion/cancellation accounting, migration,
forged proofs, both entry conditions, ordering refusal, geometry and
bounded access to hint rung four. No old test was removed or weakened.

| Gate | Result / evidence under /home/phaedrus/.local/share/pax/niflheim-floor8-step1-build-20261110 |
|---|---|
| fmt apply + check | PASS; fmt.log |
| cargo test | PASS; 243 shipped unit tests, 0 failed / ignored, plus 1 transient probe; test.log |
| clippy --all-targets -- -D warnings | PASS; clippy.log |
| Theodolite | PASS; producer existed and was current before session; same-session count=3 E0308 control, byte restore sha256 dd5ea6ad275a3bdab51949bc906c51857305ef93b9c8fedf9efda1753b2c1c76, then all seven Rust sources count=0; theodolite.json |
| Determinism + load-back | PASS; save-run1.json == save-run2.json; sha256 5ea972ecb4cf271dce0a4643e2b778e5b34f92f34f0ed3a26088d759ed4d1a9d |
| Pinned Android lib | PASS; android.log; 1351671560 bytes |
| Desktop build + fresh-save Xvfb | PASS; build.log, xvfb-smoke.log; cold_start_ms=547, exit 124 (expected timeout), zero panics |
| Unsafe forbid / frozen gate.rs | Intact; no changes to lib.rs or gate.rs |

Artifact and source hashes: artifact-sha256.json / source-sha256.json.
Markdown (brief and report): not-LSP-validated; no configured Markdown
route. The transient determinism probe and bacon configuration/export were
removed from the worktree after use and preserved in evidence. No fresh
worktree target directory and no downloads/channel syncs occurred. Cargo
used the prescribed pinned PATH, CARGO_HOME, RUSTUP_HOME and offline route,
shared target FLAG (not environment), nice 10, with foreign cargo/rustc
checks. The producer was stopped before platform builds for serialization.

Variances / pending reviews: SPEC's Pinstone (not design's Herald Stone),
Unsequenced Bell in the FOE (not Blind Eye miniature), six-line ledger and
F14 C5 executor destination govern. REVIEW-F8-CONTENT chamber tables and
REVIEW-F8-CANCELLATION land with their steps; REVIEW-F8-PIN is settled by
run-5 ruling and will be recorded at Step 2. REVIEW-F8-ARTIFACTS: no F7
artifact gate or new queue mechanic was added. Later-step reviews do not
block this rung.

Incidents: four ambiguous/stale splice anchors refused writes; narrower
current anchors cured them. An intermediate scaffold had four duplicated
header match arms (warnings), removed before clean gates. First clippy
found two nested-if warnings in the new executor; fixed without weakening
checks. Intermediate suites passed 238/0 and 240/0 (+ probe). One subsequent
suite attempt was interrupted to include the new hint route, and only its
verified own compiler/linker children were stopped; interrupted-test.log
is not a test PASS. The first LSP attempt queried RA with checks disabled
(zero discarded); a bacon-ls attempt saw two E0308 diagnostics with an
incomplete flag (also discarded). The final recorded session uses RA's
compiler diagnostics with schema-verified check/build-script override
commands through the serialized offline launcher; producer-first, count 3,
restore and clean payloads all verified. Earlier unverified logs remain.
The initial Xvfb run returned 0 but logged a Bevy panic because the selected
Vulkan device exposed no present modes / DRI3 under Xvfb, and executable-relative
assets were absent. This was detected as FAIL, not accepted on exit code.
Retry used VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json,
DISABLE_LSFGVK=1 (installed layer manifest), and BEVY_ASSET_ROOT pointing
at this worktree (verified Bevy source). That retry reached cold_start_ms=547
and timed out at 124 with zero panics / missing assets. Failed logs remain.
DS-9 cleanup also removed this run's 1,169,798,480-byte probe executable,
identified by the successful probe log, owned UID and ELF header; no process
held its executable inode. Its source, saves and logs remain in evidence;
no foreign/dependency cache cleanup occurred. Disk pressure still leaves
less than 0.6 GB available; floor work stops after this gated rung pending
coordinator capacity restoration. See staging REPORT.md.
Pre-existing absent deeper-hint routes for Floors 3–7 were flagged and left
untouched; Floor 8 has its own working route. No custom skill was used.

Commit hash is recorded after committing in the staging REPORT / commit
artifact, avoiding a self-referential hash inside this commit.

## Step 2 — Countdown Bell and Pinned Tapestry

Base: 58a737ebe90293d03668353cbe66b47a96e5aa6a. Prior disk blocker cleared:
122,256,388,096 bytes available at this run's preflight; no cache cleanup by this lane.
Added defaulted polls/rung/torn/pinned/moved fields, pure simulation-backed
bell judge and tapestry judge, guarded transitions, chamber-specific mismatch
refusals and four-rung hints, drawn/hit-tested controls and progress records.
Both chambers file proof and repair through the existing shared seal.
Six shipped tests (3 validation / 3 adversarial) cover earned save/load,
wrong poll with no transition, early/duplicate ring, post-Ready poll refusal,
pin-first/second-unpinned-move refusal, real pointer play and premature seal.
No baseline test changed. Save stays v8; gate.rs and unsafe forbid untouched.

REVIEW-F8-CONTENT derivation: retained Step 1's five boxes (begin loading,
wait, load arrives, prepare bell, complete). Load arrival is poll 3, whole
future completion poll 5. The nonfinal boxes Pending and final box Ready
are actual executor outcomes; reports are projected from Sleep/Completed
events, not a typed answer table. Wrong reports preserve work and name the
current box. The bell can ring exactly once after Ready; no further poll.
REVIEW-F8-PIN: per run-5 ruling, unpinned referent moves symbolic address
A to B and tears, is rehung at A, then pinned. Sound movement moves only
the handle/frame token; referent and self-reference remain at A. The three
ordered work flags, controls, judge and visible explanation use this meaning.
No unsafe operation or real self-referential allocation is performed.
Controls share one drawn/hit-test description at x380, y154/198/242,
380x36, 8px vertical clearance, status below at y310; seal/panel untouched.

| Gate | Actual result / evidence directory /home/phaedrus/.local/share/pax/niflheim-floor8-step2-build-20261110 |
|---|---|
| fmt apply/check | PASS; fmt.log |
| Full test | PASS: 249 shipped, 0 failed/ignored + 1 transient; final-test.log |
| Clippy all targets -D warnings | PASS; clippy.log |
| Theodolite | PASS; producer current before session, control count=3 E0308, SHA restore 5c0dbee0f0a5a89ad753529dc17863a4dc3e96d1e4f2b5a5f3bd0752544e01e7, then 5 touched Rust files zero; theodolite.json |
| Determinism/load-back | PASS; two earned C1/C2 repaired v8 saves identical; SHA256 a8c16cbf0af4ceec62fa8f556d41ce114279758ad6b9588128c6606f2d8b11a4 |
| Android pinned lib | PASS; android.log; 1352047144 bytes |
| Desktop/Xvfb fresh save | PASS; build.log, xvfb-smoke.log; cold_start_ms=2362, expected exit 124, zero panics |

Source/artifact hashes: source-sha256.json / artifact-sha256.json. Markdown
report is not-LSP-validated (no configured route). Probe and producer files
removed after preservation in evidence; not committed. No custom skill.
All cargo uses corrected offline pinned route, shared target flag, nice 10,
foreign-process checks; no channel sync or downloads. Producer stopped
before platform gates. Prior Step 1 claims are retained as prior-run evidence.

Incidents: initial compile caught missing module qualification and a wrong
save method name; fixed (staging step2-first-test.log). First clippy caught
pointer helper scoped inside an older test; added local pointer helpers
(clippy-first-failed.log). Initial corrected suite 247/0 preceded two added
GUI tests; final suite 249/0 covers all changes. One staging git inspection
ran outside the repository and failed; no gate claim relies on it.
Later-step CONTENT/CANCELLATION remain for their steps; ARTIFACTS unchanged,
no F7 prerequisite. Commit hash recorded externally after commit.

## Step 3 — Hand-Built Waker and Single-Attendant Executor

Base: cc3928e (Step 2). Added defaulted parts/trial_walked/roster/judged,
ordered waker assembly, real sleep/spin trial, twelve prediction steppers,
explicit roster judge and proof invalidation after edits. Added individual
Read buttons to expose each full computed event with seal and generation;
the real pointer test reads every event, predicts and seals both chambers.
The sleep demonstration dims the desk without animation or flashing.
Each chamber has its own four-rung hint ladder; proof mismatch names kind.

REVIEW-F8-CONTENT derivations: four errands borrow alternate hearts 0/1
using the existing F6 flag rules. Every started errand needs a Poll, terminal
Completed and Accounting event, so four one-poll Ready drives give exactly
the required twelve events, without filtering or omitting executor output.
The roster is computed, never an answer table. The sleeper's two Pending
states and final Ready produce two sleeps/two wakes and three polls.
For the authored large spinning contrast, use a decimal hundred runnable
polls per required sleep interval (200 Continue polls plus final Ready).
This is an authored bounded drive, not a host measurement. Displayed 201
polls, all wake/sleep counters and generation log are computed by run().
No F7 queue mechanic, new prerequisite or executor-surface extension.

Six shipped tests, 3 validation / 3 adversarial: assembly/trial and save/load,
wrong part/early trial/forged proof, real twelve-event heart trace and reload,
wrong predictions/invalid index/edited-proof invalidation, both chambers
through pointer handlers and seals, pairwise control clearance/wrong-room
refusals. Existing tests unchanged; suite grows 249 to 255. Prediction grid
uses two columns of 140px steppers + 42px Read buttons with 4px gaps;
44px row pitch, 36px height. Status text stands below, judge at y452.

Incidents: a formatted multiline anchor refused the initial script's final
call-site splice; resumed against the current exact call anchor. The partial
intermediate clippy correctly failed on missing controls arguments; fixed.
Initial 255/0 runs preceded the event-read and compact wake-log refinements;
final full suite rerun on the finished source, with prior logs preserved.
No custom skill; Markdown report not-LSP-validated (no configured route).
Save remains v8; gate.rs / unsafe forbid untouched. Later CONTENT and
CANCELLATION decisions remain for Step 4. Pin/stone/secondary rulings stand.

| Gate | Result / evidence under /home/phaedrus/.local/share/pax/niflheim-floor8-step3-build-20261110 |
|---|---|
| fmt apply/check | PASS; fmt.log |
| Full tests | PASS: 255 shipped, zero failed/ignored + 1 transient; final-test.log |
| Clippy all-targets -D warnings | PASS; clippy.log |
| Theodolite | PASS; producer-first, control 3 E0308, restore SHA 38c155a2f657a8a899eaded1aed514799a52c57cc5b5b8bcd7fe8384c41bdb4a, all five touched Rust files zero; theodolite.json |
| Determinism/load-back | PASS; four earned repaired chambers, v8, identical SHA 87300a17f6616e91a04a863ca981a844a0a2f821c15cdefdd4def34dff78412c |
| Android pinned lib | PASS; android.log; 1352490280 bytes |
| Desktop/Xvfb | PASS; build.log, xvfb-smoke.log; cold_start_ms=838, expected exit 124, zero panics |

Source and artifact SHA-256 manifests filed. Probe and producer files preserved
in evidence and removed from worktree. Producer stopped before platform jobs.
Corrected pinned/offline cargo route, shared target flag, nice 10 and foreign
process checks throughout; no downloads/syncs/fresh target. Commit recorded
externally after commit.

<!-- END FLOOR 8 STEP REPORTS -->
