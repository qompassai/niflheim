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

## Step 1B — Chamber 1: Traits and impls, playable (2026-11-10, local-runs lane, D-1145)

Recorded by the lane lead from gate artifacts. Provenance, plainly:
this step was authored by the lane lead directly, in Floor 1's
established patterns — after the Versemend Contract 3 close-out
consumed the night's model-run hours, no phlow runs were dispatched
for 1B. The gates below are the same gates every step answers to.
Corpus-first (Owner's rule): the Lane D corpus
(~/workspace/restricted-remakes-20261108/games/) was searched before
building (menagerie / bestiary / roster / trait system / puzzle
room / admission / closure) — **the corpus had nothing for this
floor's systems**; its dossiers are remake RE for other titles.
The build follows FLOOR2-BRIEF.md's chamber spec and the 1A
metadata. Note: FULL-DESIGN.md, named in the brief as the design
of record, is present in neither the repo, the packet, nor the
workspace; the brief's Floor 2 section carries the complete
chamber spec and governed.

### What landed

- `src/content.rs`: the Anvil Call — TRICK_NAME / TRICK_CHARTER
  ("Forgehand"), CHARTER_CANDIDATES [FireWard, Forgehand, Swift]
  (the two worn sigils are the resemblance traps), the house and
  variant performance texts, TrickPerformer + TRICK_PERFORMERS
  (Ember keeps the house body; Soot performs its own variant),
  BLIND_RUN_PERFORMANCE (the variant text), and floor2_hints —
  Chamber 1's four-rung ladder; chambers 1–4 (index) carry the
  Sigilwright's placeholder line until their steps carve them.
- `src/puzzles.rs`: the pure judges — trick_charter_ok,
  blind_run_performer (resolves the run by behaviour text alone),
  identification_ok. Tests: validation (charter + blind run) and
  adversarial (worn sigils are not charters; the house body is not
  the runner; out-of-range identification never holds).
- `src/state.rs`: Floor2Work (charter choice/ok, watched, run
  choice/ok) on Floor2ChamberProgress, serde-defaulted so saves
  written before it load whole; floor2_file_proof (both slots
  re-judged on every filing, Floor 1's law; refusals name the
  reason — unwatched, empty slot, worn-sigil charter, wrong body),
  floor2_repair_chamber (facet at full standard), floor2_view_hint
  / floor2_hints_used / floor2_reset_chamber. Tests: diagnosis +
  repair with facet; unwatched filing refused; resemblance and
  wrong body refused; forged out-of-range choice refused without
  panic; rung-4 repair earns no facet; work round-trips and
  defaults on old saves.
- `src/game.rs`: Screen::Floor2Chamber; El::F2Watch / F2Charter /
  F2RunChoice; ArmedFor::Floor2Chamber with the two-step seal;
  verbs, readings (inscription / demonstration / hints resolve to
  the Menagerie texts), examine, reset, and hint-deeper all branch
  to the Floor 2 forms; the Floor2Hub's Chamber 1 door now enters
  the chamber (doors 2–5 keep the scaffolding narration).
- `src/shell.rs`: build_floor2_chamber in build_chamber's shape
  (forge block, rule stone, slot, seal, bottom bar) and the
  build_f2c1 panel — watch button, both labelled performances, the
  blind run, the three charter chips, the two naming chips; header
  title for the new screen.

### Gates (lane-run, primo)

- cargo fmt --check: clean.
- cargo clippy --all-targets -- -D warnings: clean.
- cargo test: **55 passed / 0 failed** (Floor 1's 44 intact, +4
  from 1A, +7 new: 2 puzzles, 5 state).
- theodolite (MCP, protocol 2026-07-28) over the five touched .rs
  files: 0 diagnostics, 0 errors. Transcript:
  ~/.local/share/pax/local-runs-20261109/logs/f2-1b-theodolite.jsonl
- Launch smoke (Xvfb :97, BEVY_ASSET_ROOT at the repo root):
  cold_start_ms=744, no panics, no missing-font errors, still
  running at the 15 s bound. A driven GUI playthrough of the
  chamber is Step 5's evidence, as on Floor 1's programme; the
  repo has no screenshot harness yet.

### Production axes (D-1151) and aspects, for this step

- Characters: the roster beasts authored in 1A serve as the
  chamber's performers (Ember, Soot); no new character work.
- Levels: Chamber 1 of 5 fully playable. Chambers 2–5, the
  Herald beat, and the FOE remain scaffolding — Steps 2–4 open.
- Objects: Chamber 1's facet path to the Lens Stone is live
  (facet on a full-standard repair); the floor's loot lands with
  the FOE step.
- Music: none — the design's platform note for this game is "No
  audio"; not an open axis.
- UI-UX: the chamber screen delivered in Floor 1's idiom (panel,
  proof slot, two-step seal, hint ladder, readings), on the shared
  layout/element machinery, so touch parity and the no-hover rule
  hold by construction.
