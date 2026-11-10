# FLOOR 3 BRIEF — The Gatehouse (Niflheim completion programme)

You are building Floor 3 of Niflheim on top of the completed Floors 1–2
build (Floor 2 complete at commit 3b36aa8, branch
`pax/floor2-build-20261109`; the branch continues — do not fork a new one
without Pax's word). Local commits only — NEVER push. The per-rung gates
are the programme's: cargo test (counts grow, 0 failures), clippy
`-D warnings`, fmt, theodolite 0 error diagnostics on touched files,
determinism (two state-level playthrough runs, byte-identical saves),
Android lib build under the pinned toolchain (rustc 1.99.0,
`~/.cargo/bin` first on PATH). File the step report in
FLOOR3-STEP-REPORT.md in-tree as you go.

## The floor (design of record: FULL-DESIGN.md, Floor 3 + §5.2/§5.4/§5.6)

- Domain 3 — Idiomatic API design. Class: Gatewright. Stratum I — the
  valley's gatehouse, where everything the guild sends into the world
  is inspected, documented, and released.
- Theme: a release office: drafting desks, an audit wall with a public
  checklist, a seal press, and a great lantern over the gate.
- Learning objective: a public surface is a promise — errors named by
  kind, misuse made unrepresentable, documentation as part of the
  artifact, and a stated oldest road (MSRV) the surface can be
  travelled on.
- Prerequisites (§5.2): Floors 1 and 2 — `floor3_reachable()` is
  FOE 1 fallen AND FOE 2 fallen. (Reaching Floor 3 does NOT pass the
  Stratum I stairs; the stairs gate Stratum II, §5.6.)
- Navigation (coordinator ruling, Floor 3 step 1, reversible by Pax):
  each floor's door stands on its deepest prerequisite's hub, on open
  ground, never occluding text. Floor 3's door stands on Floor 2's
  hub, upper-right field, stairs-stone size: geom
  `door_floor3() = Rect::new(640, 150, 124, 124)`, El::DoorFloor3,
  drawn "FLOOR 3 / The Gatehouse" with an OPEN/BARRED tag like the
  Floor 2 door on Floor 1's hub (barred until floor3_reachable()).

### Chambers (ordinary encounters, in design order)

All five follow the standing resolution model (§5.4): work fields on
the chamber's progress record, a pure judge in puzzles.rs over
authored tables in content.rs, proof = the design's proof for that
chamber, repair through the shared seal. Hint ladders: four rungs
each, the fourth naming the seam.

1. **Error taxonomy — the Named Failures desk.** Travellers arrive
   with broken charters. For each traveller: name the failure's KIND
   (candidate kinds include one generic "Broken" — sorting to the
   generic bin is refused by the Gatewright, with the traveller's
   unanswerable question attached), then issue the remedy that kind
   prescribes. Judge: kind and remedy both match the authored table.
2. **Iterator internals — the Conveyor.** A belt portions goods
   lazily, one pull at a time. The player builds three attachments —
   portioner (take the first N), deduplicator (drop consecutive
   repeats), halt-after-failure (the belt stops at the first spoiled
   crate) — setting each attachment's parameter, then runs the belt
   and polls it past its end: it must keep answering "empty",
   forever. Judge: puzzles.rs runs the REAL iterator chain
   (authored goods incl. spoiled crates; take / dedup-by-consecutive
   / take-while-Ok semantics) and compares the delivered sequence;
   past-the-end polling is judged with a fused-iterator proof (after
   the first None, always None).
3. **Panic discipline — the Audit of Shouts.** The office clerk
   shouts (panics) at every malformed form. For each case on the
   audit wall: if a traveller can reach the failure, it must become a
   NAMED REFUSAL — the player names it; if only a broken office
   invariant can reach it, the shout may stand — the player writes
   (chooses) the invariant that stands behind the clerk. Judge per
   case: route and name/invariant match the authored table.
4. **Typestate builders — the Sealed Request.** Petitions are
   drafted on state-stamped paper: unaddressed → addressed →
   addressed-and-bodied; only the final state's paper fits the seal
   press, and attempts to seal early fail physically (the press does
   not close — a refusal, never a penalty). Petitions arrive in
   authored states (one pre-addressed, one bodied-but-unaddressed —
   its body cannot be sealed until addressed); the player performs
   the steps each petition still needs, in an order the states allow.
   Judge: every petition sealed, none sealed early (state machine in
   the work record; the seal act refuses until addressed && bodied).
5. **The Guidelines audit + the oldest road — the Inspection.** The
   player runs the office's full public checklist against the
   Interner's Hoard (Floor 1's loot, now a public artifact),
   recording per item: conforms / repaired / not-applicable-with-
   reason. Items whose verdict is "repaired" carry a small repair
   act (pick the correct doc line; mark the must-use; name the error
   kind). Then the Hoard's travelling papers are stamped with its
   OLDEST ROAD (the MSRV declaration): candidates are authored, the
   true one determined by the Hoard's own authored facts (the newest
   language feature its recipe uses). Side desk — the amendment
   rite: order the rite's steps (deprecate the old door, mark it,
   announce it for a full season, demonstrate the replacement, only
   then brick it up) and name the replacement door. Judge: checklist
   verdicts and repair acts, the stamp, and the rite's order all
   match the authored tables. Secondary beat (Stranger's Anvil, in
   miniature): the Hoard's recipe is re-forged by a stranger-smith
   from the paper alone; one line is missing — the player finds the
   gap (one of the checklist items IS the gap) — folded into this
   chamber's checklist as its final item; no separate screen.

