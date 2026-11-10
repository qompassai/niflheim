# FLOOR 2 BRIEF — The Menagerie Grounds (Niflheim, local-model programme, D-1145)

You are building Floor 2 of Niflheim on top of the completed Floor 1 build
(commit 9e40c0a, branch pax/floor1-build-20261109). Work in this repository
on branch `pax/floor2-build-20261109`. Local commits only — NEVER push.

## The floor (design of record: FULL-DESIGN.md, Floor 2)

- Domain 2 — Traits & the type system. Class: Sigilwright. Stratum I.
- Theme: glass plugin-houses; each beast (plugin) wears a collar of sigils
  (traits) stating what it may do. Two admission doors: the Narrow Door
  (kind known individually — generics/static dispatch) and the Wide Door
  (any beast answering to a sigil — trait objects/dynamic dispatch).
- Learning objective: a trait is a contract of behaviour; a bound is the
  full price of admission; the same service is reachable by two dispatch
  doors at different costs.

### Chambers (ordinary encounters, in design order)
1. **Traits and impls** — teach a beast a chartered trick from its
   sigil-book; one beast performs its own variant of the house trick; the
   player identifies which body ran by watching behaviour, not by label.
2. **Generics and bounds** — the Generic Stable: stalls accept any beast
   meeting the posted bound. The player starts from an over-strict bound
   and loosens/tightens it per service, justifying each sigil demanded. A
   bound carrying an unjustified sigil is refused by the Sigilwright with
   the reason shown.
3. **Associated types** — Two Kennels: the same courier service from two
   kennels, one where the kennel chooses its beast-kind, one where the
   customer does. The player runs deliveries from both and finds the
   errand natural in one and contorted in the other.
4. **Trait objects and the Wide Door** — the registry: heterogeneous
   beasts behind one sigil, driven through one service routine. Three
   beasts cannot pass the Wide Door, each barred for a different named
   reason on its placard (not object-safe, generic method, Sized/self
   return — per Rust's actual object-safety rules).
5. **Coherence** — Foreign Marks: a foreign beast and a foreign sigil may
   be joined only through a guild-issued collar (a newtype) of the
   player's own making. A staged collision — two collars claiming the
   same sigil for the same beast — is shown and read as a specification,
   not an error.
- Secondary beat (Herald, miniature): sealed letters arrive through the
  floor; one announces a service the beast never offered. Acting on it
  wastes a free miniature expedition — the floor's first wrong-room
  signal.

### FOE — The Menagerie (the house itself, as one mechanism)
- Seam: the admission desk checks appearance where the charter demands a
  sigil: a mist-hound was admitted to the fire-watch because it is grey
  like the ash-hounds; it reports the valley's weather as smoke.
- Diagnosis: audit admissions against charters — for each working beast,
  exhibit its sigils against the service's bound; then calibrate the
  suspect against one known landmark task (a lit brazier) and watch it
  report fog.
- Proof required: the admission audit — every service's roster with each
  beast's satisfying sigils exhibited, plus the calibration record.
- Repair strike: re-admit by bound, not resemblance — re-post the
  fire-watch bound at the desk (a mechanism change, not a beast change);
  the mist-hound is honourably reassigned to the mist-watch it is
  actually sigilled for; the desk thereafter refuses domain-less
  admissions.
- The FOE gate is REAL, as on Floor 1: the player's repair is expressed
  as actual Rust (a bound + admission predicate) and checked by the
  bounded cargo gate (scratch project, validation + adversarial tests,
  output cap, deadline, process-group kill) — never a quiz.
- Codex stone: The Lens Stone. "Bounds are admissions." Loot: The Plugin
  Menagerie — the two-door host (field beasts by kind or by sigil);
  carried in the pack, wielded on later floors.
- Stairs relevance: one of the two FOEs (2 or 3) the Stratum I stairs
  require.

### Platform / UI notes for this floor
- Audit lists are long: grouped, collapsible rosters, identical on phone
  and desktop; NO information may exist only in hover text; full touch
  parity; no timed elements anywhere on this floor.

## Standing patterns (Floor 1's, binding)
- Layering: engine-independent core (bounded, zero-copy where Floor 1 is)
  + authored content + pure proof validators + versioned save + Bevy
  shell. `#![forbid(unsafe_code)]`.
- Encounter loop: diagnose → proof slots by kind → wrong evidence refused
  WITH the reason → two-step seal → 4-rung hints with facet forfeiture.
- Save: `NIFLHEIM-SAVE` format, validate-then-commit, forged-state
  rejection, autosave on every state change. Floor 2 state extends the
  save without breaking Floor 1 saves (version honestly).
- Accessibility: no state conveyed by colour or glyph alone; glyphs only
  from the covered set (● ○ ■ ×) plus words. No audio.
- Run from the repo root (fonts resolve via Bevy's asset root).
- Tiger Style Rust: explicit contracts, bounded work, no hidden
  behaviour; tests split half validation / half adversarial.

## Step plan — ONE step per run; Pax gates between steps
- STEP 1: Floor 2 scaffolding (floor exists on the Ferronia chart + hub
  entry + stairs state; roster UI pattern: grouped/collapsible) +
  Chamber 1 fully playable with tests.
- STEP 2: Chambers 2 and 3. STEP 3: Chambers 4 and 5 + Herald beat.
- STEP 4: The FOE (audit, calibration, real cargo gate, repair, codex,
  loot, stairs wiring). STEP 5: integration gates + playthrough evidence
  + report (Pax dispatches it when 1–4 are gated).

## Per-step gates (all must pass before the step's commit)
- `cargo fmt --check` clean; `cargo clippy --all-targets -- -D warnings`
  clean; `cargo test` fully green with Floor 1's 44 tests intact and
  passing (regression = step failure).
- Commit on `pax/floor2-build-20261109`, message `Floor 2 step N: <what>`.
- Append a step report to `FLOOR2-STEP-REPORT.md` in the workspace root:
  what was built, gate numbers, anything refused or left open.
- If a gate cannot pass: STOP, record the blocker in the step report, do
  not weaken a test, do not improvise around the design.
