# FLOOR 4 STEP REPORT — The Foundation Galleries (in-tree continuity record)

Coordinator-authored (blocks + splicer) on branch
`pax/floor2-build-20261109`, per FLOOR4-BRIEF.md. Design of
record: FULL-DESIGN.md Floor 4 + §5.2/§5.4/§5.6.

## Step 1 — Scaffolding

### What landed

- **`core.rs`** — `SAVE_FORMAT_VERSION` 3 → 4.
- **`state.rs`** — `Floor4Progress` (five
  `Floor4ChamberProgress` + `Floor4FoeProgress` +
  `foe_hints_used`), `FoundationWork` (the `watched` flag;
  the chambers' fields land with their steps, each
  serde-defaulted), the `Expedition.floor4` field wired
  through `SaveFile` (serde-defaulted, so a version-3 save
  loads with a fresh Floor 4), `save_json`, and `load_json`.
  Accessors in the house shape: `floor4_diagnosed` (no
  chamber proves yet), `floor4_file_proof`,
  `floor4_repair_chamber` (the facet text names the Brass
  Eye Stone), hint rung recorders, `floor4_chambers_repaired`,
  `floor4_complete`. `floor4_reachable()` = the Stratum
  I → II stairs stand fully lit (§5.6 over the §5.2
  prerequisite): FOE 1 fallen, FOE 2 or FOE 3 fallen, every
  Stratum I lantern keeper-lit — exactly `stairs_clauses()`
  all lit. `validate_invariants` holds five Floor 4
  chambers and recomputes their proofs like its elders'.
  Fixtures: the future-version forge now rewrites
  `"version":4`; both legacy-version tests re-save as 4;
  a new version-3 test proves a v3 save loads with a fresh
  Floor 4 and re-saves as 4.
- **`content.rs`** — `FLOOR4_NAME` / `FLOOR4_DOMAIN` /
  `FLOOR4_KEEPER` (the Plumbmason), `FLOOR4_INTRO`, and
  `FOUNDATION_CHAMBERS`: all five ChamberMeta fully
  authored (inscriptions: The Committed Slate, The Counted
  Voids, The Free Mark, The Promised Offsets, The Mason's
  Head).
- **`game.rs`** — `Screen::Floor4Hub`; `El::DoorFloor4` /
  `Floor4Door(u8)`; `geom::door_floor4()` =
  (640, 150, 124, 124) on **Floor 1's hub** — the
  upper-right field below the chamber row's end, clear of
  the panel: the spot Floor 3's door proved on Floor 2's
  hub. The hub layout pushes the door; the Floor4Hub layout
  carries the camp stone and five chamber doors. The door's
  press is gated on `floor4_reachable()`; barred, it speaks
  the first unmet stairs clause by its inscription text
  (§5.6: never a locked-door buzz). Chamber doors speak the
  Plumbmason's not-open-yet line until Step 2. verbs,
  token visibility, cell blocking, Examine, and both
  reading arms know the hub.
- **`shell.rs`** — `build_floor4_hub` (five doors with
  state tags, the camp stone, the panel with the chambers
  list and the FOE's standing), the build dispatch, the
  header title, and the map's Floor 4 square (BARRED /
  UNCHARTED / CHARTING / FOE FALLEN, like its neighbours).

Build notes: three anchors that carried a closing brace
put the new field after its struct (SaveFile, Expedition,
and the load construction) — the parse error named each;
all three moved inside. Clippy refused two placeholder
`match { _ => … }` shapes in the scaffolding judges; both
are plain statements now, and the judges grow their real
arms in Steps 2–4.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **122 passed / 0 failed** (was 119; +3:
  state 2 — the stairs-gated reachability, the version-3
  load; game 1 — the door answering the pointer exactly
  when the stairs stand lit).
  Log: `~/.local/share/pax/niflheim-floor4-step1-build-20261110/cargo-test.log`.
- Theodolite: **0 diagnostics** on all six touched files.
- Determinism: the Floor 3 Step 4 probe re-run on the new
  tree; saves byte-identical, sha256
  `80786536fa9829e64eff5ec4052bbc6e52840c70322301e3e230e1e15e60dbbd`
  (the hash moved from Floor 3's, as it must: the save
  gained the version bump and the Floor 4 record).
- Android lib under the pin: 1,346,148,760 bytes. Launch
  smoke under Xvfb: `cold_start_ms=895`, no panics.

### Left open