### FOE — The Gatehouse Release

- Primary failure class **#5, The Lantern That Lights Itself** — a
  gate reporting green because the runner who would have reported
  red died first. Named explicitly in the fiction (the inscription,
  the scaffold's header, hint rung 4).
- The seam: the great release of the season — the Hoard's new
  edition, its lantern already green over the gate — was lit by the
  SCHEDULE, not the inspection: the keeper's chair is empty, and the
  log-slate shows the standing checks (formatting, lint, tests —
  §2.1's gates) were never run for this release.
- Diagnosis: check BOTH flames — the signal flame (green) and the
  keeper's flame (cold); read the log-slate, the record of checks
  actually performed; reconcile it against the release's claimed
  checklist (a matching interaction: claimed checks down one side,
  the slate's records down the other; the player marks each claimed
  check RUN (with its record) or NEVER RUN — judged against the
  authored slate, which records fmt and clippy as never run and one
  claimed test run as a record that does not exist).
- Proof required: the reconciled log-slate (every claimed check
  marked run-with-record or never-run), PLUS the release's two
  stamps graded: the oldest road, proven by an actual travelling of
  the Hoard on the earliest road claimed (in the gate, below); and
  one door carried through the amendment rite — deprecated, marked,
  announced for its full season, its replacement demonstrated
  (the rite performed on the room's side desk, judged as in
  Chamber 5 but on the release's own door).
- Repair strike: run the standing checks, FOR REAL, in the chamber —
  the third cargo gate. The release's crate (the Hoard's new
  edition) stands in the editor as the player's draft. Three
  lanterns over the desk, one per standing check, lit BY THE DOING:
  - **Format lantern:** run the formatter on the draft — the press
    straightens the paper: the draft is REPLACED by its rustfmt
    output (gate.rs `run_fmt`, bounded like every gate run), and
    the lantern lights when the draft stands formatted.
  - **Lint lantern:** `cargo clippy --all-targets -- -D warnings`
    on the draft (gate.rs partial run); the record shows the
    compiler's words; the player edits until it passes.
  - **Tests lantern:** `cargo test` against the hidden suite
    (validation + adversarial), as gates 1–2.
  Any edit to the draft douses all three lanterns (the standing
  unfile rule, generalized). GateReport gains fmt_checked /
  fmt_passed; `passed()` = tests && clippy && (!fmt_checked ||
  fmt_passed), so gates 1–2 are untouched in behaviour.
  The gate crate is the Hoard's new edition: its fiction is a
  small public surface (the Hoard as a library: store/retire/tally
  with named error kinds) — the scaffold ships documented,
  formatted-but-one-function-wrong, AND clippy-dirty in one named
  way the player must fix by editing (the tests lantern cannot
  pass until the bodies are written; the lint lantern cannot pass
  until the named wart is removed).
  The OLDEST ROAD is proven inside the tests lantern: the hidden
  suite is compiled as edition 2021 with no post-1.xx features
  beyond the stamped road — the stamp the player chose in the
  proof is checked by keeping the suite's language level at the
  stamped road's level (the record states the road travelled).
  (Honest bound: the gate's toolchain is the pinned one; the
  "travelling" is a language-level proof, and the fiction's slate
  records it as such.)
- When the slate is reconciled, the rite performed, and all three
  lanterns stand lit on the current draft, the strike arms: the
  keeper's flame is lit by the doing, and the player lights the
  great lantern BY HAND through the shared seal (two-step, as
  every strike).
- Codex stone: **The Lantern Stone. "A result unexamined is a gate
  unrun."** (text exact, per FULL-DESIGN §Floor 3).
- Loot: **The Gatehouse Key** — the Hoard raised to release grade
  (pack). Wielded later: Floor 17 (the publication rite is
  performed on this artifact) and everywhere as the standing
  example of "done".
- Stairs relevance (§5.6): FOE 3 is one of the two Stratum I FOEs
  (2 or 3) the stairs accept; its lantern joins clause 3. Floor 3's
  steps must extend stairs_clauses(): clause 2 lit when FOE 2 OR
  FOE 3 has fallen (text names the fallen one(s)); clause 3's
  lanterns: lit(f) = f's gate passed OR f's five chambers repaired
  (the §5.6 gloss — the remaining valley FOE's lantern is lit by
  completing its chambers, FOE attempt not required); clause 3
  lights when all three Stratum I lanterns stand lit. The Floor 2
  era texts are superseded accordingly; keep the variants honest
  per state.

### Steps (mirror Floor 2's build order)

1. Scaffolding: save v3 (Expedition gains `floor3`, serde-defaulted
   so v2 saves load; version bump with the version_one/two load
   tests extended), Floor3Progress skeleton (five chambers with a
   generic work record to be specialized per step, FOE stub with
   slate/rite/lantern fields), floor3_reachable, Screen::Floor3Hub
   + DoorFloor3 on Floor 2's hub + hub screen (keeper panel,
   chamber doors barred in order), stairs_clauses extension,
   map square for Floor 3, lib.rs doc.
2. Chambers 1–2 (Named Failures; the Conveyor).
3. Chambers 3–4 (Audit of Shouts; the Sealed Request).
4. Chamber 5 (the Inspection) + the FOE (slate, rite, gate 3).
5. Integration + driven playthrough evidence for the whole
   floor (the Floor 2 step-5 pattern: an in-suite GUI
   playthrough test + Xvfb screenshots; keyboard drive —
   XTEST pointer events do not reach this shell's cursor
   polling under Xvfb; record the channel honestly).

## Standing cautions (from the Floor 2 lanes' record)

- The editor's two-draft routing is floor-keyed on screen; a third
  draft (the release crate) needs its own record on Floor3FoeProgress
  and the accessor extended — remember Floor 2 Step 5's lesson: an
  "empty means untouched" draft convention resurrects the scaffold
  over a select-all delete. Use an explicit started flag from birth.
- Every new screen's elements must be IN layout() — Floor 2's door
  was drawn for two steps before anyone could click it. The
  integration playthrough is not optional garnish; it is the gate
  that catches this class.
- Theodolite starts its server lazily: warm-up diagnostics call
  first, never cited as evidence.
