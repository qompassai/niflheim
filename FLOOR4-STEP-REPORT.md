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
