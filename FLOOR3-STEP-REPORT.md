# Niflheim — Floor 3 Step Reports

Continuity record for the Floor 3 build (the Gatehouse), kept
in-tree per the programme's standing rule: a rung is not landed
until its commit and its report exist. The brief is
`FLOOR3-BRIEF.md`; the design of record is `FULL-DESIGN.md`.

Production axes (D-1145): Steps 1–5 of this floor are authored
directly by the coordinator lane with the blocks-and-splicer
method, because the plan's files exceed what the local stack
carries reliably in one pass; each step's gates are run on primo
under the pinned toolchain and recorded below with their
artifacts. This is stated plainly rather than dressed up.

## Step 1 — scaffolding (save v3, the Gatehouse hub)

Landed 2026-11-10. Floor 3 exists in the world: its door stands
on Floor 2's hub, its hub renders, the save format carries its
record, and the stairs read the whole Stratum I state.

### What landed

- **`core.rs`** — `SAVE_FORMAT_VERSION` 2 → 3.
- **`content.rs`** — Floor 3's authored text: `FLOOR3_NAME`
  ("The Gatehouse"), `FLOOR3_DOMAIN`, `FLOOR3_KEEPER`,
  `FLOOR3_INTRO`, the Lantern Stone and its text ("A result
  unexamined is a gate unrun."), the Gatehouse Key and its
  text, and `GATEHOUSE_CHAMBERS`: the five chamber metas
  (Named Failures; The Conveyor; The Audit of Shouts; The
  Sealed Request; The Inspection).
- **`state.rs`** — `Floor3Progress` beside Floor 2's record:
  five `Floor3ChamberProgress` (state + `GatehouseWork`, an
  empty record the later steps fill facet by facet), the FOE
  record (`Floor3FoeProgress`), helpers
  (`floor3_reachable` — both valley FOEs fallen, §5.2;
  `floor3_chambers_repaired`; `floor3_complete`), the save
  field with `#[serde(default)]`, and the five-chamber
  invariant. **`stairs_clauses` rewritten to read the whole
  Stratum I state** (§5.6): clause 2 is lit when FOE 2 OR FOE 3
  has fallen, clause 3 reports each floor's lantern
  individually (a lantern is lit by its gate **or** its five
  chambers, the keeper-lit substitute), and the unmet texts
  name what is missing.
- **`game.rs`** — `Screen::Floor3Hub`; `El::DoorFloor3` and
  `El::Floor3Door(u8)`; `geom::door_floor3()` at (640, 150)
  on Floor 2's hub (open ground, clear of the roster board
  and the Herald strip — the navigation ruling from the
  brief); the door's press (reachable → the hub and its
  intro; barred → the plain refusal); the Floor 3 chamber
  doors answer with the Gatewright's honest not-open-yet
  line until their steps land; the hub joins the tab row,
  the token's home screens, the walk rules, the header, the
  reading text, and the Examine narration. The Step 5
  playthrough's clause-3 assertion was updated to the new
  per-lantern wording (same standing asserted, new text).
- **`shell.rs`** — `build_floor3_hub` (five chamber doors
  with state tags, the floor stone, the panel: keeper line,
  the chambers list, the FOE line, the stairs standing); the
  Floor 3 door drawn on Floor 2's hub with its OPEN/BARRED
  tag; the shell header's Floor 3 title; the floor map's
  Floor 3 square (BARRED / UNCHARTED / CHARTING (n/5) / FOE
  FALLEN — date).
- **`lib.rs`** — the crate doc records the scaffolding.

Two fixture updates accompanied the format bump, both keeping
the behaviour they assert: the v99 future-version probe now
forges from `"version":3`, and the v1-loads test's re-save
assertion now expects version 3.

Build notes, honestly: the splice put `Screen::Floor3Hub`
outside its enum on the first pass (the anchor carried the
closing brace) and stranded two doc comments above the wrong
functions (insert-before anchors that carried doc comments);
all three were caught by the compiler and clippy
(`empty line after doc comment`) and fixed in place. Three
exhaustiveness sites the plan had not enumerated — the
tab-bar match, `cell_blocked`, and the shell's header title —
were found by E0004 and armed for the new screen.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean.
- `cargo clippy --all-targets -- -D warnings`: exit 0.
- `cargo test`: **95 passed / 0 failed** (was 90; +5: four
  state tests — v2 loads with default Floor 3, reachability
  needs both FOEs, the stairs read Floor 3, a forged Floor 3
  length is refused — and one game test, the Floor 3 door
  answering the pointer). Log:
  `~/.local/share/pax/niflheim-floor3-step1-build-20261110/cargo-test.log`.
