# FLOOR 4 BRIEF — The Foundation Galleries (Niflheim completion programme)

You are building Floor 4 of Niflheim on top of the completed
Stratum I build (Floor 3 complete at commit 4e60392, branch
`pax/floor2-build-20261109`; the branch continues — do not fork a
new one without Pax's word). Local commits only — NEVER push.
The per-rung gates are the programme's: cargo test (counts grow,
0 failures), clippy `-D warnings`, fmt, theodolite 0 error
diagnostics on touched files, determinism (two state-level
playthrough runs, byte-identical saves), Android lib build under
the pinned toolchain (rustc 1.99.0, `~/.cargo/bin` first on
PATH). File the step report in FLOOR4-STEP-REPORT.md in-tree as
you go. Method: coordinator-authored blocks + splicer, anchors
asserted exactly once before any write, as Floors 2–3.

## The floor (design of record: FULL-DESIGN.md, Floor 4 + §5.2/§5.4/§5.6)

- Domain 4 — Memory layout. Class: Plumbmason (the keeper is
  **the Plumbmason**). Stratum II — the machine warrens: the
  labyrinth's foundations, where stones are laid by measure and
  every wall shows its own cross-section.
- Theme: galleries of foundation stones, each stone stencilled
  with its true dimensions; a plumb line hangs in every room,
  and what the line says outranks what anyone remembers.
- Learning objective: layout is measurable fact — sizes,
  alignments, padding, and encodings are proven by assertion,
  never described in prose.
- **Primary failure class: #8, The Astronomer's Brass Eye** —
  measuring a proxy instead of the quantity. It is named
  explicitly in the FOE's inscription. Secondary class **#14,
  Last Mile, in miniature**, lives in Chamber 4.
- Prerequisites (§5.2): Floor 1. Entry also crosses the
  Stratum I → II stairs (§5.6), so `floor4_reachable()` = FOE 1
  fallen AND (FOE 2 OR FOE 3 fallen) AND every Stratum I
  lantern keeper-lit — exactly `stairs_clauses()` all lit.
  The door's refusal speaks the first unmet clause's text
  (the stairs communicate by inscription, never by a buzz).
- Navigation (standing coordinator ruling): each floor's door
  stands on its deepest prerequisite's hub, on open ground,
  never occluding text. Floor 4's door stands on **Floor 1's
  hub**; `geom::door_floor4()`, stairs-stone size, the exact
  spot chosen at build time on verified open ground,
  El::DoorFloor4, drawn "FLOOR 4 / The Foundation Galleries"
  with an OPEN/BARRED tag like its neighbours.
- This FOE carries **no cargo gate**: its gate is the plumb
  line itself. `gate.rs` is untouched by this floor.
- `#![forbid(unsafe_code)]` stands: unions are simulated by
  honest bit reinterpretation (`f32::from_bits` /
  `f32::to_bits`), never by a real union read.

### Chambers (ordinary encounters, in design order)

All five follow the standing resolution model (§5.4): work
fields on the chamber's progress record, a pure judge in
puzzles.rs over authored tables in content.rs, proof = the
design's proof for that chamber, repair through the shared
seal. Every chamber's Watch demonstration is its charter text.
Hint ladders: four rungs each, the fourth naming the seam.
Quantities render in large tabular figures (§7.3) in the
shell's chamber panels.

1. **Sizes and alignment — the Predict-and-Measure bench.**
   Six stones, each a real Rust type defined in content.rs
   (`u8`, `(u8, u32)`, `&str`, `Option<u8>`, `Option<&u64>`,
   `Vec<u8>`); the bench's facts are `std::mem::size_of` /
   `align_of` of those very types, taken at runtime — the
   line measures the quantity itself. For the selected stone
   the player first commits a predicted size (stepper,
   1..=64) and alignment (stepper over 1, 2, 4, 8, 16) to the
   slate; only then may the line drop (measure). A miss is a
   **surprise**, and a surprise must be annotated before the
   chamber completes: the annotation cycles authored causes
   (the true one plus two decoys per stone — padding, the
   niche, the fat pointer, …). Judge: every stone measured,
   and every stone whose prediction missed carries its true
   annotation. Work: `pred_size`/`pred_align: [Option<u32>; 6]`,
   `measured: [bool; 6]`, `annotate: [Option<usize>; 6]`.
   The selected stone is ephemeral Game state, never saved.
2. **Padding and field order — the smallest wall.** Five
   stones with authored (size, align) pairs lie on the bench;
   clicking one seats it next in the wall (a reset stone
   clears the wall). `puzzles::wall_layout` computes any
   order's (total, voids) by the standard algorithm — each
   stone offset aligned up, the total rounded to the wall's
   alignment — and the panel tallies the current wall's voids
   byte by byte against the bench order's. Judge: all five
   seated and the wall's total equals the minimum the same
   function finds over the aligned order (any order achieving
   the minimum passes; the size is the proof). Work:
   `wall_order: Vec<u8>`.
3. **Enum encoding and the niche — the Seal Stones.** Three
   crests, each with authored candidate encodings that are
   themselves real types (a tagged struct, an `Option`, a
   niche-carrying `Option<NonZero…>` / `Option<&str>`), so a
   candidate's size is again `size_of` of a real type. The
   player chooses an encoding per crest, then proves it on
   the bench (per-crest Prove). Judge: every crest proven,
   and every chosen encoding's real size equals the smallest
   real size among its candidates — the judge computes the
   minimum from the same `size_of` facts, never from a
   hand-written answer. Work: `crest_choices: [Option<usize>; 3]`,
   `crest_proven: [bool; 3]`.
