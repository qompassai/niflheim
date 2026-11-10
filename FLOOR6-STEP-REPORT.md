# Floor 6 Step Report — Niflheim Completion (Part A)

Branch: `pax/floor2-build-20261109` (local only, no pushes).
Start tip: `985a91c` "Floor 5 Step 5: Integration — Floor 5
Complete" (suite 161/0, Floor 5 determinism saves sha256
b7c288f6733204b0e38cbc0eb1d65088e8c1e0457daeff60739ec9fe5921d321).
Governing documents: `SPEC.md` (the completion spec) §4 Floor
6, `FULL-DESIGN.md` Floor 6, `FLOOR6-BRIEF.md` (the assembly
brief, in-tree with this report).

Per-step evidence directories on primo:
`~/.local/share/pax/niflheim-floor6-step<N>-build-20261110/`.

## Step 1 — Scaffolding

The floor framed end to end, mirroring Floor 5 Step 1 part
for part: the save format reaches v6 (DS-2, the one bump,
at this step); `floor6_reachable()` = Floor 5's chambers
0–2 repaired — the spec's ruling, unlike Floor 5's
all-five gate, and the door test proves it adversarially
(the fourth chamber repaired with the third unworked still
bars the door); the door stands on Floor 5's hub at
`geom::door_floor6()` (640, 150, 124, 124) — the open field
Floor 5's own door proved on Floor 4's hub, verified clear
of the chamber row above (ends y=136), the FOE's door below
(starts y=292), the camp stone, and the panel. The chambers
carry intros, charters, examine texts, and named refusals;
no chamber can be worked yet (every judge lands with its
step); the FOE's records exist with no screen behind them.
The shell reaches the hub and the chamber skeletons, the
map gains the floor's square (index 5, number 6), and the
Lockheart Stone is named in the repair facet text, ready
for Step 5's implementation. One parity note, pre-existing
and not a regression: like Floors 4 and 5 before it, the
next-floor door has no drawn visual on its hub — the door
lives in layout, press, and map exactly as Floor 5's did;
drawing hub door visuals for Floors 3–6 is out of this
floor's scope.

Gates, all on primo under pinned rustc 1.99.0 (evidence:
`~/.local/share/pax/niflheim-floor6-step1-build-20261110/`):

- `cargo fmt --check`: clean (after one applying fmt pass).
- `cargo test`: **164 passed, 0 failed** (161 at the Floor
  5 tip + the door test and two state tests; no test
  deleted, skipped, or weakened).
- `cargo clippy --all-targets -- -D warnings`: exit 0, no
  warnings.
- Theodolite (cured-protocol session record): planted run —
  game.rs count=9, all E0308 from the planted
  `theodolite_route_control` at line 8349, while core /
  content / state / shell read 0 in the same session;
  restore verified by sha256 (1c958003… before and after);
  clean run — all five touched files (core, content, state,
  game, shell) count=0; THEODOLITE: PASS both modes.
- Determinism: the Floor 5 Step 5 probe (state-level
  playthrough, load-back asserted), renamed
  `floor6_step1_playthrough_save`, run twice: saves
  byte-identical, sha256
  7d84e8281affcdde2cfa47d0691455bc45ddeb250145755aa5ecbd27f2748bd1.
  Probe removed after the runs; the tree carries only the
  five source files.
- Android: `cargo build --target aarch64-linux-android
  --lib` exit 0 (libniflheim.so 1,348,837,560 bytes).
- Xvfb smoke: fresh save path, `cold_start_ms=1051`, zero
  panics. `#![forbid(unsafe_code)]` intact (src/lib.rs:26).

Commit: `3994899` on `pax/floor2-build-20261109`.

## Step 2 — The Cell Heart and the Borrow-Flag Brazier

The first two chambers work. The Cell Heart walks a
scripted day of six operations over a real flag machine in
puzzles.rs (Free / Shared(n) / Exclusive; exclusive opens
only from Free): both shared openings, a closing, the
exclusive opening refused while Bryn's share stands — the
bell strikes, set from the replay, not by hand — the last
closing, and the exclusive opening from free. Operations
walk only in the day's order. The Brazier runs five calls
over a lazy cell that kindles exactly once: three calls for
the flame and two reads, the light-count shown after every
call and standing at one at the end, computed by the sim.
Both judges require the demonstration watched. One curative
fix on the way: clippy's result-unit-err sent `flag_step`
from `Result<_, ()>` to `Option` — refused is None — with
callers and tests following; no lint allowed or narrowed.

Gates, all on primo under pinned rustc 1.99.0 (evidence:
`~/.local/share/pax/niflheim-floor6-step2-build-20261110/`):

- `cargo fmt --check`: clean.
- `cargo test`: **173 passed, 0 failed** (164 at Step 1 +
  six puzzles tests, two state tests, one GUI test; no test
  deleted, skipped, or weakened).
- `cargo clippy --all-targets -- -D warnings`: exit 0.
- Theodolite (cured protocol): planted run — puzzles.rs
  count=9, all E0308 from the planted control at line 1981,
  the other four touched files 0 in the same session;
  restore sha256-verified (9e96416e…); clean run — all
  five files (puzzles, content, state, game, shell)
  count=0; PASS both modes.
- Determinism: the playthrough probe extended through
  both chambers at state level (day replayed op by op,
  proofs filed, repairs struck, load-back asserted), run
  twice on the final tree: byte-identical, sha256
  a23c36d3703b435762e9e573b89f6be2901cdabf84ad9379056ad5bc7deef8a6.
  Probe removed after the runs.
- Android: lib builds, exit 0 (libniflheim.so
  1,348,981,552 bytes).
- Xvfb smoke: `cold_start_ms=1276`, zero panics.
  `#![forbid(unsafe_code)]` intact.

Commit: recorded at Step 3.
