# FLOOR 9 BRIEF — The Hoard Beneath (Niflheim completion programme)

Floor 9 completes Part A. Its Step 5 commit is **the freeze
commit** of SPEC §5: the tip of the programme branch at Part A
completion, on which every Part B branch forks. Treat Step 5
accordingly — the frozen surfaces (§5.1) must be exactly as
Part B's contracts rely on them when this floor closes.

Base: the programme branch `pax/floor2-build-20261109` tip
AFTER Floor 8's integration (Pax merges the Floor 8 lane
branch at his Floor 8 boundary gate; Floor 9 does not start
before that merge). RE-READ the live branch tip before basing
any work on it; record the actual base. Floor 9 builds in a
fresh worktree on branch `pax/floor9-build-20261110` per the
coordinator's commission. Local commits only — NEVER push.
Per-rung gates: fmt, cargo test (counts grow from Floor 8's
final count, 0 failures), clippy `-D warnings`, theodolite
verified zero errors on touched files, two byte-identical
state-level determinism saves, pinned-toolchain Android lib
build, Xvfb launch smoke, and `#![forbid(unsafe_code)]`
intact. File evidence and append FLOOR9-STEP-REPORT.md
in-tree as you go. Method (D-1189): coordinator-sequenced
Part A lane build, per SPEC §3; anchors asserted exactly
once against the CURRENT tree before any write. Build
contract of record: SPEC.md §3 standing pattern + §4
Floor 9; §5 interface freeze and §9 DS-1…DS-15 bind. This
brief is authoring, not evidence that any build gate has
passed.

## The floor (design of record: FULL-DESIGN.md, Floor 9 — The Deep Hoard)

- Domain 9 — Collections & allocators. Class and keeper:
  **Heapwarden**. Stratum III — Systems: the working
  frontier, vault-mines where the guild's collections are
  grown, counted, and stored.
- Learning objective: the standard collections are choices
  with known prices — growth, probing, and allocation are
  mechanisms the player has operated by hand, and counts
  about a hoard are claims that must carry their evidence.
- **Primary failure class: #1, The Toll of Tallies** — a
  number that travels farther than its evidence.