- Theodolite (rust-analyzer route): **0 diagnostics** on all
  six touched files (`core.rs`, `content.rs`, `state.rs`,
  `game.rs`, `shell.rs`, `lib.rs`); log in the same evidence
  directory.
- Determinism: the Step 5 full-floor state-level playthrough
  probe run twice on the new tree; saves byte-identical,
  sha256
  `76647e3a7647b4f75b597f4f203dfab07e79a61d94056473e5d9138f4866e5d0`
  (the hash moved from Step 5's, as it must: the save gained
  the version bump and the Floor 3 record). A version-2 save
  loads on the new tree (in-suite test).
- Android: `cargo build --lib --target aarch64-linux-android`
  under the pin passes; `libniflheim.so` 1,344,576,808 bytes.
- Launch smoke under Xvfb: `cold_start_ms=1054`, no panics.

### Left open

- Chambers 1–5 and the FOE land in Steps 2–4 per the brief;
  `GatehouseWork` is deliberately empty until then.
- The Stratum II floors' doors (Floor 4 onward) arrive with
  their own floors' scaffolding steps; the stairs already
  report their clauses truthfully in the meantime.

## Step 2 — Chambers 1–2 (Named Failures; the Conveyor)

Landed 2026-11-10. The Gatehouse's first two desks are open
and playable end to end on the GUI surface.

### What landed

- **`content.rs`** — Chamber 1: `FAILURE_CHARTER`,
  `FAILURE_PAIRS` (five kind+remedy pairs; the fifth is the
  generic **Broken** bin, present as a candidate precisely so
  the desk can refuse it), `FAILURE_REPORTS` (four reports,
  each with its true pair). Chamber 2: `CONVEYOR_CHARTER`,
  `BeltItem` (Sound/Spoiled), `CONVEYOR_BELT` (gear, gear,
  spring, cracked gear, spring, cam, cam, bell),
  `CONVEYOR_REQUIRED` (gear, spring, cam, bell), and the
  three stations' option lists. `FLOOR3_CHAMBER_INTROS` for
  the Read verb, and `floor3_chamber_hints` (real ladders for
  Chambers 1–2; the unopened desks answer honestly).
- **`puzzles.rs`** — the judges. `failure_pair_ok`: a report
  sorts only under its own pair, so the Broken pair sorts
  nothing — the refusal is the table. `conveyor_run`: the
  player's three stations composed as one real iterator chain
  over the belt (take → dedup-by-closure → halt-mode), polled
  past its end; `conveyor_diagnosed` requires the exact slate
  and a fused chain. Unit tests prove the correct setting
  (whole belt, first sightings, lift the cracked piece) is
  **unique across all 27 settings**.
- **`state.rs`** — `GatehouseWork` gains the facets
  (`watched`, `pair_choices`, the three station choices), all
  serde-defaulted. `floor3_diagnosed` / `floor3_file_proof` /
  `floor3_repair_chamber` mirror Floor 2's proof discipline,
  facet rule included; hint counters beside them. The save
  invariant now recomputes Chambers 1–2: a claimed proof its
  work does not hold is refused at load.
- **`game.rs`** — `Screen::Floor3Chamber`, `ArmedFor::
  Floor3Chamber`, elements `F3Watch` / `F3Pair` / `F3Station`
  (cycling choices, with the belt's live delivery spoken at
  each setting), the chamber layout and seal wiring, the
  Prove/Hint verbs, the reading views (the charters as the
  demonstrations), and the hub door opening the two desks.
- **`shell.rs`** — `build_floor3_chamber` with per-chamber
  fields: the wall of reports with pair buttons and remedies
  (the Broken remedy prints in the warning colour), and the
  belt view — belt, slate, three station buttons, and the
  current setting's computed delivery.

