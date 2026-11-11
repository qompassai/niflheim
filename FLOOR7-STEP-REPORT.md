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

Commit: (this commit)
