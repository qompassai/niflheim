# Floor 7 Step Report — Niflheim Completion (Part A)

Branch: `pax/floor2-build-20261109` (local only, no pushes).
Start tip: `a6e3de2` "Floor 6 Step 5: Integration — Floor 6
Complete" (suite 191/0, Floor 6 determinism saves sha256
6386a8b71efc90fba5a20c441a3b3d9f6597f55438a29a987239fa4e8abb9d6b,
earned save at version 6 — Pax-verified).
Governing documents: `SPEC.md` (the completion spec) §3 +
§4 Floor 7, `FULL-DESIGN.md` Floor 7, `FLOOR7-BRIEF.md`
(the assembly brief, in-tree with this report).

Per-step evidence directories on primo:
`~/.local/share/pax/niflheim-floor7-step<N>-build-20261110/`.

Name variance, flagged per the brief (not harmonized):
the SPEC (§4 Floor 7, the contract of record) names the
codex stone **The Loomstone** — "A global order forbids
cycles." FULL-DESIGN.md's Floor 7 calls the same stone
*The Porters' Stone* with the same text. This build uses
the SPEC's name throughout.

## Step 1 — Scaffolding

The floor framed end to end, mirroring Floor 6 Step 1 part
for part: the save format reaches v7 (DS-2, the one bump,
at this step); `floor7_reachable()` = `floor5_complete()`
— the spec's ruling (Floor 5 complete; Floor 6 recommended
advisory only, and the door test + state test both prove
the door does not wait on the Lockheart); Floor 6's records
ride the bump unjudged, spin annotations included (DS-7).
The chambers carry intros, charters, examine texts, and
named refusals; no chamber can be worked yet (every judge
lands with its step); the FOE's records exist with no
screen behind them. The shell reaches the hub and the
chamber skeletons, and the map gains the floor's square
(index 6, number 7).