Build notes: the Prove-verb insertion first landed between
match arms (the splicer's anchor sat at the arm's end, not
inside it) and two doc comments were stranded by
insert-before anchors again — all caught by the compiler or
clippy and fixed in place; the tab-bar op briefly duplicated
a pattern line (unreachable-pattern error) for the same
reason. These are the splicer's known failure shapes; the
gates caught every one.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets --
  -D warnings`: exit 0.
- `cargo test`: **102 passed / 0 failed** (was 95; +7:
  puzzles 2 — the pair judge and the conveyor's unique
  setting; state 4 — the judged sort, the unwatched sort
  refused, the judged belt, the forged proof refused; game 1
  — both chambers played on the GUI surface, including a
  re-sort under a wrong pair unmaking the proof mid-play).
  Log: `~/.local/share/pax/niflheim-floor3-step2-build-20261110/cargo-test.log`.
- Theodolite: **0 diagnostics** on all five touched files;
  log in the same evidence directory.
- Determinism: the state-level probe extended through both
  chambers, run twice; saves byte-identical, sha256
  `8c465b0506fb05b4fd45be77b3e27cd45fd55a7349018b4417993424e61ce5e4`.
- Android lib under the pin: 1,345,063,320 bytes. Launch
  smoke under Xvfb: `cold_start_ms=1629`, no panics.

### Left open

- Chambers 3–4 land in Step 3, Chamber 5 and the FOE in
  Step 4, the driven whole-floor playthrough in Step 5.

## Step 3 — Chambers 3–4 (the Audit of Shouts; the Sealed Request)

Landed 2026-11-10. The Gatehouse's middle desks are open.

### What landed

- **Chamber 3 — panic discipline.** `SHOUT_CHARTER`,
  `SHOUT_CASES` (four ledger entries: two traveller-reachable
  failures that must become named refusals; two invariant-only
  failures whose shouts stand), `INVARIANT_CANDIDATES` (two
  sworn by the house, two travellers' premises as decoys).
  The judge (`shout_case_ok`) requires verdict **and**
  invariant to match: a standing shout without its own
  invariant named is unsorted. The booth renders the ledger
  with a verdict and an invariant button per entry, and the
  candidate list numbered in the panel.
- **Chamber 4 — typestate builders.** `RAIL_CHARTER`,
  `REQUEST_SLIPS` (three slips at authored stages; the third's
  body page is missing). The rail is a state machine in
  `floor3_slip_act`: advance walks unaddressed → addressed →
  bodied; the seal press refuses any stage before bodied
  ("Iron, not policy"); advancing the body-less slip is
  refused at the empty tray; the returns tray refuses honest
  slips. Judged complete only at [sealed, sealed, returned].
- **`state.rs`** — `GatehouseWork` gains `shout_choices`,
  `invariant_choices`, `slip_stages`, and a manual `Default`
  (the slips arrive at their authored stages, [0, 1, 1]).
  `floor3_diagnosed` and `floor3_file_proof` now cover four
  chambers; the save invariant's recompute extends to
  Chambers 3–4.
- **`game.rs` / `shell.rs`** — elements `F3Shout`,
  `F3Invariant`, `F3Slip(slip, action)`; the hub door opens
  four desks; hint ladders for Chambers 3–4 replace the
  placeholder rungs; the charters serve as the demonstrations.

Build notes: one borrow error (E0499) in the new press arms —
the verdict text was computed from the borrowed slot while
`say` needed `self`; fixed by scoping the borrow and speaking
after it ends, the same shape Step 4 of Floor 2 taught. One
splice dropped a struct field that shared its anchor with a
closing brace; the compiler caught it and it was restored.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets --
  -D warnings`: exit 0.
- `cargo test`: **108 passed / 0 failed** (was 102; +6:
  puzzles 2 — the audit judge and the rail judge; state 3 —
  the judged audit, the rail's enforced order, the forged
  rail proof refused; game 1 — both chambers played on the
  GUI surface, the press refused early on the record).
  Log: `~/.local/share/pax/niflheim-floor3-step3-build-20261110/cargo-test.log`.
- Theodolite: **0 diagnostics** on all five touched files.
- Determinism: the probe extended through both chambers,
  run twice; saves byte-identical, sha256
  `3083015a1bee9a4977b9d665e40a779a2c201f4bc2324d47be146d4217db5411`.
- Android lib under the pin: 1,345,008,040 bytes. Launch
  smoke under Xvfb: `cold_start_ms=2168`, no panics.

### Left open

- Chamber 5 (the Inspection) and the FOE (the Gatehouse
  Release) land in Step 4; the driven whole-floor playthrough
  in Step 5.

