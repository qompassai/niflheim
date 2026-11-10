# Floor 5 (The Veilworks) — Step Report

Programme: Niflheim completion, fifth lane (D-1189:
CLAUDE-DIRECT authoring throughout this floor — no phlow
local-stack routing; the lane authors and gates its own
code). Brief: FLOOR5-BRIEF.md (this workspace and in-tree).
Gates per rung: cargo test growing with 0 failures, clippy
--all-targets -D warnings clean, fmt clean, theodolite 0
diagnostics on touched .rs (the cured route: bacon producer
first, positive control in-session), determinism (two
playthrough saves byte-identical), Android lib under pinned
rustc 1.99.0, Xvfb launch smoke. Local commits only on
pax/floor2-build-20261109; no pushes.

## Step 1 — Scaffolding

### What landed

- **`core.rs`** — `SAVE_FORMAT_VERSION` 4 → 5.
- **`state.rs`** — `Floor5Progress` (five
  `Floor5ChamberProgress` + `Floor5FoeProgress` +
  `foe_hints_used`), `VeilWork` (the `watched` flag; the
  chambers' fields land with their steps, each
  serde-defaulted), the `Expedition.floor5` field wired
  through `SaveFile` (serde-defaulted, so a version-4 save
  loads with a fresh Floor 5), `save_json`, and `load_json`.
  Accessors in the house shape: `floor5_diagnosed` (no
  chamber proves yet — the judges land with their chambers),
  `floor5_file_proof` (per-chamber refusals naming each
  chamber's bar), `floor5_repair_chamber` (the facet text
  names the Ice Stone), hint rung recorders,
  `floor5_reset_chamber` (hint usage stays on the record),
  `floor5_chambers_repaired`, `floor5_complete`.
  `floor5_reachable()` = `floor4_complete()`.
  `validate_invariants` holds five Floor 5 chambers and
  recomputes their proofs like its elders'. Fixtures: the
  future-version forge now rewrites `"version":5`; the
  three legacy-version tests re-save as 5; a new version-4
  test proves a v4 save loads with a fresh Floor 5 and
  re-saves as 5.
- **`content.rs`** — `FLOOR5_NAME` / `FLOOR5_DOMAIN` /
  `FLOOR5_KEEPER` (the Veilwalker), `FLOOR5_INTRO`, and
  `VEIL_CHAMBERS`: all five ChamberMeta fully authored
  (The Read Card, One Writer or Readers, Checkable or
  Vapour, The Seam Contract, Fix the Card Not the Symptom),
  `FLOOR5_CHAMBER_INTROS`, `F5_EXAMINE_TEXTS`, the five
  Watch charters, and `floor5_chamber_hints` (the general
  ladder; per-chamber ladders land with their chambers).
- **`game.rs`** — `Screen::Floor5Hub` / `Floor5Chamber`;
  `El::DoorFloor5` / `Floor5Door(u8)` / `F5Watch`;
  `geom::door_floor5()` = (640, 150, 124, 124) on **Floor
  4's hub** — verified open ground: below the chamber row
  (ends y=136), left of the panel (x=792), above the FOE
  door (y=292), clear of the camp stone (352..416 ×
  288..352). The Floor 4 hub layout pushes the door; the
  Floor 5 hub layout carries the camp stone and five
  chamber doors (the FOE door lands with Step 4). The
  door's press is gated on `floor5_reachable()`; barred, it
  names the unmet completion in words (never a locked-door
  buzz). Chamber doors enter and speak their intros.
  Verbs, token visibility, cell blocking, Examine, Prove,
  Hint, ResetChamber, the seal press/confirm, the readings
  (inscription / demonstration / hint / floor intro), and
  the layout tab match all know the new screens.
- **`shell.rs`** — `build_floor5_hub` (five doors with
  state tags, the camp stone, the panel with the chambers
  list and the FOE's standing), `build_floor5_chamber`
  (header, room, desk, rule stone, slot — the per-chamber
  builders land with their steps), the build dispatch, the
  header titles, and the map's Floor 5 square (BARRED /
  UNCHARTED / CHARTING / FOE FALLEN, like its neighbours).

Build notes: two curative fixes on the way. (1) serde's
`#[serde(default)]` on the new save field needs
`Floor5Progress: Default` — the manual impl mirroring
Floor 4's landed in the same step. (2) Clippy's
derivable-impls lint took VeilWork's manual Default —
derived instead (Floor 4's work record keeps its manual
impl: its treaty field's start value is not the type's
default). **Environment note:** one Android build attempt
failed E0463 because that shell resolved `/usr/bin/cargo`
(Arch's system Rust 1.99.1, no Android std) ahead of the
rustup proxy; the pinned 1.99.0 toolchain is intact (its
aarch64 std compiles directly). All primo cargo work in
this lane now prefixes `PATH=$HOME/.cargo/bin:$PATH`.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **139 passed / 0 failed** (was 136; +3:
  state 2 — the version-4 load, Floor 5 reachability
  following Floor 4's completion; game 1 — the door
  answering the pointer exactly when Floor 4 stands
  complete). Log:
  `~/.local/share/pax/niflheim-floor5-step1-build-20261110/cargo-test.log`.
- Theodolite (cured route; `bacon --headless -j bacon-ls`
  producer first, `.bacon-locations` present at session
  start): planted control appended to game.rs read
  count=3 (E0308) in the same session where core.rs,
  content.rs, state.rs and shell.rs read 0; clean session
  reads **0 on all five touched files** — the zeros are
  verified, not assumed.
- Determinism: the Floor 4 Step 4 probe re-run on the new
  tree; saves byte-identical, sha256
  `8511521e424be8aa541aee209669d0e3bf38fa7852095cf6b572027498dd6981`
  (the hash moved from Floor 4's, as it must: the save
  gained the version bump and the Floor 5 record).
- Android lib under the pin: 1,347,685,640 bytes. Launch
  smoke under Xvfb: `cold_start_ms=929`, no panics.

### Left open

- Chambers 1–2 land in Step 2 (the Pointer Walks, the Two
  Tenants); Chambers 3–4 in Step 3 (the Contract Cards, the
  Split Beam); Chamber 5 and the Thing Behind the Veil in
  Step 4; the driven whole-floor playthrough in Step 5.
- Floor 6's door hangs on this hub at Floor 6's
  scaffolding, in the same (640, 150) spot Floor 5's door
  proved here.
