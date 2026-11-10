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

## Step 2 — Chambers 1–2: the Pointer Walks, the Two Tenants

Committed by the Part A coordinator lane under the
completion spec (D-1190; start authorization D-1205),
per DS-10: lane #5 built and gated this step before the
stop order; the commit waited for the spec. Every gate
below was RE-RUN at commit time on the uncommitted tree
(D-567) and re-passed before the commit was made.
Evidence: `~/.local/share/pax/niflheim-f5s2-regate-20261110/`
on primo.

### What landed

- **`state.rs`** — VeilWork gains the two chambers' fields,
  all serde-defaulted: C1 `cards_read: [bool; 6]`, `walked`,
  `refused`; C2 `predictions: [u8; 8]`, `judged`,
  `annotations: [Option<usize>; 8]`. The Floor 5 judges and
  proof refusals now cover chambers 1–2.
- **`puzzles.rs`** — the pure judges: `step_cracked` (a
  Pointer Walks step's card claim is false),
  `tenancy_correct` / `tenancy_missed` (the Two Tenants
  verdicts against the authored tenancy table).
- **`content.rs`** — the Pointer Walks' six contract cards,
  the eight tenancy placards with their deciding rules, and
  the two chambers' four-rung hint ladders (the fourth rung
  names each seam).
- **`game.rs` / `shell.rs`** — the chamber surfaces: card
  reading before stepping, the step/refuse presses, the
  prediction, judging and annotation presses, and the
  per-chamber builders for chambers 1–2.

### Gates (primo, pinned rustc 1.99.0) — re-run at commit time

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **145 passed / 0 failed** (was 139 at
  Step 1; +6). Log: `cargo-test.log` in the evidence dir
  above.
- Theodolite (cured route; bacon producer first,
  `.bacon-locations` present at session start): a planted
  control on puzzles.rs read live E0308 diagnostics in the
  same session where content.rs, state.rs, game.rs and
  shell.rs read 0; the clean session reads **0 on all five
  touched files** — the zeros are verified, not assumed.
  puzzles.rs restored byte-identical after the plant
  (sha256 `d7254e3d…` on both sides).
- Determinism: the Step 2 probe (the earlier playthrough
  extended through the Pointer Walks and the Two Tenants)
  run twice; saves byte-identical, sha256
  `456c7b47ffbc91f13e4bf45759267938a9bdb79a85c3a8609dc155c1679f9c33`
  — the same hash lane #5 recorded, as it must be: the tree
  did not move between its gate run and this commit.
- Android lib under the pin: 1,347,792,040 bytes (the size
  lane #5 recorded). Launch smoke under Xvfb:
  `cold_start_ms=4639`, no panics (the start time runs
  higher under shared load; the gate is the line's presence
  plus zero panics, and both hold).

### Left open

- Chambers 3–4 land in Step 3 (the Contract Cards, the
  Split Beam); Chamber 5 and the Thing Behind the Veil in
  Step 4; the driven whole-floor playthrough in Step 5.

## Step 3 — Chambers 3–4: the Contract Cards, the Split Beam

Built Claude-direct by the Part A coordinator lane under
the completion spec (splice53.py + blocks/, anchors
asserted exactly once against the tree at 7e8b767).
Evidence: `~/.local/share/pax/niflheim-floor5-step3-build-20261110/`
on primo.

### What landed

- **`state.rs`** — VeilWork gains, all serde-defaulted:
  C3 `card_lines: [[u8; 3]; 4]`; C4 `split: u8`,
  `seamed: bool`, `access_refused: bool`,
  `worked: [bool; 2]`, `dropped: [bool; 2]`.
  `floor5_diagnosed` now judges chambers 3–4 (twelve
  checkable lines; a sound seam plus the referee's full
  walk).
- **`content.rs`** — `CARD_TOOLS` (a `CardTool` alias names
  the row shape): the four tools — the Length-Reader, the
  Splitter, the Keeper, the Lens — three lines each, three
  phrasings per line, exactly one checkable; the key of
  record is T0 [1,0,1], T1 [0,2,1], T2 [2,0,1], T3 [0,1,2]
  per the spec. Plus the two chambers' four-rung hint
  ladders (the fourth rung names each seam) and the
  dispatch arms that serve them.
- **`puzzles.rs`** — `card_line_checkable` /
  `card_lines_checkable` / `cards_all_checkable`; the beam
  itself as a real `[u32; 6]` = [2, 5, 1, 4, 2, 3] with
  `beam_sound_split` and `beam_halves` — a test asserts
  `size_of_val(&BEAM) == 24`, so the Plumb Line's fact the
  chamber states is one the suite proves (DS-13).
- **`game.rs` / `shell.rs`** — the card-line cycling
  presses; the beam's seam steppers (a new split unseams
  and spends the walk, per the spec), the saw, the
  over-reach the face refuses, the two-handed work, and
  the drops (a right-first drop refused with its reason);
  the per-chamber builders showing the rack's phrasings
  and the beam with its chalked seam.

