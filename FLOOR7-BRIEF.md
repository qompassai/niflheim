# FLOOR 7 BRIEF — The Loom of Many Hands (Niflheim completion programme)

You are building Floor 7 of Niflheim on top of the completed
Floor 6 build (Floor 6 complete at commit a6e3de2, branch
`pax/floor2-build-20261109`; the branch continues — do not fork a
new one without Pax's word). Floor 6's boundary is Pax-verified:
suite re-run 191 passed / 0 failed, determinism saves
byte-identical (sha256 6386a8b7…), earned save at version 6.
Local commits only — NEVER push. The per-rung gates are the
programme's: cargo test (counts grow, 0 failures), clippy
`-D warnings`, fmt, theodolite 0 error diagnostics on touched
files (verified zeros: producer first, same-session-shape
positive control, per the cured Rust route), determinism (two
state-level playthrough runs, byte-identical saves), Android
lib build under the pinned toolchain (rustc 1.99.0,
`~/.cargo/bin` first on PATH), Xvfb launch smoke. File the
step report in FLOOR7-STEP-REPORT.md in-tree as you go.
Method (D-1189): Claude-direct — coordinator-authored blocks +
splicer, anchors asserted exactly once before any write, as
Floors 2–6. Build contract of record: the completion spec
(~/workspace/niflheim-completion-spec-20261110/SPEC.md), §3
standing pattern + §4 Floor 7; its DS decisions bind.

## The floor (design of record: FULL-DESIGN.md, Floor 7 — The Loom Halls + §5.2/§5.4/§5.6)

- Domain 7 — Concurrency & atomics. Class: Threadweaver (the
  keeper is **Threadweaver**). Stratum II — long halls of
  looms where many hands pull one thread, and the air is full
  of almost-simultaneous events.
- Learning objective: concurrency is a contract discipline —
  orderings are chosen with written happens-before arguments,
  queues are bounded with a stated full-policy, and deadlock
  is a shape that a global order makes impossible.
- **Primary failure class: #15, The Two Porters** — deadlock
  as a cycle in the waits-for graph. Secondary class **#13,
  Tide Table**, lives in Chamber 3 in miniature.
- Prerequisites (§5.2): Floor 5. `floor7_reachable()` =
  `floor5_complete()`. (Floor 6 is recommended — its hearts
  are this floor's furniture — advisory text only, never a
  gate.) The door's refusal speaks the unmet condition, never
  a buzz.
- Navigation (standing coordinator ruling): each floor's
  door stands on its deepest prerequisite's hub, on open
  ground, never occluding text. Floor 7's door stands on
  **Floor 5's hub** (SPEC §4 Floor 7); `geom::door_floor7()`,
  stairs-stone size (124×124), `El::DoorFloor7`, drawn
  "FLOOR 7 / The Loom of Many Hands" with an OPEN/BARRED tag.
  The spec does not name the spot: Floor 6's door already
  stands at the (640, 150) ground Floor 5's door proved, so
  choosing and verifying this door's open ground — against
  the hub's camp stone, chamber row, the FOE door below, and
  Floor 6's door — is a Step 1 design task, recorded in the
  step report like Floor 6 Step 1 recorded its spot.
- **SYSTEM LANDED HERE (frozen for Part B, §5): the model
  loom** — `puzzles::loom`: authored thread programs as op
  lists (`LoomOp`: Send / Recv / Claim / Release / Join over
  named channels and claimables); `explore(programs, cap)`
  enumerates interleavings in deterministic (lexicographic
  by thread-choice) order up to a stated cap and returns
  `LoomReport { interleavings_tested: u64, deadlock:
  Option<Vec<u8>>, outcomes: BTreeSet<Vec<u8>> }` (final
  channel-state signatures). Deterministic, bounded, no RNG.
  Consumed by this floor's C5/FOE and by F16 C4. The loom
  module skeleton + its first unit tests land in Step 1,
  not Step 4 (SPEC §4 Floor 7, Steps).
- The Lockheart Pair (Floor 6's loot — the borrow-flag heart
  and the spin heart, built from parts) is wielded here: its
  hearts are this floor's furniture, and two of the FOE's
  four claimables (the brazier, the staff) are Floor 6's own.
- Floor 6's four spin annotations ride the save into this
  floor PROVISIONAL (recorded at Floor 6 Chamber 3). **DS-7
  binds: they are never re-judged.** Chamber 3 displays them
  beside the wall's own four claims; the wall judges only its
  own annotations. (Re-judging would soft-lock a player who
  annotated provisionally-wrong at F6.)
- This FOE carries **no cargo gate**: `gate.rs` is untouched
  (DS-4). The FOE's 1,000,000-message exact-set proof is a
  real cargo test over the puzzles queue model (DS-13) — a
  suite test, not a gate.rs surface.
- `#![forbid(unsafe_code)]` stands. Every mechanism is a real
  computation in puzzles.rs over authored tables — the loom
  exploration, the strong/weak counter simulation, the queue
  model — never hand-typed answer tables (DS-13).
- Mastery note (FULL-DESIGN): from here down, an ordering
  may be chosen only with a written happens-before argument;
  "strongest ordering everywhere" is allowed — as a stated
  decision with a stated cost, never as an unexamined
  default. Chamber 3's authored claims are written under it.
- Platform (FULL-DESIGN): the waits-for graph is the floor's
  signature diagram — it must render legibly at phone width
  with labelled (not colour-only) cycle highlighting
  (§6.7, §7.3). No live-timing input anywhere: the model
  loom is turn-stepped.

### Chambers (ordinary encounters, in spec order)

All five follow the standing resolution model (§5.4): work
fields on the chamber's progress record, a pure judge over
authored tables in content.rs, proof = the spec's proof for
that chamber, repair through the shared seal. Work record:
`LoomWork`. Hint ladders: four rungs each, the fourth naming
the seam. The keeper throughout is **Threadweaver**.

1. **The Scoped Loom.** Six thread-cards are borrowed for one
   task; every thread must be joined before the task's scope
   closes. The player arranges the join order (steppers
   assign each thread a join slot 1..=6; the scope's own two
   threads have fixed lifetimes the order must respect — two
   threads end early and cannot be joined late). Judge: the
   arrangement is a valid scoped order (all joined inside
   the scope, early threads joined before their expiry
   slot). Work: `join_order: [u8; 6]`, `scoped: bool` (set
   by the judge press).
2. **The Border Seals.** Six tools to carry across the
   thread border; each tool's card states its parts. The
   player seals each tool Send / Sync / Neither. One tool is
   phantom-sealed (its card's fine print names a raw pointer
   inside) — it is Neither. Judge: all six seals right.
   Work: `seals: [u8; 6]`, `judged: bool`.
3. **The Charter Wall.** Secondary #13 in miniature: one
   wall claim is a stale tide (its annotation predates a
   change to the spin heart). The F6 spin heart's four
   provisional annotations are displayed beside the wall's
   own four claims (DS-7: displayed, never re-judged — no
   soft-lock). The player (a) places the happens-before
   edges: five candidate pairs, exactly four are true edges;
   (b) marks the one weakened-ordering claim; (c) finds the
   stale tide claim. Judge: edges exactly the true set,
   weakened claim marked, stale claim found. Work:
   `edges: [bool; 5]`, `weak_marked: u8`,
   `stale_found: bool`, `graded: bool`.
4. **Arc and Weak Echoes.** A counter held by many hands,
   and echoes that may outlive it. An authored eight-step
   script (clone / drop / upgrade attempts); the player
   predicts each step's outcome (upgrade succeeds / fails;
   counts after). The referee runs a REAL strong/weak
   counter simulation in puzzles.rs (a counted model, not a
   table). Judge: all eight predictions match the simulated
   trace. Work: `predictions: [u8; 8]`, `judged: bool`.