## Step 4 — Chamber 5 (the Inspection) and the FOE (the Gatehouse Release)

Authored directly by the coordinator (blocks + splicer), as
Steps 1–3 of this floor. Two halves: 4a the judged core
(gate, content, puzzles, state), 4b the rooms (game, shell).

### What landed

- **`gate.rs`** — `GateReport` gains `fmt_checked` /
  `fmt_passed`; `passed()` now requires the format verdict
  when one was taken. `RELEASE_SPEC` and `run_gate3`: the
  Release's gate runs the hidden tests and clippy like its
  elders, then holds the draft *as written* up to a bounded
  `rustfmt --edition 2024 --check` — the format lantern is
  part of the gate, not a courtesy. Two lantern runners
  stand beside the gates: `run_fmt` (applies rustfmt in a
  claimed scratch dir and returns the formatted text — the
  lantern lights by *doing*, and the draft comes back
  formatted) and `run_lint` (the Release's own clippy over
  the assembled project). Both answer a `LanternReport`.
  The hand-authored reference was verified rustfmt-true by
  the gate itself.
- **`content.rs`** — the Inspection's wall: the charter
  (stating the oldest road: the gate honors every road
  **from 1.85 onward**), five checklist items with true
  verdicts [Repaired, Conforms, Not applicable, Conforms,
  Repaired] — the fifth is the Stranger's Anvil line — the
  set-aside line's candidate reasons (the true one: "The
  Hoard seals nothing; it lends and holds."), the oldest-
  road stamp candidates (true: 1.85), the amendment text,
  and the rite's three stages. The FOE: the Gatehouse
  Release, whose inscription names **failure class #5, The
  Lantern That Lights Itself**; the log slate's three
  claimed passes, of which only the third — fresh paper,
  the draft's own mark — ever ran. Gate 3's crate
  (`the_gatehouse_release`), scaffold, reference, and six
  hidden tests. Chamber 4's hint ladder is real now.
- **`puzzles.rs`** — the judges: `inspection_item_ok`,
  `inspection_reason_ok`, `msrv_ok`, `slate_entry_ok`.
- **`state.rs`** — Chamber 5's work (checklist choices, the
  set-aside reason, the stamp, the rite stage); the rite
  **gates the seal**: a Diagnosed Chamber 5 with the rite
  unfinished refuses repair on the record. The Release's
  record: slate taken/judged/done, the stamp, its own rite
  entered in the book, three lanterns, the draft (with the
  Step 5 `editor_started` discipline), and the gate record.
  Proof complete = slate reconciled ∧ the stamp is the
  oldest road ∧ the rite stands entered. Any edit douses
  all three lanterns and the gate mark — they were lit for
  the draft as it stood. The strike awards **the Lantern
  Stone** ("A result unexamined is a gate unrun.") to the
  codex and **the Gatehouse Key** to the pack.
- **`game.rs` / `shell.rs`** — Chamber 5's desk (verdict
  cycles, the reason and stamp fields, the rite button on
  the side desk) and the Release's room on Floor 3's hub
  (`geom::door_foe3()`, the spot Floor 2's FOE door proved
  clear): the slate and its verdicts, the reconciliation,
  the stamp, the rite button, three lantern buttons, the
  editor, and the two-step seal. `IoRequest` learns
  `floor3` and a `RunLantern` request; the shell runs gate
  3 and both lanterns on threads and delivers their reports
  through a `LanternChannel` twin to the gate's. Step 2's
  placeholder refusal at the fifth desk's door is lifted.

Build notes: the FOE door's layout push first landed in
Floor 2's hub arm and was moved to Floor 3's; one splice
set the Examine arm inside its neighbour's body (the parse
error named it; relocated); one E0499 in the slate press
(the verdict is computed inside the borrow's scope and
spoken after it ends). The driven test sets `gate_floor3`
exactly as the RunGate press leaves it before answering
gate results through the handlers.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **118 passed / 0 failed** (was 108; +10:
  gate 4 — the scaffold refused, the reference passing with
  its format verdict, gates 1–2 leaving format unchecked,
  both lanterns answering; puzzles 2 — the inspection
  judges and the slate judge; state 3 — the judged
  inspection with the rite gating its seal, the Release's
  proof and strike, the doused lanterns; game 1 — Chamber
  5 and the Release played on the GUI surface, the rite
  refusing the seal on the record until walked).
  Log: `~/.local/share/pax/niflheim-floor3-step4-build-20261110/cargo-test.log`.