Build notes: two borrow-checker fixes on the way (the seam
steppers now compose their message inside the work borrow
and speak after it closes — the F5Walk read-first shape).
Clippy's first pass caught four lints in the new code —
a stranded doc comment where the shell builders were
spliced (the hub's doc returns to its fn; the splicer's
anchor now names the doc line), the `CARD_TOOLS` type
complexity (the alias above), a manual range-contains,
and a needless range loop in a test — all fixed
curatively and re-gated; no lint was allowed or narrowed.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0 (after the four fixes above).
- `cargo test`: **153 passed / 0 failed** (was 145 at
  Step 2; +8: six puzzles — the key exact, the key scoring
  twelve, the found rack failing, one vapour line
  unmaking it, the beam's facts real, the ends not seams;
  one state judge test; one GUI playthrough of both
  chambers, including the refused right-first drops).
- Theodolite (cured route; bacon producer first,
  `.bacon-locations` present at session start): planted
  control on puzzles.rs read live E0308 diagnostics in
  the same session where the other four files read 0;
  clean session **0 on all five touched files** —
  verified zeros. puzzles.rs restored byte-identical
  after the plant (sha256 `5e5a61df…` both sides).
- Determinism: the Step 3 probe (the playthrough extended
  through the Contract Cards and the Split Beam) run
  twice; saves byte-identical, sha256
  `6ee78608c2f0db7a34220f48a45525ad5c1fee95a6f6438d00d6dd1bd09fb451`.
- Android lib under the pin: 1,348,158,304 bytes. Launch
  smoke under Xvfb: `cold_start_ms=1179`, no panics.

### Left open

- Chamber 5 and the Thing Behind the Veil land in Step 4;
  the driven whole-floor playthrough in Step 5.

## Step 4 — Chamber 5 (the UB Hunt) and the FOE (the Thing Behind the Veil)

Built Claude-direct by the Part A coordinator lane under
the completion spec (splice54.py + blocks/, anchors
asserted exactly once against the tree at 3017a97).
Evidence: `~/.local/share/pax/niflheim-floor5-step4-build-20261110/`
on primo.

### What landed

- **Chamber 5, the UB Hunt.** `content::STATIONS` — six
  stations (a `Station` struct: name, duty, fault,
  contracts, true contract) with three seeded faults at
  miniature size: the Twin Pens (two writers fused, true
  contract 2), the Second Reading (a room read after it
  was emptied, true 0), the Blank Ledger (a read of an
  unfilled blank, true 1); `FAULT_KINDS` names the three
  shapes. Work fields `lensed: [bool; 6]`,
  `verdicts: [u8; 6]`, `cards: [u8; 6]`, all
  serde-defaulted. Judges in `puzzles`:
  `station_verdict_right`, `hunt_complete` (every station
  lensed, every verdict true, every faulted card on its
  true contract). The lens press speaks the fault it
  finds; a faulted station's card button appears once its
  verdict stands at FAULT.
- **The FOE, the Thing Behind the Veil** (class #10, the
  Moving Floor). Full FOE surface in the house pattern:
  `Screen::Floor5Foe`, `El::DoorFoe5` on the Floor 5 hub
  (door drawn with its Standing/Exposed/Fallen tag),
  `ArmedFor::Foe5`, and every shared site wired (verbs,
  tab-bar match, seal_armed, layout_seal, examine,
  enumerate, hint branch, reading title/inscription/
  demonstration/hint). Diagnosis:
  `floor5_foe_exhibit` takes the trace's three sights IN
  ORDER — a live shared loan, a growth while it was out,
  the lens's verdict — and refuses out-of-order exhibits
  naming the missing prior sight. Gate:
  `floor5_foe_walk_case` runs the referee's five cases
  against `puzzles::BeamModel`, a real modelled beam
  (length, stated capacity, loans out, filled cells) —
  growth under a loan refused; access past the end
  refused; the drop order a true permutation; the empty
  element refusing access while growth beside it is
  allowed; overflow at capacity refused (DS-13: the
  cases are computed, not asserted). Repair: the beam's
  card (`FOE5_CARD`, key [1, 2, 1]) rewritten to its
  three checkable truths; the strike arms only on proof
  + gate + card, and the fall records the Ice Stone to
  the codex and the Veilwalker's Buffer to the pack.
  Miri becomes the standing instrument from this floor
  onward — content-level framing only, per the spec; no
  runtime seam was built. The FOE also forced the
  layout() tab-bar match and two stranded doc comments
  to be set right; all fixed curatively.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **160 passed / 0 failed** (was 153 at
  Step 3; +7: four puzzles — the station truth table
  exact, the hunt unmade by any single omission, the
  found card unrepaired, all five referee cases holding;
  two state — the hunt judged, the FOE's arc with the
  out-of-order refusal naming its missing sight; one GUI
  playthrough of the chamber and the FOE through both
  seals, ending at `floor5_complete()`).
