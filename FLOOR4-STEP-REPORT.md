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

## Step 3 — Chambers 3–4: the Seal Stones and the treaty wall

**Method note (D-1189, Owner 2026-11-10):** from the next
rung boundary the completion programme is Claude-direct
throughout; no further phlow-stack dispatches. Floor 4
Steps 1–3 were in fact also authored directly by the
coordinator (blocks + splicer, recorded here), so the
authorship line is unbroken; the formal change applies
from Step 4.

### What landed

- **`content.rs`** — `CRESTS` and `CREST_CANDIDATES`, and
  `crest_candidate_facts(crest, candidate)`: prices from
  the real types — the tagged pair, `Option<u8>`,
  `Option<NonZeroU8>`; the flagged word tuple,
  `Option<&str>`, the hand-packed triple; the wide pair,
  `Option<u64>`, `Option<NonZeroU64>`. `TREATY_STONES`
  (four treaty stones + the free-cut twin) and
  `TREATY_PROMISES` [0, 3, 8, 10]; the two charters; real
  hint ladders for both chambers (the treaty ladder names
  failure class #14 in miniature at its seam).
- **`puzzles.rs`** — `crest_best` (argmin over measured
  sizes) and `crest_choice_ok`; `c_offsets`, the treaty
  accumulation returning the offsets themselves. The unit
  test asserts every candidate's measured facts outright,
  the niche winner per crest [2, 1, 2], and that the
  book's promises differ from the line's offsets in
  exactly one seat — the last, by exactly the six bytes of
  forgotten padding.
- **`state.rs`** — `FoundationWork` gains the crest cuts
  and their proofs, the treaty seating and its proof, and
  `final_promise`, which starts at the book's naive figure
  (hand-written `Default` + serde field default, so fresh
  and migrated records agree). Judges: Chamber 3 — all
  crests proven at a smallest cut; Chamber 4 — the treaty
  order seated, the line dropped, the promise amended to
  the true final offset. Both chambers refuse the free
  twin and the unamended book in-suite.
- **`game.rs`** — elements `F4Crest`, `F4CrestProve`,
  `F4TreatySeat`, `F4TreatyReset`, `F4TreatyProve`,
  `F4PromiseUp/Down`, all bounded (the promise walks
  0..=32); choosing a new cut spends the old proof; the
  line refuses a short wall and stops at the free stone
  with the reason spoken; the prove line reads the wall
  seat by seat, naming the one seat where book and line
  disagree.
- **`shell.rs`** — `build_f4c3` (cuts and their measured
  prices) and `build_f4c4` (the tray with each stone's
  seat and promise, the amendment steppers, the line's
  reading against the standing promise).

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **129 passed / 0 failed** (was 126; +3:
  puzzles 1 — facts, winners, and the one-careless-promise
  audit; state 1 — both judges with their refusals; game 1
  — both chambers played on the GUI surface).
  Log: `~/.local/share/pax/niflheim-floor4-step3-build-20261110/cargo-test.log`.
- Theodolite: **0 diagnostics** on all five touched files.
- Determinism: the probe extended through both chambers;
  saves byte-identical, sha256
  `92f009fa2be28adf5cd5b1bd71a1f057fcb58797ffc0e6776e47e8f665ba61b4`.
- Android lib under the pin: 1,346,849,760 bytes. Launch
  smoke under Xvfb: `cold_start_ms=1415`, no panics.

### Left open

- Chamber 5 and the Crooked Foundation land in Step 4;
  the driven whole-floor playthrough in Step 5. A small
  separately-gated fix for Floor 3's inert chamber reset
  (flagged in Step 2's section) lands before Step 4 per
  D-1189.

## Step 4 — Chamber 5 and the Crooked Foundation

**Method note (D-1189):** authored Claude-direct by the
coordinator (the programme's fifth lane), blocks + splicers,
no phlow-stack dispatch — the first rung under the formal
D-1189 arrangement, as this report's Step 3 section foresaw.

### What landed

- **`content.rs`** — `HOLLOW_CHARTER` and a real hint ladder
  for Chamber 5 (the general ladder no longer serves it);
  `HOLLOW_STONES` (7 with 2.5; 1,000,000 with 0.75 — cross-read,
  the number's bits are next to nothing and the ratio's bits
  are a count in the millions). The FOE furniture:
  `FOE4_NAME` / `FOE4_INTRO` / `FOE4_INSCRIPTION` (failure
  class #8, **the Astronomer's Brass Eye**, named in the
  fiction) / `FOE4_DEMONSTRATION`; `floor4_foe_hints`.
  `DIMENSION_BOOK` (five load-bearing entries) with
  `dimension_book_true` — `size_of` of the real stones
  ((u32, u32), &[u8], Vec<u16>, (u8, u32), Option<&u64>) — and
  `SHADOW_OFFSET` 6: exactly entries 1, 3, and 4 are false,
  each by exactly the offset, and identifiable only by
  measuring. `RECUT_FOREIGN` (the spec's four + the
  shadow-cut beam; stencils [2, 4, 8, 1, 14]) and `RECUT_FREE`
  (wedge, pier, slab, rod — tray span 24, budget 16).
  `BRASS_EYE_STONE` ("Distinct quantities get distinct
  types.") and `PLUMB_LINE` (wielded on Floors 5, 9, 10, 13,
  14; from this fall, prose about sizes is no longer evidence).
- **`puzzles.rs`** — `book_disagrees`; `hollow_bits` /
  `hollow_as_ratio`: honest `to_bits`/`from_bits`
  reinterpretation, never a union read
  (`#![forbid(unsafe_code)]` untouched). No new layout
  algorithm: the re-cut prices through the existing
  `c_offsets` / `wall_layout` / `wall_minimum`.
- **`state.rs`** — `FoundationWork` gains `fillings`,
  `tags`, `blank_cells`, `blank_used`, `referee_catches`
  (all serde-defaulted). Chamber 5's judge: watched, both
  stones filled with tag == filling, the blank full and
  used whole; the referee's catches stay on the record but
  are not the judge's business. `Floor4FoeProgress` gains
  `book_taken`, `measured`, `exhibited`, `foreign_order`,
  `foreign_proven`, `free_order`, `free_proven` (all
  serde-defaulted), and the FOE accessors
  (`floor4_foe_take_book` / `measure` / `exhibit` /
  `proof_complete` / `prove_foreign` / `prove_free` /
  `arm_strike` / `confirm_strike`, plus hint recorders).
  `prove_foreign` refuses the shadow-cut stone by its
  stencil before order is judged; `gate_passed` = both sets
  verified by the line. This FOE carries no cargo gate —
  `gate.rs` untouched.
- **`game.rs`** — `Screen::Floor4Foe`, `ArmedFor::Foe4`,
  `El::DoorFoe4` and fifteen new elements. Chamber 5's
  presses: filling and tag cycle separately (the tag lives
  in the mason's head); Read speaks the cavity as the tag
  claims (`ratio_text` keeps nonsense readable as nonsense);
  a Use on a part-filled blank is refused and
  `referee_catches` grows — Miri is named in the fiction as
  the syllabus's referee. The FOE's presses: take, measure,
  exhibit; seating a stone spends its set's proof and the
  gate, as the treaty's seating spent its proof. The strike
  rides the shared two-step seal; inscription, demonstration,
  and hint readings wired; the hub gains the FOE door (the
  Release door's proven ground, 600·292); all panel controls
  stay clear of the seal block (x < 780).
- **`shell.rs`** — `build_f4c5` (fillings, tags, the live
  readings, the blank's cells, the referee's count) and
  `build_floor4_foe` (the book's rows with measured truths
  and exhibit buttons, both trays under the line, the free
  set's budget computed from `wall_minimum` on screen, the
  re-survey and the gate spoken in words); the hub's FOE
  door with its state tag; header "Floor 4 · The Crooked
  Foundation".

### Defect found by the gates, fixed curatively

- The first clippy run failed on exhaustiveness:
  `layout()`'s tab-bar match — a second screen list, away
  from the layout arms — had no `Floor4Foe`. The arm was
  added; the tab bar renders on the Foundation's screen as
  on the Release's. Recorded because it is the rung's
  lesson: a new `Screen` variant has more match sites than
  the obvious ones, and the compiler is the census.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **135 passed / 0 failed** (was 130; +5:
  puzzles 3 — the book false in exactly the shadow's three
  entries, each by the offset; hollow bits cross-read both
  ways; the re-cut priced, budget brute-forced over all 24
  seatings — state 1: Chamber 5's judge, the FOE's refusals
  (book still framed, honest entry exhibited, unmeasured
  entry exhibited, decoy seated, tray order over budget),
  the strike, the awards, and the serde-strip save-compat
  proof; game 1: Chamber 5 and the Foundation played on the
  GUI surface through real layout clicks, the referee's
  catch landing on the way).
  Log: `~/.local/share/pax/niflheim-floor4-step4-build-20261110/cargo-test.log`.
- Theodolite (the cured route, the review's Battery C
  shape): the producer (`bacon --headless -j bacon-ls`) ran
  before each session, locations current at session start.
  Planted session: the control
  `pub fn theodolite_route_control() -> i32 { "not an int" }`
  appended to `puzzles.rs` read **count=3** (E0308, push and
  pull) while content, state, game, and shell read **0 in
  the same session** — verified zeros. Clean session (control
  restored): all five files count=0, no possibly_incomplete,
  session ready on rust-analyzer 1.100.0-nightly.
  Logs: `theodolite-planted.log`, `theodolite.log`.
- Determinism: the probe extended through Chamber 5 and the
  Foundation; saves byte-identical, sha256
  `8a6493c666a90be8ee4e8b8a97212a912bbf74ec44caaf6bc9be6cbb6f358864`
  (was `92f009fa…` — the playthrough grew).
- Android lib under the pin: 1,347,263,208 bytes. Launch
  smoke under Xvfb: `cold_start_ms=754`, no panics.

### Left open

- Step 5: the driven whole-floor playthrough — all five
  chambers and the FOE on the GUI surface — and Floor 4's
  close-out.

## Step 5 — Integration: the whole floor, played

**Method note (D-1189):** Claude-direct, as Step 4.

### What landed

- **`game.rs` (tests only)** — `floor4_whole_floor_played_on_the_gui`:
  the shipped whole-floor playthrough. From the Step 1
  stairs-lit fixture, every chamber is entered through its
  door and solved through real layout clicks — the bench's
  six slates with both surprises annotated, the wall at its
  least span, the crests cut to their niches, the treaty
  wall seated and its promise amended six presses, the
  hollow stones filled and tagged with the referee's catch
  taken on the way — each chamber proved and sealed before
  the next door opens. Then the Foundation: the book taken,
  five stones measured, the false three exhibited, the
  seal's refusal while the re-cut stands unverified, both
  sets laid and verified, the strike. The test closes on
  the floor's own terms: `floor4_complete()`, all five
  chambers Repaired **at full standard** (no hint spent
  anywhere on the floor — every facet kept), the Brass Eye
  Stone in the codex, the Plumb Line in the pack.

### Gates (primo, pinned rustc 1.99.0)

- `cargo fmt --check`: clean. `cargo clippy --all-targets
  -- -D warnings`: exit 0.
- `cargo test`: **136 passed / 0 failed** (was 135; +1, the
  playthrough).
  Log: `~/.local/share/pax/niflheim-floor4-step5-build-20261110/cargo-test.log`.
- Theodolite (cured route, Battery C shape): producer
  first; the planted control in `game.rs` read **count=3**
  (E0308 live), the restored file read **count=0** in the
  clean session — a verified zero on the rung's one touched
  file. Logs: `theodolite-planted.log`, `theodolite.log`.
- Determinism: the Step 4 probe re-run on the new tree;
  saves byte-identical, sha256
  `8a6493c666a90be8ee4e8b8a97212a912bbf74ec44caaf6bc9be6cbb6f358864`
  — unchanged, as it should be: this rung changed no state
  code.
- Android lib under the pin: 1,347,263,208 bytes —
  byte-size identical to Step 4 (a test-only rung). Launch
  smoke under Xvfb: `cold_start_ms=1709`, no panics (the
  number is contention weather, not a regression signal;
  Step 4 read 754 on a quieter machine).

### Floor 4 close-out

Floor 4 — **the Foundation Galleries** — is complete:
five chambers (the Surveyor's Bench, the Alignment Wall,
the Seal Stones, the Treaty Wall, the Hollow Stones) and
the Crooked Foundation, FOE 4, whose fall teaches the
descent its eighth failure class and puts the Plumb Line
in the pack for Floors 5, 9, 10, 13, and 14. The floor's
doctrine held at every gate: every size, alignment, and
offset the game asserts was measured from a real type or
priced by the one layout algorithm — including the FOE's
own book, whose three false entries are false only against
the ground. Suite at floor close: **136/0**.