4. **The repr family — Treaty Stones vs Free Stones.** A
   treaty wall of four seats under a foreign spec: each
   stone's offset is *promised* in the treaty book. The tray
   holds the four treaty stones plus one **free twin** of the
   last (same dimensions, no promise). Clicking seats stones
   in order; the line proves offsets by the C layout rules
   (`puzzles::c_offsets`). Seating the free twin is shown to
   be under no obligation — the wall cannot complete with it.
   The book's promise for the **final** stone was written by
   the naive sum and forgets the padding (**#14 in
   miniature**): the line proves the true offset, and the
   player must amend the treaty — a stepper walks the final
   promise to the true offset. Judge: the wall seated with
   the four treaty stones in order, the line dropped
   (proven), and the final promise amended to the true
   offset. Work: `treaty_order: Vec<u8>`,
   `treaty_proven: bool`, `final_promise: Option<u32>`
   (the stepper starts from the book's naive figure).
5. **Unions and uninitialized memory — the Hollow Stones.**
   Two hollow stones, each a 4-byte cavity with two possible
   fillings: a whole number (authored `u32`) or a ratio
   (authored `f32`). The filling cycles (empty / number /
   ratio); the mason's tag cycles separately (untagged /
   number / ratio) — the tag is kept only in the mason's
   head. The demonstration reads the cavity *as the tag
   claims*, by honest bit reinterpretation, so a wrong tag
   shows its nonsense plainly. Beside them, a blank stone of
   four cells: cells fill one by one, and a Use press on a
   partly-filled blank is caught by the gallery's referee
   (the catch is recorded on the record; Miri is named in
   the chamber's text as the syllabus's referee). Judge:
   both hollow stones filled with tag == filling, the blank
   full, and the blank used whole. Work:
   `fillings: [u8; 2]`, `tags: [u8; 2]`,
   `blank_cells: [bool; 4]`, `blank_used: bool`,
   `referee_catches: u32`.

### The FOE — The Crooked Foundation

The warrens' master foundation, laid a generation ago from a
celebrated survey, now settling crooked. Its inscription
names **failure class #8, The Astronomer's Brass Eye**.

- **The seam:** the old survey measured the foundation by
  its *shadow at noon* — a proxy that tracks the true width
  only when the sun cooperates — and recorded the shadow
  numbers in the dimension book as if they were the stones'
  sizes. The book's arithmetic is flawless; every stone cut
  from it is true to the book and false to the ground.
- **Diagnosis (the re-survey):** take the dimension book
  (five load-bearing entries: name, book figure; the true
  sizes are real stones' facts). Drop the line on each stone
  in turn. Entries whose book figure disagrees with the
  measured stone are exhibited beside their true values —
  the wrong entries (three of five) all disagree by exactly
  the shadow's offset, and they are identifiable only by
  measuring the stones, never by auditing the book.
- **Proof required:** every load-bearing dimension measured
  at the stone, the proxy entries exhibited beside their
  true values.
- **Repair strike (the re-cut, laid before the seal):** as
  the guild requires, the model is re-cut twice — one set of
  stones to the foreign spec (four stones seated to promised
  offsets, proven by `c_offsets`) and one free set under a
  stated size budget (four stones ordered so `wall_layout`'s
  total meets the budget, the budget set at the true
  minimum). The line verifies each set.
- **Strike:** the shared two-step seal, armed only when the
  proof stands complete (re-survey done, both sets laid
  true).
- **Codex stone:** **The Brass Eye Stone** — "Distinct
  quantities get distinct types."
- **Loot:** **The Plumb Line** — wielded on Floors 5, 9, 10,
  13, 14; from this fall onward, prose about sizes is no
  longer evidence anywhere in the descent.

## Steps

1. **Scaffolding.** Save format v4; `Floor4Progress` +
   `FoundationWork` (empty) + `Floor4FoeProgress`;
   `Screen::Floor4Hub`, `El::DoorFloor4` / `Floor4Door(u8)`;
   `geom::door_floor4()` on Floor 1's hub on verified open
   ground; `build_floor4_hub` in shell.rs; the map's Floor 4
   square; `floor4_reachable()` gating the door on the
   stairs clauses. The v4 fixture updates ride the bump, as
   they did for v3.
2. **Chambers 1–2** — the bench and the smallest wall.
3. **Chambers 3–4** — the Seal Stones and the treaty wall.
4. **Chamber 5 + the FOE** — the Hollow Stones, the book,
   the re-survey, the two re-cuts, the strike.
5. **Integration** — the shipped whole-floor GUI playthrough
   (all five chambers and the FOE on the GUI surface), full
   gates, the floor's close-out. Floor 4 completes there.

## Standing cautions (from the earlier lanes' record)

- Splicer anchors assert exactly once against the CURRENT
  tree before any write; an anchor carrying a closing brace
  puts inserted code outside the item; insert-before at a
  match-arm boundary can land between arms.
- The local splice mirrors are unformatted: after shipping
  to primo, `cargo fmt` (apply) then `cargo fmt --check`;
  primo's committed tree is the fmt-clean source of truth.
- Theodolite starts its language server lazily: a warm-up
  diagnostics call first, never cited as evidence.
- Real layout facts only: any size/alignment/offset the game
  asserts comes from `size_of`/`align_of` of real types or
  from the one layout algorithm in puzzles.rs — never from a
  hand-typed answer table. That is this floor's whole
  doctrine, and its tests hold it: a unit test cross-checks
  every authored (size, align) pair in Chambers 2 and 4
  against the algorithm, and every crest candidate against
  its real type.
