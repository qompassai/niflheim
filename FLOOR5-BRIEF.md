# FLOOR 5 BRIEF — The Veilworks (Niflheim completion programme)

You are building Floor 5 of Niflheim on top of the completed
Floor 4 build (Floor 4 complete at commit 3c6c6b0, branch
`pax/floor2-build-20261109`; the branch continues — do not fork a
new one without Pax's word). Local commits only — NEVER push.
The per-rung gates are the programme's: cargo test (counts grow,
0 failures), clippy `-D warnings`, fmt, theodolite 0 error
diagnostics on touched files (verified zeros: producer first,
same-session-shape positive control, per the cured Rust route),
determinism (two state-level playthrough runs, byte-identical
saves), Android lib build under the pinned toolchain (rustc
1.99.0, `~/.cargo/bin` first on PATH), Xvfb launch smoke. File
the step report in FLOOR5-STEP-REPORT.md in-tree as you go.
Method (D-1189): Claude-direct — coordinator-authored blocks +
splicer, anchors asserted exactly once before any write, as
Floors 2–4.

## The floor (design of record: FULL-DESIGN.md, Floor 5 + §5.2/§5.4/§5.6)

- Domain 5 — Unsafe Rust & the aliasing model. Class:
  Veilwalker (the keeper is **the Veilwalker**). Stratum II —
  deeper warrens: a workshop where every tool is sharp and
  every bench carries its contract card — what the tool
  assumes, written where the hand can read it before it lifts.
- Learning objective: unsafe work is contract work — every
  assumption stated, checkable, and refereed; aliasing is a
  promise about the whole value, not the touched part.
- **Primary failure class: #10, The Ice Ledger** — a freeze
  (a borrow) is a contract over the whole value. It is named
  explicitly in the FOE's inscription. Secondary class **#7,
  Two Charms, in miniature**, lives in Chamber 5.
- Prerequisites (§5.2): Floor 4. `floor5_reachable()` =
  `floor4_complete()` (five chambers repaired AND the Crooked
  Foundation fallen). The door's refusal speaks the unmet
  condition, never a buzz.
- Navigation (standing coordinator ruling): each floor's door
  stands on its deepest prerequisite's hub, on open ground,
  never occluding text. Floor 5's door stands on **Floor 4's
  hub**; `geom::door_floor5()`, stairs-stone size (124·124).
  Leading candidate: (640, 150) — the spot `door_floor4`
  proved on Floor 1's hub — below Floor 4's chamber row,
  clear of the panel and of the FOE door at (600, 292);
  verify open ground at build time. `El::DoorFloor5`, drawn
  "FLOOR 5 / The Veilworks" with an OPEN/BARRED tag.
- The Plumb Line is wielded here (Floor 4's loot): layout
  claims on this floor are line-proven — Chamber 4's beam
  and the FOE's ledger carry their sizes from real types.
- This FOE carries **no cargo gate**: its gate is the
  referee's behaviour-suite walk. `gate.rs` is untouched.
- `#![forbid(unsafe_code)]` stands — on this floor above
  all: the game *teaches* unsafe; it never *performs* it.
  Every mechanism is simulated in safe code over authored
  tables; the referee is a pure judge in puzzles.rs.

### Chambers (ordinary encounters, in design order)

All five follow the standing resolution model (§5.4): work
fields on the chamber's progress record, a pure judge over
authored tables in content.rs, proof = the design's proof
for that chamber, repair through the shared seal. Every
chamber's Watch demonstration is its charter text. Hint
ladders: four rungs each, the fourth naming the seam. The
referee throughout is **Miri** — the syllabus's referee,
here the workshop's own mechanism (Floor 4 named her; this
floor's mastery unlock makes her a standing instrument).

1. **Raw pointers — the Pointer Walks.** A marked path of
   six steps across a memory-yard of crates, walked with a
   surveyor's pole that can point anywhere and guarantees
   nothing. Each step carries a contract card (its
   precondition, in words); some cards' claims are false
   (the crate stands emptied; the step leaves the yard).
   The step and refuse presses refuse until the step's card
   has been read — the chamber's first lesson is mechanical:
   read the card before the step. Judge: every safe step
   walked, every false step refused, each after its card
   was read. Work: `cards_read: [bool; 6]`,
   `walked: [bool; 6]`, `refused: [bool; 6]`.
2. **The aliasing model — the Two Tenants.** Eight placarded
   tenancy situations (readers and writers over one room).
   The player predicts each verdict — sound, or a violation
   — on the placard; then the referee judges and marks each
   placard right or wrong. Seven of eight must stand right;
   every miss must be annotated with its deciding rule (the
   annotation cycles three authored rules per situation,
   one true). Re-prediction and re-judging are allowed; the
   annotations are judged against the final predictions.
   Judge: judged at least once, final correct count ≥ 7,
   every missed situation annotated with its true rule.
   Work: `predictions: [u8; 8]` (0 undecided, 1 sound,
   2 violation), `judged: bool`,
   `annotations: [Option<usize>; 8]`.
3. **Unsafe contracts — the Contract Cards.** Four tools on
   a bench, each with a card of three lines; most lines are
   vapour ("safe because it is safe") or circular. Each line
   cycles three authored phrasings, exactly one checkable.
   The sceptical Veilwalker accepts a card only when every
   line's claim can be *checked*, not merely believed.
   Judge: all twelve lines checkable. Work:
   `card_lines: [[u8; 3]; 4]` (the chosen phrasing index per
   line; cycling is the rewrite).
4. **Safe abstractions — the Split Beam.** A beam (a buffer
   of six words — a real `[u32; 6]`, its size the Plumb
   Line's fact) is split into two halves that may be worked
   simultaneously, the contract at the seam. The player sets
   the split (stepper over word boundaries 0..=6), seams the
   beam (sound only strictly inside: both halves non-empty),
   then walks the referee's three checks: an access past a
   half's end, refused by the face; both halves worked at
   once; the halves dropped in the contract's order (left
   first — the face's stated rule; dropping right first is
   refused with the reason). Choosing a new split unseams
   the beam and spends the walk. Judge: seamed at a sound
   split, the refused access walked, both halves worked,
   dropped in order. Work: `split: u8`, `seamed: bool`,
   `access_refused: bool`, `worked: [bool; 2]`,
   `dropped: [bool; 2]`.
5. **The UB hunt.** A small machine of six stations; three
   hide seeded faults — a tenancy violation (two writers
   fused into one tenancy: #7 in miniature), a use after the
   room was emptied, a read of an unfilled blank — and three
   are sound. The referee's lens (an Examine press per
   station) reveals a station's contract state; verdicts
   (sound / fault) may be entered only for lensed stations.
   A fault is fixed at the contract level, never at the
   symptom: the faulted station's card cycles three authored
   contracts, one true. Judge: all stations lensed, all
   verdicts right, every faulted station's card rewritten
   truly. Work: `lensed: [bool; 6]`,
   `verdicts: [u8; 6]` (0 unentered, 1 sound, 2 fault),
   `cards: [u8; 6]`.

### The FOE — The Thing Behind the Veil

The workshop's great self-growing store-beam (a raw-parts
buffer) has begun letting a lent section be *resized by a
second hand* while the first hand's reference is still out.