- Prerequisites (design §5.2): Floors 4 + 5.
  `floor9_reachable()` = `floor5_complete()` (Floor 4 is
  implied by Floor 5's own reachability). This is not a
  previous-floor chain: Floor 8 is not a gate. Refusal
  names the unmet condition, never a buzz.
- Navigation (standing coordinator ruling): door on
  **Floor 5's hub**, deepest prerequisite's hub, on open
  ground, never occluding text. `geom::door_floor9()`,
  stairs-stone size 124×124; `El::DoorFloor9`, with the
  floor's name and OPEN/BARRED tag following the current
  hub convention. The exact spot is a Step 1 design task:
  inspect the live hub's existing furniture, chamber row,
  camp, FOE door, panels and any other doors (Floor 10's
  door also lands on this hub later — leave it room);
  verify the chosen rectangle against them and record the
  spot and clearances in the step report, as Floors 6–8
  did. No coordinates are supplied by the sources.
- **SYSTEMS LANDED HERE (frozen at this floor's Step 5,
  §5): the Trio, as real structures in `puzzles`** —
  `ShelfVec`, `CellarMap`, `Hoard` (a bump arena), plus
  `Stamped<T>` — with EXACTLY the §5.1 surface:
  `ShelfVec { new(reserve), push, get, len, cap, stamps() }`;
  `CellarMap { new(capacity), insert (counted probes), get,
  len, probes_total }`; `Hoard { issue(bytes) ->
  Option<u32>, reset, used, blocks_used }`;
  `Stamped<T> { seal(value, census) -> Stamped<T>, value(),
  census_id() }`. Crossover rules are part of the surface:
  `Hoard::issue` refuses (None) over capacity, never wraps;
  `CellarMap` is open addressing with linear probe,
  tombstone-free, load factor under one so a probe always
  terminates; every `Stamped` carries the census id it was
  sealed under. The structures + their unit tests land in
  **Step 1**, not Step 4. Consumed by Floor 12 C1,
  Floor 13 C5, Floor 16, and this floor's own C4/FOE. Land
  the Trio's full required behaviour here; defer nothing
  a Part B contract relies on.
- Save format **v9**, one bump at Step 1 (DS-2); work
  record **`HoardWork`**. All added fields serde-defaulted;
  existing fields never renamed, removed or re-typed.
  Version-load fixtures and the future-version forge test
  ride the bump.
- No cargo gate: `gate.rs` remains untouched (DS-4).
  `#![forbid(unsafe_code)]` stands. **Real facts only
  (DS-13): identities, census counts, probe counts and
  allocation counts in assertions derive from the REAL
  structures run over authored inputs; the differential
  trials are real cargo tests over at least 10,000
  operations per structure, never a hand-typed result
  table.** Authored inputs are content; outputs are
  computed.
- Stairs relevance: this floor's FOE fallen is the
  load-bearing half of the Stratum III stairs; the
  `stairs_23()` predicate itself lands at Floor 10 Step 1
  per SPEC — no stairs predicate lands here.
- Mastery note (FULL-DESIGN): reaching for a standard
  collection on later floors becomes a stated choice with
  a known cost, never a reflex; keepers quote prices.
- Platform: census reconciliation runs in batches with
  progress shown by count and bar (never a spinner alone);
  the ten-thousand-operation trials are presented as a
  summarized record, not simulated in real time,
  identically on all platforms. No live-timing inputs,
  host timing gates or RNG. Preserve reduced motion,
  labelled non-colour signals and phone-width readability.
- **Name variances (build to SPEC, flag, never harmonize):**
  SPEC names this floor *The Hoard Beneath*; FULL-DESIGN's
  section title is *The Deep Hoard* (its FOE name is The
  Hoard Beneath in both). SPEC's codex stone is **The
  Hoardstone**; FULL-DESIGN calls it *The Toll Stone*,
  same text. FULL-DESIGN's "wielded later" for the Trio
  lists Floors 12, 13, 14; SPEC §5.1 lists consumers
  Floor 12 C1, Floor 13 C5, Floor 16. Record all three
  variances in the step report.

### Chambers (ordinary encounters, in spec order)

All five follow the standing resolution model (design
§5.4): UNWORKED → DIAGNOSED → REPAIRED, work fields on the
chamber's progress record, pure judges in puzzles/state
over authored content.rs inputs, proof = the specified
proof, repair via the shared seal. Work record:
`HoardWork`. Four-rung hint ladders each, fourth naming
the seam; keeper **Heapwarden**. Chambers open strictly
in order after the prior repair. Proof slots labelled by
KIND, never answer content; wrong proof names its
mismatch. Facet = 
...[truncated 7899 chars]repair at hints_used ≤ 3, with facet text naming **The
Hoardstone**. Extend only this floor's invariant section;
recompute proofs, five chambers.

1. **The Shelf.** `ShelfVec` over Floor 5's buffer
   discipline: predict (len, cap) for each of 8 pushes,
   one of them forcing a reserve; growth costed on the
   counting-house slate. Judge: all 8 predictions exact
   against the REAL `ShelfVec`. Fields:
   `predictions: [[u8; 2]; 8]`, `judged: bool`. Secondary
   **#14** boundary encounter: growth at the exact
   capacity boundary — the general growth rule expires
   exactly there, and the floor says so.
2. **Honest Chains.** Secondary **#4, Twin Bells**. A
   hand-linked chain of crates fights both the ledger and
   the road; the player takes it as far as soundness
   allows, then redesigns it as an indexed shelf-row
   (arena indices) and reads the post-mortem wall, which
   names both failure mechanisms. Judge: the arena
   redesign's slot and next arrays exact. Fields:
   `slots: [u8; 6]`, `nexts: [u8; 6]` (255 = none).
3. **The Cellar.** `CellarMap` walked with a
   probe-lantern: place an authored colliding set
   (content.rs), predict each key's probe count, then
   flood the cellar and state the flood's total probe
   cost. Judge: probe predictions + flood cost exact
   against the REAL `CellarMap`. Fields:
   `probes: [u8; 5]`, `flood_cost: u8`, `judged: bool`.
   The authored colliding set's exact keys are
   REVIEW-F9-CONTENT, not an invented answer key; counts
   are computed from the real map, never hand-typed.
4. **The Counting Steward.** A stated ten-operation
   workload run across the whole Trio; predict its exact
   allocation count AND explain it — every predicted miss
   accounted per operation. Judge: predicted count exact
   against the REAL structures + all ten miss-explanations
   judged. Fields: `predicted: u8`,
   `misses_explained: [bool; 10]`, `ran: bool`.
5. **The Interner's Hoard, Rebuilt.** Secondary **#14,
   The Uncounted Last Mile**. The Floor 1 loot rebuilt on
   `Hoard` blocks: blocks issued in order, reset as one.
   Its behaviour must pass its original suite unchanged
   (the suite walks here), and its allocation count for a
   fixed corpus drops to a stated number of blocks.
   Judge: original suite passes on the rebuild + the
   player names the corpus's block count, the last mile
   counted. Fields: `blocks: [u8; 8]`,
   `suite_walked: bool`, `last_mile: u8`. The fixed
   corpus and its stated block count are authored
   content (REVIEW-F9-CONTENT if unsettled); the count
   the player names is verified against the REAL arena.

### The FOE — The Hoard Beneath

Three honest tallies, one lying sum: the master hoard's
census sums the shelf-count, the cellar-count and the
arena-count kept by three stewards whose domains overlap —
everything stored through the arena is also shelved, and a
third of the cellar's keys index shelved goods.

- **Diagnosis (exhibits, IN SEQUENCE):** (1) the three
  tallies as they stand, each honest; (2) the overlap
  classes, sorting six goods into their classes;
  (3) the reconciled census: a stamped count fused to its
  census-scroll. Out-of-sequence Exhibit refused with the
  missing prior exhibit named. Reconciliation is by
  identity, not arithmetic: sampled items exhibited
  against the three ledgers, overlap classes enumerated —
  shelved-only, cellar-only, arena-and-shelved, all-three —
  with counts proven against the physical hoard.
- **Pressure (DS-3):** the bare census travels: frontier
  desks keep quoting the summed number during diagnosis —
  bounded authored misleading evidence. Never damage,
  never destroy proof, never impose a real-time limit.
- **The gate:** the differential-trial ledger, actually
  run: the player's Trio against naive reference models,
  at least 10,000 operations per structure, zero
  divergence — real cargo tests over the real structures
  (DS-13), presented to the player as a summarized
  record. Plus three adversarial cases walked: a growth
  at capacity refused; a collision flood taken from
  Chamber 3; arena exhaustion returning a documented
  refusal, never a panic or a wraparound.
- **Repair (strike's substance):** the summed census is
  abolished; tallies are re-issued as `Stamped` counts
  fused to the census-scroll that produced them, in a
  form that cannot be detached from its evidence or added
  to another stamped count. A bare re-issue is refused.
  Shared two-step seal strike arms only with proof
  complete, gate walked, and re-issue accepted.
  STANDING → EXPOSED → FALLEN; no strike can bypass
  proof, and proven repair cannot fumble.
- **Codex stone:** **The Hoardstone** — "Provenance is
  part of the value." FULL-DESIGN calls it *The Toll
  Stone* with the same text. Build to SPEC, flag the
  name variance in the step report, do not silently
  harmonize it (Floor 7's Loomstone and Floor 8's
  Pinstone precedents). Preserve the design's mechanism
  explanation: the check moves from discipline —
  remembering to verify — to structure: the desk itself
  refuses anything else.
- **Loot:** **The Heapwarden's Trio** — the growing
  shelf, the key-cellar, and the bump quarry,
  differentially proven. Wielded later per SPEC §5.1:
  Floor 12 C1, Floor 13 C5, Floor 16 (FULL-DESIGN lists
  Floors 12, 13, 14 — destination-list variance flagged
  above, never harmonized).

## Steps

1. **Scaffolding.** Re-read live tip and Floor 8's
   boundary report. Save v9; `Floor9Progress` +
   `HoardWork` (empty at S1, standing watched flag as the
   current precedent requires) + `Floor9FoeProgress`;
   serde-defaulted expedition/save field and mapping
   touch-points. `Screen::Floor9Hub`, `El::DoorFloor9` /
   `Floor9Door(u8)`; `geom::door_floor9()` on F5 hub on
   verified open ground, spot/clearances in report;
   `build_floor9_hub` in shell.rs; map's Floor 9 square;
   `floor9_reachable()` gates door. All five ChamberMeta,
   intros, examine texts and general hint ladder.
   **The Trio + `Stamped<T>` land HERE**, as real
   structures with their unit tests, on exactly the §5.1
   surface. Update v9 fixtures, preserve old records.
   Resolve any Trio-shape REVIEW with Pax before turning
   unsettled choices into the frozen API — this floor's
   Step 5 freezes it.
2. **Chambers 1–2** — The Shelf and Honest Chains.
3. **Chambers 3–4** — The Cellar and The Counting
   Steward, including real probe/allocation accounting.
4. **Chamber 5 + FOE** — The Interner's Hoard, Rebuilt;
   sequenced exhibits, bounded bare-census pressure, the
   differential-trial ledger and three adversarial cases,
   stamped re-issue and strike.
5. **Integration + THE FREEZE.** Shipped whole-floor GUI
   playthrough: a `#[test]` driving real press handlers
   through all five chambers and the FOE, not setters
   substituting for play; full gates, close-out. **This
   step's commit is the freeze commit of SPEC §5** — say
   so in the commit message. Floor 9 and Part A complete
   here.

Per-step gates (Floor 6/7/8 set; all on primo; evidence in
`~/.local/share/pax/niflheim-floor9-step<N>-build-20261110/`):

- `cargo fmt` apply on primo then `cargo fmt --check`
  clean.
- `cargo test`: **counts grow from Floor 8's final count,
  0 failures**; no test deleted, skipped or weakened.
  Roughly half validation, half adversarial: exact
  predictions paired with off-by-one counts, probe
  mispredictions, capacity-boundary refusals, exhaustion
  refusals, overlap misclassifications, unstamped-count
  refusals and premature strikes. Version fixtures run
  every rung. The differential trials (≥10,000 ops per
  structure, zero divergence) are part of this suite.
  Record actual counts.
- `cargo clippy --all-targets -- -D warnings`: exit 0.
- Theodolite: cured protocol, producer first; planted
  control seen in-session, restore sha256-verified, clean
  count=0 on all touched files with a supported route.
  See cautions.
- Determinism: state-level playthrough probe run twice
  with `NIFLHEIM_SAVE_OUT`, saves byte-identical, sha256
  recorded. Transient per DS-9: copy into `tests/` for
  gate, remove afterward, never commit it. Cover earned
  current-step state and load-back, culminating in
  completed v9 save.
- Android `cargo build --target aarch64-linux-android
  --lib` exit 0; record library size. Pinned rustc 1.99.0
  from repo rust-toolchain.toml, the pinned toolchain's
  own bin first on PATH before every cargo call.
- Xvfb smoke: `cargo build`, fresh `NIFLHEIM_SAVE_PATH`,
  `timeout 25 xvfb-run -a ./target/debug/niflheim`;
  `cold_start_ms=` recorded, zero panics. Expected timeout
  is distinguished from a crash. Unsafe forbid intact.

Per-step local commit on the Floor 9 branch; brief
committed with Step 1, as Floors 6–8 were. No pushes.
Reports follow FLOOR8-STEP-REPORT.md: actual base, built
delta, geometry/API decisions, named variances and
REVIEWs, per-gate evidence paths/hashes, counts/exits,
incidents and fixes, commit hash; close-out lists all
five commits, final count, earned save v9, determinism
hash, Android size, GUI evidence and open items — and
names the Step 5 commit as the freeze commit. No artifact
means REVIEW, not PASS; Pax verifies boundary. Pax alone
merges the Floor 9 branch into the programme branch; the
freeze is the merged tip.

## Standing cautions (from the earlier lanes' record)

- Splicer anchors assert exactly once against CURRENT
  tree before any write. Closing-brace anchors may put
  code outside the item; insert-before at an arm boundary
  can land between arms. New `Screen` variants have more
  match sites than the obvious ones; exhaustiveness errors
  are the census, including layout's tab-bar match.
- Local splice mirrors are unformatted: apply `cargo fmt`
  on primo after shipping, then check. Primo's committed
  tree is the fmt-clean source of truth. Inspect control
  geometry and hit testing: F7 integration found a real
  order-button/seal overlap missed by the prior gate path.
- Validate every touched file through primo
  `~/.local/bin/theodolite`. Zero error diagnostics
  before reporting done; cite diagnostics per file type.
  Name any type without a route, or file without an
  arriving payload, **not-LSP-validated**, never silently
  skip it. For Rust, locations file exists and is current
  WHEN SESSION STARTS: `bacon --headless -j bacon-ls`
  producer FIRST, bridge afterward. Producer session
  `niflheim-bacon`: restart if gone before the first
  session; stop at floor boundary. Same-session-shape
  planted `theodolite_route_control` must yield count=3
  E0308 while clean files yield 0; restore with sha256
  verification, rerun clean on every touched Rust file.
  Warm-up calls are not evidence.
- Record outcomes for any custom skill materially used
  via `~/workspace/bin/skill-trust.py record`. This is
  Pax's standing recorder path; on primo the recorder
  lives in the lane workspace if present. Coordinator
  resolves the dispatch path; do not invent one or claim
  an unrun record.
- Real facts only: census counts, probe counts, growth
  sequences and allocation counts derive from the real
  Trio over authored inputs, never answer tables. The
  differential ledger is a real test, not a displayed
  claim.
- Primo good-neighbour rules: one heavy job at a time,
  `nice -n 10`, Owner's interactive work wins. Shared
  cargo target via the `--target-dir` FLAG, never the
  `CARGO_TARGET_DIR` env var (the env var is inherited by
  gate.rs's nested cargo and fails gate tests — Floor 8
  commission record).
- Anything SPEC/FULL-DESIGN does not settle is a named
  REVIEW for Pax, not permission to invent constants,
  structures, policies or new mechanics.