- Chambers 1–2 land in Step 2 (the Predict-and-Measure
  bench, the smallest wall); Chambers 3–4 in Step 3;
  Chamber 5 and the Crooked Foundation in Step 4; the
  driven whole-floor playthrough in Step 5.
- The Stratum II → III stairs recitation (§5.6) arrives
  with the first Stratum III door (Floor 9's scaffolding),
  as the I → II recitation arrived with Floor 3.

## Step 2 — Chambers 1–2: the bench and the smallest wall

### What landed

- **`content.rs`** — Chamber intros and Examine texts for
  all five galleries; the bench's furniture: `BENCH_STONES`
  (six stones), `bench_stone_facts(index)` — the floor's
  doctrine in one function: (size, alignment) taken from
  the **real types** (`u8`, `(u8, u32)`, `&str`,
  `Option<u8>`, `Option<&u64>`, `Vec<u8>`) with
  `size_of`/`align_of` at runtime, so the bench can never
  drift from the truth; `BENCH_CAUSES` / `BENCH_TRUE_CAUSE`
  (each stone's one true surprise-cause among three);
  `WALL_STONES` (five authored (size, align) pairs); both
  charters; the hint ladders (Chambers 1–2 real, the rest
  on the gallery's general ladder until their steps).
- **`puzzles.rs`** — `bench_prediction_ok` /
  `bench_missed`; `wall_layout` — the descent's one layout
  algorithm (each offset aligned up, the span rounded to
  the widest alignment; returns span and void bytes);
  `wall_minimum` through the same algorithm over the
  falling-alignment seating. The unit tests assert the
  bench facts outright [(1,1), (8,4), (16,8), (2,1), (8,8),
  (24,8)], the bench order's wall at (24, 8 voids), and —
  the honest part — brute-force all 120 seatings to prove
  no seating beats the claimed minimum of 16.
- **`state.rs`** — `FoundationWork` gains the bench slate
  (`pred_size`/`pred_align`/`measured`/`annotate`) and
  `wall_order`. `floor4_diagnosed` judges: Chamber 1 —
  watched, every stone measured, every surprise carrying
  its true annotation; Chamber 2 — watched, all five
  seated at the algorithm's minimum. Proof texts per
  chamber; `floor4_reset_chamber` in the house shape.
- **`game.rs`** — `Screen::Floor4Chamber` and the whole
  chamber plumbing (verbs, Prove/Hint branches, the seal
  and its confirm, the reset arm — Floor 4's chambers
  reset properly), elements `F4Watch`, `F4Stone`,
  `F4SizeUp/Down`, `F4AlignUp/Down`, `F4DropLine`,
  `F4Annotate`, `F4Seat`, `F4WallReset`; the ephemeral
  `f4_stone` slate selection; the chamber doors navigate.
  The steppers are bounded (§ platform notes): size walks
  1..=64, alignment walks 1, 2, 4, 8, 16. The line refuses
  to drop on a blank slate.
- **`shell.rs`** — `build_floor4_chamber` with the bench
  panel (each stone's slate/line state under its button,
  the selected stone's detail in the panel, steppers and
  buttons drawn from state) and the wall panel (seatings,
  the bench order's tally vs the player's, "the least
  span" when it stands found). Quantities render large
  and tabular (§7.3).

Observed while mirroring, **not** fixed here (pre-existing,
Floor 3's surface): Floor 3's chambers lay out a
ResetChamber button whose press falls through to the
match's catch-all — `floor3_reset_chamber` does not exist.
Floor 4's chambers reset properly; the Floor 3 gap is
recorded for Pax's word.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **126 passed / 0 failed** (was 122; +4:
  puzzles 2 — the real-type facts, the brute-forced
  minimum; state 1 — both judges with their refusals;
  game 1 — both chambers played on the GUI surface, the
  surprises annotated through the steppers).
  Log: `~/.local/share/pax/niflheim-floor4-step2-build-20261110/cargo-test.log`.
- Theodolite: **0 diagnostics** on all five touched files.
- Determinism: the probe extended through both chambers;
  saves byte-identical, sha256
  `60261eff8786d71d956ac5e36b9aaf3f3abfe239d957d0c3dfaee13c9b10ae39`.
- Android lib under the pin: 1,346,617,352 bytes. Launch
  smoke under Xvfb: `cold_start_ms=910`, no panics.

### Left open

- Chambers 3–4 land in Step 3 (the Seal Stones, the
  treaty wall); Chamber 5 and the Crooked Foundation in
  Step 4; the driven whole-floor playthrough in Step 5.
