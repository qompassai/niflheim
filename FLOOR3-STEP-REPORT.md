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