- Aspects: mechanics — delivered in full for Chamber 1
  (identify-by-behaviour + charter judgement under the standing
  encounter loop). Animation — n/a (no animation system in the
  design). Economy — n/a (none in the design). Other systems —
  the save format extended compatibly (see above).

### Left open, honestly

- Steps 2 (Chambers 2–3), 3 (Chambers 4–5 + Herald), 4 (the FOE
  with the real cargo gate), and 5 (integration + playthrough
  evidence) are not started.
- Floor 2's doors are not yet sequentially gated the way Floor
  1's are; Chamber 1's door stands open and doors 2–5 arrive
  with their steps.
- No HTTP 500 was seen in this step (no model runs were made).

## Errata — FULL-DESIGN.md reconciliation (2026-11-10, completion programme)

The Step 1B report above records that FULL-DESIGN.md, the brief's
named design of record, "is present in neither the repo, the packet,
nor the workspace." That is corrected here: the document exists at
`~/workspace/goals/rust-all-the-way-down-to-the-kernel/files/FULL-DESIGN.md`
in the sandbox (the goal's files, outside the primo packet the lane
searched). It is now landed in this repo as `FULL-DESIGN.md`
(sha256 54bda4c4bc0a077e32d165cc5253b924c855b3bf1010d376be7d48b4847f39bd,
1144 lines, byte-identical to the sandbox copy), and its Floor 2
section (§4) has been diffed against the brief and the landed
Steps 1A/1B.

### What the diff found — agreement

The brief is a faithful compression of the design's Floor 2 spec.
Floor name, domain, keeper, theme, and learning objective match.
The five chambers match in name, order, and core mechanic
(chartered trick identified by behaviour; the Generic Stable's
bound-tightening with unjustified sigils refused with the reason;
the Two Kennels; the Wide Door registry with three placarded
barred beasts; Foreign Marks and the guild collar). The Herald
beat (#2, secondary) matches and is Step 3's, per the brief. The
FOE's seam, diagnosis, proof, and repair strike in the brief are
the design's: admission by resemblance; the audit plus the lit-
brazier calibration; the admission audit as proof; re-post the
fire-watch bound, reassign the mist-hound, refuse domain-less
admissions. The Lens Stone's name, the stairs relevance, and the
platform notes (grouped collapsible rosters, no hover-only
information) all match. The brief's two elaborations are
consistent with the design and stand: the three barred beasts'
reasons are named per Rust's object-safety rules, and the FOE
repair is expressed as real Rust checked by the bounded cargo
gate (Floor 1's standing pattern).

### What the diff found — conflicts in the landed 1A content, fixed here

The brief was faithful; the content landed under it was not, in
three places. All three are corrected in this errata commit, in
`src/content.rs`, following the design:

1. **The FOE was reframed as a beast.** The 1A texts described the
   Menagerie as "a beast with every sigil and none" that "crosses
   every post unchallenged" (FLOOR2_INTRO, FOE2_INSCRIPTION,
   FOE2_DEMONSTRATION). The design's FOE is the house itself as
   one mechanism: an admission desk that checks appearance where
   the charter demands a sigil. The three texts now carry the
   design's framing — the desk admitting by resemblance, the
   audit instruction, and the mist-hound reporting fog at the
   lit brazier.
2. **The loot was misnamed and misconceived.** PLUGIN_MENAGERIE
   held "The Sigilwright's Lens", a lens that reads sigils. The
   design's loot is **The Plugin Menagerie**, the two-door host
   that fields beasts by kind (Narrow Door) or by sigil (Wide
   Door). Name and text corrected. (The constant was not yet
   consumed anywhere — the loot lands with Step 4 — so nothing
   persisted the wrong string.)
3. **The Lens Stone's text drifted.** It read "Traits: a bound is
   an admission — say what a thing must do..." The design's
   codex entry is "**Bounds are admissions.**", and the
   design's consistency gate (§8.5) makes stone-text drift a
   defect; Floor 1's landed pattern (the Plank Stone's text is
   its principle sentence alone) agrees. Text corrected to
   "Bounds are admissions."

