# Floor 2 — Step Reports

## Step 1A — Scaffolding (2026-11-09, local-model lane, D-1145)

Recorded by the lane lead from gate artifacts; the code was authored
by the local model stack (phlow planner gpt-oss-20b / coder
devstral-24b / reviewer granite-4.2-8b via rose-serve) through the
editor bridge, in bounded region files the lane spliced back between
unique anchors (the bridge offers whole-file read/write only and the
served context is 16384 tokens, so the three large files were edited
as anchored regions; see the lane report for the full account).

### What landed

- `src/core.rs`: SAVE_FORMAT_VERSION 1 → 2.
- `src/content.rs`: the Floor 2 section — FLOOR2_NAME / DOMAIN /
  KEEPER / INTRO, LENS_STONE + text, PLUGIN_MENAGERIE (the
  Sigilwright's Lens) + text, MENAGERIE_CHAMBERS [ChamberMeta; 5]
  (Traits and impls / Generics and bounds / Associated types / Trait
  objects and the Wide Door / Coherence), Beast + ROSTER_GROUPS
  (The Fire Watch, The Mist Watch) + BEASTS [;6], AdmissionService +
  ADMISSION_SERVICES [;3] (Hearth Post [FireWard], Mist Post
  [MistSense], Valley Post [Swift]), FOE2_NAME / INSCRIPTION /
  DEMONSTRATION.
- `src/state.rs`: Floor2ChamberProgress, Floor2FoeProgress,
  Floor2Progress (::new sizes from the content tables; Default via
  new), `Expedition.floor2`, `SaveFile.floor2` with
  `#[serde(default)]` (v1 saves load with a fresh Floor 2),
  floor2_reachable / floor2_chambers_repaired / floor2_complete,
  stairs clause 2 now reads Floor 2's FOE, validate_invariants
  covers the Floor 2 lengths and the facet rule. Tests: the
  future-version literal moved to version 2; four new tests
  (round-trip, v1 compat, reachable arc, forged-length refusal).
- `src/game.rs`: Screen::Floor2Hub; El::DoorFloor2 / Floor2Door /
  RosterToggle; geom::door_floor2 + roster_group; token/cell/verbs
  cover Floor2Hub; toggle_roster_group; DoorFloor2 refuses until
  Floor 1's FOE falls; Floor2Door names the later step; layout
  header tabs + Hub door + Floor2Hub elements; FloorIntro reading
  resolves to Floor 2's texts on Floor2Hub.
- `src/shell.rs`: build_floor2_hub (doors, camp stone, roster board
  with collapsible groups and per-beast admission words, chamber
  progress with marks and words, FOE row, stairs line), the Floor 2
  door on Floor 1's hub (OPEN / BARRED), header title, the Ferronia
  chart's Floor 2 square carries a real status (filled only when
  FOE 2 has fallen), the codex reads the Lens Stone and lists
  Floor 2 facets.
- `src/lib.rs`: crate doc records the scaffolding.

### Gates (lane-run, primo)

- cargo fmt --check: clean.
- cargo clippy --all-targets -- -D warnings: clean.
- cargo test: **48 passed / 0 failed** (Floor 1's 44 intact, +4).
- theodolite (MCP, protocol 2026-07-28) over the six touched .rs
  files: 0 diagnostics, 0 errors. Transcript:
  ~/.local/share/pax/local-runs-20261109/logs/f2-1a-theodolite-final.jsonl
- phlow's named theodolite check ran inside every run (rc 0).

### Honest notes

- Every phlow run's report status is "failed" for one reason: the
  reviewer role never returned a parseable structured JSON verdict
  ("Reviewer must return a structured JSON verdict" — the D-1076
  class). Coder completions were verified by the lane's own gates
  above; no run is claimed as phlow-verified.
- Host (lane-lead) corrections to model output, all at gate review:
  content section doc-comment/prose polish; one line appended after
  a mis-stated instruction (load_json wiring); a duplicated tuple
  close from a region-boundary spec error; build_floor2_hub field
  fixes (meta.state / beast.index / meta.facet do not exist) and
  house-style alignment of its door and camp-stone drawing.
- No HTTP 500 was seen in any run. One HTTP 400 class was root-caused
  during the lane: requests over the served 16384-token context are
  rejected (exceed_context_size_error); the bounded region pattern
  is the workaround.
