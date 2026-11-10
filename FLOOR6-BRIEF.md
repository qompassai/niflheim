# FLOOR 6 BRIEF — The Lockheart (Niflheim completion programme)

You are building Floor 6 of Niflheim on top of the completed
Floor 5 build (Floor 5 complete at commit 985a91c, branch
`pax/floor2-build-20261109`; the branch continues — do not fork a
new one without Pax's word). Local commits only — NEVER push.
The per-rung gates are the programme's: cargo test (counts grow,
0 failures), clippy `-D warnings`, fmt, theodolite 0 error
diagnostics on touched files (verified zeros: producer first,
same-session-shape positive control, per the cured Rust route),
determinism (two state-level playthrough runs, byte-identical
saves), Android lib build under the pinned toolchain (rustc
1.99.0, `~/.cargo/bin` first on PATH), Xvfb launch smoke. File
the step report in FLOOR6-STEP-REPORT.md in-tree as you go.
Method (D-1189): Claude-direct — coordinator-authored blocks +
splicer, anchors asserted exactly once before any write, as
Floors 2–5. Build contract of record: the completion spec
(~/workspace/niflheim-completion-spec-20261110/SPEC.md), §3
standing pattern + §4 Floor 6; its DS decisions bind.

## The floor (design of record: FULL-DESIGN.md, Floor 6 + §5.2/§5.4/§5.6)

- Domain 6 — Interior mutability. Class: Lockheart (the
  keeper is **Lockheart**). Stratum II — a quiet warren of
  strongboxes and reading hearts, where things change from
  inside without their shells ever moving.
- Learning objective: all interior change is one mechanism
  underneath (`UnsafeCell`); borrow-flag discipline turns
  "two writers" from corruption into a loud, survivable
  refusal.
- **Primary failure class: #7, The Two Charms, One Socket**
  — two mechanisms forced into one zone, fused into a third
  useless thing. It is named explicitly in the FOE's
  inscription. Secondary class **#15, Two Porters, in
  miniature**, lives in Chamber 2.
- Prerequisites (§5.2): Floor 5 **through chamber 3 only**.
  `floor6_reachable()` = Floor 5 chambers 0, 1, 2 repaired
  (the Pointer Walks, the Two Tenants, the Contract Cards).
  The door's refusal speaks the unmet condition, never a
  buzz.
- Navigation (standing coordinator ruling): each floor's
  door stands on its deepest prerequisite's hub, on open
  ground, never occluding text. Floor 6's door stands on
  **Floor 5's hub**; `geom::door_floor6()`, stairs-stone size
  (124×124). The spot Floor 5's door proved on Floor 4's
  hub — (640, 150) — is this hub's open ground too (Floor
  5's Step 1 report left it open for exactly this); verify
  at build time against the hub's camp stone, chamber row,
  and the FOE door below. `El::DoorFloor6`, drawn
  "FLOOR 6 / The Lockheart" with an OPEN/BARRED tag.
- The Veilwalker's Buffer is wielded here (Floor 5's loot):
  its contract card is Chamber 1's bench card — a content
  reference only, no new code surface.
- This FOE carries **no cargo gate**: its gate is the
  model-loom preview walk. `gate.rs` is untouched (DS-4).
- `#![forbid(unsafe_code)]` stands. Every mechanism is
  simulated in safe code: the flag machine, the lazy cell,
  the spin heart, the reachable-state enumeration, and the
  loom table are real computations in puzzles.rs over
  authored tables — never hand-typed answer tables (DS-13).

### Chambers (ordinary encounters, in spec order)

All five follow the standing resolution model (§5.4): work
fields on the chamber's progress record, a pure judge over
authored tables in content.rs, proof = the spec's proof for
that chamber, repair through the shared seal. Every
chamber's Watch demonstration is its charter text. Hint
ladders: four rungs each, the fourth naming the seam. The
keeper throughout is **Lockheart**.

1. **The Cell Heart.** A heart behind glass, worked by a
   flag machine — the Buffer's card posted beside it.
   The player walks a scripted day of six operations:
   open shared, open shared, close one, OPEN EXCLUSIVE —
   refused while a shared flag is out (the bell strikes,
   the heart survives; the refusal is a demonstration, not
   a failure), close all, open exclusive. The day is walked
   in order; the machine is a real flag simulation in
   puzzles.rs, and the bell is the simulation's refusal.
   Judge: all six walked AND the bell struck. Work:
   `day_walked: [bool; 6]`, `bell_struck: bool`.
2. **The Borrow-Flag Brazier.** Secondary #15 in
   miniature. Two porters share one brazier, lit lazily on
   first call. The player sequences five calls: A calls,
   B calls, A reads the flame, B calls again, both read.
   The brazier's light-count is shown after each call.
   Judge: all five walked AND the light-count stands at
   exactly 1 — lit once though both porters called —
   computed by a real lazy-cell simulation in puzzles.rs.
   Work: `calls: [bool; 5]`.