- **Inscription** names failure class #10, the Ice Ledger.
- **Diagnosis (the trace):** the player traces the reported
  corruption through the loan ledger and exhibits, **in
  sequence**: one loan (a live shared loan on the beam),
  one growth (the beam grew while the loan was out), and
  the lens's verdict (the reference reads a room that no
  longer exists — another tenant's goods). Exhibiting out
  of sequence is refused with the missing prior exhibit
  named. Proof = the three exhibits in order.
- **The gate (the standing evidence):** the repaired
  buffer's behaviour suite, walked under the referee —
  five named cases, each a demonstration the player runs:
  (i) growth refused while a loan is out; (ii) the panic
  path — an access past the end refused, not UB; (iii) drop
  order — the elements drop in order, none twice, none
  leaked; (iv) the empty element — an empty store refuses
  access and allows growth; (v) overflow — growth at the
  stated capacity limit refused by contract. `gate_passed`
  = all five walked.
- **Repair (the strike's substance):** the beam's contract
  card rewritten (three lines, cycled to their checkable
  truths — a loan freezes the whole beam; growth is refused
  while any loan is out; the face exposes no operation that
  can move the beam under a reference), laid before the
  seal. Strike: the shared two-step seal, armed only when
  proof stands complete and the gate stands walked.
- **Codex stone:** **The Ice Stone** — "A borrow is a
  contract over the whole value."
- **Loot:** **The Veilwalker's Buffer** — wielded on Floors
  6, 9, 12. Mastery: the referee (Miri) is unlocked as a
  standing instrument; unsafe mechanisms on later floors
  are operable only with their contract cards.

## Steps

1. **Scaffolding.** Save format v5; `Floor5Progress` +
   `VeilWork` (empty) + `Floor5FoeProgress`;
   `Screen::Floor5Hub`, `El::DoorFloor5` / `Floor5Door(u8)`;
   `geom::door_floor5()` on Floor 4's hub on verified open
   ground; `build_floor5_hub` in shell.rs; the map's Floor 5
   square; `floor5_reachable()` gating the door on Floor 4
   complete. The v5 fixture updates ride the bump, as they
   did for v4.
2. **Chambers 1–2** — the Pointer Walks and the Two Tenants.
3. **Chambers 3–4** — the Contract Cards and the Split Beam.
4. **Chamber 5 + the FOE** — the UB hunt, the trace, the
   suite walk, the strike.
5. **Integration** — the shipped whole-floor GUI playthrough
   (all five chambers and the FOE on the GUI surface), full
   gates, the floor's close-out. Floor 5 completes there.

## Standing cautions (from the earlier lanes' record)

- Splicer anchors assert exactly once against the CURRENT
  tree before any write; an anchor carrying a closing brace
  puts inserted code outside the item; insert-before at a
  match-arm boundary can land between arms. A new `Screen`
  variant has more match sites than the obvious ones —
  exhaustiveness errors are the census; expect them in
  places like layout()'s tab-bar match.
- The local splice mirrors are unformatted: after shipping
  to primo, `cargo fmt` (apply) then `cargo fmt --check`;
  primo's committed tree is the fmt-clean source of truth.
- Theodolite: the locations file must exist and be current
  WHEN THE SESSION STARTS — producer (`bacon --headless
  -j bacon-ls`) first, then the bridge; a zero counts as
  verified only with a same-session-shape positive control.
  A warm-up diagnostics call is never evidence.
- Real facts only: any size the game asserts comes from
  `size_of` of a real type or from the floor's one priced
  algorithm — never from a hand-typed answer table.
- Primo good-neighbour rules: one heavy job at a time,
  `nice -n 10`, the Owner's interactive work wins.