5. **The Lock-Free Stack.** Linearizability spot-check: six
   recorded histories (push/pop with turn-count intervals).
   The player marks each history linearizable or not; for
   the one non-linearizable history, names the witness pair
   (two operations whose order cannot be reconciled).
   Judge: all six verdicts right + witness right. Work:
   `verdicts: [u8; 6]`, `witness: u8`.

### The FOE — The Loom of Many Hands

Two servants claim two service stairs in opposite orders;
the waits-for graph has a cycle of two.

- **Diagnosis (the exhibits, IN SEQUENCE):** (1) each
  servant's claim order exhibited; (2) the waits-for graph —
  the player connects the four waits-for edges; (3) the
  deadlocking interleaving RUN on the loom (a real `explore`
  call from the GUI path; the deadlock trace is exhibited).
  Exhibiting out of sequence is refused with the missing
  prior exhibit named.
- **The gate:** (i) the message walk — the Arc & Queue
  carries three stated loads (authored counts), the
  received set reconciles exactly each time (the
  1,000,000-message exact-set proof is a real cargo test
  over the puzzles queue model — DS-13; the GUI walks the
  reconciled summaries); (ii) at-capacity back-pressure —
  the queue filled to capacity; the next send WAITS
  (exhibited as a pending state, never a drop, never a
  spin).