### Recorded for the later steps (no action in this commit)

- **The failure class was never named.** The design assigns
  Floor 2 the primary failure class **#6, The Cartographer's
  Blind Eye** (§3.2, §4) — a fine instrument used outside the
  domain it was made for; the resemblance desk is that class
  wearing this floor's clothes. Neither the brief nor the landed
  content names it. Step 4 must weave it explicitly, as Floor
  1's build names #12.
- **The FOE's cast is not in the 1A roster.** The design's audit
  turns on the ash-hounds (the fire-watch's legitimate kind) and
  the mist-hound admitted among them; the landed roster has
  neither (its watches are Ember/Cinder/Soot and
  Mistveil/Drizzle/Vapor). Step 4 should carry the FOE's audit
  as its own admission table rather than resizing BEASTS, so
  the 1A roster and the save shape built on it stay untouched;
  if Step 4 instead extends the roster, it must version the
  save consequence honestly.
- **Chamber doors are not yet sequentially gated** (already
  recorded in Step 1B's left-open list; the design's §5.2 makes
  chambers strictly ordered). The step that opens Chamber 2
  (Step 2) should gate Chamber 1's door on Chamber 1 repaired,
  and so on down the floor.

### Gates (this errata commit)

- FULL-DESIGN.md landed byte-identical (sha256 above).
- Six exact-match content replacements, each verified to occur
  exactly once before substitution.
- cargo gates and counts recorded in the programme report; the
  test count must hold at 55/0 (the changed strings are display
  content; no test asserts them).

## Step 2 — Chambers 2 and 3 (2026-11-10, completion programme)

Recorded by the coordinator of the completion programme. Honest
provenance: the chamber code below was authored by the prior lane,
which ended without filing its final report; Pax verified the
dirty tree from disk (cargo test 64 passed / 0 failed) before this
programme took over. The coordinator audited the uncommitted diff
against the brief and the errata's forward notes, closed one gap
the audit found (below), ran every gate, and made the commit.

### What landed

- `src/content.rs`: Chamber 2, the Generic Stable — STABLE_SIGILS
  (FireWard / MistSense / Swift / KeenNose), POSTED_BOUND (every
  sigil demanded), STABLE_SERVICES (the Hearth Post, the Letter
  Run, the Stores Audit, each with its work stated and the sigils
  that work requires). Chamber 3, the Two Kennels — KENNELS (the
  Hearth Kennel chooses its kind; the Open Kennel's customer
  chooses), ERRANDS, and DELIVERIES, the records of both houses,
  holding exactly one contorted run: the River Parcel from the
  Hearth Kennel, whose ash-hound cannot swim. Hint ladders for
  both chambers, carved in the house form.
- `src/puzzles.rs`: the pure judges — bound_unjustified /
  bound_missing / bound_exact for the stable (a bound is exact
  when it demands exactly what the work names; stray bits beyond
  the four sigils make any bound inexact), and for the kennels
  contorted_errand / errand_choice_ok / kennel_choice_ok, judged
  from the delivery records alone. Four tests: two validation,
  two adversarial.
- `src/state.rs`: Floor2Work gains the chambers' fields, every
  one serde-defaulted so Step 1B saves load whole; floor2_file_proof
  judges Chamber 2's three bounds and Chamber 3's two finding
  slots on every filing, refusals naming the unjustified or
  missing sigil, the unrun deliveries, or the wrong body of the
  finding, in the floor's own words. Five tests: diagnosis +
  repair with facet for both chambers (validation), the posted
  bound and wrong findings refused with reasons (adversarial),
  and the work round-trip / old-save defaults (validation).
- `src/game.rs`: the chambers' elements and handlers (per-service
  sigil toggles, the run-deliveries verb, the two finding chips),
  and — discharging the errata's forward note — the floor's doors
  are now sequentially gated: Chamber N's door (N = 2, 3) stands
  barred until Chamber N−1 is REPAIRED, the refusal naming both
  chambers; doors 4–5 keep the scaffolding narration until
  Step 3.
