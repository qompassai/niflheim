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
