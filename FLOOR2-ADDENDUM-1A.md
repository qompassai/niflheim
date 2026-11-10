# ADDENDUM — Step 1 split (Pax, after the first run's measured anatomy)

The first Step 1 run died in the planner: 6 iterations spent on directory
listing (two calls wasted on an empty path argument) and brief reading;
no plan was emitted and no code was written. The contract is therefore
tightened — the plan is authored here; the run executes it.

## File map of the Floor 1 build (do not re-survey)
- `src/lib.rs` (35) — crate root, layering. `src/main.rs` (5).
- `src/core.rs` (505) — engine-independent core: bounded interner,
  zero-copy line index, plan census.
- `src/content.rs` (761) — authored content data for Floor 1.
- `src/puzzles.rs` (437) — pure proof validators.
- `src/state.rs` (1107) — game state + versioned save (NIFLHEIM-SAVE v1,
  validate-then-commit, forged-state rejection, autosave).
- `src/gate.rs` (599) — the real cargo gate (bounded scratch project,
  validation + adversarial tests, output cap, deadline, group kill).
- `src/game.rs` (2245) — Bevy game logic: title/hub/chart, the encounter
  loop (diagnose → proof by kind → refusal with reason → two-step seal →
  4-rung hints with facet forfeiture).
- `src/shell.rs` (3917) — UI shell: screens, editor overlay, input.
- `src/platform.rs` (107) — platform glue.
Tests are inline plus the suites counted in Floor 1's report (44 total).

## Tool-call discipline (planner and coder)
- `file_list` requires a NONEMPTY relative path argument. Use "." for
  the workspace root. An empty path is a wasted turn.
- Do not list directories to explore: the map above is complete. Read
  only the files the current step names.

## Step 1A (this run) — scaffolding only
1. `src/state.rs`: add Floor 2 state — the five Menagerie chambers'
   progress, the roster/admissions state the chambers will use, FOE
   state — and extend the save honestly (version the format; Floor 1
   saves must still load and validate; forged-state rejection intact).
2. `src/content.rs`: add the Floor 2 content structures + the floor's
   static data (chamber names/order, keeper, codex/loot ids) — no
   chamber encounter content yet.
3. `src/game.rs`: register Floor 2 — the Ferronia chart shows The
   Menagerie Grounds; hub entry exists; stairs state reflects the
   design (Floor 2 reachable once Floor 1's FOE has fallen; Stratum I
   stairs still require one of FOE 2/3).
4. `src/shell.rs`: the Floor 2 entry screen with the grouped,
   collapsible roster UI pattern (empty rosters are fine at this step),
   touch + keyboard parity, no hover-only information.
5. Tests: state/save round-trip for Floor 2 state, Floor 1 save
   compatibility, registry/chart state. Floor 1's 44 tests intact.
Gates and commit per the brief (`Floor 2 step 1A: scaffolding`), step
report appended to FLOOR2-STEP-REPORT.md.

## Step 1B (next run, Pax dispatches after gating 1A)
Chamber 1 (Traits and impls) fully playable with tests, per the brief.