3. **The Spin Heart.** A heart that spins for a flag. The
   player (a) sets a spin bound (stepper 1..=9) and runs
   the heart with the flag unset: it gives up honestly at
   the bound — the run is a real bounded spin simulation
   in puzzles.rs; (b) annotates four ordering claims —
   payload write, flag store, flag load, payload read —
   each cycled through three authored orderings (drawn
   from Relaxed / Acquire / Release / SeqCst, exactly one
   strongest-correct per Floor 7's key). The annotations
   are marked PROVISIONAL on the chamber's face — Floor 7
   grades them; this chamber does NOT judge their
   correctness (DS-7). Work: `spin_bound: u8`,
   `gave_up: bool`, `spin_annotations: [u8; 4]`
   (0 = unrecorded). Judge: bound set, the honest give-up
   exhibited, all four annotations recorded.
4. **The Day Census.** A worked day of the heart: eight
   events — flag operations by three porters, timestamped
   as turn counts across four turns. The player submits
   the census of reachable flag-states across the day —
   counts per class: free / shared / exclusive — steppers
   per class. The truth is a REAL reachable-state
   enumeration in puzzles.rs: breadth-first over the day,
   turn by turn, every ordering of a turn's events
   explored, refused operations standing as self-loops;
   the census counts the distinct states of each class
   the day can reach. Judging is explicit (a judge
   press); a wrong census may be re-submitted. Work:
   `census: [u8; 3]`, `census_judged: bool`.
5. **Choosing the Cell.** Three data to house: a config
   read often and written rarely; a counter two porters
   increment; a log one hand appends. Three cell kinds:
   the plain cell, the many-readers lock heart, the
   borrow-flag heart. The player matches each datum to
   its kind AND writes the one-sentence invariant for
   each pairing (three authored sentences cycled per
   datum, one true — the sentence IS the deliverable).
   Judge: all matches right, all invariants true. Work:
   `choices: [u8; 3]`, `invariants: [u8; 3]`.

### The FOE — The Heart Locked Twice

The warren's great heart, whose flag-staff an
improving smith has re-cut to permit a writer flag and a
reader flag out simultaneously — a fused state the type
should have made unrepresentable.

- **Inscription** names failure class #7, the Two Charms.
- **Diagnosis (the exhibits, IN SEQUENCE):** (1) the
  staff's reachable-state census, exhibiting the fused
  state — a real enumeration over the broken staff's
  machine in puzzles.rs, its counts spoken from the
  computation, never typed; (2) the fused state reached
  in a worked day — the two porters' steps interleaved
  on the staff model to the torn read; (3) the heart
  read mid-write — the torn value itself, computed by
  the model. Exhibiting out of sequence is refused with
  the missing prior exhibit named.
- **The gate (the model-loom PREVIEW walk):** four named
  two-porter interleavings walked on a fixed loom-table
  (the full loom lands at Floor 7; this is a table, not
  the engine). Each interleaving is run on the staff
  model in puzzles.rs and judged sound or torn by its
  computation — a read that mixes two generations of
  the heart's two-word value is torn. `gate_passed` =
  all four walked.
- **Repair (the strike's substance):** the
  single-occupancy token staff taken from the chest —
  the staff's issue-rule card rewritten to its three
  truths (three lines, cycled): the staff is an enum —
  Free / Shared(n) / Exclusive; a writer flag issues
  only from Free; the count is part of the state.
  Strike: the shared two-step seal, armed only when
  proof stands complete, the gate stands walked, and
  the card stands rewritten.
- **Codex stone:** **The Lockheart Stone** — "One slot,
  one occupant — say so in the type."
- **Loot:** **The Lockheart Pair** — the borrow-flag
  heart and the spin heart, built from parts. Wielded on
  Floors 7 and 8.

## Steps

1. **Scaffolding.** Save format v6 (DS-2's one bump);
   `Floor6Progress` + `LockWork` (empty but the watched
   flag) + `Floor6FoeProgress`; `Screen::Floor6Hub`,
   `El::DoorFloor6` / `Floor6Door(u8)`;
   `geom::door_floor6()` on Floor 5's hub on verified
   open ground; `build_floor6_hub` in shell.rs; the
   map's Floor 6 square; `floor6_reachable()` gating the
   door on Floor 5 charted through its third chamber.
   The v6 fixture updates ride the bump, as they did
   for v5.
2. **Chambers 1–2** — the Cell Heart and the
   Borrow-Flag Brazier.
3. **Chambers 3–4** — the Spin Heart and the Day Census.
4. **Chamber 5 + the FOE** — Choosing the Cell, the
   exhibits, the loom-preview walk, the strike.
5. **Integration** — the shipped whole-floor GUI
   playthrough (all five chambers and the FOE on the GUI
   surface), full gates, the floor's close-out. Floor 6
   completes there; Stratum II's third FOE falls.

## Standing cautions (from the earlier lanes' record)

- Splicer anchors assert exactly once against the CURRENT
  tree before any write; an anchor carrying a closing
  brace puts inserted code outside the item; insert-before
  at a match-arm boundary can land between arms. A new
  `Screen` variant has more match sites than the obvious
  ones — exhaustiveness errors are the census; expect
  them in places like layout()'s tab-bar match.
- The local splice mirrors are unformatted: after shipping
  to primo, `cargo fmt` (apply) then `cargo fmt --check`;
  primo's committed tree is the fmt-clean source of truth.
- Theodolite: the locations file must exist and be current
  WHEN THE SESSION STARTS — producer (`bacon --headless
  -j bacon-ls`) first, then the bridge; a zero counts as
  verified only with a same-session-shape positive control.
  A warm-up diagnostics call is never evidence.
- Real facts only: any count the game asserts comes from
  a real simulation or enumeration in puzzles.rs — the
  census numbers, the light-count, the loom verdicts —
  never from a hand-typed answer table.
- Primo good-neighbour rules: one heavy job at a time,
  `nice -n 10`, the Owner's interactive work wins.