- Theodolite (cured route; bacon producer running,
  `.bacon-locations` present at session start): planted
  control on puzzles.rs read live E0308 diagnostics in
  the same session where the other four files read 0;
  clean session **0 on all five touched files** —
  verified zeros. puzzles.rs restored byte-identical
  after the plant (sha256 `7e8bd3dd…` both sides). (The
  first attempt's harness omitted the plant append and
  read a clean file in "planted" mode; the re-run above
  is the evidence.)
- Determinism: the Step 4 probe (the playthrough extended
  through the UB Hunt and the Thing's fall, fixed date)
  run twice; saves byte-identical, sha256
  `632e975a65c66ca97d2524249b93030dd2555506fff114b41d5ec44834556383`.
- Android lib under the pin: 1,348,458,576 bytes. Launch
  smoke under Xvfb: `cold_start_ms=729`, no panics.

### Left open

- Step 5: the driven whole-floor playthrough (all five
  chambers plus the FOE from the title screen), the full
  battery, and the floor close-out.

## Step 5 — Integration (Floor 5 close-out)

Built Claude-direct by the Part A coordinator lane under
the completion spec. Evidence:
`~/.local/share/pax/niflheim-floor5-step5-build-20261110/`
on primo.

### What landed

- **The shipped whole-floor playthrough**
  (`floor5_whole_floor_played_on_the_gui`): one test
  driving the real press handlers from a fresh expedition
  through all five chambers — the Pointer Walks (with the
  cracked stone refused by name), the Two Tenants (one
  deliberate miss, stamped, annotated by its rule), the
  Contract Cards (all twelve lines cycled to checkable),
  the Split Beam (refused drops, the seam at three, the
  referee's rulings, halves dropped in order), the UB
  Hunt (six stations lensed, verdicts true, faulted cards
  rewritten) — each chamber through the two-step seal —
  and the Thing Behind the Veil (out-of-order exhibit
  refused, the trace in order, the referee's five cases,
  the card rewritten, the strike), closing on
  `floor5_complete()` with all five chambers repaired.
- **The earned-save round-trip.** The determinism probe's
  closing assertion now loads its own save back:
  `Expedition::load_json` returns the identical
  expedition, invariants recomputed. Getting there
  surfaced the probe's one fixture line — Floor 1's FOE
  marked Fallen without its census — which the loader
  rightly refused; the probe now earns that arc too
  (enumerate, exhibit the double-lent row, verify every
  uncopiable row, mark the gate, strike on the fixed
  date), so the byte-identical save is an earned
  five-floor state end to end.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **161 passed / 0 failed** (was 160 at
  Step 4; +1, the whole-floor playthrough).
- Theodolite (cured route; bacon producer running,
  `.bacon-locations` present at session start): planted
  control on puzzles.rs read live E0308 diagnostics in
  the same session where the other four files read 0;
  clean session **0 on all five shared files** — verified
  zeros. puzzles.rs restored byte-identical after the
  plant (sha256 `7e8bd3dd…` both sides).
- Determinism: the Step 5 probe run twice; saves
  byte-identical, sha256
  `b7c288f6733204b0e38cbc0eb1d65088e8c1e0457daeff60739ec9fe5921d321`.
- Android lib under the pin: 1,348,458,576 bytes. Launch
  smoke under Xvfb: `cold_start_ms=712`, no panics.
  `#![forbid(unsafe_code)]` intact (src/lib.rs:26).

### Floor 5 close-out

Floor 5 — The Veilworks (interior mutability's limits,
failure class #10) is **complete**: five chambers and the
Thing Behind the Veil, landed in five gated steps on top
of the DS-10 Step 2 commit — `7e8b767` (Step 2, re-gated
at commit time), `3017a97` (Step 3), `1111fde` (Step 4),
and this Step 5 commit, over Step 1's `167069d`. The
suite stands at **161/0** (139 at the floor's first
commit). Save format v5 throughout, per DS-2's one bump
at Step 1. `gate.rs` untouched, per DS-4. The floor's
design record is FLOOR5-BRIEF.md; its step evidence
lives in the five per-step evidence directories on primo
named in each section above.