- Theodolite: **0 diagnostics** on all six touched files.
- Determinism: the probe extended through Chamber 5 and
  the Release's whole arc, run twice; saves byte-identical,
  sha256
  `aa4ed6795880c35976adbbb4ebbc5fa139b292724c581d05cec66715691cc236`.
- Android lib under the pin: 1,345,769,504 bytes. Launch
  smoke under Xvfb: `cold_start_ms=901`, no panics.

### Left open

- Step 5: the driven whole-floor playthrough (all five
  chambers and the Release on the GUI surface, the gate
  answered through the io slot as the shell delivers it)
  and the floor's close-out. Floor 3 completes there.

## Step 5 — Integration: the whole floor on the GUI surface

### What landed

- **`game.rs` (tests only)** — `floor3_gui_playthrough_fells_the_release`:
  both prerequisite FOEs felled at the state level (the
  Gatehouse's §5.2 price), then the doors walked honestly
  (Floor 1 hub → Floor 2 → Floor 3), all five chambers
  played through their desks and sealed through the
  two-step seal, Chamber 5's rite walked between its Prove
  and its Seal, and the Release played end to end: slate
  judged and reconciled, the stamp set, the rite entered,
  the reference typed into the game's own editor, the
  format and lint lanterns lit by the **real** engines
  (`run_fmt` / `run_lint`) answered through the io slot
  exactly as the shell files them, the third gate run for
  real (`run_gate3`, its own format verdict asserted), and
  the strike through the seal. The record asserts the
  Lantern Stone in the codex, the Gatehouse Key in the
  pack, `floor3_complete()`, clause 3 of the stairs reciting
  all three Stratum I lanterns lit, and the whole record
  round-tripping through the save byte-for-byte.

Build note: the first run of the playthrough caught the
lantern buttons' true place — they are room furniture, and
the editor overlay owns the layout while it stands open,
so the honest order is the one a player walks: type the
draft, close the editor, light the lanterns in the room,
reopen the editor to file the gate. The test walks that
order; no game code changed.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **119 passed / 0 failed** (was 118; +1:
  the whole-floor playthrough).
  Log: `~/.local/share/pax/niflheim-floor3-step5-build-20261110/cargo-test.log`.
- Theodolite: **0 diagnostics** on all six Floor 3 files.
- Determinism: the Step 4 probe re-run on the final tree;
  saves byte-identical, sha256
  `aa4ed6795880c35976adbbb4ebbc5fa139b292724c581d05cec66715691cc236`
  (unchanged — Step 5 touched test code only).
- Android lib under the pin: 1,345,769,504 bytes. Launch
  smoke under Xvfb: `cold_start_ms=1097`, no panics.

### Floor 3 — COMPLETE

The Gatehouse stands finished: five chambers (Named
Failures, the Conveyor, the Audit of Shouts, the Sealed
Request, the Inspection), the Gatehouse Release fallen,
the Lantern Stone in the codex, the Gatehouse Key in the
pack — the key the descent wields again on Floor 17.
Stratum I is complete: Floors 1, 2, and 3 all stand, and
the stairs' third clause reads all three lanterns lit.

## Addendum (2026-11-10, D-1189 fold-in) — the chamber reset, repaired

Floor 3's chambers laid out a ResetChamber button from
their first step, but the press fell through to the match's
catch-all: `floor3_reset_chamber` did not exist. Found while
mirroring the pattern for Floor 4 (whose chambers reset
properly), flagged in FLOOR4-STEP-REPORT.md Step 2, and
fixed here as its own gated commit at the Owner's word:
`floor3_reset_chamber` in the house shape (state →
Unworked, facet cleared, work defaulted, hints kept), the
`Screen::Floor3Chamber` arm in the reset match, and a
regression test that works a chamber on the GUI surface,
resets it, and reads the work given back. Gates: 130
passed / 0 failed (was 129, +1 regression), clippy clean,
fmt clean, theodolite 0 diagnostics on state.rs/game.rs,
determinism unchanged (sha256 92f009fa…), Android lib
1,346,850,528 bytes, smoke cold_start_ms=884, no panics.
Evidence: `~/.local/share/pax/niflheim-f3reset-build-20261110/`.