- **Repair (the strike's substance):** a single total claim
  order posted — the player orders the floor's four
  claimables (two stairs, the brazier, the staff) into one
  total order; judge: a valid total order (all four, no
  repeats) AND both servants' claim sequences consistent
  with it. Strike: the shared two-step seal, armed only when
  proof stands complete, the gate stands walked, and the
  order stands posted.
- **Codex stone:** **The Loomstone** — "A global order
  forbids cycles." (SPEC §4 Floor 7 names the stone so;
  FULL-DESIGN's Floor 7 calls it The Porters' Stone with the
  same text. Build to the SPEC — the contract of record —
  and flag the name variance in the step report; do not
  silently harmonize it.)
- **Loot:** **The Threadweaver's Arc & Queue** — a
  jointly-owned counter and a bounded many-handed queue,
  model-verified. Wielded on Floors 8 and 14.

## Steps

1. **Scaffolding.** Save format v7 (DS-2's one bump);
   `Floor7Progress` + `LoomWork` (empty but the watched
   flag) + `Floor7FoeProgress`; `Screen::Floor7Hub`,
   `El::DoorFloor7` / `Floor7Door(u8)`;
   `geom::door_floor7()` on Floor 5's hub on verified open
   ground (see Navigation — the spot is this step's design
   task); `build_floor7_hub` in shell.rs; the map's Floor 7
   square; `floor7_reachable()` = `floor5_complete()`
   gating the door. The loom module (`puzzles::loom`)
   skeleton + its first unit tests land HERE. The v7
   fixture updates ride the bump, as they did for v6 —
   Floor 6's records, spin annotations included, carry
   forward unjudged.
2. **Chambers 1–2** — the Scoped Loom and the Border Seals.
3. **Chambers 3–4** — the Charter Wall and Arc and Weak
   Echoes.
4. **Chamber 5 + the FOE** — the Lock-Free Stack, the
   exhibits, the loom-run deadlock, the message walk and
   back-pressure, the strike.
5. **Integration** — the shipped whole-floor GUI playthrough
   (all five chambers and the FOE on the GUI surface), full
   gates, the floor's close-out. Floor 7 completes there.

Per-step gates (the Floor 6 set, evidence in
`~/.local/share/pax/niflheim-floor7-step<N>-build-20261110/`
on primo): `cargo fmt --check` clean; `cargo test` — counts
grow from 191, 0 failures, no test deleted, skipped, or
weakened; `cargo clippy --all-targets -- -D warnings` exit
0; theodolite (cured protocol) — planted run seen in-session,
restore sha256-verified, clean run count=0 on all touched
files; determinism — the state-level playthrough probe run
twice, saves byte-identical (probe transient per DS-9:
copied into `tests/` for the gate run, removed after, never
committed); Android `cargo build --target
aarch64-linux-android --lib` exit 0; Xvfb smoke — zero
panics, cold-start recorded; `#![forbid(unsafe_code)]`
intact. Per-step local commit on the branch; this brief is
committed with the Step 1 commit, as Floor 6's was.

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
  A warm-up diagnostics call is never evidence. The lane's
  bacon producer session is `niflheim-bacon` on primo
  (started at dispatch; Floor 6's was stopped at its
  boundary) — if it is gone, restart it before the first
  theodolite session of a step, and stop it at this floor's
  boundary.
- Real facts only: any count the game asserts comes from a
  real simulation, enumeration, or exploration in
  puzzles.rs — the loom's interleaving counts and deadlock
  traces, the counter's trace, the queue's reconciliations —
  never from a hand-typed answer table.
- Primo good-neighbour rules: one heavy job at a time,
  `nice -n 10`, the Owner's interactive work wins.