- `src/shell.rs`: build_f2c2 (each service's posted work beside
  its bound toggles, the posted bound's story told in text) and
  build_f2c3 (both kennels' delivery records in full text, then
  the finding's chips). No information lives only in hover text.

### The audit gap, closed in this step

The load invariants recomputed Floor 1 chambers' proofs
(`proof_complete`) but never Floor 2's: a forged save could stand
a Floor 2 chamber Repaired — facet and all — on empty work and
load clean. The fix, authored by the local model stack (phlow,
two runs; the first run's fragments carried a shadowed binding
and wrong API shapes and were rejected at review, the second
run's were spliced verbatim) and spliced by the coordinator:
`floor2_proof_complete` recomputes each landed chamber's proof
from its work, chambers not yet landed never hold a proof, and
`validate_invariants` now refuses any Floor 2 chamber standing
beyond Unworked without a recomputed proof, Floor 1's law on
Floor 2's side. Two tests: repaired chambers survive a load
(validation); forged Repaired/Diagnosed states on empty work
are refused as corrupt (adversarial).

### Gates (coordinator-run, primo)

- cargo fmt --check: clean (rc 0)
- cargo clippy --all-targets -- -D warnings: clean (rc 0)
- cargo test: **66 passed / 0 failed** (Floor 1's 44 and Steps 1A/1B's 11 intact;
  +4 puzzles, +5 state from the chambers, +2 from the audit fix).
- theodolite (MCP, protocol 2026-07-28) over the five touched
  .rs files: 0 diagnostics, 0 errors, 0 warnings. Transcript: ~/.local/share/pax/niflheim-step2-fix-20261110/evidence/theodolite-step2.jsonl
- Toolchain note, honestly: this session's first gate pass ran
  under primo's Arch system rustc 1.99.1 (primo-ssh's PATH lacks
  ~/.cargo/bin, and the system cargo ignores rust-toolchain.toml).
  Every gate was re-run with ~/.cargo/bin first on PATH so the
  pin governs (rustc 1.99.0, b940084d7): identical results —
  fmt clean, clippy clean, 66/0, the same save hash. The Android
  build exists only under the pin (the system sysroot carries
  no aarch64-linux-android std; its attempt failed E0463 and is
  recorded in the lane evidence, not hidden).
- Determinism: a scripted state-level playthrough solving
  Floor 2 Chambers 1–3 through the state API, run twice, produced
  byte-identical saves (sha256 a3370b31f97b7446e053da248b21a1f6a8f42ed06809e2209ce46af3525e5286 both runs). The driven GUI
  playthrough of the floor remains Step 5's evidence, as on
  Floor 1's programme.
- Android: cargo build --target aarch64-linux-android --lib:
  Finished in 1m 16s under the pinned toolchain; libniflheim.so 1,343,771,952 B (debug profile)
- Launch smoke: Xvfb :97, BEVY_ASSET_ROOT at the repo root: cold_start_ms=1089, no panics, still running at the 20 s bound (Vulkan loader validation noise under Xvfb only)

### Production axes (D-1151) and aspects, for this step

- Characters: the stable's and kennels' casts are the 1A roster
  and the authored kennel kinds; no new character work.
- Levels: Chambers 2 and 3 of 5 fully playable; the floor now
  teaches in strict order end to end for the landed chambers.
- Objects: no new loot; the facet paths for Chambers 2 and 3 are
  live.
- Music: none — the design's platform note is "No audio".
- UI-UX: both chambers delivered in Floor 1's idiom on the shared
  machinery; the stable's long postings and the kennels' records
  are full text on the panel, touch parity by construction.
- Aspects: mechanics delivered in full for both chambers (bound
  revision judged sigil-by-sigil; associated-type finding judged
  from records). Animation/economy: none in the design.

### Left open, honestly

- Steps 3 (Chambers 4–5 + Herald), 4 (the FOE with the real
  cargo gate; failure class #6 to be named; the FOE's own
  admission table), and 5 (integration + GUI playthrough
  evidence) remain.
- One residual from the audit: the stable panel's toggles read
  the mask live, so a mid-revision bound is visible state, not
  yet judged — by design; judgement happens at filing, as on
  Floor 1.

## Step 3 — Chambers 4 and 5 + the Herald (coordinator lane 3)

Chamber 4 (Trait objects and the Wide Door, failure class
"Unsafe Detour"), Chamber 5 (Coherence and the orphan rule,
failure class "Orphan's Claim"), and the Herald's secondary beat
are built, in the landed Steps' idiom throughout.

- content.rs: the Wide Door's service text and the registry's
  book — six beasts with kind and charter prose, three passing
  the Dawn Round in their own bodies (Ember, Drizzle, Vapor) and
  three barred, each with its bar named (Brindle the Mule —
  Returns Self; Tessell the Turtle — Generic method; The
  Unmounted Bell — No self). The bar reasons and their placard
  labels are one list, so judge, panel, and refusal read the same
  words. Chamber 5: Auzi of the Silt House and the TideLedger
  sigil (FOREIGN_MARK_STORY, design prose verbatim), the three
  joining routes with their authored refusals (direct stamp and
  blanket joining illegal; the guild collar of the player's own
  forging lawful), and the staged collision with its three
  readings, only the one-claim reading lawful. The Herald's three
  sealed letters, the Frost Post letter the false one (the
  floor's first wrong room), with HERALD_FILED and
  HERALD_WRONG_ROOM narrations. Hint ladders for both chambers,
  four rungs each, the fourth naming the working.
- puzzles.rs: four pure judges — `bar_label_ok` (one beast's bar
  against a named label), `registry_bars_ok` (every barred
  beast's bar, keyed in book order; a passing beast's bar is
  never read), `collar_route_ok`, `collision_reading_ok` — each
  total over forged indices. Four tests (validation/adversarial
  pairs): the true bars hold and a passing beast's bar is never
  consulted; swapped bars and forged labels never hold; exactly
  one route and one reading are lawful, forged indices refused.
- state.rs: Floor2Work gains drove_round, bar_choices (indexed
  by the barred beasts in book order), bars_ok, route_choice,
  route_ok, collision_choice, collision_ok — all serde-defaulted.
  Floor2Progress gains herald_filed (serde-defaulted; new saves
  carry one flag per letter; the load invariant accepts a short
  record as letters unfiled and refuses a wrong-sized one).
  floor2_file_proof arms 3 and 4 judge in the house refusal
  order: Chamber 4 asks the driven round before the naming, names
  the first unnamed barred beast, and answers a mislabelled bar
  with the placard's own wording; Chamber 5 asks the route before
  the reading, answers an unlawful route with the Sigilwright's
  authored refusal verbatim, and refuses a misread collision
  without blaming either keeper. floor2_proof_complete arms 3
  and 4 recompute both proofs, so the load invariants now guard
  all five chambers. floor2_herald_act files a letter once and
  narrates the outcome — the false letter wastes the miniature
  expedition, as designed. Nine tests: per-chamber
  diagnose+repair (validation) and refusal batteries
  (adversarial: undriven filing, unnamed beast named, swapped
  bars, out-of-range forgeries, both unlawful routes with their
  authored refusals, first-forged reading); the Herald files, the
  false letter wastes the ride, repeats and nonexistent letters
  refused, pre-beat saves load unfiled; the Step 3 work
  round-trips and defaults on old saves; all five chambers
  repaired survive a save/load and two identical playthroughs
  produce byte-identical saves; forged Step 3 repairs on empty
  work refused as corrupt.
- game.rs: El gains F2DriveRound, F2BarLabel, F2Route,
  F2Reading, F2PanelTab (Chamber 5's panel sub-view, mirroring
  Chamber 1's), and F2HeraldAct. The Floor2Door gate widens from
  chambers 0–2 to 0–4; door and examine narrations land for both
  chambers; hub actuation gains the Herald's buttons; panel
  actuation gains the drive button, the bar-naming chips, the
  panel tabs, the route buttons, and the reading chips. Layout
  geometry lives in geom, as before.
- shell.rs: build_f2c4 (the registry book before the drive; the
  round's outcome and the three placards with their naming chips
  after) and build_f2c5 (two views — the joining: story, route
  buttons with their detail lines; the collision: the staged
  text, the three readings, and the reading chips). The hub
  renders the Herald's strip room-side: the letters' texts with
  an act button each, reading Filed once acted. Desk labels:
  Chamber 4 "THE REGISTRY", Chamber 5 "FOREIGN MARKS".
- lib.rs: the crate doc now records all five chambers playable
  with the Herald at the desk; the FOE and the gate remain
  Steps 4–5.

### Gates (coordinator-run, primo, pinned toolchain throughout)

- cargo fmt: applied; cargo fmt --check clean afterwards.
- cargo clippy --all-targets -- -D warnings: clean (rc 0).
- cargo test: **79 passed / 0 failed** (66 at Step 2's close
  intact; +4 puzzles, +9 state from this step).
- theodolite (MCP, protocol 2026-07-28) over the six touched
  .rs files: 0 diagnostics each, session quiescent (no
  possibly_incomplete results read as proof). Log:
  ~/.local/share/pax/niflheim-step3-build-20261110/theodolite-step3.log
- Determinism: a scripted state-level playthrough of the Step 3
  surface through the state API (Chambers 4 and 5 solved and
  repaired, all three Herald letters acted on), run twice in
  separate processes, produced byte-identical saves (sha256
  aae272f3c5ab25896091aba4793633c47d27dc321068d2692e97103dddc6adab
  both runs). The in-suite playthrough test covers all five
  chambers the same way. The driven GUI playthrough of the floor
  remains Step 5's evidence, as on Floor 1's programme.
- Android: cargo build --target aarch64-linux-android --lib:
  Finished in 19.7 s under the pinned toolchain (incremental over
  Step 2's dependency build); libniflheim.so 1,344,168,864 B
  (debug profile).
- Launch smoke: Xvfb :97, BEVY_ASSET_ROOT at the repo root:
  cold_start_ms=763, no panics, still running at the 20 s bound.

### Production axes (D-1151) and aspects, for this step

- Characters: no new character work; Auzi and the registry's six
  are roster prose, the floor-2 roster as designed.
- Levels: Chambers 4 and 5 of 5 fully playable — the floor's
  chambers now teach in strict order end to end; only the FOE
  and the gate remain on the floor.
- Objects: no new loot; the facet paths for Chambers 4 and 5
  are live.
- Music: none — the design's platform note is "No audio".
- UI-UX: both chambers and the Herald strip delivered in Floor
  1's idiom on the shared machinery; the registry book and the
  collision's readings are full text on the panel, touch parity
  by construction. Chamber 5's two panel views mirror Chamber
  1's tabbed panel.
- Aspects: mechanics delivered in full for both chambers (the
  bar-naming judged beast by beast; the joining judged by route
  and the collision by its reading) and for the Herald (filing
  free and final; the false letter's cost is the wasted ride,
  exactly the design's first wrong room). Animation/economy:
  none in the design.

### Left open, honestly

- Steps 4 (the FOE with the real cargo gate; failure class #6,
  The Cartographer's Blind Eye, to be named in the fiction; the
  FOE's own admission table, never a BEASTS resize) and 5
  (integration + GUI playthrough evidence) remain.
- The Herald's narration returns through the status line, as
  Floor 1's secondary beats do; no dedicated reading overlay
  was built for it — the letters stand in full on the hub.