The door spot (this step's design task, per the brief):
Floor 6's door already holds (640, 150) on Floor 5's hub,
so Floor 7's door stands at `geom::door_floor7()` =
(440, 150, 124, 124), the upper field west of it. Verified
against the hub's furniture: the chamber row ends at
y=136 (the door starts at y=150, the same 14-unit gap
Floor 6's door keeps); Floor 6's door starts at x=640
(this door ends at x=564, 76 units clear); the camp stone
(cell (5,4): x 334–396, y 332–394) stands below the door's
foot (y ends 274); the FOE's door (600, 292, 168×124)
starts at y=292, 18 units below the foot, and the panel
starts at x=792, far east of the door's east edge (564).

The system landed here (frozen for Part B): the model
loom, `puzzles::loom` — authored thread programs as op
lists (`LoomOp`: Send / Recv / Claim / Release / Join over
indexed channels and claimables); `explore(programs, cap)`
walks interleavings depth-first in lexicographic
thread-choice order, counting terminal paths to the cap,
recording the first deadlock's choice trace and the set of
final channel-state signatures of completed paths. Its
first unit tests land with it: the empty loom, two
senders, the two porters' cycle (with and without
releases), the lonely receive, the waiting join, and the
cap's bound — half validation, half adversarial.

One parity note, pre-existing and not a regression: like
Floors 4–6 before it, the next-floor door has no drawn
visual on its hub — the door lives in layout, press, and
map exactly as Floor 6's did.

Gates, all on primo under pinned rustc 1.99.0 (evidence:
`~/.local/share/pax/niflheim-floor7-step1-build-20261110/`):

- `cargo fmt --check`: clean (after one applying fmt pass).
- `cargo test`: **202 passed, 0 failed** (191 at the Floor
  6 tip + the door test, two state tests, and the loom's
  first tests; no test deleted, skipped, or weakened).
- `cargo clippy --all-targets -- -D warnings`: exit 0, no
  warnings.
- Theodolite (cured-protocol session record): planted run —
  game.rs count=3, all E0308 from the planted
  `theodolite_route_control` at line 9539, while core /
  content / puzzles / state / shell read 0 in the same
  session; restore verified by sha256 (b6d1851d… before
  and after); clean run — all six touched files count=0;
  THEODOLITE: PASS both modes.
- Determinism: the Floor 6 Step 5 probe (the full earned
  playthrough, load-back asserted), renamed
  `floor7_step1_playthrough_save`, run twice: saves
  byte-identical, sha256
  cbe8bd8cc2d952bfd279b74b7b59eb9fed8ffa3068941c2009761625b96d86e0
  (the earned save now at version 7, Floor 6 complete
  inside it). Probe removed after the runs; the tree
  carries only the six source files, this report, and the
  brief.
- Android: `cargo build --target aarch64-linux-android
  --lib` exit 0 (libniflheim.so 1,350,003,448 bytes).
- Xvfb smoke: fresh save path, `cold_start_ms=1026`, zero
  panics. `#![forbid(unsafe_code)]` intact (src/lib.rs:26).

Commit: `d461754` on `pax/floor2-build-20261109`.

## Step 2 — The Scoped Loom and the Border Seals

**Built.** Chamber 1 (The Scoped Loom): `LoomWork.join_order[6]` +
`scoped`; content `SCOPED_THREADS` — six named hands with expiry
slots (Bryn 2, Elin 3, the rest 6); the judge in puzzles
(`scoped_order_fault` names the first fault — unassigned, doubled
slot, Bryn late, Elin late — and `scoped_order_valid` requires the
permutation plus every slot within its expiry); presses
`F7JoinCycle` (per-thread slot cycle, clears the judgement) and
`F7ScopeJudge`. The reference enrolment [4,2,5,6,3,1] — Bryn second,
Elin fifth, both inside their scopes — is exercised end-to-end in
the GUI test with per-slot click counts. Chamber 2 (The Border
Seals): `LoomWork.seals[6]` + the shared `judged` stamp; content
`SEAL_TOOLS` with per-tool `true_seal` (the key is derived, never
stored: [Send, Sync, Send, Sync, Neither, Send]); the Phantom-Sealed
Kit is the trap — its fine print names a raw pointer, and
`seals_complete` refuses it under Send and under Sync alike in the
puzzles tests; presses `F7SealCycle` / `F7SealJudge`. Diagnosed for
both chambers recomputes from the work (stamp + watched + judge
function). State tests: the judged path for both chambers plus the
adversarial reset/shape checks. One GUI playthrough test drives
both chambers through watch → work → judge → Prove → Seal →
SealConfirm → Repaired, including the refused empty-order judge and
the phantom sealed Send first, then corrected to Neither.

**Gates** (primo, rustc 1.99.0; evidence
`~/.local/share/pax/niflheim-floor7-step2-build-20261110/`):
- `cargo fmt --check` — clean.
- `cargo test` — **210 passed / 0 failed** (202 + 5 puzzles + 2
  state + 1 GUI), re-run after the clippy cure: still 210/0.
- `cargo clippy --all-targets -- -D warnings` — exit 0
  (`clippy2.log`). One curative fix on the way: clippy's
  `manual_contains` in `scoped_order_fault`
  (`order.iter().any(|slot| *slot == 0)` → `order.contains(&0)`);
  first clippy run preserved at `clippy.log`.
- Theodolite (cured protocol; bacon producer live,
  `.bacon-locations` present at session start): planted run —
  game.rs count=3, E0308 from `theodolite_route_control` at line
  9775, the other four touched files count=0 in the same session;
  restore sha256 `39d3d259…` verified against the pre-plant backup;
  clean run — all five touched files (content, puzzles, state,
  game, shell) count=0. PASS both modes.
- Determinism: state-level probe through Floor 7 chambers 1–2
  (transient `tests/floor7_probe.rs`, removed after the run, per
  DS-9) — two runs byte-identical, sha256
  `81cb25d4c72aa5ba6d807b7d22cbae0c29e438eef173c287e7d5534db33a3b006`;
  the save carries `"version":7` and round-trips with
  `floor7_chambers_repaired() == 2`.
- Android: `cargo build --target aarch64-linux-android --lib` —
  exit 0 (libniflheim.so 1,350,212,856 bytes).
- Xvfb smoke: cold_start_ms=932, zero panics (timeout exit 124 as
  expected).
- `#![forbid(unsafe_code)]` intact.

**Process note.** One mirror-drift incident, self-caught: the
local src/ mirrors were stale (pre-fmt) when the clippy fix was
first shipped, which regressed fmt on primo's puzzles.rs for one
cycle. Cured by re-running `cargo fmt` on primo and re-syncing all
five mirrors primo→local with sha256 verification; the standing
rule from here on is mirrors are only ever refreshed FROM primo
after its fmt pass.

Commit: `d042e9a` on `pax/floor2-build-20261109`.

## Step 3 — The Charter Wall and Arc and Weak Echoes

**Built.** Chamber 3 (The Charter Wall): `LoomWork.edges[5]`,
`weak_marked`, `stale_found`, `graded`; content `WALL_EDGES` (five
candidate happens-before pairs), `WALL_EDGE_KEY`
[true,true,true,false,true] — the fourth chalked pair runs backwards
(a read ordering the write it reads); `WALL_CLAIMS` — the weakened
claim is the third (Relaxed tally increments asked to carry an
edge), `WALL_WEAK_INDEX` 2; `WALL_TIDE_NOTE` (the dawn note: "the
shuttle stands free"). puzzles `wall_fault` names the first fault in
order (edges, then the mark, then the unfound tide); the diagnosis
recomputes `wall_graded` behind the `graded` stamp. Presses:
`F7EdgeToggle`, `F7WeakCycle`, `F7StaleWitness` (the witness text
carries the shuttle's token from the Spin Heart — claimed, held
since — against the dawn note), `F7WallGrade`. The shell panel
displays Floor 6's four spin annotations beside the wall's claims,
read from `floor6.chambers[2].work.spin_annotations` with their
chosen orderings — displayed only, never read by any Floor 7 judge
(DS-7); a state test proves a "wrong" (by Floor 7's key) set of
annotations neither blocks the wall nor is modified by it.
Chamber 4 (Arc and Weak Echoes): `LoomWork.predictions[8]` reusing
the shared `judged` stamp; content `ARC_SCRIPT` — eight authored
steps (operations + per-step questions: strong count, weak count,
or the raising's outcome); puzzles runs a REAL counted strong/weak
model (`ArcModel`) over the script and `arc_trace()` is the model's
own output — [2,1,2,1,1,1,0,2], the last raising refused by an
empty book; `arc_foretold` compares the player's foretellings
against the computed trace. Presses `F7PredictCycle` / `F7ArcJudge`.
One GUI playthrough drives both chambers through the seal,
including the grade refusing while the backwards pair stands
chalked.

**Gates** (primo, rustc 1.99.0; evidence
`~/.local/share/pax/niflheim-floor7-step3-build-20261110/`):
- `cargo fmt --check` — clean.
- `cargo test` — **217 passed / 0 failed** (210 + 4 puzzles + 2
  state + 1 GUI).
- `cargo clippy --all-targets -- -D warnings` — exit 0, no fixes
  needed this step.
- Theodolite (cured protocol; bacon producer live): planted run —
  game.rs count=3, E0308 from `theodolite_route_control` at line
  10053, the other four touched files count=0 in the same session;
  restore sha256 `f764c721…` verified; clean run — all five touched
  files count=0. PASS both modes.
- Determinism: state-level probe through Floor 7 chambers 1–4
  (transient, removed after, DS-9) — two runs byte-identical,
  sha256 `8f9769e01ad429fdafcf44938bee0aae4feaba4825723bcf99f7f8d0d9988058`;
  round-trip asserts `floor7_chambers_repaired() == 4`.
- Android: exit 0 (libniflheim.so 1,350,198,120 bytes).
- Xvfb smoke: cold_start_ms=1108, zero panics.
- `#![forbid(unsafe_code)]` intact.

Commit: `d456574` on `pax/floor2-build-20261109`.

## Step 4 — The Lock-Free Stack and the Standing Seamstress

**Built.** Chamber 5 (The Lock-Free Stack): `LoomWork.verdicts[6]`
+ `witness`; content `STACK_HISTORIES` — six recorded histories as
(push/pop, value, start, end) operations — and `WITNESS_PAIRS` with
`WITNESS_KEY` 1 (the push of B, the pop that returned A). The
referee in puzzles is a REAL linearizability search:
`history_linearizable` tries every sequential order the turn
intervals allow (real-time precedence: an operation may be placed
only when nothing unplaced must precede it) against a sequential
stack; five histories order, the third — B pushed and never popped,
yet the pop returns A beneath it — does not. `stack_verdicts_true`
recomputes all six; the chamber has no judge press — the diagnosis
itself recomputes (verdicts + named witness). Presses
`F7VerdictCycle` / `F7WitnessCycle`.

The FOE (The Standing Seamstress, class #15 — Two Porters, One
Stairwell): `Floor7FoeProgress` gains exhibits[3], edges[4],
loom_run, loads[3], pressure, order[4] (all serde-defaulted, so
Step 1–3 era v7 saves still load). Her demonstration, in her order:
exhibit 0 the servants' claim orders (Ansel North→South, Berta
South→North); exhibit 1 the waits-for chart, its four edges drawn
by `F7FoeEdge` presses (edges refuse before the chart); exhibit 2
runs the deadlock on the model loom itself — state runs
`loom::explore` on the servants' opposite-order programs and
requires a deadlock found; the press text reports the found trace
and the searched count. Her gate is the bounded queue: three
stated loads (2×60, 3×45, 4×250 at capacity 8) walked on puzzles'
`queue_run` — a real turn-based producer/consumer model with
back-pressure (at capacity a producer's turn passes; nothing drops,
nothing spins) whose received set is reconciled exactly by dense
id — plus the at-capacity exhibit (`queue_at_capacity`: the queue
stands full, the next send pends). The repair is the posted order:
four claimables ranked (the two stairs plus the brazier and the
staff, carried from the floor below); `posted_order_holds`
requires a true permutation AND `loom::explore` on both servants
re-cut to the posted stair order returning no deadlock. Strike
arming (`floor7_foe_arm_strike`, Refusal-spoken) requires proof
(exhibits + loom run + order holds) and gate; the confirm seats
her — loomstone text to the codex, "The Threadweaver's Arc &
Queue" to the pack. Full wiring mirrors Floor 6's FOE: Screen::
Floor7Foe, DoorFoe7 (hub layout + drawn door), exhibits/edges/
loads/pressure/order elements, seal arm/confirm arms, Enumerate =
exhibit 0, her own 4-rung hint ladder, inscription/demonstration
readings, token/cell-blocked/tab lists. The puzzles suite carries
the DS-13 million-message exact-set proof over `queue_run`
(10 × 100,000 at capacity 64: sent = received = 1,000,000,
reconciled).

**Incidents, self-caught and cured in-step.** (1) The first
splice's Enumerate insert duplicated the verb arm's opener —
game.rs failed to parse; restored from git and re-spliced with
the insert inside the existing arm. (2) Two splicer guards were
non-unique across floors (`pub exhibits: [bool; 3]`,
`pub verdicts` exist in elder floors' structs), so the FoeProgress
and LoomWork field blocks were silently skipped while their
consumers landed — E0609s at first compile; both blocks applied
by a direct fix-up splice with unique anchors. Guards in this
programme are now checked for cross-floor collisions before use.

**Gates** (primo, rustc 1.99.0; evidence
`~/.local/share/pax/niflheim-floor7-step4-build-20261110/`):
- `cargo fmt --check` — clean.
- `cargo test` — **228 passed / 0 failed** (217 + 7 puzzles + 3
  state + 1 GUI), re-run after the clippy cure: still 228/0.
- `cargo clippy --all-targets -- -D warnings` — exit 0
  (`clippy2.log`). One curative fix: `queue_run`'s producer loop
  rewritten to `next.iter_mut().enumerate()` (first run at
  `clippy.log`).
- Theodolite (cured protocol; bacon producer live): planted run —
  game.rs count=3, E0308 from `theodolite_route_control` at line
  10515, the other four touched files count=0 in the same session;
  restore sha256 `9338b49e…` verified; clean run — all five touched
  files count=0. PASS both modes.
- Determinism: state-level probe through the whole floor — all
  five chambers repaired and the Seamstress seated,
  `floor7_complete()` asserted in-probe (transient, removed after,
  DS-9) — two runs byte-identical, sha256
  `84346cd774e7a193a486764db5f61bba4d6ae6f2d04981d91c1358e97495bca3`.
- Android: exit 0 (libniflheim.so 1,350,816,104 bytes).
- Xvfb smoke: cold_start_ms=1049, zero panics.
- `#![forbid(unsafe_code)]` intact.

Commit: (this commit)
