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

Commit: `STEP1_SHA` on `pax/floor2-build-20261109`.
