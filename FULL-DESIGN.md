# Niflheim — Full Design

*The production design for the Niflheim game. Companion to `CURRICULUM.md` (the syllabus), `MAP.md` (the tracker), and `GAME-DESIGN.md` (the concept design). Where the concept design sketched, this document decides: every floor is fully specified, every failure class in the concept's mapping table is assigned to a floor and a FOE, and the systems, presentation, platform, and production rules are stated as requirements a build team can be held to.*

*Status: full design pass, 2026-11-03. This document contains no exercise solutions and no tutorials; it designs a game, it does not walk anyone through the syllabus. All names, places, and problems in it are invented and abstract.*

---

## 1. Product definition

### 1.1 The product in one paragraph

Niflheim is a single-player, turn-considered dungeon-descent RPG about repairing mechanisms. The player is a delver-cartographer of a guild that maintains the Great Labyrinth — the water gates, tally bridges, signal towers, ice roads, and ward engines of Ferronia, a hostile iron country — and descends through four strata and seventeen floors to the sealed depth called Niflhel. Each floor is a domain of the Rust craft wearing a class's name; each floor's boss (a FOE) is a great mechanism broken by one specific failure class. FOEs are never defeated by attrition: the player diagnoses the seam, produces the proof the floor demands, strikes the repair, and earns an artifact, a principle stone for the codex, and the right to draw that floor on the map. The game is the curriculum drawn at walking scale — the same seventeen domains, the same prerequisites, the same stairs gates — so that finishing the game and finishing the descent are two drawings of one achievement.

### 1.2 Genre

- **Primary genre:** dungeon-descent RPG in the Etrian manner — first-person-considered exploration of a gridded labyrinth, party-of-one, progress by charting, bosses as gates rather than milestones.
- **Secondary genre:** diagnostic puzzle. The moment-to-moment verb is not *hit* but *examine, reconcile, prove, repair*.
- **Explicitly not:** an action RPG (no reflex combat), a quiz game (no question is ever answered by selecting text), a coding game (the player never types Rust into the game; the game teaches the principles Rust embodies, as physics).

### 1.3 Audience

- **Primary:** programmers and programming students who know some code and want Rust's mental model in their hands — including players who will go on to the curriculum itself.
- **Secondary:** RPG players who have never written a line of code. The game must be fully winnable, and fully enjoyable, by a player who never reads the curriculum and never opens the codex's deeper inscriptions.
- **Not targeted:** children (the reading level is advanced, though the content is entirely wholesome), speedrunners (determinism is a fairness property, not a leaderboard), or competitive players (no multiplayer, no rankings).

### 1.4 Player fantasy

You are not the strongest person in the labyrinth; you are the most careful. Your weapons are attention, exactness, and a growing codex of principles. Where an ordinary hero out-damages a broken mechanism, you out-diagnose it: you find the seam where it fails, you prove what is actually true, and you repair it so the *failure class* — not just the failure — is gone. Every victory leaves the world measurably more trustworthy and leaves you holding a tool, a principle, and a map square you drew yourself and can defend. The fantasy's emotional core: **competence that can be pointed at.** Nothing you earn is a number going up; everything you earn is a thing you can show.

### 1.5 Pillars

1. **Proof, not claim.** No square fills, no FOE falls, no gate opens on the player's say-so or on an indicator's say-so. Every advancement is backed by an artifact the game itself can display: a reconciled ledger, a held witness, a matched pair, a verified keep. If a system ever asks the player to trust a light, the pillar is violated.
2. **Diagnosis before intervention.** Every challenge is winnable by noticing, and the intervention is locked until the diagnosis is proven to the floor's standard. Haste is never necessary; attention is always sufficient.
3. **The failure class dies, not the instance.** Repairs remove the *shape* of a failure — a type that cannot be half-built, an order that cannot cycle — so a repaired floor stays repaired and a learned principle transfers to floors that never taught it.
4. **The map is yours because you drew it.** Cartography is a core system, not a cosmetic. The map is evidence: late in the descent, other characters ask to see it before they trust the player's word, and they are right to.
5. **Principles are physics.** Rust's principles are taught as the way the world behaves, encountered in the hands before they are ever read in the codex. Text confirms experience; it never substitutes for it.
6. **One descent, three doors.** The game is identical in content, difficulty, and outcome on phone, Linux desktop, and Windows desktop. Platform changes the window, never the labyrinth (see §7).

### 1.6 Core loop

The loop of record, unchanged from the concept design, with state made explicit:

1. **Enter** a floor whose prerequisites are charted. State: floor becomes ACTIVE; its five chambers are revealed in order, each initially UNWORKED.
2. **Learn the mechanism by watching it work once.** Each chamber opens with the floor's mechanism shown functioning correctly — a demonstration the player can replay at will. State: chamber DEMONSTRATED.
3. **Work the chamber's challenge.** A diagnosis-first puzzle built on the chamber's mechanism and, per the floor spec (§4), a woven failure class. The repair action is present but *locked*; it unlocks when the player assembles the chamber's proof (see §5.4 for the proof model). State: challenge moves UNWORKED → DIAGNOSED (proof assembled) → REPAIRED (strike landed). Only REPAIRED advances the chamber.
4. **Face the FOE** when all five chambers are REPAIRED (or earlier, at a stated penalty of information: the FOE may be *attempted* at any time, but its seam telegraphs live in the chambers — see §1.9). The fight is the same three-beat shape at boss scale: diagnose under the FOE's pressure, assemble the floor-scale proof, strike the repair. State: FOE STANDING → EXPOSED (seam proven) → FALLEN.
5. **Take the loot and the stone.** FOE FALLEN yields the floor's artifact (equipment, §5.3) and its principle stone (codex, §5.4 of the concept design; mechanics in §5.5 here).
6. **Draw the map.** The floor's squares fill — chamber squares on REPAIRED, the floor square on FOE FALLEN — under the marking convention (§5.1). 
7. **Descend** when the stairs gate (§4, per stratum) holds.

### 1.7 Session loop

A session (target length 15–45 minutes on phone, unbounded on desktop) is: arrive at a rest point (guildhall or a floor's camp) → choose a floor whose prerequisites are charted → work one to five chambers or one FOE → return or camp → the game saves, shows the map as it now stands, and offers the codex at the fire. Design requirements:

- **Any single chamber is completable in 3–10 minutes** and is a valid stopping point; the game never holds progress hostage to a long unbroken stretch.
- **Resume is exact:** closing the game mid-chamber and returning restores the chamber's state, including partial proof assembly, with no penalty and no repeated demonstration (§7.4).
- **A session always ends with the map.** The last thing the player sees is their own chart — what filled, what stands blank. Blank is honest, and the game says so.

### 1.8 Campaign loop

Across the descent: floors open by prerequisite (never by level or currency); loot from fallen FOEs is wielded by name on deeper floors, so the campaign's difficulty curve is a *knowledge* curve — later floors are harder because they combine earlier mechanisms and earlier failure classes, not because numbers grew. The guildhall grows with the player: keepers remember what was proven, the workshop's recipe wall fills with the recipes of artifacts the player has sealed, and NPC trust is gated on the map (§5.9). The campaign ends at Niflhel (§5.10). There is no experience-point grind and no gear treadmill; a floor is finished when its proof is produced, and not before.

### 1.9 Failure and retry philosophy

- **Failure is information, priced at zero.** A failed diagnosis, a struck repair that misses the seam, a FOE attempt that collapses — each costs nothing but the attempt, and each returns *specific* knowledge: the game always shows which part of the proof did not hold, in the world's terms (the warden names the double-counted crate; the lantern's keeper-flame is shown cold). Failure is never communicated as a bare red X.
- **No fail states that end a run.** There is no permadeath, no lost progress, no resource spiral. The worst outcome of any action is *the world stays as broken as it was, and you know more about why*.
- **FOE attempts are free and repeatable.** A FOE can be attempted before its chambers are finished; the attempt is legal but under-informed — the seam telegraphs are chamber content. A collapsed attempt marks nothing on the map and consumes nothing. This mirrors the curriculum, where a failed gate run costs nothing but the run.
- **Wrong interventions are survivable exactly once per teaching beat, then instructed.** Where the concept design's puzzles let the player "sail by a wrong measurement exactly once," the rule is: the first wrong strike produces the *consequence in miniature* (a demonstration-scale version of the harm, nobody hurt), and the floor then names what happened. The game teaches prices by showing small bills, never by ambush.
- **Retries never repeat unskippable content.** Demonstrations, dialogues, and set-pieces seen once are skippable forever.

### 1.10 Difficulty model

Difficulty is **diagnostic depth**, not execution demand:

- **Depth of combination.** Stratum I floors weave one failure class into one mechanism. Stratum II adds pressure (the FOE acts while the player diagnoses). Stratum III floors combine the floor's primary class with a secondary class in the FOE. Stratum IV FOEs combine two classes and add a *false seam* — a plausible wrong diagnosis the game will let the player fully assemble before showing why it cannot hold.
- **No reflex difficulty.** Nothing in the game is gated on reaction time, input precision, or frame timing, on any platform (§7.7). Timed elements (the Ice Ledger escort) run on a generous, pauseable clock and are about routing decisions, never dexterity.
- **No hidden difficulty.** A floor's spec (§4) states its classes openly in the codex once encountered; the game hides *instances*, never *rules*.
- **Assistance is structural, not a mode.** There is no difficulty selector. Help comes from the hint system (§1.11), the codex, and keeper dialogues — all available to every player at all times, so "difficulty" is a property of how much of the game's own help the player chooses to use.

### 1.11 Hint philosophy

Hints are a ladder the player climbs down voluntarily, rung by rung, per challenge:

1. **The Keeper's Remark** — free, always available: the floor's keeper restates the mechanism in the world's terms ("The warden doesn't distrust your arithmetic. He distrusts your *paper*.").
2. **The Codex Nudge** — free: the game names the stone (if earned) or the principle (if not) that governs this seam, without applying it.
3. **The Worked Parallel** — free: a *different, smaller instance* of the same failure class, shown solved, in a side alcove. The player still diagnoses their own instance.
4. **The Seam Named** — costs the challenge its *full-standard* mark (the chamber still completes and the floor still finishes; the forfeited mark only affects the optional Cartographer's Standard record, §5.8): the game names the seam and the missing proof element, but the player still assembles the proof and strikes the repair themselves.

Hints never solve, never strike, and never fill a map square. A player who uses rung 4 on everything can still finish the game and reach Niflhel; the record simply shows how the descent was made.

### 1.12 Learning without lectures

The game teaches Rust's principles the way the concept design specifies — as physics — under these production rules:

- **Mechanism before name.** The player *uses* provenance for two floors before the word appears in a codex stone; uses single-occupancy sockets before hearing "enum"; routes traffic off a frozen river before reading "borrow." Names arrive as recognition, not introduction.
- **No quizzes, ever.** There is no point in the game where the player is asked a question about a principle and graded on the answer. Assessment is entirely performative: if you repaired it, you understood it enough.
- **The codex is a mirror, not a textbook.** Stones are inscribed *after* the proof that earns them (§5.5), phrase the principle plain-first, then at mechanism depth, and exist to be re-read at fires — where deeper floors' keepers will casually assume them.
- **Refusals teach.** The game's most important sentences are its refusals: the warden refusing an unstamped number, the socket refusing a second charm, the stairs refusing the unready. Each refusal names the missing proof, which is the principle wearing its working clothes.
- **Curriculum handoff.** A player who finishes the game and opens the curriculum should feel recognition, not repetition: the game taught the *shape* of each principle; the curriculum teaches the principle itself, in the language. Neither document pretends to be the other.

---

## 2. The House Dialect: Tiger Style Rust

*This section is binding. It governs what the game teaches, what the game's own text claims, and — when a build exists — the standard its code is written to. It exists because Niflheim's subject is not "Rust" in the abstract: it is a particular, disciplined way of writing Rust, and the game would betray its subject if its principles were phrased as syntax tricks that the next edition can obsolete.*

### 2.1 The standard

Niflheim's taught dialect is **Tiger Style Rust**, with its priority order stated exactly:

1. **Safety** — before everything.
2. **Performance** — measured, never asserted.
3. **Developer experience** — real, and third.

Its load-bearing rules, as the game teaches and models them:

- **Explicit contracts.** Every operation — in the craft the game teaches, and every mechanism in the game world — states what it accepts, what it refuses, and what it guarantees. In the game this is literal: seals, inscriptions, and treaties are contracts the player reads and enforces.
- **Assertions are the contract mechanism.** Facts a correct implementation has already established are asserted, not re-checked forever and not left as vibes. The game's analogue: the Plumb Line (Floor 4's loot) — layout claims are proven by assertion, never described in prose.
- **Bounded work.** Loops, queues, retries, and allocations carry named limits with units. Nothing in the taught craft grows without a bound, and nothing in the game world does either: the queue at the Loom is bounded and says what it does when full; the Freeze's meter is a bound made visible.
- **No hidden behaviour, no hidden allocation.** What a thing does is what its contract says; costs that exist are shown. In the game: no mechanism does a second, undocumented thing — the one apparent exception (a lantern that lights itself) is a *failure class*, presented as a fault to repair, never as a system to trust.
- **Typed errors over broad error types.** A failure the caller must decide about arrives as a distinct, named kind — in the world, as a distinct refusal with a distinct remedy, never a generic "it failed."
- **Minimal dependency and feature surfaces.** Take the smallest tool that carries the contract; every addition is a surface someone must trust. In the game: the player's pack is small, every artifact in it is wielded by name, and there is no vendor trash.
- **Determinism.** Wherever a result is compared, it is reproducible: same inputs, same artifact (the Stranger's Anvil is this rule as a shrine), same map, same proof.
- **Standing gates.** In the craft: `cargo fmt --check` and strict clippy at `-D warnings` are not release-week events but the standing condition of work. In the game: the Gatehouse (Floor 3) makes this a place — nothing is "released" out of the Gatehouse that has not passed its standing checks, and its FOE is precisely a gate that reported green without running them.

### 2.2 How the dialect surfaces in the Codex

Principle stones are phrased so the Tiger Style principle underneath is visible where it maps. Concretely:

- Stones name **mechanisms, not syntax**: "one slot, one occupant — say so in the type," not "use enum X with syntax Y." A stone must still be true, and still be useful, if the language's surface changes.
- Where a stone's principle *is* a Tiger Style rule, the stone says so in its own terms: the Lantern stone is **a result unexamined is a gate unrun** (the `#[must_use]` discipline); the Toll stone is **provenance is part of the value** (distinct types for distinct concepts); the Ice Ledger stone is **a borrow is a contract over the whole value** (contracts are total, or they are not contracts); the Last Mile stone is **boundaries are part of the contract** (validate the range before computing its end).
- Stones never teach a construct the dialect forbids: no stone praises an unbounded queue, a swallowed error, a hidden cost, or a broad error type where a caller must decide.

### 2.3 Forward readiness: editions, not versions

Rust does not evolve by breaking versions. It evolves by **editions** — 2015, 2018, 2021, 2024 — each opt-in per crate, each leaving the others compiling. There is no announced "Rust 2.0," and designing for one would be designing for a fiction. Niflheim's readiness doctrine is therefore **edition fluency**, and it is taught, not just practised:

- **All taught idioms are current-edition (2024) idioms.** Nothing in the game's principles, and nothing in the curriculum the game mirrors, depends on a pre-2024 form.
- **No deprecated or edition-fragile constructs appear in taught material.** If a construct's meaning differs across editions, it is either avoided or — in the two fenced domains below — studied *as* an edition difference.
- **Unstable and nightly-only features appear only where they are the subject:** inside the Mirror Hall (Domain 15, compiler internals) and the Roadwarden's Terminus (Domain 17, toolchain internals), fenced and labelled as unstable ground. No other floor's principle depends on one. A player can trust every other stone to be stable-language truth.
- **Edition migration is a named Domain 17 theme** (§4, Floor 17): the rite of walking an old crate across an edition boundary — the automated fix pass, the edition guide, the lints that guard the border — presented as road maintenance, which is exactly what it is.
- **MSRV and `rust-version` declaration are Domain 3 content** (§4, Floor 3): a public surface states the oldest road it can be travelled on, and semver and deprecation discipline — how a surface changes without betraying its travellers — are taught as part of API design, in the Gatehouse where surfaces are released.
- **Reading RFCs and release notes is a Domain 17 habit:** the Roadwarden reads the road's own future — proposals, stabilization reports, edition guides — as part of the trade, not as an afterthought. The game models this in the Terminus's notice-board, whose postings change as the player's descent proceeds: the road announcing its own next repairs.

### 2.4 What this section forbids in production

- No marketing or in-game text may promise "Rust 2.0 readiness" as a version claim; the honest claim is edition fluency, and the game's own copy must be honest in the way it teaches players to be.
- No floor's puzzle may hinge on memorizing syntax. If a challenge cannot be restated as a mechanism, it does not ship.
- No taught material may be added that fails the dialect: if a proposed encounter models an unbounded process, a hidden cost, or an untyped failure as *normal*, it is redesigned or cut, however charming.

---

## 3. The descent at a glance

### 3.1 Floor matrix (summary)

The full specification of each floor is §4. This table is the index.

| Floor | Stratum | Domain | Class | FOE | Primary failure class | Loot |
|---|---|---|---|---|---|---|
| 1 | I — The Language as a System | Ownership, borrowing & lifetimes | Oathwarden | The Borrowed Hoard | #12 The Plank Promised Twice | The Interner's Hoard |
| 2 | I | Traits & the type system | Sigilwright | The Menagerie | #6 The Cartographer's Blind Eye | The Plugin Menagerie |
| 3 | I | Idiomatic API design | Gatewright | The Gatehouse Release | #5 The Lantern That Lights Itself | The Gatehouse Key |
| 4 | II — The Machine Model | Memory layout | Plumbmason | The Crooked Foundation | #8 The Astronomer's Brass Eye | The Plumb Line |
| 5 | II | Unsafe Rust & the aliasing model | Veilwalker | The Thing Behind the Veil | #10 The Ice Ledger | The Veilwalker's Buffer |
| 6 | II | Interior mutability | Lockheart | The Heart Locked Twice | #7 The Two Charms, One Socket | The Lockheart Pair |
| 7 | II | Concurrency & atomics | Threadweaver | The Loom of Many Hands | #15 The Two Porters | The Threadweaver's Arc & Queue |
| 8 | II | Async & Pin | Pinwright | The Suspended Hour | #2 The Herald in the Wrong Hall | The Pinwright's Executor |
| 9 | III — Systems | Collections & allocators | Heapwarden | The Hoard Beneath | #1 The Toll of Tallies | The Heapwarden's Trio |
| 10 | III | FFI & ABI | Marchenvoy | The Treaty of Two Tongues | #9 The Tapestry That Holds the Queen | The Marchenvoy's Treaty |
| 11 | III | Macros | Runesmith | The Second Tongue | #3 The Forester's List | The Runesmith's Die |
| 12 | III | no_std & embedded | Anchorite | The Cell Without Windows | #4 The Twin Bells of the Water Gate | The Anchorite's Cell |
| 13 | III | Performance | Quicksilver Adept | The Quicksilver Trial | #14 The Uncounted Last Mile | The Quicksilver Lens |
| 14 | IV — The Depths | Kernel & OS | Rootwarden | The Root of the Tree | #13 The Tide Table | The Rootwarden's Kernel |
| 15 | IV | Compiler internals | Mirrorwright | The Mirror Turned Inward | #5 The Lantern That Lights Itself (second order) | The Mirrorwright's Lens |
| 16 | IV | Verification | Assayer | The Assay of All Things | #1 The Toll of Tallies (second order) | The Assayer's Touchstone |
| 17 | IV | Toolchain internals | Roadwarden | The Road to Everywhere | #11 The Stranger's Anvil | The Roadwarden's Charter |

### 3.2 Failure-class assignment ledger

Every row of the concept design's mapping table, assigned. "Primary" means the class is woven into that floor's FOE as its gimmick and seam. "Secondary" means the class drives ordinary chamber encounters on that floor (recurrence as weather, per the concept design's own note). Every class has exactly the primaries and secondaries listed — no class is left as palette, and none appears where it is not listed.

| # | Failure class (game form) | Primary floor(s) | Secondary floors |
|---|---|---|---|
| 1 | The Toll of Tallies | 9; 16 (second order) | 1 |
| 2 | The Herald in the Wrong Hall | 8 | 2 |
| 3 | The Forester's List | 11 | 15 |
| 4 | The Twin Bells of the Water Gate | 12 | 9 |
| 5 | The Lantern That Lights Itself | 3; 15 (second order) | 16 (as the battery's standing temptation — see §4.16) |
| 6 | The Cartographer's Blind Eye | 2 | 8 |
| 7 | The Two Charms, One Socket | 6 | 5 |
| 8 | The Astronomer's Brass Eye | 4 | 10, 13 |
| 9 | The Tapestry That Holds the Queen | 10 | — (Floor 10 carries it alone; its lesson is that some seams are single) |
| 10 | The Ice Ledger | 5 | 12 |
| 11 | The Stranger's Anvil | 17 | 3 |
| 12 | The Plank Promised Twice | 1 | 11 |
| 13 | The Tide Table | 14 | 7, 17 |
| 14 | The Uncounted Last Mile | 13 | 4, 9 (in growth-boundary encounters) |
| 15 | The Two Porters | 7 | 6, 14 |

### 3.3 Changes from the concept design

The concept design (`GAME-DESIGN.md`) is preserved in substance. This pass makes exactly these moves, all recorded here:

1. **Assignment, not invention.** The concept left the mapping table as a palette ("row assignments are made floor by floor in the full design pass"). All fifteen rows are now assigned (§3.2). No row was renamed; no codex principle was reworded in substance; the game forms keep their concept names.
2. **Two deliberate second-order recurrences.** Rows #1 and #5 each serve as a primary twice — Floors 9 and 16, Floors 3 and 15. This is not a shortage of classes; it is the design of the deep floors. The Assay Office's subject *is* the tallying of proofs, and the Mirror Hall's subject *is* the gating of code — so their FOEs are the tally and the lantern returning at second order, where the thing being verified is verification itself. The recurrences are named as such in the floor specs so no player or builder mistakes them for new classes.
3. **The false seam (Stratum IV).** The concept says each FOE carries one woven class. This pass adds, for Stratum IV only, the *false seam* (§1.10): a fully assemblable wrong diagnosis that is a misreading of the FOE's primary class, not a second class. It exists because the depths test whether the player verifies their own diagnosis — the curriculum's final lesson, stated as a boss mechanic.
4. **Platform moved from concept note to binding spec.** The concept recorded the Owner's platform ruling (Android phones, Linux desktop, Windows desktop) as a constraint. §7 of this document makes it a complete platform design; nothing in the concept's ruling was altered.

Nothing else in the concept design is changed. In particular: the core loop, the codex, the win state (reaching Niflhel), the marking convention, and the stairs gates are as the concept and the tracker state them.

---

## 4. Floor-by-floor specifications

Each floor spec uses the same fields. **Chambers** are the floor's five rooms, mirroring the curriculum domain's five levels in order; a chamber's challenge teaches its level's mechanism by demonstration and diagnosis. The curriculum's exercise specs are the syllabus twin of each chamber — the game chamber teaches the same mechanism as physics, and deliberately does not reproduce the exercise.

---

### Floor 1 — The Lending Vaults

- **Stratum / region:** Stratum I — the guild's home valley; the Oathwarden's vault-house stands at the labyrinth's mouth, where every expedition is outfitted.
- **Domain · Class:** 1, Ownership, borrowing & lifetimes · Oathwarden.
- **Prerequisites:** none. This is the entrance hall.
- **Floor theme:** a lending library of tools, charters, and beasts, where everything given out is written in the great ledger and everything returned is checked against it.
- **Learning objective:** ownership as a physical law — one thing, one holder; lending is not giving; what is lent comes back or the ledger does not balance.
- **Chambers & ordinary encounters:**
  1. *Moves and copies* — the player outfits an expedition from the vault: heavy tools (a saw, a chart) can be *given* to one porter only, while tokens and tally-sticks can be copied freely. The encounter is a packing puzzle: a plan that hands the same saw to two porters fails at the vault door, and the ledger-keeper shows the double entry. Proof: a packing list with exactly one holder per uncopiable item.
  2. *Borrowing* — the Reading Room: many delvers may read one chart at the reading table; the annotator alone may write in it, and only when no reader holds it. The player schedules readers and one annotator through a day; overlapping claims are refused at the desk with the conflicting holds shown. Proof: a day-plan with no overlap between the annotator's hours and any reader's.
  3. *Lifetimes as contracts* — the Long Lease: charters are lent for stated terms, and a charter may be sub-lent only within its term. The player chains three sub-leases; a chain whose last term outlives the first is refused, and the refusal draws the nested terms as brackets so the player *sees* the lifetime. Proof: a lease chain whose brackets nest cleanly.
  4. *Drop and scope guards* — the Faithful Steward: a store-room whose steward unmakes every temporary arrangement when an expedition leaves a room, unless the expedition has *committed* it in the book. The player runs a delivery through two rooms, one committed and one not, and watches the uncommitted work roll back. Proof: the delivery arrives with the committed half intact and the uncommitted half demonstrably restored.
  5. *Reborrows and the hard cases* — the Gauntlet Yard: ten lending situations on placards, some legal and some not, each decided by the player and then judged by the yard's mechanism — a gate that physically opens or bars. Proof: ten correct judgements, each barred gate naming the rule that barred it.
  - *Secondary class (#1, Toll of Tallies, in miniature):* in Chamber 1, the vault's crate count comes from two overlapping tally-keepers; the player reconciles by name before the count is accepted — the game's first unstamped number is refused here, gently.
- **Primary failure class:** #12, The Plank Promised Twice — a resource promised to two uses in one plan.
- **FOE:** **The Borrowed Hoard** — the vault's great hoard-engine, which sorts the guild's lent treasures and has begun issuing the unique Heartstone Chart to two expeditions at once.
- **The seam:** the Hoard's issue desk treats the Chart as if it were a copiable token: its plan-ledger contains the Chart in two expedition columns, and every other item in the plan is consistent — the single double-promise is the seam.
- **Diagnosis:** the player audits the Hoard's master plan (a large packing ledger spanning all five chambers' item kinds), cross-checking each uncopiable item against the holder columns. The double entry is findable by census, not by guessing: the player must enumerate the uncopiables and count their holders.
- **Proof required:** the completed census — every uncopiable item in the plan listed with exactly one holder, the Chart's double entry exhibited with both columns shown. The Hoard accepts no accusation without the census attached (provenance, from the first floor).
- **Repair strike:** re-plan, not confiscation: the player strikes the second expedition's claim and re-routes it through a *lent copy of a lesser chart* the vault legitimately holds — demonstrating that the repair is a legal plan, not a second plank conjured from nowhere.
- **Codex stone:** *The Plank Stone.* **Ownership: one value, one owner.** A move transfers the thing; the giver's name for it stops working. It works because double-use is not caught by vigilance but by structure — the language itself refuses the second claim, every time, before the expedition leaves.
- **Loot:** **The Interner's Hoard** — in the game, a self-keeping ledger-chest that stores each distinct thing once and issues references to it; it becomes the player's inventory backbone.
- **Wielded later:** Floor 3 (its public surface is the Gatehouse's specimen), Floor 9 (rebuilt over the Trio), Floor 14 (its discipline is the Kernel's string discipline).
- **Mastery unlock:** lending and giving become readable everywhere: every later floor's "who holds this?" puzzle is stated in this floor's language, and the borrow-checker's judgements — quoted in later keeper dialogue — are assumed legible.
- **Stairs relevance:** load-bearing. The Stratum I stairs (§5.6) require this FOE fallen.
- **Platform / input notes:** the census is a list interaction — full touch parity via tap-to-exhibit; no timed elements anywhere on this floor.

### Floor 2 — The Menagerie Grounds

- **Stratum / region:** Stratum I — the plugin-houses of the home valley: glass houses where trained beasts (plugins) perform services for the guild.
- **Domain · Class:** 2, Traits & the type system · Sigilwright.
- **Prerequisites:** Floor 1.
- **Floor theme:** each beast wears a collar of sigils stating what it may do; two doors admit beasts to service — the Narrow Door, where each beast's kind is known individually, and the Wide Door, where any beast answering to a sigil may pass.
- **Learning objective:** a trait is a contract of behaviour; a bound is the full price of admission; the same service can be reached by two dispatch doors at different costs.
- **Chambers & ordinary encounters:**
  1. *Traits and impls* — the player teaches a beast a chartered trick from its sigil-book; one beast in the house performs its own variant of the house trick, and the player must identify which body ran by watching, not by label.
  2. *Generics and bounds* — the Generic Stable: stalls accept any beast meeting the posted bound. The player starts with an over-strict bound and loosens/tightens it per service, justifying each sigil demanded by the service that needs it. A bound with an unjustified sigil is refused by the Sigilwright with the reason shown.
  3. *Associated types* — Two Kennels: the same courier service run from two kennels, one where the kennel chooses its beast-kind, one where the customer does. The player runs deliveries from both and finds the errand that is natural in one and contorted in the other.
  4. *Trait objects and the Wide Door* — the registry: heterogeneous beasts behind one sigil, driven through one service routine. Alongside, three beasts that cannot pass the Wide Door, each barred for a different named reason displayed on its placard.
  5. *Coherence* — Foreign Marks: a beast from a foreign house and a foreign sigil; the player may join them only by way of a guild-issued collar (a newtype) of the player's own making. A staged collision — two collars claiming the same sigil for the same beast — is shown and read as a specification, not an error.
  - *Secondary class (#2, Herald):* sealed letters arrive at the Menagerie throughout; one announces a service the beast never offered. Acting on it wastes a (free, miniature) expedition — the floor's first wrong-room signal, and the reason Floor 8 will feel familiar.
- **Primary failure class:** #6, The Cartographer's Blind Eye — a fine instrument used outside the domain it was made for.
- **FOE:** **The Menagerie** (the house itself, as one mechanism) — the grounds have begun admitting beasts to services by *resemblance*: a mist-hound admitted to the fire-watch because it is grey like the ash-hounds, reporting the valley's weather as smoke.
- **The seam:** the admission desk checks appearance where the charter demands a sigil: the mist-hound carries no fire-watch sigil, and its admission slip names a bound it does not satisfy.
- **Diagnosis:** the player audits admissions against charters — for each working beast, exhibit its sigils against the service's bound. The mist-hound is the one admission whose slip and sigils disagree; the cheap calibration the floor teaches is to run the suspect beast against one known landmark task (a lit brazier) and watch it report fog.
- **Proof required:** the admission audit — every service's roster with each beast's satisfying sigils exhibited, plus the calibration record showing the mist-hound's confident wrong report.
- **Repair strike:** re-admit by bound, not by resemblance: the player re-posts the fire-watch bound at the desk (a mechanism change, not a beast change) — the mist-hound is honourably reassigned to the mist-watch it is actually sigilled for, and the desk thereafter refuses domain-less admissions.
- **Codex stone:** *The Lens Stone.* **Bounds are admissions.** A trait bound states the domain a function is valid over. It works because admission is decided at the door, by the checker, instead of being discovered later from a confident wrong map.
- **Loot:** **The Plugin Menagerie** — the two-door host: a travelling menagerie that can field beasts by kind or by sigil.
- **Wielded later:** Floor 8 (async traits restate this floor's problem), Floor 11 (the Die's target trait), Floor 15 (trait resolution read from inside).
- **Mastery unlock:** bounds may be authored freely on later floors; choosing the Narrow or Wide Door becomes a stated decision with a stated cost in keeper dialogue and puzzle design alike.
- **Stairs relevance:** one of the two FOEs (2 or 3) the Stratum I stairs require.
- **Platform / input notes:** audit lists are long — the UI must support grouped, collapsible rosters identically on phone and desktop (§7.3); no information may exist only in hover text.

### Floor 3 — The Gatehouse

- **Stratum / region:** Stratum I — the valley's gatehouse, where everything the guild sends into the world is inspected, documented, and released.
- **Domain · Class:** 3, Idiomatic API design · Gatewright.
- **Prerequisites:** Floors 1 and 2.
- **Floor theme:** a release office: drafting desks, an audit wall with a public checklist, a seal press, and a great lantern over the gate that shows the road's state to travellers.
- **Learning objective:** a public surface is a promise — errors named by kind, misuse made unrepresentable, documentation as part of the artifact, and a stated oldest-road (MSRV) the surface can be travelled on.
- **Chambers & ordinary encounters:**
  1. *Error taxonomy* — the Named Failures desk: travellers arrive with broken charters; the player sorts each failure to its named kind and issues the remedy that kind prescribes. Failures sorted to a generic "broken" bin are returned by the Gatewright with the traveller's unanswerable question attached.
  2. *Iterator internals* — the Conveyor: a belt that portions goods lazily, one pull at a time. The player builds three belt attachments (portioner, deduplicator, halt-after-failure) and learns the belt's contract by polling it past its end — it must keep answering "empty," forever, and the chamber proves it does.
  3. *Panic discipline* — the Audit of Shouts: a clerk who *shouts* (panics) at every malformed form. The player re-trains the office: every traveller-reachable failure becomes a named refusal; the only remaining shouts are for broken office invariants, each with its invariant written on the wall behind the clerk.
  4. *Typestate builders* — the Sealed Request: petitions are drafted on state-stamped paper — unaddressed, addressed, addressed-and-bodied — and only the final state's paper fits the seal press. Attempts to seal early fail physically, not administratively: the press does not close.
  5. *The Guidelines audit + the oldest road* — the Inspection: the player runs the office's full public checklist against the Interner's Hoard (Floor 1 loot, now a public artifact), recording *conforms / repaired / not-applicable-with-reason* per item — and stamps the Hoard's travelling papers with its **oldest road** (the MSRV declaration): the earliest road-conditions under which the guild guarantees it. A side desk teaches the amendment rite: how a released surface changes by *deprecation* — old doors kept open, marked, and announced for a full season before any door is bricked up. (Semver and deprecation discipline are this floor's law; see §2.3.)
  - *Secondary class (#11, Stranger's Anvil, in miniature):* the Hoard's recipe is sent to a stranger-smith in the valley, who re-forges it from the paper alone; one missing line fails the seal and the *recipe* goes back for repair — the full rite waits at Floor 17.
- **Primary failure class:** #5, The Lantern That Lights Itself — a gate reporting green because the runner who would have reported red died first.
- **FOE:** **The Gatehouse Release** — the great release of the season: the Hoard's new edition, ready to ship, its lantern already green over the gate.
- **The seam:** the lantern is lit by the *schedule*, not by the inspection: the keeper's chair is empty, the log-slate shows the standing checks (the dialect's gates — formatting, lint, tests; §2.1) were never run for this release, and the green flame burns anyway.
- **Diagnosis:** the player checks both flames — the signal flame (green) and the keeper's flame (cold) — then reads the log-slate, the record of inspection steps actually performed, and reconciles it against the release's claimed checklist.
- **Proof required:** the reconciled log-slate: every claimed check marked *run* (with its record) or *never run*, exhibited against the checklist. The release cannot proceed on the slate's evidence; that is the point of the slate. The slate also grades the release's two stamps: the oldest road, proven by an actual travelling of the Hoard on the earliest road its papers claim; and one door carried through the amendment rite — deprecated, marked, and announced for its full season, its replacement demonstrated — each recorded like any other check: run, with its record, or never run.
- **Repair strike:** run the standing checks, for real, in the chamber: the player performs the inspection sequence step by step (each a small mechanism interaction, none a quiz), the keeper's flame is lit by the doing, and only then may the lantern be lit by hand. The release ships with the slate attached.
- **Codex stone:** *The Lantern Stone.* **A result unexamined is a gate unrun.** Fallible results are marked must-use because an unlooked-at outcome is indistinguishable from success. It works because it makes the *looking* mandatory — the evidence, not the indicator, becomes the gate.
- **Loot:** **The Gatehouse Key** — the Hoard raised to release grade: documented, checklisted, stamped with its oldest road.
- **Wielded later:** Floor 17 (the publication rite is performed on this artifact) and everywhere: it is the standing example of "done" for a public surface, and later keepers cite it.
- **Mastery unlock:** the API checklist becomes the review standard for all later loot; undocumented public items in later floors' mechanisms are treated as defects the player may call out — several later chambers hide exactly that call-out as their diagnosis.
- **Stairs relevance:** one of the two FOEs (2 or 3) the Stratum I stairs require; separately, the Stratum I stairs require all Stratum I boss crates clippy-clean — in game terms, no descent until every valley lantern is keeper-lit.
- **Platform / input notes:** the checklist is the floor's long-text surface — minimum text sizes and reflow rules of §7.3 apply; the log-slate reconciliation is a matching interaction with full touch and keyboard equivalence.

---

### Floor 4 — The Foundation Galleries

- **Stratum / region:** Stratum II — the machine warrens: the labyrinth's foundations, where stones are laid by measure and every wall shows its own cross-section.
- **Domain · Class:** 4, Memory layout · Plumbmason.
- **Prerequisites:** Floor 1.
- **Floor theme:** galleries of foundation stones, each stone stencilled with its true dimensions; a plumb line hangs in every room, and what the line says outranks what anyone remembers.
- **Learning objective:** layout is measurable fact — sizes, alignments, padding, and encodings are proven by assertion, never described in prose.
- **Chambers & ordinary encounters:**
  1. *Sizes and alignment* — the Predict-and-Measure bench: for each of a row of stones (values of many kinds), the player first commits a predicted size and alignment to the bench slate, then drops the plumb line. Predictions are graded by the mechanism; surprises must be annotated on the slate before the chamber completes.
  2. *Padding and field order* — a wall built of ill-fitting stones with visible voids (padding); the player reorders the same stones into the smallest wall, and the void-count is tallied byte by byte so every saved unit is attributed.
  3. *Enum encoding and the niche* — the Seal Stones: crests (enums) whose signet ring hides a niche — a value that can never occur, into which the "absent" mark fits for free. The player shrinks a provided crest by finding its niches; before/after sizes are proven on the bench.
  4. *The repr family* — the Treaty Stones vs the Free Stones: stones cut to a foreign specification (every offset promised) versus stones cut freely (no offset promised to anyone). The player seats a foreign-spec wall and proves each offset with the line; a free stone substituted into the treaty wall is shown to be under no obligation — and the wall fails honestly.
  5. *Unions and uninitialized memory* — the Hollow Stones: stones with one cavity and two possible fillings, the tag kept only in the mason's head; and blank stones (uninitialized) that may be filled one by one and used only when full. An attempt to use a partly-filled blank is caught by the gallery's own referee mechanism — the chamber's demonstration of why the referee (Miri, in the syllabus) exists.
  - *Secondary class (#14, Last Mile, in miniature):* the treaty wall's final stone overhangs the plinth by exactly the padding the player's plan forgot — the boundary stretch of the wall, where the general rule silently expired.
- **Primary failure class:** #8, The Astronomer's Brass Eye — measuring a proxy instead of the quantity.
- **FOE:** **The Crooked Foundation** — the warrens' master foundation, laid a generation ago from a celebrated survey, now settling crooked.
- **The seam:** the old survey measured the foundation by its *shadow at noon* (a proxy that tracks the true width only when the sun cooperates) and recorded the shadow numbers in the dimension book as if they were the stones' sizes. Every stone cut from the book is true to the book and false to the ground.
- **Diagnosis:** the player cross-checks the dimension book against the plumb line in two frames — book-space and ground-space — and finds the entries disagree by exactly the shadow's offset. The wrong entries are identifiable only by measuring the quantity itself (the stones), not by auditing the book's arithmetic, which is flawless.
- **Proof required:** the re-survey: every load-bearing dimension measured at the stone and asserted on the bench, with the book's proxy entries exhibited beside their true values.
- **Repair strike:** re-cut the foundation's model twice, as the guild requires: one set of stones to the foreign spec (every offset asserted) and one free set under a stated size budget — the player lays both and the line verifies each.
- **Codex stone:** *The Brass Eye Stone.* **Distinct quantities get distinct types.** A measurement in view-space and the same number in model-space are different kinds of thing, so a proxy cannot be silently substituted for the quantity. It works because the substitution, when attempted, fails at the door instead of failing at sea.
- **Loot:** **The Plumb Line** — a layout-assertion instrument: any size or offset claim made in its presence is proven or refused on the spot.
- **Wielded later:** Floors 5, 9, 10, 13, 14 — layout claims on all of them are Plumb-Line claims.
- **Mastery unlock:** prose about sizes stops being evidence anywhere in the descent; later keepers accept only line-proven dimensions.
- **Stairs relevance:** load-bearing — the Stratum II stairs require this FOE fallen.
- **Platform / input notes:** prediction entry is a numeric interaction — the same bounded stepper/keyboard field on all platforms; the bench must render quantities in large, tabular figures (§7.3).

### Floor 5 — The Veilworks

- **Stratum / region:** Stratum II — deeper warrens, behind a veil of gauze workshops where the safe world's rules are hand-made and hand-checked.
- **Domain · Class:** 5, Unsafe Rust & the aliasing model · Veilwalker.
- **Prerequisites:** Floor 4.
- **Floor theme:** a workshop where every tool is sharp and every bench carries its contract card: what the tool assumes, written where the hand can read it before it lifts.
- **Learning objective:** unsafe work is contract work — every assumption stated, checkable, and refereed; aliasing is a promise about the whole value, not the touched part.
- **Chambers & ordinary encounters:**
  1. *Raw pointers* — the Pointer Walks: marked paths across a memory-yard of crates, walked with a surveyor's pole (a raw pointer) that can point anywhere and guarantees nothing. Every step's safety is read off the path's contract card before stepping.
  2. *The aliasing model* — the Two Tenants: rooms (values) with tenancy rules: many readers, or one writer, never both at once. Eight tenancy situations are placarded; the player predicts each verdict (sound, or a violation and why) in writing on the placard, then the workshop's referee mechanism judges. Seven of eight must be right; the miss must be annotated with the deciding rule.
  3. *Unsafe contracts* — the Contract Cards: a bench of tools with missing or circular cards ("safe because it is safe"). The player rewrites each card until a sceptical Veilwalker accepts it — the acceptance test is whether the card's claims can be *checked*, not whether they sound careful.
  4. *Safe abstractions* — the Split Beam: a beam (a buffer) divided into two halves that may be worked simultaneously, built from raw parts behind a safe face, with the contract at the seam. The player assembles it and the referee walks its panic paths and drop order.
  5. *The UB hunt* — three seeded faults hidden in a small machine (a tenancy violation, a use after the room was emptied, a read of an unfilled blank): the player hunts with the referee's lens, and each fix must be made at the contract level — the card rewritten — not at the symptom.
  - *Secondary class (#7, Two Charms, in miniature):* one hunted fault is two writers fused into one tenancy — the socket lesson arriving early, in veilcloth.
- **Primary failure class:** #10, The Ice Ledger — a freeze (a borrow) is a contract over the whole value.
- **FOE:** **The Thing Behind the Veil** — the workshop's great self-growing store-beam (a raw-parts buffer), which has begun letting a lent section be *resized by a second hand* while the first hand's reference is still out.
- **The seam:** the store-beam's keepers treat an outstanding loan as freezing only the lent *section*. Under the true contract, the loan freezes the whole beam — growth moves every section — so the outstanding reference is left pointing at a room that no longer exists.
- **Diagnosis:** the player traces a reported corruption (a reference that read another tenant's goods) through the loan ledger, and demonstrates with the referee's lens that the fault is not in the reading but in the growth: the beam moved under a live loan. The demonstration requires exhibiting one loan, one growth, and the lens's verdict in sequence.
- **Proof required:** the loan-freeze demonstration, plus the workshop's standing evidence: the repaired buffer's full behaviour suite walked under the referee (the Miri-analogue), including panic paths, drop order, and the empty-element and overflow cases, each refused or handled by contract.
- **Repair strike:** rewrite the beam's contract card and re-cut its loan desk: growth is refused while any loan is out (the freeze made structural), and the safe face exposes no operation that can move the beam under a reference.
- **Codex stone:** *The Ice Stone.* **A borrow is a contract over the whole value.** A shared reference freezes the whole of the value, not the fields you happen to touch, for as long as the borrow lives. It works because the guarantee is total: readers can rely without re-checking precisely because writers are excluded *everywhere*, not merely nearby.
- **Loot:** **The Veilwalker's Buffer** — a growable raw-parts store with an honest safe face.
- **Wielded later:** Floors 6, 9, 12 — the storage primitive the deeper floors keep rebuilding on.
- **Mastery unlock:** unsafe mechanisms may be operated on later floors provided every one carries its contract card; the referee (Miri) is unlocked as a standing instrument the player can invoke in later workshops — Floor 16 will formalize it into a gate.
- **Stairs relevance:** load-bearing — the Stratum II stairs require this FOE fallen.
- **Platform / input notes:** the referee's lens view is a diagram surface — it must be legible at phone width without pinch-zoom (§7.3); no chamber on this floor is timed.

### Floor 6 — The Chamber of the Twice-Locked Heart

- **Stratum / region:** Stratum II — a quiet warren of strongboxes and reading hearts, where things change from inside without their shells ever moving.
- **Domain · Class:** 6, Interior mutability · Lockheart. (Entered on Floor 5 charted through its third chamber, per the prerequisite chart.)
- **Prerequisites:** Floor 5 through chamber 3.
- **Floor theme:** hearts in chests: a heart can beat differently inside an unmoving chest, but only through mechanisms whose rules are posted on the chest.
- **Learning objective:** all interior change is one mechanism underneath (`UnsafeCell`); borrow-flag discipline turns "two writers" from corruption into a loud, survivable refusal.
- **Chambers & ordinary encounters:**
  1. *The one true interior* — the player builds a simple changing heart (a `Cell` for copyable feelings) inside a chest, stating exactly which tenancy rule the wrapper upholds; the referee walks it.
  2. *The borrow-flag machine* — a heart with a flag-staff: reading flags fly many at once; the writing flag flies alone; a second writing flag does not corrupt the heart — it *strikes the bell* (panics) and the heart keeps its last true state. The player operates the machine through scripted requests including the illegal one, and watches the bell strike and the heart survive.
  3. *Lazy initialization* — the First-Caller Brazier: a brazier that the first visitor lights and all later visitors simply use. The player designs the lighting rite for one thread of visitors, and writes down — on the chamber wall, as a deferred note — why many-thread lighting is a deeper floor's problem.
  4. *The spin heart* — a heart guarded by a spinning watchman (a spin mutex): visitors who find it busy wait their turn by watching. Under a scripted crowd, the count of beats comes out exact; the player annotates the watchman's ordering choices as *provisional* — Floor 7 will grade them.
  5. *Choosing the cell* — three tangled dollhouses (designs knotted in shared hearts): the player untangles two into clean ownership and deliberately leaves one interior-mutable, writing its one-sentence invariant on its door. The invariant sentence is the deliverable.
  - *Secondary class (#15, Two Porters, in miniature):* in Chamber 2, two writing requests arrive in perfect courtesy, each waiting for the other to finish first — the flag machine's refusal is shown as the mercy it is, one floor before the porters make it a legend.
- **Primary failure class:** #7, The Two Charms, One Socket — two mechanisms forced into one zone, fused into a third useless thing.
- **FOE:** **The Heart Locked Twice** — the warren's great heart, whose flag-staff has been *improved* by a well-meaning smith: it now permits the writing flag and a reading flag to fly together "to save time," and the heart's state has fused into something that is neither truly read nor truly written.
- **The seam:** the heart's flag machine allows a state its own design makes meaningless — two occupants of incompatible kinds in one occupancy zone. Every symptom (torn readings, half-written states) is downstream of that one illegal combination.
- **Diagnosis:** the player enumerates the flag-staff's reachable states from its mechanism (not from its placard), exhibiting the fused state, then demonstrates on the workshop model that the fused state — and only it — produces the reported symptoms.
- **Proof required:** the state census: all reachable flag combinations listed, the illegal one exhibited, plus the referee's clean walk of the corrected machine and the loom-model's (Floor 7's instrument, previewed here as the guild's model loom) exploration of the spin heart's interleavings with a thread that fails mid-beat.
- **Repair strike:** re-cut the flag-staff as a single-occupancy mechanism: the staff holds *exactly one* state-token at a time — the states become a set of mutually exclusive tokens (an enum in a chest), so the fused state cannot be constructed, not merely discouraged.
- **Codex stone:** *The Socket Stone.* **One slot, one occupant — say so in the type.** Two flags that must never fly together are a bug farm; a single token with mutually exclusive states makes the fused state unrepresentable. It works because the type *is* the occupancy rule, and the checker is the socket-warden who never sleeps.
- **Loot:** **The Lockheart Pair** — the borrow-flag heart and the spin heart, built from parts.
- **Wielded later:** Floor 7 (they are what memory orderings protect), Floor 8 (the executor's shared state).
- **Mastery unlock:** interior change may be used on later floors only through mechanisms whose invariant the player can state in one sentence; keepers will ask for the sentence.
- **Stairs relevance:** counts toward the Stratum II stairs' "any two more."
- **Platform / input notes:** the state census is a set-building interaction; tokens must be distinguishable by shape and label, never by colour alone (§6.7).

### Floor 7 — The Loom Halls

- **Stratum / region:** Stratum II — long halls of looms where many hands pull one thread, and the air is full of almost-simultaneous events.
- **Domain · Class:** 7, Concurrency & atomics · Threadweaver.
- **Prerequisites:** Floor 5. (Floor 6 recommended — its hearts are this floor's furniture.)
- **Floor theme:** looms, shuttles, and porter-stairs; every hall posts its *happens-before* charters on the wall: which events are guaranteed ordered, and which are merely likely.
- **Learning objective:** concurrency is a contract discipline — orderings are chosen with written happens-before arguments, queues are bounded with a stated full-policy, and deadlock is a shape that a global order makes impossible.
- **Chambers & ordinary encounters:**
  1. *Threads and scoped lending* — the Scoped Loom: short-lived weavers borrow the hall's own thread for one pattern and return it; the player runs a parallel tally over borrowed cloth with no shared ownership anywhere, exact under several weaver counts.
  2. *Send and Sync as contracts* — the Border Seals: some tools may cross between halls (Send) and some may be used by many hands at once (Sync). The player makes one tool deliberately border-sealed with a phantom seal and proves the refusal at the border desk.
  3. *Orderings I* — the Charter Wall: the player builds a one-shot message shuttle and a spin watchman on raw signals (atomics), annotating each signal with the happens-before edge it creates; a sceptical Threadweaver re-reads every annotation.
  4. *The shared counter (Arc)* — the hall builds a jointly-owned pattern-book whose last holder closes it: the player assembles the ownership counter and its weak echoes, and a deliberately weakened ordering in a practice copy is caught by the model loom (Chamber 5's instrument, introduced here).
  5. *The lock-free stack, model-checked* — a shuttle-stack no hand locks: the player operates it while the model loom explores the bounded interleavings, and replays its operation record against a sequential model to spot-check linearizability.
  - *Secondary class (#13, Tide Table, in miniature):* a charter on the wall was true when posted (a shuttle *was* free at dawn); the player must carry the witness (the shuttle's token) rather than the note — the first stale-observation refusal.
- **Primary failure class:** #15, The Two Porters — deadlock as a cycle in the waits-for graph.
- **FOE:** **The Loom of Many Hands** — the halls' master loom, fed by a bounded queue from many producer-hands, which has begun — rarely, unrepeatably — to stop dead with every hand waiting and no hand waiting *on* anything the keepers can name.
- **The seam:** two of the loom's servants claim the hall's two service stairs in opposite orders. Most days the claims don't overlap; when they do, each waits on the other forever — a cycle of exactly two, invisible to any keeper watching a single stair.
- **Diagnosis:** the player draws the waits-for graph from the loom's own event record (an authored log the game presents), finds the cycle, and then uses the model loom to explore the bounded interleavings of the FOE's mechanism until the deadlocking interleaving is exhibited — not merely theorized.
- **Proof required:** the exhibited deadlocking interleaving from the model loom (no race, no lost wakeup, the cycle closed), plus the stress evidence at scale: a million messages through contended producers with the received set matching the sent set exactly, and the at-capacity behaviour demonstrated to be the documented back-pressure, never a silent drop.
- **Repair strike:** impose the house rule: the player re-posts the stairs charter with a single total order in which stairs may be claimed, and re-cuts the servants' claim plaques to it. The model loom then explores the same bounded space and finds no cycle to exhibit — the shape is gone, not the instance.
- **Codex stone:** *The Porters' Stone.* **A global order forbids cycles.** If every lock in a program is taken in the same total order, the waits-for graph can never contain a cycle — and deadlock *is* a cycle. It works because it removes the shape of the failure, not the instance: no diplomacy, just topology.
- **Loot:** **The Threadweaver's Arc & Queue** — a jointly-owned counter and a bounded many-handed queue, model-verified.
- **Wielded later:** Floor 8 (the executor's task queue), Floor 14 (kernel synchronization, read with experienced eyes).
- **Mastery unlock:** from here down, an ordering may be chosen only with a written happens-before argument; "strongest ordering everywhere" is allowed — as a stated decision with a stated cost, never as an unexamined default.
- **Stairs relevance:** counts toward the Stratum II stairs' "any two more"; load-bearing for Floor 14's prerequisites.
- **Platform / input notes:** the waits-for graph is the floor's signature diagram — it must render legibly at phone width with labelled (not colour-only) cycle highlighting (§6.7, §7.3). No live-timing input anywhere: the model loom is turn-stepped.

### Floor 8 — The Stillroom of Hours

- **Stratum / region:** Stratum II — a hushed warren of suspended bells and sleeping messengers, where time is kept in jars.
- **Domain · Class:** 8, Async & Pin · Pinwright.
- **Prerequisites:** Floors 2 and 6.
- **Floor theme:** hours that can be stopped mid-swing and resumed without the bell ever moving from its pin; an executor-hall where one attendant runs every errand by a strict roster.
- **Learning objective:** a future is a state machine you could have written by hand; `Pin` is a promise about not moving; a wakeup is a signal with provenance, and a stale one is a wrong-room herald.
- **Chambers & ordinary encounters:**
  1. *Futures as state machines* — the Countdown Bell: the player assembles a bell that rings after three visits, from explicit states in a state-box — no "async" magic, the states drawn as a diagram they must complete.
  2. *Pin and self-reference* — the Pinned Tapestry: a tapestry whose threads point into itself; moved, it tears. The player mounts it on its pin-frame and demonstrates the unpinned version's tear (referee-judged), then the pinned version's soundness.
  3. *Wakers and the attendant's contract* — the player builds the attendant's wake-bell by hand: when an errand pends, the attendant *sleeps* (the hall darkens) and only the correct bell wakes them. A wake-counter proves the attendant slept rather than spinning in place.
  4. *The single-attendant executor* — errands queue on the Lockheart hearts (Floor 6 loot, at home here); the player runs several errands interleaved and the roster's order is asserted event by event — deterministically.
  5. *Cancellation and async traits* — dropping an errand-scroll is cancelling it, silently: the player runs a timed errand with a scope-guard witness (Floor 1's steward, returning) and reconciles the cleanup ledger against the documented semantics — what ran, what didn't, exactly.
  - *Secondary class (#6, Blind Eye, in miniature):* a Floor 2 beast is offered as an errand-runner for a service outside its sigils — its admission to the roster is refused at the door by bound, familiarly.
- **Primary failure class:** #2, The Herald in the Wrong Hall — a completion signal believed in the wrong room.
- **FOE:** **The Suspended Hour** — the Stillroom's master hour, whose errands have begun completing *before they are run*: attendants wake, find completion bells already rung, and mark errands done that never happened — because a waker from a finished errand of the previous watch still holds a live bell for this one, and its ring is indistinguishable from a true completion.
- **The seam:** completion is signalled by *sound-alike bells* — an untyped signal any stale waker can ring. The errand system cannot tell "the bell for *this* errand, rung by *its* waker" from "a bell," and so a stale completion publishes a result for work that never ran.
- **Diagnosis:** the player traces one phantom completion through the roster: the errand's state-box never left its first state (exhibited), yet the completion bell was rung (the wake-log exhibited), by a waker whose errand had already ended (the generation mismatch exhibited). Three exhibits, one conclusion.
- **Proof required:** the three exhibits above, plus the repaired roster's deterministic interleaving suite run event-for-event, timers in order, and the wake-counter proving no busy-wait; a never-waking errand is shown to reach the executor's documented shutdown behaviour, and a panicking errand is contained per the documented policy.
- **Repair strike:** re-found the bell system on typed signals: each errand's completion becomes its own kind of message, carrying its errand's seal and generation, and the attendant's roster accepts a completion only when seal and generation match the living errand. The stale waker may ring forever; its bell is now a letter in a language the hall no longer mistakes.
- **Codex stone:** *The Herald Stone.* **Let the type carry the errand.** Signals are distinct types, not sounds to be pattern-matched on hope — one message kind per meaning. It works because the matching is checked: a handler for *Errand Completed* cannot be handed a *Bell Rung*; they were never the same type. (And the freshness half of the lesson is the dialect's own: a generation token means stale completions cannot publish.)
- **Loot:** **The Pinwright's Executor** — the attendant, the roster, the timer jars, and the typed bell system.
- **Wielded later:** Floor 13 (its overhead is a profiling specimen), Floor 15 (async desugaring read in the mirror with recognition).
- **Mastery unlock:** async stops being weather: in later floors, a suspended task is a state machine the player could in principle have written by hand, and `Pin` is a promise they can state.
- **Stairs relevance:** counts toward the Stratum II stairs' "any two more."
- **Platform / input notes:** the roster is an event list — it must be fully operable by keyboard and touch with identical ordering (§7.7); the hall's "sleep" state uses dimming, never flashing (§6.7).

---

### Floor 9 — The Deep Hoard

- **Stratum / region:** Stratum III — the working frontier: vault-mines where the guild's collections are grown, counted, and stored.
- **Domain · Class:** 9, Collections & allocators · Heapwarden.
- **Prerequisites:** Floors 4 and 5.
- **Floor theme:** a hoard that grows in the dark — rows of self-raising shelves (vectors), a great key-cellar (a hash map), and the counting-house where every allocation is written in a ledger with its reason.
- **Learning objective:** the standard collections are choices with known prices — growth, probing, and allocation are mechanisms the player has operated by hand, and counts about a hoard are claims that must carry their evidence.
- **Chambers & ordinary encounters:**
  1. *The growing shelf (Vec)* — built over Floor 5's buffer discipline: the player grows a shelf past its boards and watches capacity and length part company; every growth is costed on the counting-house slate.
  2. *Linked structures, honestly* — a chain of hand-linked crates that fights both the ledger and the road (cache): the player takes it as far as soundness allows, then redesigns it as an indexed shelf-row (arena indices) and reads the post-mortem wall, which names both failure mechanisms.
  3. *The key-cellar (hash map)* — open addressing walked with a probe-lantern: the player places keys, follows probe sequences, and floods the cellar with colliding keys to feel the load factor in their feet.
  4. *Allocators as an interface* — the Counting-House: a counting steward wraps every allocation in the floor; the player must predict a stated workload's exact allocation count *and explain it*, not merely match it.
  5. *Arenas and the Hoard rebuilt* — a bump-arena quarry: blocks issued in order, reset as one. The player rebuilds the Interner's Hoard (Floor 1 loot) on arena blocks; its behaviour must pass its original suite unchanged, and its allocation count for a fixed corpus drops to a stated number of blocks.
  - *Secondary class (#4, Twin Bells, in miniature):* the Trio is issued as a set — shelf, cellar, quarry — and the counting-house refuses to seal a partial issue: the hoard's instruments are a whole, or they are not issued.
  - *Secondary class (#14, Last Mile, in encounters):* growth at the exact capacity boundary and the arena's last block are boundary encounters in Chambers 1 and 5 — the general growth rule expires exactly there, and the floor says so.
- **Primary failure class:** #1, The Toll of Tallies — a number that travels farther than its evidence.
- **FOE:** **The Hoard Beneath** — the master hoard, whose census — the number every guild decision on this floor trusts — is computed by *summing the shelf-count, the cellar-count, and the arena-count*, three tallies kept by three stewards whose domains overlap: everything stored through the arena is *also* shelved, and a third of the cellar's keys index shelved goods.
- **The seam:** the census is arithmetic over overlapping ledgers. Each steward's tally is honest; the sum is a lie, because the overlap was counted twice and thrice — and the census travels through the whole frontier stamped as a bare number, believed everywhere because it is a number.
- **Diagnosis:** the player reconciles by identity, not arithmetic: sample the hoard's contents item by item against the three ledgers (a full census at FOE scale, operable in batches), exhibiting items that appear in two and three tallies. The diagnosis completes when the overlap classes are enumerated — shelved-only, cellar-only, arena-and-shelved, all-three — with counts proven against the physical hoard.
- **Proof required:** the reconciled census — a count *stamped with the census-scroll it came from*, its overlap classes exhibited — plus the floor's standing evidence: the differential trials, in which the player's Trio is run against the guild's reference instruments over at least ten thousand operations per structure with zero divergence, including the adversarial cases (empty-kind values, growth at capacity boundaries, collision flooding, arena exhaustion returning a documented refusal rather than a panic or a wraparound).
- **Repair strike:** abolish the summed census: the counting-house thereafter issues only stamped counts — a number fused to the census that produced it, in a form (a sealed tally-scroll) that cannot be detached from its evidence or added to another stamped count. Bare numbers are refused at every frontier desk from this floor onward — the player will feel this refusal working *for* them on Floor 16.
- **Codex stone:** *The Toll Stone.* **Provenance is part of the value.** Wrap a bare number in a type that carries where it came from, so an unverified count cannot be handed where a verified one is required. It works because the check moves from discipline — remembering to verify — to structure: the desk itself refuses anything else.
- **Loot:** **The Heapwarden's Trio** — the growing shelf, the key-cellar, and the bump quarry, differentially proven.
- **Wielded later:** Floors 12, 13, 14 — the kernel's allocator is this Trio's descendant.
- **Mastery unlock:** reaching for a standard collection on later floors becomes a stated choice with a known cost, never a reflex; keepers quote prices.
- **Stairs relevance:** load-bearing — the Stratum III stairs require this FOE fallen.
- **Platform / input notes:** census reconciliation runs in batches with progress shown by count and bar (never a spinner alone); the ten-thousand-operation trials are presented as a summarized record, not simulated in real time, identically on all platforms.

### Floor 10 — The Treaty Port

- **Stratum / region:** Stratum III — the frontier's border port, where the guild's road meets the C-speaking country across the water.
- **Domain · Class:** 10, FFI & ABI · Marchenvoy.
- **Prerequisites:** Floors 4 and 5.
- **Floor theme:** a customs house on a strait: two languages, one strait, and a treaty wall where every crossing's ownership is written before anything moves.
- **Learning objective:** across a border, representation is shared and meaning is not — every crossing states its ABI, its ownership, and its panic firewall, or it does not cross.
- **Chambers & ordinary encounters:**
  1. *Both directions* — the player opens the port both ways: a guild machine worked by foreign hands through the customs desk, and a foreign machine worked by the guild. Both crossings are *run*, not inspected by eye.
  2. *Structures across the border* — Floor 4's Plumb Line at work: a treaty-stone (a `repr(C)` struct) passes by value in both directions, its offsets asserted on the guild side and counter-asserted on the foreign side; a deliberately mis-cut twin fails on *both* sides.
  3. *Callbacks and the panic firewall* — a foreign sorting engine calls back into the guild for comparisons. One comparison-clerk is scripted to panic on demand; the firewall at the desk converts the panic to a treaty error code, and the sort completes or aborts per the written contract — panic never unwinds across the water.
  4. *Ownership across the border* — the only question that matters, posted at the desk: *who frees what.* The player drafts crossings whose ownership terms make double-freeing unrepresentable from the guild side; the classic abuses (double-free, use-after-free, freeing foreign goods) are attempted at a practice desk and each is refused by construction or caught by Floor 9's counting steward.
  5. *The generated manifest* — a foreign header is machine-translated into guild bindings by the port's translating engine; the generated manifest is a build artifact, never hand-edited — the port's law is stencilled on the engine itself — and the player wraps it in a safe guild layer.
  - *Secondary class (#8, Brass Eye, in miniature):* a foreign gauge and a guild gauge give the strait's width in the same numerals and different units; a crossing planned on the proxy number grounds a barge, gently, on the demonstration shoal.
- **Primary failure class:** #9, The Tapestry That Holds the Queen — boundaries are semantic, not representational.
- **FOE:** **The Treaty of Two Tongues** — the port's master sieve-bridge, which sorts all cross-border goods by *appearance*: colour, weight-class, and container shape. It has begun admitting a painted decoy consignment — scenery panels painted as treaty-stones, touching real stones in the same crate — into the treaty's protected store, because where the painted panel touches a real stone, the sieve lifts them as one.
- **The seam:** the sieve reads representation; the border is a boundary of meaning. Goods that merely *touch* a protected item are not protected, but every separator the port owns keys on adjacency of appearance, so touching scenery crosses with the figure, every time, predictably.
- **Diagnosis:** the player demonstrates the failure predictably (three staged crates, three lifts, a panel crossing each time), then finds the cut the sieve cannot make: the semantic seam — the stitch line where panel meets stone, the story of which thing is which — and shows that along that seam, and only there, the consignment separates cleanly.
- **Proof required:** the staged demonstrations (the failure, repeated on demand, is the evidence), plus the treaty trials: both crossing directions passing their behaviour suites, the ownership abuses attempted against the finished treaty and refused, a panicking callback contained at the border, and a forbidden null presented at the desk detected and refused, never dereferenced.
- **Repair strike:** re-found the sieve on the treaty itself: goods cross under *declared kind*, stated in the manifest and checked against the treaty wall's types — representation becomes a hint the desk may use, never the criterion. The painted panels, declared as what they are, cross openly as scenery, in the scenery store, harming no one.
- **Codex stone:** *The Tapestry Stone.* **Type is what a thing is; representation is what it looks like.** Two values can share a representation and differ in type; a system keyed on representation inherits every coincidence of appearance. It works because meaning, once named as a type, can be checked — colour cannot testify about belonging.
- **Loot:** **The Marchenvoy's Treaty** — the two-way library and its wrapper pattern: one artifact, both directions, one ownership treaty.
- **Wielded later:** Floor 12 (vendor bindings are this, at scale), Floor 14 (the kernel's Linux-shore binding model is this treaty, formalized).
- **Mastery unlock:** foreign mechanisms appear on later floors only behind a safe guild wrapper with a stated ownership contract; raw foreign machinery in the open is a defect from here down.
- **Stairs relevance:** counts toward the Stratum III stairs' "any two more."
- **Platform / input notes:** the staged demonstrations are stepwise (advance on player input), never auto-playing timed sequences; treaty text is long-form — §7.3 text rules apply.

### Floor 11 — The Scriptorium of Echoes

- **Stratum / region:** Stratum III — a frontier scriptorium where scribes write books that write books, and every echo is inspected before it is allowed to speak.
- **Domain · Class:** 11, Macros · Runesmith.
- **Prerequisites:** Floor 2.
- **Floor theme:** stamps, dies, and echo-chambers: a die cut once stamps a thousand pages, and a flaw in the die is a flaw in every page it will ever touch.
- **Learning objective:** code that writes code is held to a higher standard, not a lower one — generated text must be hygienic, its errors deliberate and located, and its repetitions earned.
- **Chambers & ordinary encounters:**
  1. *Patterns* — the Hashmark Die: the player cuts a die that stamps collection-pages (lists and maps) from shorthand, handling trailing marks and the empty page; its stampings are tested including the empty and one-item cases.
  2. *Hygiene* — the die is carried to a foreign scriptorium whose scribes use conflicting local names; it must stamp correctly there owing nothing to that house's scope. A die that resolves through the *caller's* names fails in that house, publicly.
  3. *The declarative engine* — a state-machine press: a readable declaration is compiled by token-munching into a sigil-set and a transition table in which an illegal transition is *unrepresentable*, not merely unhandled — the player tries to declare one and the press refuses the shape.
  4. *The derive die* — the great die that stamps a Floor 2 trait's implementation onto any suitably-shaped house. Used from a foreign scriptorium, it works; offered an unsuitable shape (a house with no rooms, a round tower), it refuses with a located, legible error pointing at the offending span.
  5. *Error quality* — the die is hardened: malformed briefs are pinned in the scriptorium's UI-test gallery, every one producing its deliberate error at the right span — never a die-panic, never an inscrutable expansion dump.
  - *Secondary class (#12, Plank, in miniature):* a faulty practice die expands a brief into text that *uses the same plank twice* — the player reads the expansion and finds the double move in generated text, learning that generated code is code.
- **Primary failure class:** #3, The Forester's List — exhaustiveness is a census; what is not enumerated gets decided by default.
- **FOE:** **The Second Tongue** — the scriptorium's master press, which compiles the guild's road-orders into transition tables. To save time, the press *summarizes*: any destination not explicitly named in a brief is routed to a default road — "and elsewhere, the usual way." A felling order for deadwood, summarized by the press, is about to send the woodwards down the usual way through a grove of ribboned ancient oaks that the brief's author assumed went without saying.
- **The seam:** the press treats an unenumerated case as a default, and the default is where the destruction hides. The oaks were never named as keeps in the generated table — not because anyone decided against them, but because the table never had to name them.
- **Diagnosis:** the player does the forester's work before any axe falls: transcribe the press's output table onto a wax tablet (the exact list, counted), walk the boundary of the grove, and physically verify the *keeps* — every ribboned oak — against the table, finding the keeps the table decides by default.
- **Proof required:** the wax-tablet census: every keep named and checked, the default-routed entries exhibited as a class, plus the press's own trials — its dies behaving per their galleries from a foreign scriptorium, and every malformed brief producing its pinned, located error.
- **Repair strike:** re-cut the master press so its tables are *exhaustive by construction*: a brief that does not name every destination does not compile — the press refuses the brief itself, upstream of any axe, and demands the census be completed by the author rather than defaulted by the machine.
- **Codex stone:** *The Forester's Stone.* **Exhaustiveness is a census.** An exhaustive match must name every arm; a plan must name every keep. It works because whatever is not enumerated gets decided by default — and defaults, unexamined, are where destruction hides.
- **Loot:** **The Runesmith's Die** — the proc-macro die and the declarative press, hardened.
- **Wielded later:** Floor 15 (macro expansion read inside the mirror), Floor 17 (build-time code generation with judgement).
- **Mastery unlock:** code generation becomes a legal tool on later floors — which means hand-written repetition must thereafter justify itself against the die that could replace it.
- **Stairs relevance:** counts toward the Stratum III stairs' "any two more."
- **Platform / input notes:** the wax-tablet census is a long-list interaction with check-off in any order; expansion views are code-like text under §7.3's monospace and size rules.

### Floor 12 — The Cells of the Wanting-Nothing

- **Stratum / region:** Stratum III — a frontier hermitage: small stone cells where Anchorites keep mechanisms that run on almost nothing.
- **Domain · Class:** 12, no_std & embedded · Anchorite.
- **Prerequisites:** Floor 5. (Floor 9 recommended.)
- **Floor theme:** cells with `core` alone: no stores, no servants, no ambient anything — what a mechanism needs, it carries in its hands, stated.
- **Learning objective:** the standard library is a provision, not a law of nature — a mechanism can be built to need almost nothing, and everything it *does* need is named.
- **Chambers & ordinary encounters:**
  1. *Core alone* — the player ports the Floor 9 quarry (the arena) into a cell: it must work with the cell's provisions only, and the cell's stores-manifest proves nothing extra crept in — the manifest (the dependency tree, shown as a stores list) is the chamber's proof.
  2. *Panic handlers and entry* — a bare cell-dwelling (firmware): the player gives it its own way to begin and its own way to fail, and it calls its greeting out of the cell's speaking-tube and departs cleanly, on cue, in the scripted demonstration.
  3. *Volatile and memory-mapped doors* — registers are not memory: the cell's message-door (a UART) is worked through a practice door-frame, every touch made deliberately and justified; a "helpfully optimized" twin that caches the door's status is shown failing the same test its honest twin passes.
  4. *Interrupts and critical sections* — a bell-timer interrupts the cell's work to feed a counter-hearth; the shared state is arranged so the race is impossible by construction (one producer, by design), and the player interleaves bell and work adversarially and loses nothing.
  5. *Driver shape* — a sense-organ (a sensor driver) built generic over two kinds of road (I2C/SPI analogues), proven against practice organs, structured so swapping the practice road for a real one changes nothing inside the driver.
  - *Secondary class (#10, Ice Ledger, in miniature):* during the bell-hearth's critical section, the whole cell goes quiet — the origin frozen — and the player must route their own actions around the freeze rather than through it.
- **Primary failure class:** #4, The Twin Bells of the Water Gate — halves installed separately, each looking complete.
- **FOE:** **The Cell Without Windows** — the hermitage's master cell-dwelling, delivered by two builders: one fitted its bell (its entry and greeting), another its clapper (its panic handler and exit rite), each builder signing their half complete. Assembled, the dwelling stands silent: it can begin, or it can fail loudly, but as delivered it can do neither reliably — under trial it sometimes greets, sometimes hangs, and its failure rite was cut for a different bell.
- **The seam:** the dwelling was never whole: two halves, each internally complete, never constructed as one thing — the clapper's contract names a bell the first builder did not hang. Nothing is *broken*; the whole was simply never built, and every inspection that examined a half reported green.
- **Diagnosis:** the player inspects not the halves but the *join*: the interface contract between bell and clapper, exhibiting the mismatch (the failure rite's expected signal against the entry's actual one), and demonstrates that each half passes its own trial while the assembled dwelling fails the boot trial.
- **Proof required:** the boot trial itself, run as the syllabus prescribes in spirit: the scripted demonstration log containing the greeting banner, the bell-count reaching its stated threshold, and the clean exit — plus the poverty proofs (stores-manifest shows `core` only; the failure rite is defined in-cell), and the storm trial: bells arriving during a long critical section lose no count beyond the documented bound.
- **Repair strike:** rebuild the dwelling as a single construction: one builder's rite, one contract — the cell is re-founded so its entry, failure rite, and exit are a single artifact that cannot be delivered in halves; the hermitage's gate thereafter accepts only whole dwellings, and a half-delivered one locks at the door rather than half-working.
- **Codex stone:** *The Twin Bells Stone.* **Wholes that cannot be built in halves.** A structure that can only be constructed complete makes the half-installed state unrepresentable. It works because a failure class that cannot be constructed cannot be encountered: there is no path that meets a half-made mechanism.
- **Loot:** **The Anchorite's Cell** — the firmware skeleton: boot, failure rite, bell, and exit as one whole.
- **Wielded later:** Floor 14 (it is the Kernel's chassis), Floor 17 (built through a hand-written road-spec).
- **Mastery unlock:** the standard library stops being an assumption anywhere below; allocation becomes opt-in — taken deliberately, with its cost stated — never ambient.
- **Stairs relevance:** load-bearing — the Stratum III stairs require this FOE fallen.
- **Platform / input notes:** the scripted demonstration log is a text record the player steps through; the storm trial is turn-stepped, never real-time (§7.7).

### Floor 13 — The Quicksilver Proving Grounds

- **Stratum / region:** Stratum III — open proving fields above the frontier, all chronometers and measured miles.
- **Domain · Class:** 13, Performance · Quicksilver Adept.
- **Prerequisites:** Floors 4 and 9.
- **Floor theme:** a proving ground where every claim of speed is made twice: once by the clock, and once by the output — and the second claim outranks the first.
- **Learning objective:** speed is a measurement with an equality proof attached, or it is a rumour — and the cost of a workload hides at its boundaries.
- **Chambers & ordinary encounters:**
  1. *Measurement discipline* — the chronometer house: the player benchmarks the Trio (Floor 9) against the guild's reference instruments on stated workloads; a run counts only if repeated runs agree within noise, and the harness must *report the variance*, not just the median.
  2. *Reading the machine's handwriting* — a hot mechanism's works are laid open (its assembly): the player finds one wasted check and one missed broadening (a vectorization), and removes both by restructuring the design, with the behaviour record unchanged.
  3. *Shape of the field (cache)* — the same troop drilled in two formations (array-of-structs vs struct-of-arrays): the player measures both, with *output equality proven between the formations* — the direction of improvement must be shown, not asserted.
  4. *Broad strokes (SIMD)* — the wide scythe: a broad-stroke sum with a narrow fallback, differentially proven against the narrow version on the adversarial lengths — none, one, and the lengths that are not multiples of the scythe's width. Where the wide scythe *loses*, the record says so; the crossover point is written on the grounds' board honestly. (The wide scythe's broader magic is practised only in its stable form here; its unstable experimental forms are fenced ground, belonging to Floors 15 and 17's study — §2.3.)
  5. *Allocation and errand economy* — a hot road re-walked with Floor 9's counting steward in attendance: allocations counted, then cut with the quarry; errands batched. The count must fall to the stated number, and the clock must follow the count.
  - *Secondary class (#8, Brass Eye, in miniature):* an early enthusiast on the grounds has been timing the *lantern-lighting* instead of the race — a proxy metric, exquisitely precise, wholly wrong — and the player must name the quantity that matters before the grounds will record their own runs.
- **Primary failure class:** #14, The Uncounted Last Mile — the general rule expires at the boundary, silently.
- **FOE:** **The Quicksilver Trial** — the grounds' great trial: a naive frontier workload (parse and aggregate the frontier's whole ledger-corpus, held in the Interner's Hoard) must be made ten times faster, with its output byte-identical. Every adept who has attempted it has hit the same wall: a fast start, a glowing chronometer — and a total that comes up *short by exactly the last portion*, or a speedup that evaporates on the full run.
- **The seam:** the workload's cost and its correctness both live in its final partial stretch — the tail chunk that is not a whole number of scythe-widths, milestones, or blocks. Attempts that treat the road as uniform either skip the tail (fast, wrong) or fall off their stride exactly there (right, slow); the trial's budget cannot be met by a uniform method.
- **Diagnosis:** the player profiles the naive workload honestly (the Lens, Chamber 1's discipline) and localizes both the missing output and the lost time to the terminal segment; the demonstration requires showing the shortfall equals the tail, to the item — the difference is the diagnosis.
- **Proof required:** the trial's two claims, both, always: the chronometer's median improvement meets the budget (tenfold), *and* the output's hash matches the naive reference's on every run — with any unsafe mechanism used on the fast path carrying its Floor 5 contract and passing the referee's named subset walk. An attempt that hits the number by changing the output fails by definition, however fast it is.
- **Repair strike:** re-lay the workload's road with its boundary as a designed stretch: the main road runs at full stride, and the last partial mile gets its own deliberate method (a named remainder, walked, not counted) — the split itself is the repair, and the proof covers both stretches.
- **Codex stone:** *The Last Mile Stone.* **Boundaries are part of the contract.** Half-open ranges, slice ends, the final partial chunk: the edges are where a general rule's assumptions expire. It works because the boundaries are stated in the open — in the shape of the range, in named remainders — where they can be handled deliberately instead of discovered, short of food, at dusk.
- **Loot:** **The Quicksilver Lens** — the benchmark harness and its handwriting notes: measurement with variance, equality with proof.
- **Wielded later:** Floor 14 (kernel hot paths), Floor 15 (code generation read with a profiler's eye).
- **Mastery unlock:** a performance claim anywhere later requires two things — a measured number and an output-equality proof; speed asserted without both is treated as a rumour, and keepers will say so.
- **Stairs relevance:** counts toward the Stratum III stairs' "any two more."
- **Platform / input notes:** all measurements in the fiction are *records*, not real device benchmarks — the grounds report the same numbers on every platform (§7.7, parity of information); no encounter measures the player's actual hardware.

---

### Floor 14 — The Root of the Tree

- **Stratum / region:** Stratum IV — the deep dark: the root chamber of the whole labyrinth, where the world's own operating machinery lives.
- **Domain · Class:** 14, Kernel & OS · Rootwarden.
- **Prerequisites:** Floors 12, 9, and 7. *Toolchain note: the root-engine's waking route — the hand-drawn road-spec and the build-std machinery beneath it — runs on nightly infrastructure. That dependence is Floor 17's subject, used here under its fence (§2.3); the machinery this floor itself teaches is stable throughout.*
- **Floor theme:** a tree's root hollow fitted as a working engine-room: boot-levers, a fault bell, page-ledgers, a heap grown in the dark, and two tireless servants (tasks) working by cooperation.
- **Learning objective:** nothing below this floor is magic — boot, memory, and scheduling are mechanisms the player has personally operated; every abstraction above them is now visibly a convenience, priced and understood.
- **Chambers & ordinary encounters:**
  1. *Boot* — on the Anchorite's Cell chassis (Floor 12 loot, visibly the same bones), the player brings the root-engine up: the scripted waking shows its banner in the waking-log and the harness's success sign.
  2. *Exceptions and interrupts* — the fault bell: the player registers the bell-keepers, rings a deliberate small fault, and survives a page-fault with a printed diagnosis (the faulting address, the cause) instead of the engine's death — the waking-log shows the diagnosis lines and the engine continues.
  3. *Paging and frames* — the page-ledger: the player maps a fresh page from the frame-store and proves the mapping by writing and reading through it; the proof lines appear in the log, and the frame-store's own trials run green in the side room.
  4. *A heap in the root* — the player grows a heap in the engine-room (first a linked chain of plots, then fixed blocks) until collections live in the root: a great shelf grown past one page, dropped, regrown, with the allocator's invariants holding throughout.
  5. *Tasks, and the far shore* — two servants work cooperatively by the roster (Floor 8's executor, recognized) with a keyboard-servant among them; then the capstone visit: a traced design from the far shore's real kernel tradition, mapped line by line onto Floor 7's synchronization charters and Floor 10's treaty — the correspondences named, in the player's hand.
  - *Secondary class (#15, Two Porters, in the root):* the engine-room's two service doors are claimed under Floor 7's house rule — the player posts the total order themselves; the root accepts no other arrangement.
- **Primary failure class:** #13, The Tide Table — a check performed honestly, acted on in a changed world.
- **FOE:** **The Root of the Tree** — the root-engine fully raised, which proves four subsystems alive on its waking-log — and which has begun, intermittently, to *grant pages it checked a moment ago*: a servant asks whether a frame is free; the ledger says yes; by the time the servant writes, another servant — woken by the fault bell between the check and the act — holds the same frame. Two servants' memories bleed into one page, and every post-mortem finds the check was performed, honestly, every time.
- **The seam:** the engine's grant is a note about the past ("frame 7 was free when asked") used as if it were the present. Between check and act, the world — an interrupt, a woken servant — changes. No one lied; the tide table was true at breakfast.
- **False seam (Stratum IV):** the seductive wrong diagnosis is *the frame-store's arithmetic* — its counts can be audited into glowing correctness (a Brass-Eye-flavoured misreading), and the player may fully assemble that case before the log shows a fault occurring with perfect counts.
- **Diagnosis:** the player reconstructs one bleeding from the waking-log's interleaving, exhibiting the check, the bell, and the act in order — and shows that the failing grants, and only they, lack a *held* witness: the servants who kept hold of the frame-token itself never bleed.
- **Proof required:** the engine's four proof lines on the waking-log (fault survived with diagnosis, a mapped page written and read back, a shelf grown past one page, two interleaved servants completing) with the harness's success exit — plus the adversarial evidence: a deliberate page-fault diagnosed, not fatal, and heap exhaustion under the stress producing the engine's documented behaviour (a named failure with the allocator's state printed), never silent corruption.
- **Repair strike:** abolish the note: the grant *is* the token. The ledger thereafter hands over the frame-token itself, held across the whole operation — check and act become one held state, and a token cannot be in two servants' hands. The Tide Table is replaced by the held rope.
- **Codex stone:** *The Tide Stone.* **Hold the proof, not a note about the proof.** Keep the parsed value — the held witness, the guard held across the operation — instead of a boolean about the past. It works because a held value *is* the state; a boolean merely describes a world that may have left.
- **Loot:** **The Rootwarden's Kernel** — the running root-engine itself: the terminal artifact of the descent, a system the player can point at.
- **Wielded later:** Niflhel — the engine running in the final room is this one, recognized as the player's own (§5.10); and all later systems work.
- **Mastery unlock:** boot, memory, and scheduling stop being magic anywhere in the player's future work; the abstractions above them are conveniences with known prices.
- **Stairs relevance:** one of the four FOEs whose fall unseals Niflhel.
- **Platform / input notes:** the waking-log is a steppable text record with the success-sign stated in words and symbol, not colour; the cooperative roster is turn-stepped (§7.7).

### Floor 15 — The Mirror Hall

- **Stratum / region:** Stratum IV — a hall of mirrors where the language looks at itself: every surface shows a mechanism reading a text about mechanisms.
- **Domain · Class:** 15, Compiler internals · Mirrorwright.
- **Prerequisites:** Floors 2 and 11. *This floor and Floor 17 are the fenced ground of §2.3: unstable and nightly-only machinery is studied here as a subject, labelled as such, and nothing outside these two floors depends on it.*
- **Floor theme:** mirrors within mirrors — the pipeline as a series of translations, each shown as a room the player walks a small text through.
- **Learning objective:** the compiler is a text that can be read — its stages, solver, and borrow-checker are mechanisms with traces, and its output is a primary source that outranks theories.
- **Chambers & ordinary encounters:**
  1. *The pipeline* — one small text is walked through the translation rooms (wording, parsing, and the descending forms of its meaning); the player annotates what each room *changed* — a mechanism per room, not a label.
  2. *Resolution and expansion* — a Floor 11 die's expansion traced through the mirror end-to-end, with the player naming where hygiene is decided; the chamber's proof is predictive: the player forecasts an unseen expansion's shape before the mirror shows it.
  3. *Type and trait checking* — a real failing bond is taken to the solver's well: the player reads the obligation tree and identifies the exact obligation that cannot be satisfied; the diagnosis is confirmed by the minimal change that makes the bond hold.
  4. *Borrow checking in the mirror* — a real borrow refusal is mapped onto the mirror's flow-form: the player maps which statements and region-constraints produce it, and predicts — before compiling — whether a stated change moves, alters, or dissolves the error.
  5. *The descent to iron* — one function followed from flow-form to machine handwriting, reconciled with Floor 13's handwriting skill; the write-up accounts for every nontrivial difference between the forms.
- **Primary failure class:** #5, The Lantern That Lights Itself — *second order:* the gate's own gate unrun.
- **FOE:** **The Mirror Turned Inward** — the hall's great lint-mirror, which reads the guild's texts and flags a named carelessness (the Floor 3 clerk's shout-suppressed habit: discarding fallible results bare-handed — the `unwrap` analogue of this world) outside the practice rooms. The mirror is celebrated. Its readings are green across the guild. And its *negative gallery* — the fixtures that must stay silent: the sanctioned invariant-shouts, the practice rooms, the infallible cases — has never once been run, because the mirror's keeper "knows it works." A text in the deep floors has begun exploiting exactly the infallible-case gap: formally within the mirror's pattern, actually a live hazard the mirror certifies daily.
- **The seam:** the mirror's positive readings are real and its verdict travellers trust is *unexamined in the half that matters*: an instrument validated only on what it must catch, never on what it must spare, is a lantern lit by confidence.
- **False seam:** the seductive wrong diagnosis is *the mirror's pattern is too narrow* — widen it, and it flags the practice rooms and the invariant-shouts, flooding the guild with false alarms while still certifying the true hazard's cousin. The player may assemble this case fully before the negative gallery's dust gives it away.
- **Diagnosis:** the player asks the Lantern question of the mirror itself — *show me the keeper's flame*: demands the mirror's full fixture record, exhibiting that the positive gallery ran and the negative gallery is dust-covered and unrun; then runs the negative gallery and watches the infallible-case fixture fail to stay silent.
- **Proof required:** the mirror's UI-trial record in the hall's own harness style: every positive fixture firing with its pinned wording and span, and every negative fixture silent — sanctioned shouts, practice rooms, infallible cases — or, where a case is genuinely out of scope, the mirror's own documentation stating the limit in its text (a stated limit is a pass; a silent miss is not).
- **Repair strike:** the negative gallery is hung beside the positive one, permanently, and the mirror's charter is amended: its green now means *both galleries ran*. The exploitation text is then re-read and refused — by the repaired mirror, in the player's presence.
- **Codex stone:** *The Second Lantern Stone.* **Examine the examiner.** A gate is itself a mechanism whose results are must-use: an instrument proven only on what it must catch has not been proven. It works because the looking is made mandatory at the second order too — the evidence about the evidence becomes part of the gate.
- **Loot:** **The Mirrorwright's Lens** — the lint-mirror plus the pipeline dissection dossier.
- **Wielded later:** Floor 17; and standing — the Lens is the exemplar of a gate the guild can run on itself, cited by keepers whenever a new gate is proposed anywhere in the descent.
- **Mastery unlock:** compiler output becomes a primary source for the player: when code fails below this floor, the mirror gets read before theories get formed.
- **Stairs relevance:** one of the four FOEs whose fall unseals Niflhel.
- **Platform / input notes:** fixture galleries are long lists — grouped, collapsible, identical across platforms (§7.3); mirror diagrams carry text equivalents (§6.7).

### Floor 16 — The Assay Office

- **Stratum / region:** Stratum IV — the depths' assay office: touchstones, balances, and furnaces where claims about metal are settled by test, not by shine.
- **Domain · Class:** 16, Verification · Assayer.
- **Prerequisites:** Floor 5. (Floor 7's standing required for the office's fourth chamber.)
- **Floor theme:** an office that grades evidence: every artifact that passes through leaves with a stated evidence class — example-tested, property-tried, model-walked, or proven — and the class is stamped on its papers.
- **Learning objective:** "it passes the tests" is never a sufficient claim — every artifact can state what *kind* of evidence stands behind it, and the statement means something because the office's own evidence is in order.
- **Chambers & ordinary encounters:**
  1. *Property trials* — the Touchstone Benches: a property suite for the Floor 9 key-cellar against a reference cellar — trials generated, results compared, failures *shrunk* by the bench to their minimal case. A seeded fault in a practice cellar is found and shrunk by the harness itself, in view.
  2. *The furnace (fuzzing)* — a furnace fed malformed charters from the Floor 3 parser's family for a stated budget of time; any crash is triaged to a root cause, repaired there, and pinned as a regression; the furnace's corpus is kept with the artifact, as law.
  3. *The referee as a gate* — the player assembles the referee's walks over Floors 5 and 9's loot into one scripted office gate, its covered suites named and its slow exclusions justified in the script's header; a deliberately seeded tenancy fault in a practice copy turns the gate red, on cue.
  4. *The model loom, exhaustively* — Floor 7's queue re-verified under the model loom with a stated exploration budget, and the player writes the account of what the bound does *and does not* prove — the account's limits stated as precisely as its claims.
  5. *Proofs proper* — the Proof Vault: bounded proofs over the Floor 9 quarry's invariants — allocations never overlap, the issuing pointer advances monotonically, reset truly empties — each verified by the vault's engine; a mutated quarry (one notch off in the bump pointer) makes exactly the expected proof fail, and the player names which before running it.
  - *The standing temptation (#5, as weather):* at every chamber, the office clerk offers the player the *summary page* — green totals, ready-made — instead of the records. Accepting summaries is never punished; the office simply stamps the resulting evidence class one grade lower, and the stamp is visible. The floor teaches by price tag, not by scolding.
- **Primary failure class:** #1, The Toll of Tallies — *second order:* the tally of proofs itself, summed past its evidence.
- **FOE:** **The Assay of All Things** — the office's master battery, which grades the player's whole accumulated hoard — and whose certificate of the descent is computed by *adding the green counts of its benches*: benches sharing fixtures, corpora, and one shared reference cellar, so the same evidence is counted twice and thrice; worse, one bench's green count includes trials that a sister bench *generated but never ran*, their scrolls copied into the tally by the copying-clerk. The certificate is a beautiful number. It is stamped, sealed, and wrong in exactly the way Floor 9 taught the player to refuse.
- **The seam:** the battery commits the Toll fallacy over *evidence itself*: green counts from overlapping suites, summed as if disjoint, including counts of trials that exist only as intentions. The office that exists to grade evidence has been grading its own with a bare number.
- **False seam:** the seductive wrong diagnosis is *one bench is miscounting* — each bench, audited alone, counts its own scrolls honestly; the lie is in the addition, and the player may audit two benches clean before the copying-clerk's duplicate scrolls surface.
- **Diagnosis:** reconciliation by identity, at second order: the player takes the certificate's total apart scroll by scroll — exhibiting the duplicated fixtures across benches and the copied-but-unrun trial scrolls — until the true evidence set remains: each distinct trial, run once, counted once, stamped with the bench that ran it.
- **Proof required:** the battery's full record, every leg green on the reconciled evidence: the property suites over two structures; the furnace's fixed 300-heartbeat budget run with its corpus kept; the scripted referee gate green end-to-end; the quarry's proofs green in the vault. Adversarial by construction: any fault any leg finds must be repaired at the root and added as a regression — a FOE with an unfixed find has not fallen, however green the other legs are.
- **Repair strike:** the office's certificate is re-founded as Floor 9's stamped count: the certificate becomes a sealed evidence-scroll — totals fused to the reconciled trial set, benches' overlaps exhibited on its face, unrun trials structurally uncountable (a scroll exists only when a trial ran). The copying-clerk is retired; the desk refuses bare green totals from anyone, including the office itself.
- **Codex stone:** *The Second Toll Stone.* **Evidence about evidence is still evidence.** A total over proofs obeys the same law as a total over crates: provenance is part of the value, overlaps are enumerated or the number does not travel. It works because the certificate's authority now rests on a checkable set, not on the office's reputation — the stranger can re-count.
- **Loot:** **The Assayer's Touchstone** — the battery as a reusable harness: benches, furnace, gate script, and vault, with the sealed-scroll certificate rite.
- **Wielded later:** retroactively over all Strata I–III loot (the office re-grades the player's hoard, and keepers' dialogue acknowledges the new stamps), and forward over everything built after.
- **Mastery unlock:** every artifact the player holds can state its evidence class, and the statement means something; "it passes the tests" stops being a sufficient claim in any later room — including, pointedly, the last one.
- **Stairs relevance:** one of the four FOEs whose fall unseals Niflhel.
- **Platform / input notes:** the reconciliation is the game's largest list interaction — batch operations and progress-by-count are required, with identical capability on touch and keyboard (§7.7); the furnace's budget is fictional time, stepped by the player, never wall-clock.

### Floor 17 — The Roadwarden's Terminus

- **Stratum / region:** Stratum IV — the end of the maintained road: the terminus where every road the guild keeps is surveyed, signposted, and handed over.
- **Domain · Class:** 17, Toolchain internals · Roadwarden.
- **Prerequisites:** Floor 11. Standing in Stratum III recommended. *Fenced ground (§2.3), as Floor 15: the road's unstable machinery is studied here as a subject.*
- **Floor theme:** a road-works terminus: feature-signposts, a generating mill, a surveyor's chain for dependencies, hand-drawn road-specs for foreign country, and the departure board where journeys are published.
- **Learning objective:** the toolchain is not weather — resolver, features, code generation, and packaging are mechanisms the player has operated by hand, and the road's future (editions, proposals) is read as part of the trade.
- **Chambers & ordinary encounters:**
  1. *Signposts and feature-graphs* — a crate-house whose option-signposts have tangled into a non-additive thicket (choosing one sign forbids another, silently); the player re-plants them as an additive graph and proves it with a journey-matrix over the combinations: every cell travelled, every trap named and shown removed.
  2. *The generating mill* — a mill that grinds a data-table into typed road-furniture at build time: the player wires its trip-stones so touching the data regrinds exactly the furniture, no more; the ground furniture is never stored in the warehouse (generated code is never checked in — the mill-house rule).
  3. *The surveyor's chain (the resolver)* — a staged dependency dispute in a fixture yard: houses requiring incompatible versions of one road. The player surveys who requires what, before and after, and applies the minimal re-survey that resolves it — changing no version the dispute didn't force.
  4. *Hand-drawn roads and the edition rite* — two rites in one chamber: the player builds Floor 12's Cell through a hand-written road-spec for foreign country (its smoke trial passing unchanged), and then performs the **edition rite**: an old house, built to an earlier edition's road-rules, is walked across the boundary to the current edition — the automated fix-cart makes its pass, the player reads the edition guide's border notices, and every changed joint is inspected. The chamber wall carries the Roadwarden's habit, posted as a duty roster: *read the road's future* — proposals (RFCs), stabilization notices, and edition guides are read here as routine maintenance, and the terminus notice-board (whose postings have been changing throughout the player's descent, §2.3) is finally read end to end.
  5. *The release rite* — the Gatehouse Key (Floor 3 loot) is taken through the full departure rite: packing, a publish dry-run, warning-free papers. The package's contents are audited line by line — exactly the intended files, nothing stray, nothing missing — because the stray file is exactly the file that ships to strangers.
- **Primary failure class:** #11, The Stranger's Anvil — an artifact is trusted because its maker is trusted, and fails in a stranger's hands because its recipe was never complete enough to rebuild it.
- **FOE:** **The Road to Everywhere** — the terminus's great departure: a multi-house expedition (the Die's house, the Cell's firmware, and the Gatehouse Key as the publishable artifact) to be sent out over every road at once — and its manifest has been prepared by the guild's most trusted quartermaster, from memory, in one sitting. The feature-matrix has cells no one travelled; the package contains a scratch log and a local signpost file nobody remembers adding; and the Cell's recipe, sent ahead to a stranger-smith in the far town as the rite demands, has come back *wrong* — not sabotaged, simply unrebuildable as written, one unrecorded step missing.
- **The seam:** every defect in the departure is a place where trust replaced a run: the matrix trusted, the listing trusted, the recipe trusted. The quartermaster is excellent; that is the seam. The rite exists because excellence is not evidence.
- **False seam:** the seductive wrong diagnosis is *the stranger-smith's incompetence* — the far town's smith can be audited into blamelessness (their forge, their measures, their other work all check out), and the player may fully assemble the case for blaming them before the missing recipe-step surfaces at home.
- **Diagnosis:** the player runs the rite instead of reading the manifest: travels the matrix cells until an untravelled one fails, audits the package listing line by line until the stray files surface, and re-forges the Cell at a *local* stranger's anvil from the recipe alone, reproducing the far town's failure at home — the three exhibits converging on the recipe, not the smith, and the manifest, not the roads.
- **Proof required:** the completed drill, every leg run: every matrix cell green; the departure dry-run verified; the papers built warning-free; the Cell built under the hand-written road-spec; the old house's edition crossing re-performed for the departure record — the fix-cart's pass made and every changed joint inspected against the guide's border notices; and the package listing audited line by line with the strays exhibited. A single stray file fails the drill — because that is exactly the file that ships to strangers.
- **Repair strike:** the recipe is repaired (never the blade, never the smith): the missing step is written in, the manifest is regenerated by the rite rather than by memory, and the departure is re-run end-to-end green before the expedition leaves. The terminus posts the drill as its standing charter.
- **Codex stone:** *The Anvil Stone.* **An artifact is its recipe.** Deterministic builds and a pinned manifest mean the same inputs rebuild the same artifact on any road. It works because trust moves from a person to a process anyone can re-run — the stranger's anvil is just the most honest build-yard.
- **Loot:** **The Roadwarden's Charter** — the workspace and its release checklist: the drill, made permanent.
- **Wielded later:** real projects, from the day it exists — the Charter is the one artifact whose "later" is outside the labyrinth, and the keepers say so.
- **Mastery unlock:** the toolchain stops being weather: a build the player depends on is a build they can re-derive — resolver, features, code generation, and packaging all read as mechanisms, because they have operated each one by hand.
- **Stairs relevance:** one of the four FOEs whose fall unseals Niflhel.
- **Platform / input notes:** the listing audit is a line-by-line list — line-level check-off with keyboard and touch parity; the notice-board is long-form text under §7.3.

### The sealed depth — Niflhel

Below Floor 17 there are no more floors. Niflhel's design is §5.10; it is recorded here, in the matrix's place, only to fix its status: **Niflhel is not Floor 18.** It has no domain, no class, no chambers, no FOE, and no loot. It is the win state (§5.10), and its seal is the fourth Stratum IV stairs gate: when Floors 14, 15, 16, and 17 have all fallen, it opens.

---

## 5. Systems design

### 5.1 Cartography and map-drawing rules

The map is a first-class system with the tracker's convention as its law:

- **The grid.** Each floor is a grid of squares: five chamber squares, one FOE square, and the connecting squares the player walks. Squares have three states, matching `MAP.md`'s marks: **BLANK** (uncharted — rendered as honest empty parchment, never as a teaser silhouette), **CHARTING** (chamber squares fill individually as chambers are repaired; a floor at n/5 shows exactly n filled chamber squares), **FALLEN** (the floor square fills when the FOE falls, with the date of the verified fall inscribed).
- **A square fills only on proof seen.** The fill event is the game's rendering of the proof state change (§5.4): a chamber square fills at the moment a chamber's proof is accepted, not when the player enters, explores, or survives the chamber. There is no fog-of-war and no exploration credit; the map records *knowledge*, not *presence*.
- **The player draws.** Filling is presented as the player's own hand drawing the square (a short, skippable inking animation, §6.4). The map is diegetically the player's artifact: it is what NPCs ask to see (§5.9) and what lies on the stone table at Niflhel (§5.10).
- **The map is complete information about progress.** Every prerequisite, stairs condition, and FOE status in the game is readable from the map screen. No progress state exists that the map does not show. Conversely, the map shows nothing unverified: FOEs are drawn standing from the start (they are known dangers), but no chamber's *solution shape* is ever drawn.
- **Failed work leaves no mark.** Collapsed FOE attempts and failed diagnoses draw nothing. Doubts are not map states; they belong to the expedition log (§5.8), the map's companion record.
- **Strata and the sealed depth.** The map is one sheet per stratum plus the cover sheet; Niflhel appears on the Stratum IV sheet as a sealed square labelled only with its name and its seal condition ("unseals when all four FOEs of this stratum have fallen"). Its interior is never drawn — at the win state, its square fills *around* the player's completed map rather than revealing a floor plan (§5.10).

### 5.2 Progression: classes, skill trees, prerequisites

- **Classes are floors.** Each domain is a class; "levelling a class" is working its five chambers in order. Chambers within a floor are strictly ordered (chamber n+1 opens when chamber n is REPAIRED), mirroring the curriculum's levels.
- **Prerequisites are the tracker's chart, verbatim.** The governing edges (from `MAP.md`; the list governs over any drawing):

  | Floor | Requires |
  |---|---|
  | 1 | — (entrance hall) |
  | 2 | 1 |
  | 3 | 1, 2 |
  | 4 | 1 |
  | 5 | 4 |
  | 6 | 5 through chamber 3 |
  | 7 | 5 (6 recommended) |
  | 8 | 2, 6 |
  | 9 | 4, 5 |
  | 10 | 4, 5 |
  | 11 | 2 |
  | 12 | 5 (9 recommended) |
  | 13 | 4, 9 |
  | 14 | 12, 9, 7 |
  | 15 | 2, 11 |
  | 16 | 5 (7 required for chamber 4) |
  | 17 | 11 (Stratum III standing recommended) |

  "Requires" means the prerequisite floor's FOE has fallen, except where a chamber depth is stated (Floor 6; Floor 16's chamber 4). "Recommended" never gates: a recommended-but-unmet prerequisite is shown on the map as a keeper's marginal note, and the floor's keeper remarks on it once — the game warns, it does not bar.
- **Mastery is cross-domain legality.** When a FOE falls, its floor's mastery (stated per floor in §4) takes effect everywhere: mechanisms, instruments, and standards from that floor become usable and *expected* in later chambers. Mastery is not a stat; it is the game's content gating what diagnoses are available to the player. The codex (§5.5) is the player's record of masteries.
- **No levels, no XP, no currency of power.** The only progression resources are: charted chambers, fallen FOEs, loot (§5.3), stones (§5.5), and the expedition log (§5.8). Nothing is purchasable and nothing is grindable; a stuck player needs a hint (§1.11) or a different floor whose prerequisites they meet — the graph always offers at least one open floor until Stratum IV.
- **FOEs are routable.** A standing FOE blocks only what lists it as a prerequisite. The map's openness is deliberate: the descent is a graph, not a corridor.

### 5.3 Equipment and inventory

- **The pack is the loot list.** Seventeen artifacts exist (one per FOE, §4), each unique, each named, each carried from the moment its FOE falls. There is no other equipment, no consumables, no vendor trash, and no inventory Tetris: the pack screen is a roll of seventeen slots that fill in fall order, each opening to the artifact's card (what it is, where it was won, where it is wielded — mirroring the tracker's inventory table).
- **Loot is wielded by being relevant.** An artifact's use is contextual and explicit: a chamber whose mechanism involves the artifact *names it* ("the Plumb Line is wanted here"), and its presence unlocks the diagnosis actions it enables. The player never guesses which item applies; the design rule is that loot gates *capabilities*, and capabilities announce themselves. Where the curriculum says an artifact is wielded, §4's floor specs say how.
- **Combinations are authored, not emergent.** Where two artifacts compose (the Anchorite's Cell as the Kernel's chassis; the Trio under the Assay Office's benches), the composition is a scripted chamber beat, not a crafting system. The guildhall workshop (§5.9) exists to *show* compositions and record recipes, not to be a crafting minigame.
- **The recipe wall.** Every artifact has a recipe — its construction record — kept in the workshop. Recipes are diegetic §2 documents: complete enough for a stranger to rebuild from. Floor 17's FOE turns on exactly this.

### 5.4 The resolution model: diagnose, prove, repair

All challenges — chamber and FOE — resolve through one state machine. This is the game's combat model; there is no other.

- **States.** UNWORKED → DIAGNOSED → REPAIRED for chambers; STANDING → EXPOSED → FALLEN for FOEs. DIAGNOSED/EXPOSED means: the required proof (§4, per floor) has been assembled and accepted by the floor's own mechanism.
- **Diagnosis actions.** A small verb set, identical on every floor: **Examine** (inspect a mechanism, inscription, ledger, or log — reveals structure), **Enumerate** (produce a census/list of a named set — the Forester verb), **Exhibit** (attach a found fact to the growing proof — the act that moves provenance from the world into the record), **Calibrate** (run a mechanism against a known landmark — the Blind Eye verb), **Cross-check** (measure the same quantity in a second frame — the Brass Eye verb), **Trace** (follow one instance through a record in order — the Tide/Herald verb). Floor-specific verbs may be added by loot (e.g., the referee's **Walk** from Floor 5) but the core six never change meaning.
- **Proof assembly.** Each challenge declares its required exhibits (in-fiction: the keeper names the standard; in-system: the proof panel lists slots by *kind*, never by content — "the overlap classes," not "click the third crate"). The player fills slots by Exhibiting facts found through diagnosis actions. A slot accepts only evidence of its kind; wrong evidence is refused *with the reason* ("that is the book's number; the slot wants the stone's").
- **The repair strike.** When all slots are filled, the strike action unlocks. The strike is one deliberate act, specified per floor in §4. Striking with a complete proof always succeeds — execution cannot fumble a proven repair. Striking is impossible without the proof; there is no brute-force path and no luck roll anywhere in the model.
- **FOE pressure.** From Stratum II, FOEs act during diagnosis: each has a stated pressure behaviour (the Loom's hands keep claiming stairs; the Suspended Hour's stale bells keep ringing) that *adds misleading or time-stamped evidence to the room*. Pressure never damages the player, never destroys proof already exhibited, and never imposes a real-time limit (§1.10); it raises the *discrimination* demand — the player must Exhibit the true record among decoys. Pressure behaviours are specified per FOE in content data, bounded, and identical across platforms.
- **Failure communication.** A refused Exhibit names the mismatch; a collapsed FOE attempt shows the proof panel with the slots that did not hold highlighted *in the world's terms*, returns the player to the floor camp, and costs nothing (§1.9). The game never says "wrong"; it says what the evidence showed.
- **Determinism of resolution.** Every challenge's evidence set, decoys, and acceptance are authored (§5.7). The same proof is accepted identically on every playthrough and every platform.

### 5.5 The codex in play

- **Stones.** Each FOE yields its floor's stone (§4). Each chamber cleared *at full standard* (no rung-4 hint, §1.11) yields a **facet**: a small stone carrying the chamber's secondary-class principle where one is assigned (§3.2), or a restatement of the floor's primary principle at chamber scale where none is. The codex screen is the pack's second roll: 17 stones, up to 85 facets, each readable at any rest point.
- **Reading is free and unpenalized.** The codex may be opened anywhere, any time, including mid-challenge. Deeper floors assume it (§1.12): keepers reference stones by name, and some proof slots are labelled with the stone whose principle governs the slot's kind — a kindness the game extends only for *earned* stones; unearned slots are labelled by kind alone.
- **The codex as equipment.** Three mechanics use the codex actively: (i) **Citing** — in designated chambers, Exhibiting a *stone* (rather than a world-fact) is the required proof kind, modelling "apply the principle you already hold"; (ii) **The Assay re-grade** — Floor 16's fall triggers the codex's own stamping: every stone gains its evidence class (§4.16); (iii) **Niflhel** — the completed codex is one of the two artifacts on the stone table (§5.10).
- **Inscription format.** Every stone's text follows §2.2: plain statement first, mechanism second, no syntax, no version claims. Stone texts are content-locked to §4's codex entries; paraphrase drift between the floor spec and the shipped text is a defect class in review (§8.5).

### 5.6 Stairs gates

Between strata, the stairs state their conditions as literal inscriptions, per the curriculum:

- **Stratum I → II:** FOE 1 fallen, plus FOE 2 or FOE 3; and every Stratum I lantern keeper-lit (in system terms: Floors 1–3's FOE crates at standing-gate standard — represented in-game by each Gatehouse-style check being completable; no separate grind is imposed: if FOEs 1 and (2 or 3) have fallen, the remaining valley FOE's lantern is lit by completing its chambers, FOE attempt not required for the stairs, though its square stays blank until it falls).
- **Stratum II → III:** FOEs 4 and 5 fallen, plus any two more of Stratum II's five.
- **Stratum III → IV:** FOEs 9 and 12 fallen, plus any two more of Stratum III's five.
- **Stratum IV → Niflhel:** all four FOEs (14, 15, 16, 17) fallen. The final stairs have no gate text (§5.10).

Stairs are *in addition to* per-floor prerequisites (§5.2), exactly as the tracker states. A stairs condition that fails is communicated by the inscription lighting its unmet clauses — never by a locked-door buzz.

### 5.7 State, saves, and determinism

- **Authored state, no procedural generation.** Every floor's layout, evidence sets, decoys, keeper lines, and pressure behaviours are authored content. Randomness is used nowhere in challenge logic. Where a mechanism in-fiction samples (Floor 9's census batches), the sampled sets are authored per challenge.
- **Seeds.** There is one determinism requirement: any ambient system that varies (firelight, dust, idle animation phase) derives from a per-save seed so that a save is a complete description of the game state; ambient variation never touches challenge state. Ambient seeds never affect difficulty, evidence, or outcomes.
- **Save model.** One save per *expedition* (campaign), autosaved at every state change of §5.4's machine and at every rest point; manual save at any rest point. A save records: chamber/FOE states, proof panels in progress, pack, codex, map state, log, hint usage (including rung-4 forfeits), settings, and the ambient seed. Saves are **portable files** (§7.5) — the same save must load on phone and desktop, because the descent may change doors mid-journey.
- **Checkpoints.** The rest points are: the guildhall, and each floor's camp (unlocked on first entry). Mid-chamber exit is always safe (§1.7); there is no checkpoint starvation anywhere, by pillar 2.
- **No irreversible choices.** Nothing in the campaign can be missed permanently, locked by a decision, or consumed. Hint forfeits (§1.11) are per-challenge and recorded, never campaign-ending.

### 5.8 Scoring and records

Niflheim has no score in the arcade sense. Its records are:

- **The map** (§5.1) — the primary record: what is proven.
- **The expedition log** — a dated journal the game keeps with the player: attempts, collapses, doubts, and falls, in the tracker's spirit ("failed attempts belong in the log, where they are useful"). The log is readable at any rest point and is the credits sequence's source text (§5.10).
- **The Cartographer's Standard** — an optional distinction, shown on the map's cover: a floor is *Standard-kept* if completed without any rung-4 hint (§1.11). The Standard is a record of *how* the descent was made, never a gate: Niflhel opens for every completed descent, Standard or not, and the game states this on the cover itself so no player mistakes it for a requirement.
- **Evidence classes** — after Floor 16, artifacts and stones carry their evidence-class stamps (§5.5); before it, the concept exists in keeper dialogue but stamps appear only from the Assay Office onward. Retroactive stamping is automatic on Floor 16's fall.
- **No leaderboards, no time records, no completion percentage** displayed as a number. (A bare "% complete" would be, in this game's own terms, an unstamped number; the map *is* the percentage, drawn honestly.)

### 5.9 The guildhall and the economy of trust

The guildhall is the between-floors space: rest point, workshop (§5.3), codex fire, and the descent's social memory.

- **Keepers remember proofs, doubt claims.** Keeper dialogue keys off map state: a keeper whose floor is charted speaks as to a colleague; a keeper whose floor is blank speaks as to a hopeful. Any NPC line that asserts the player did something keys off a filled square — the game may not claim, even in flavour text, a proof the map doesn't hold.
- **The map as credential.** From Stratum III, certain keepers and two Stratum IV gatekeepers ask to *see the map* before granting an audience that a prerequisite requires. The check is always exactly the prerequisite (§5.2) — the ritual is flavour over the same rule, and the game says so in the keeper's line: the map is believed because it cannot flatter.
- **No economy.** There is no money, no shop, no trade. The guild provides; the player's wealth is the pack and the codex. This is a design position, not an omission: a game about evidence does not sell its proofs.

### 5.10 Niflhel and the endgame

- **Opening.** When the fourth Stratum IV FOE falls, the player is returned to the Stratum IV stairs. The final stairs have no inscription. The sealed square on the map is shown filling *around* its border (§5.1). The descent to Niflhel is a single authored walk: no challenges, no pressure, lit only by the player's own lantern (the Gatehouse's flame, carried).
- **The room.** At the bottom: a room, the Rootwarden's Kernel running — the player's own engine, assembled across seventeen floors from their loot, presented running its four proof behaviours in a quiet loop — and a stone table bearing two artifacts: the completed map and the completed codex. The player places them (the one interaction in the room). There is no boss, no test, no final quiz. The room's single inscription is the map's own convention, restated: *a square fills only when its proof has been seen. You have seen them all.*
- **Ending and credits.** The game acknowledges the descent complete; the credits are the expedition log, read as a scroll — attempts and falls alike. The save is marked DESCENT COMPLETE with its date, mirroring the tracker's FOE-fallen dates.
- **After.** The completed game remains open: the player may revisit any floor (chambers replayable in their repaired state — a "walking the mended road" mode in which keepers discuss what was fixed), re-read the codex, and view the map. There is no New Game Plus and no post-game content tier; completeness is the ending. A fresh descent may be started as a new expedition (separate save), with the completed one retained.

### 5.11 Replayability

Replay value is designed as *re-reading*, not re-grinding:

- **The second descent reads differently.** Keeper dialogue has authored variants for players whose map already shows later floors (a fresh expedition does not inherit these; they key off *within-save* order — the prerequisite graph permits many orders, and keepers remark on unusual ones).
- **Order freedom is the replay surface.** Because floors open by graph, not by corridor, a second expedition can take Strata II–III in a materially different order, and chambers that cite loot (§5.3) play differently depending on what is already in the pack.
- **The Standard run.** The Cartographer's Standard (§5.8) offers a self-imposed replay goal with no content locked behind it.
- **No procedural remixing** is offered or promised: the authored evidence sets are the game. (A "remix" that altered evidence would break the provenance the game teaches.)

---

## 6. Narrative and presentation design

### 6.1 Framing story

A generation ago, the guild that keeps the Great Labyrinth began to fail at its work — not from malice, and rarely from ignorance of the parts, but at the seams. The framing is told in the prologue's three beats, all playable, none a cutscene longer than ninety seconds:

1. **The prologue expedition.** The player walks with a veteran expedition that marches out on a bell rung in the wrong wing (the Herald, class #2, as lived experience). The march is wasted; nobody is hurt; the veteran's comment is the game's thesis in one line: *"The bell was real. The room was wrong."*
2. **The empty chair.** At the gatehouse, the player sees the season's caravan pass under a green lantern over an empty keeper's chair, into a flooded pass (the Lantern, class #5, as witnessed harm — the caravan turns back, wet and angry; the harm is embarrassment and cost, not death: the game's stakes are trust and livelihood, never gore).
3. **The commission.** The guildmaster — a keeper of keepers who has read the old maps — gives the player a blank map, a codex with empty settings, and the commission: descend, repair, and *draw only what you have proven*. The first blank square is shown. The game begins.

The frame plot does not develop a villain. The antagonist is a *class of error*, and the story's arc is the map filling. Character arcs belong to the keepers: each floor's keeper begins as the custodian of a broken mechanism and ends (on the FOE's fall) as its colleague-witness, with one short authored exchange each — the descent's emotional through-line is a guild slowly becoming able to trust its own instruments again, because someone finally checked.

The ending is §5.10's room. There is no twist: Niflhel was never a monster. The darkness at the bottom is simply the place where everything is the player's own work, running.

### 6.2 Floor-region identities

Each stratum has a region identity; each floor a distinct place within it (full specs in §4):

- **Stratum I — The Home Valley (surface, daylight).** Guild country: vaults, plugin-houses, the gatehouse. Palette of parchment, brass, green fields. The valley is the only stratum with weather and civilians; it establishes what the mechanisms are *for* — caravans, crossings, travellers — so deeper abstractions never lose their purpose.
- **Stratum II — The Machine Warrens (underground, lamplight).** Mechanisms run exposed: the player stands inside a lock while it is held, walks a memory-yard, watches a borrow being honoured. Cross-sections and contract cards everywhere; the warrens' beauty is *legibility* — everything shows its seams, which is why failures here are findable.
- **Stratum III — The Working Frontier (border country, dusk).** Hoards, the treaty port, hermitage cells, proving grounds: the places where the guild's work meets scale, foreigners, poverty of means, and speed. The frontier is wider, emptier, and less supervised — keepers here are peers, not teachers.
- **Stratum IV — The Deep Dark (below the maintained road).** The Root, the Mirror Hall, the Assay Office, the Terminus. Light is carried, not provided. Keepers here speak as equals or not at all. The dark is quiet, not hostile — nothing in Stratum IV jumps out; everything in it *matters*.
- **Niflhel — the unlit room.** Designed as the visual inverse of the valley: no weather, no civilians, one engine's quiet light, two artifacts on a table.

### 6.3 Visual language

- **The world is drawn as the map is drawn.** One visual hand across game and cartography: ink, parchment, brass, and stone. The map is not a menu skin over the game; the game is the map at walking scale, and the visual language must make that identity felt — floor geometry echoes grid squares; chamber thresholds are drawn as map-square borders.
- **Mechanisms are shown, not symbolized.** Every challenge mechanism is a visible machine whose moving parts are its moving parts: flags fly, bells hang, looms shuttle, shelves grow. A player must be able to *watch* the mechanism work in its demonstration and watch it fail in its broken state. Abstract glyphs stand only for abstract things (stones, seals, stamps).
- **State is read from the world first, UI second.** A chamber's state is legible from its mechanism (the lantern's flames, the flag-staff's token, the census slate's fullness) before any HUD element is consulted. HUD confirms; the world declares.
- **Failure classes have a visual accent, used sparingly.** The one recurring accent is *the unstamped thing*: objects lacking their seal/stamp/provenance render subtly *unfinished* (a missing seal-wax, a blank where a stamp belongs) — the player learns to spot the absence the way the craft spots a missing contract. The accent is an absence, never a red glow: the game does not highlight its own answers.

### 6.4 Animation principles

- **Animation demonstrates mechanism.** Idle and demonstration animations exist to show how a mechanism behaves (the executor's attendant sleeping and waking; the loom's claim order). Decorative motion is confined to ambient life (firelight, dust, water) and never competes with a mechanism's motion.
- **Repairs are the animation set-pieces.** Each FOE's repair strike has one authored animation: the mechanism *visibly becoming unable to fail that way again* (the second socket being cut, the bell system re-founded, the token replacing the note). The animation budget is spent on §5.4's commit moments, not on traversal.
- **Map inking.** Square-filling is a short hand-inking animation (under two seconds, always skippable, §1.9), identical in meaning on all platforms.
- **Motion safety.** All animation obeys §6.7's motion rules: no rapid flashing, no luminance inversion transitions, scene changes by wipe/crossfade at walking pace, and a Reduced Motion setting that replaces travel and flourish animations with fades — including, specifically, stair descents and the Niflhel walk.

### 6.5 UI/UX structure

- **Four screens, everywhere available at rest:** the **Map** (home), the **Floor** (the play view), the **Pack & Codex** (two rolls, one screen, tabbed), and the **Log & Settings**. In-challenge, the play view carries the **proof panel** (§5.4) as a persistent, collapsible element — never a modal that dismisses the world.
- **The proof panel is the signature UI.** Slots by kind, exhibits attached as physical-feeling cards, refusals annotated in place. It must be fully operable by touch, keyboard, and mouse with identical results (§7.2), and its state must survive session interruption exactly (§7.4).
- **Text is a primary surface.** Inscriptions, log-slates, treaties, and the codex are long-form reading inside the game. Typography rules are §7.3's; the UI rule here is that no inscription is ever *only* decorative — if it is written, it can be read, in full, in a dedicated reading view with the player's chosen text size.
- **No information lives only in transient presentation.** Anything a keeper says that matters is also written in the log. Anything demonstrated can be re-demonstrated. Anything refused states its reason in persistent text.
- **Diegetic first, chrome second.** Where a system has a world-object (the map, the codex, the pack), the UI is that object, framed. Menus that are not objects (settings, save management) are plain, honest chrome in the same hand.

### 6.6 Sound and music direction

- **Mechanisms have sounds; sounds have provenance.** Bells, flags, looms, and presses sound as themselves, and — by the game's own Herald rule — a sound is always attributable to its source on screen. There are no unattributed stingers. A completion bell the player cannot locate is a bug, in-fiction and in fact.
- **Music is regional and sparse.** Each stratum has one musical identity (valley: open strings and daylight; warrens: wood and mechanism rhythm; frontier: wide, few notes; depths: near-silence with a single carried theme — the valley's theme, slowed, as the descent deepens). Music never plays *over* a diagnosis-critical sound; the mix ducks for mechanism audio by rule.
- **Silence is a designed state.** The Stillroom's sleeping hall, the frozen river's held breath, and Niflhel's room are near-silent by intention, and the design treats that silence as content (the player *hears* the Freeze's totality).
- **No audio-only information.** Every fact carried by sound is also carried visually (§6.7). Audio cues are reinforcement, never the sole channel.

### 6.7 Accessibility — a designed requirement

Accessibility is specified here as requirements on the design, not as a post-production pass. A build that fails any item in this section fails its quality gate (§8.5).

- **Reduced motion.** A Reduced Motion setting (persisted, offered at first launch before the title screen) replaces all non-essential animation with fades: travel, inking flourishes, stair descents, ambient motion. Mechanism demonstrations remain *steppable* — with motion reduced, every demonstration and FOE pressure behaviour advances on player input instead of playing in time.
- **Photosensitivity safety.** No sequence in the game may flash at more than three flashes per second, and no transition may use luminance inversion (black↔white strobing) at any rate. Scene changes are wipes and crossfades. The lantern flames, forge effects, and mirror flashes are brightness-bounded by content rule, and the whole game's luminance profile must pass an automated photosafety analysis pass over captured footage before release (§8.5). This is a hard gate, not a review note: a mechanism whose honest depiction would strobe is redesigned (the design has one precedent in spirit: effects are depicted by state, not by strobing).
- **Readable text.** All text meets §7.3's size floor at the player's chosen scale (three scales plus system-scale respect on mobile), with a high-contrast text option, a dyslexia-friendly typeface option, and no text rendered over busy art without a backing plate.
- **Remappable controls.** Every input action (§7.2) is remappable on keyboard; touch targets are sized per §7.2 and their layout adjustable (left/right handed). No action requires a chord, a hold longer than one second, or simultaneous inputs.
- **Non-colour-only signals.** Every distinction the game makes by colour is also made by at least one of: shape, icon, label, pattern, or position. Named instances that must comply: flag-states on Floor 6 (tokens differ by shape and label), waits-for cycle highlighting on Floor 7 (labelled and outlined), lantern flames on Floor 3 (two flames, distinguished by position and label), evidence classes (stamps differ by wording and seal shape), map states (blank/charting/fallen differ by ink fill *and* inscription).
- **Cognitive accessibility.** The hint ladder (§1.11) is always available; no content is timed by default (§1.10); every mechanism can be re-demonstrated; the log preserves all important statements; the codex is readable anywhere. The game may be hard to *understand*; it may never be hard to *operate*.
- **Subtitles and text equivalents.** All spoken/voiced content (if any is produced) is subtitled; all audio-carried reinforcement has the visual equivalent §6.6 requires; all diagrams (waits-for graphs, state censuses, mirror forms) have text equivalents reachable in the reading view.

### 6.8 Writing voice

- **Plain first, mechanism second** — the codex's rule (§2.2) is the whole game's prose rule. Keeper dialogue states the situation plainly, then precisely. Nobody speaks in riddles; difficulty lives in mechanisms, not in obscured text.
- **The guild's register** is working language: ledgers, treaties, charters, rites. Refusals are courteous and specific; praise is specific or absent. Keepers never mock an error — the guild's failure mode was never stupidity, and the writing may not pretend otherwise.
- **Humour** is dry, sparse, and always at the expense of *systems*, never of people or of the player's mistakes (the copying-clerk of the Assay Office is remembered with weary affection, not contempt).
- **No real-world references:** no brands, no in-jokes about real tools or languages beyond the craft itself, nothing that dates. The world is sealed; its only imports are principles.

### 6.9 Tutorialisation through play

- **The first floor is the tutorial, and is not labelled one.** Floor 1's chambers introduce, in order: examination, the proof panel, census-taking, the strike, the map, the codex, and the hint ladder (the Oathwarden offers the ladder unprompted, once, in-fiction). Each system is introduced at the moment of first need, in a chamber whose mechanism is forgiving, and is then *assumed* — later floors never re-teach, per the concept design's codex doctrine.
- **New verbs are introduced one at a time** (§5.4's six core verbs across Floors 1–4; loot verbs at their floors). Each new verb is introduced by a keeper demonstration, then a low-stakes use, then the chamber's real use.
- **The FOE tutorial is Floor 1's audit** — a FOE whose seam is a census the player has already effectively performed in the chambers, so the first boss teaches the *shape* (diagnose → prove → strike) at minimal discrimination load.
- **No text-wall tutorials.** No system is explained in more than three consecutive lines of dialogue before the player does it. If a system cannot be taught by a demonstration and a small use, it is redesigned until it can.

---

## 7. Platform design (D-800)

### 7.1 The ruling

The Owner has ruled (D-800, 2026-11-03): **Niflheim must be playable on Android phones, Linux desktop, and Windows desktop.** This section is the binding interpretation of that ruling. It does not choose an engine (§7.8 states the constraints the choice must satisfy). "Playable" means: the complete game — every floor, every mechanic, every record — winnable start to Niflhel on each platform, with no platform-exclusive content, no platform-missing mechanic, and no platform advantage (§7.7).

### 7.2 Control mappings

One action set (§5.4's verbs plus navigation) is bound to all three input families. The mapping below is the default; keyboard bindings are remappable (§6.7).

| Game action | Touch (Android) | Keyboard (desktop) | Mouse (desktop) |
|---|---|---|---|
| Move / navigate floor | Tap destination square; swipe to pan the view | Arrow keys / WASD to step; Tab to cycle points of interest | Click destination; wheel to scroll view |
| Examine | Tap a mechanism or inscription | E (with focus shown) | Click / hover-pause |
| Open reading view | Tap the inscription's read glyph | R | Click the read glyph |
| Enumerate (census) | Tap "Enumerate" verb chip, then tap entries | N, then arrows + Enter per entry | Click verb chip, then click entries |
| Exhibit (attach evidence) | Drag exhibit card to proof slot, or tap card then tap slot | X, arrows to choose slot, Enter | Drag, or click card then click slot |
| Calibrate | Tap verb chip, then tap landmark | C, then select | Click verb chip, then click landmark |
| Cross-check | Tap verb chip, then tap second frame | K, then select | Click verb chip, then click |
| Trace | Tap verb chip; step with Next/Prev chevrons | T; step with , and . | Click verb chip; step with on-screen chevrons |
| Repair strike | Tap the (unlocked) strike seal and confirm | Enter on the strike seal, confirm | Click strike seal, confirm |
| Map | Map tab / two-finger tap | M | Map tab |
| Pack & Codex | Pack tab | I (pack), B (codex) | Tabs |
| Log | Log tab | L | Log tab |
| Hint ladder | Keeper portrait / hint glyph | H | Hint glyph |
| Pause / settings | System back or pause glyph | Esc | Pause glyph |
| Advance dialogue / demonstration step | Tap | Space / Enter | Click |
| Confirm / cancel | On-screen Confirm / Back | Enter / Esc | Click / right-click as Back where consistent |

Rules: every action exists in every family (no touch-only or keyboard-only verbs); tap targets are at least 48dp on Android; no action requires multi-touch, long-press over one second, chords, or precise dragging (drag always has the tap-tap alternative shown above); the strike always requires a deliberate two-step confirm on every platform, because it is §5.4's commit point.

### 7.3 Orientation, responsive UI, readability

- **Orientation:** landscape is the reference layout on all platforms; portrait is fully supported on phones with a reflowed layout (play view above, proof panel as a bottom sheet). The game never locks orientation and never loses state on rotation (§7.4).
- **Responsive rules:** the layout has three bands — phone (≤7"), tablet/small window, desktop — defined by available width, not by platform, so a resized desktop window gets the phone layout rather than a squashed desktop one. In every band: the proof panel is reachable without dismissing the play view; lists (censuses, rosters, listings) reflow to single column at phone band; diagrams (waits-for graph, state censuses) scale to band width with labels intact — pinch-zoom is permitted as an enhancement but may never be *required* to read a label.
- **Readability budgets (hard):** body text ≥ 16sp equivalent at the default scale in every band; inscriptions and codex text use the reading view with player scale (three scales + system font-scale respect on Android); contrast ≥ 4.5:1 for body text, ≥ 3:1 for large display text, with the high-contrast option (§6.7) raising both; tabular figures for all quantities (counts, measurements, tallies) in every band — the game's numbers are evidence and must look like it.
- **Safe areas:** all interactive elements respect Android gesture/nav insets and desktop window chrome; nothing interactive within 24dp of a screen edge on phones.

### 7.4 Phone session and resume behaviour

- **Session shape:** the phone is a first-class door (§1.7): chambers are 3–10 minutes; the guildhall is reachable between chambers in at most two taps; the game explicitly supports the "one chamber on the bus" session.
- **Interruption is a non-event.** Incoming call, app switch, OS process death, rotation, or battery death: on relaunch the game restores the exact §5.4 state — floor, chamber, proof panel as assembled, pressure state, scroll positions — from the autosave (§5.7), with no penalty, no repeated demonstration, and no "session expired" concept. The design target: resume to the exact prior screen in under two seconds on a mid-range phone.
- **No background obligations.** The game requires nothing of the player when closed: no notifications, no timers that run in absence, no streaks. (The frozen river's clock is fictional and pausable; §5, §7.7.)
- **Desktop resume** obeys the same rule: closing and reopening restores exact state; a desktop session and a phone session are the same descent behind different windows (§7.5).

### 7.5 Offline behaviour and save portability

- **Fully offline.** The game has no network features, accounts, telemetry, ads, or in-app purchases. It must be completable in airplane mode on a fresh install; any build whose first run requests a network permission fails §8.5's gate.
- **Save portability.** The expedition save (§5.7) is a single documented file in an open, versioned format: exportable and importable through the platform's ordinary file sharing (Android share sheet / file picker; desktop file dialogs), with a format version and a migration path — a save written by version *n* of the game loads in version *n+1* or the game states, in words, why not and what to do. Cross-platform continuation (start on desktop, continue on phone) is a supported, tested flow (§8.6), not a happy accident.
- **Settings portability** follows the save (accessibility and control settings travel with the expedition, and are also honoured device-locally when they conflict — accessibility wins over portability, in the player's favour, and the conflict is stated).

### 7.6 Performance and readability budgets

Budgets are stated as requirements on the *experience*, engine-agnostic:

- **Frame budget:** the game is not reflex-based; the requirement is smoothness of presentation — scene rendering must hold the device's native refresh on a mid-range Android phone of the release year for all ambient and mechanism animation, and must degrade by reducing ambient motion (never mechanism legibility) on weaker devices.
- **Load budgets:** cold start to interactive title ≤ 3 s on the reference phone; floor load ≤ 1 s; resume ≤ 2 s (§7.4). The full game (all 17 floors' content) must fit a download/storage budget appropriate to a phone: target ≤ 500 MB installed, stated here so content production (§8.4) plans asset weight per floor.
- **Battery/thermal:** no background work when suspended; sustained play must not thermally throttle a mid-range phone within a 45-minute session at default settings (mechanism animation is the load; it is budgeted, ambient-first to degrade).
- **Memory:** the game must run within the per-app limits of a 4 GB-RAM Android phone without eviction during a session, including at the map and census screens (the largest list states).

### 7.7 Input equivalence and parity

- **Parity of content:** identical floors, evidence, decoys, pressure behaviours, hint text, and outcomes on all platforms. There is no platform-exclusive chamber, item, or ending.
- **Parity of difficulty:** no challenge anywhere is gated on reaction time, input speed, precision pointing, or multi-input dexterity (§1.10). Timed fictional clocks (the Ice Ledger escort) advance on a turn/step basis or a generously scaled, pauseable timer whose rate is identical across platforms — device performance may never change a fictional clock's relative rate.
- **Parity of information:** every fact available on one platform is available on all: no hover-only facts on desktop (hover content is also tap/focus content), no small-print facts on phone (reading view exists in every band), no colour-only facts anywhere (§6.7).
- **Parity of record:** the map, log, Standard, and evidence stamps (§5.8) mean the same thing on every platform; a save's records are indistinguishable by the platform that produced them.

### 7.8 Engine-selection constraints (no engine chosen)

The engine decision is a later commission. The three-platform ruling plus this design impose these constraints, which any candidate must satisfy *with evidence* (a built proof on each target), not with promises:

1. **Targets:** ships production builds to Android (phone form factor, current and previous two major OS versions), Linux desktop (x86_64), and Windows desktop (x86_64) from one codebase and one content set.
2. **Input:** supports touch, keyboard, and mouse with the full action set of §7.2, including remapping support and tap-tap alternatives to drag.
3. **Text:** renders long-form text with player scaling, high contrast, and a pluggable typeface (for §6.7's options) without per-platform forks; tabular figures available.
4. **State:** the §5.4 state machine, proof panels, and save format (§7.5) are implementable as data the design owns — the engine must not force challenge state into engine-proprietary storage that would break save portability.
5. **Determinism:** presentation-layer variation only; the engine's physics/animation randomness must be seedable or excludable from anything §5.7 requires to be exact.
6. **Offline & footprint:** no mandatory runtime network calls, storefront lock-in, or account systems; installed size compatible with §7.6's budget.
7. **Accessibility hooks:** reduced-motion and photosafety controls (§6.7) implementable at the presentation layer; automated capture of footage for the photosafety pass must be possible.
8. **Longevity:** the engine's licence permits commercial and non-commercial release on all three targets without per-platform royalties that would change the product's economics (a business constraint the Owner will weigh), and its maintenance horizon plausibly exceeds the game's support horizon.

**Decision criteria (for the later commission), in the dialect's priority order:** (a) *safety of the choice* — proven three-target shipping record, save-portability feasibility, licence clarity; (b) *performance* — meets §7.6 budgets on the reference phone with headroom; (c) *developer experience* — iteration cost for authored content (this game is content-heavy and mechanism-light: tooling for floors, lists, and text outweighs rendering sophistication). The commission should build the same vertical slice (§8.2) in at most two finalists and decide on evidence, exactly as the game teaches.

---

## 8. Scope and production design

### 8.1 Content inventory (the whole game, counted)

| Content | Count | Notes |
|---|---|---|
| Strata / regions | 4 + Niflhel | §6.2 |
| Floors | 17 | one per curriculum domain, §4 |
| Chambers | 85 | 5 per floor, mirroring the domains' levels |
| FOEs | 17 | one per floor; 2 carry second-order primary classes (§3.3) |
| Failure classes | 15 | all assigned (§3.2): 17 primary assignments (15 classes, 2 recurring at second order), 18 secondary assignments |
| Artifacts (loot) | 17 | one per FOE, §5.3 |
| Principle stones | 17 | one per FOE, texts locked to §4 |
| Codex facets | up to 85 | one per chamber at full standard (§5.5) |
| Keepers | 17 | one per floor, plus the guildmaster |
| Stairs gates | 4 | §5.6 |
| Core diagnosis verbs | 6 | §5.4, invariant across floors |
| Platforms | 3 | Android phone, Linux desktop, Windows desktop (§7) |
| Endings | 1 | Niflhel (§5.10); the descent may be *stopped* anywhere, never *ended* elsewhere |

### 8.2 Vertical slice specification

The vertical slice proves the game's thesis end-to-end on real targets. It contains:

- **Guildhall shell:** rest point, map screen, pack & codex screen, log, settings (including Reduced Motion, text scale, high contrast).
- **Floor 1 complete:** all five chambers, the Borrowed Hoard FOE, loot, stone, hint ladder at all four rungs, the Stratum I stairs gate functional (with Floors 2–3 present as locked map squares whose prerequisites display).
- **Floor 4 complete** (one Stratum II floor, entered via a slice-only stairs waiver recorded as such in the build): proves loot-wielding across strata (the Plumb Line in play), a second failure-class family (measurement), and cross-check/trace verbs.
- **Systems:** full §5.4 resolution state machine; proof panel at phone and desktop bands; autosave + exact resume; save export/import.
- **Platforms:** Android phone + one desktop OS built and playable; save file moved phone → desktop and continued (the §7.5 flow, exercised, not asserted).
- **Accessibility:** §6.7's reduced-motion, non-colour signals, and text scaling functional; photosafety analysis run over captured slice footage.

**Slice exit criteria:** a first-time player, given no external help, completes Floor 1 in one sitting and can state, in their own words, what the Plank principle is; the slice passes §8.5's gates for its scope; the save-portability flow passes on the two built platforms.

### 8.3 Alpha scope

- Strata I and II complete: Floors 1–8, all chambers, FOEs, loot, stones, stairs gates I→II and II→III functional.
- All §5 systems functional end-to-end (codex facets, Standard record, Assay stamps stubbed as locked content, guildhall trust dialogue for built floors).
- All three platforms building; content complete for built floors; polish (animation set-pieces, regional music) at draft quality.
- Floors 9–17 present on the map as charted-but-sealed regions (their squares and prerequisites display; their content is not in the alpha).

### 8.4 Full-release scope

- All 17 floors and Niflhel, at the full standard of §4: every chamber, FOE (including Stratum IV false seams), secondary-class encounter, and keeper arc.
- All systems at full function, including Floor 16's retroactive evidence stamping and the workshop recipe wall.
- All three platforms at release quality, simultaneous content parity (§7.7), and the complete §6.7 accessibility set.
- Full audio: regional music and mechanism sound per §6.6.

### 8.5 Quality gates

A build is releasable only when all of these pass, with evidence (the game's own doctrine applies to its production: gates report records, not lights):

1. **Code standard:** the game's own code is written to the House Dialect (§2): `cargo fmt --check` clean, strict clippy at `-D warnings` clean, tests green on all three platform targets' CI-equivalent local runs. (If the chosen engine imposes a different primary language for some layer, that layer follows the equivalent house standard for its language, and the deviation is recorded.)
2. **Content consistency:** an automated check that every shipped stone text matches §4's codex entry for its floor; that every floor's in-game prerequisites match §5.2; that every artifact's "wielded in" card matches §4; drift is a build failure, not a review comment.
3. **Photosafety:** automated analysis over captured footage of every animated sequence, against §6.7's flash and luminance rules. Hard gate.
4. **Accessibility audit:** §6.7's items verified by test and by inspection — remapping works, targets sized, non-colour signals present on the named instances, text scales hold layout at all bands.
5. **Parity matrix:** the §8.6 parity playtest suite passes on all three platforms from the same save file.
6. **Save integrity:** migration tests for every prior save-format version; corruption of a save copy produces a stated, recoverable refusal — never a silent reset.
7. **Offline verification:** fresh install, airplane mode, full Floor 1 completion, on Android.
8. **Performance budgets:** §7.6's budgets measured on the reference devices, recorded with the measurements' environment.

### 8.6 Playtest criteria

Playtesting is gated by criteria, not by vibes:

- **Comprehension:** ≥ 8 of 10 first-time players complete Floor 1 unaided beyond rung 2 of the hint ladder; ≥ 7 of 10 can state the floor's principle unprompted afterward (exit interview, their words).
- **Diagnosis honesty:** across Floors 1–4, observe whether players attempt strikes before proofs (the design forbids them; if players *look* for a brute-force path repeatedly, the teaching beat is failing, and the chamber is revised — the metric is observed behaviour, not completion).
- **Session fit (phone):** median chamber completion 3–10 minutes on phone; resume-after-interruption tested by scripted interruption (call simulation, process kill) with zero state-loss findings.
- **Discrimination curve:** Stratum II FOE first-attempt collapse rate should be visibly nonzero and second-attempt success high — a FOE nobody fails is not testing diagnosis; a FOE nobody passes on the second attempt is not teaching.
- **Parity:** the same scripted playthrough (same inputs by action, per platform binding) reaches identical proof states on all three platforms.
- **Accessibility panel:** at least one playtest pass with players using reduced motion, largest text scale, high contrast, and remapped controls, each completing Floor 1.
- **The recognition test (release-level):** players who finish the game and are then shown a page of elementary Rust should, in interview, recognize the principles' shapes at a rate the production lead accepts as the game's learning claim — this is the game's reason to exist, and it is tested like it.

### 8.7 Production sequence

1. **Design lock** (this document) → 2. **Engine commission** under §7.8 (finalists build the §8.2 slice core) → 3. **Vertical slice** (§8.2) and its exit criteria → 4. **Content production in stratum order** (Stratum I floors first; systems built floor-driven, not speculatively — a system is built when its first floor needs it, generalized when its second floor needs it) → 5. **Alpha** (§8.3) → 6. **Strata III–IV content + Niflhel** → 7. **Hardening:** §8.5 gates, §8.6 playtest rounds → 8. **Release** on three platforms with parity.

Standing production rules: no floor enters production without its §4 spec being final (they are, in this document); stone texts are locked at floor production start and changed only by a recorded amendment; every production gate follows the game's own doctrine — records over lights, and a failed gate stops the line, not the reporter.

### 8.8 Build-readiness checklist

The build may start when every box is honestly checked:

- [ ] This design is accepted by the Owner, including §8.10's decisions that gate the build.
- [ ] Engine chosen under §7.8, with three-target proof builds in hand.
- [ ] Reference devices named (one mid-range Android phone, one Linux desktop, one Windows desktop) and §7.6 budgets re-confirmed against them.
- [ ] Vertical slice team/roles assigned (design, engineering, art, audio, QA/playtest).
- [ ] Content pipeline exists for: floor layouts, evidence sets, keeper dialogue, stone texts — with the §8.5 consistency checks automated from the first floor, not retrofitted.
- [ ] Accessibility (§6.7) and photosafety tooling in the pipeline from the slice onward.
- [ ] Save format v1 specified, with the migration test harness.
- [ ] Storefront/publishing requirements for the three platforms surveyed (accounts, policies, signing) — surveyed, not necessarily completed.
- [ ] The curriculum documents remain the syllabus of record; a change-control note states how a curriculum amendment would propagate into shipped floor content (by recorded amendment, never silently).

### 8.9 Risks

| Risk | Shape | Mitigation in this design |
|---|---|---|
| Content volume | 85 authored chambers + 17 FOEs is a large authored surface; puzzle quality may vary | Floors share one resolution model and six verbs (§5.4); production in stratum order with per-stratum exit reviews; slice validates the template before mass production |
| Difficulty miscalibration | Diagnosis puzzles can silently become either trivial or opaque | §8.6's discrimination criteria measured per stratum; hint ladder gathers per-challenge usage data (locally, in-save) that names the miscalibrated chambers |
| The game drifts into a coding game | Pressure to "teach real Rust" by adding syntax | §1.2 and §2.4 forbid it structurally: no text input of code, no syntax-dependent challenges; the curriculum is the other artifact, on purpose |
| Engine commission stalls | §7.8's three-target proof is real work | The design is engine-free by construction; content (floors, texts, evidence sets) can be produced in engine-neutral form while the commission runs |
| Platform store friction (Android) | Policies, signing, review timelines | Surveyed at §8.8; the game has no accounts, ads, or network features — the friction profile is minimal by design |
| Scope creep toward systems not in §5 | Crafting, economies, and combat stats are the genre's gravity | §8.10 requires Owner decision for any new system; §5's systems list is closed |
| Accessibility treated as a pass at the end | Retrofitting motion/text/input is expensive | §6.7 is a quality gate (§8.5.3–4) from the slice onward |
| The false seams feel unfair | A fully-assembled wrong diagnosis can feel like a trick | False seams exist only in Stratum IV, are always refuted by *evidence the player can exhibit* (never by narrator fiat), and their collapse teaches the floor's second-order lesson; playtest criterion added at Stratum IV's production review |

### 8.10 Remaining decisions that genuinely require the Owner

Everything not listed here is decided by this document or derivable from standing rulings. These are the Owner's:

1. **Engine selection** — the commission itself (§7.8), after finalist slice proofs.
2. **Art and audio production route** — who or what produces the visual and musical assets, and under what review. (This design specifies language and direction, §6.3/§6.6, not a production method.)
3. **Release model** — free, paid, or otherwise; storefront accounts and signer identity for the three platforms; whether the game and the curriculum are publicly cross-referenced (the game contains no such reference unless the Owner adds one).
4. **Name clearance** — "Niflheim" as a product name: trademark/storefront availability checks before release commitments are made under it.
5. **Voice acting** — go/no-go (the design is complete without it; §6.7's subtitling requirements apply if it is added).
6. **Localization** — English is the production language; any localization is a post-release decision, and the text architecture (§6.5's reading view, content-as-data) must not foreclose it.
7. **Curriculum amendments A-1…A-4** (§9.2) — whether and when the syllabus of record is revised to match the dialect policy this pass has stated for it.

---

## 9. Audit

This section verifies the document against its sources and states the results. Counts below were checked against `CURRICULUM.md`, `MAP.md`, and `GAME-DESIGN.md` during this pass.

### 9.1 Traceability and consistency checks

- **Seventeen floors, exactly.** §4 specifies Floors 1–17 — no more, no fewer. Strata: I = 1–3, II = 4–8, III = 9–13, IV = 14–17, matching the curriculum's strata.
- **Every curriculum domain appears exactly once.** Domain 1 Ownership → Floor 1; 2 Traits → 2; 3 API design → 3; 4 Memory layout → 4; 5 Unsafe & aliasing → 5; 6 Interior mutability → 6; 7 Concurrency & atomics → 7; 8 Async & Pin → 8; 9 Collections & allocators → 9; 10 FFI & ABI → 10; 11 Macros → 11; 12 no_std & embedded → 12; 13 Performance → 13; 14 Kernel & OS → 14; 15 Compiler internals → 15; 16 Verification → 16; 17 Toolchain internals → 17. Class names match the curriculum in every case (Oathwarden, Sigilwright, Gatewright, Plumbmason, Veilwalker, Lockheart, Threadweaver, Pinwright, Heapwarden, Marchenvoy, Runesmith, Anchorite, Quicksilver Adept, Rootwarden, Mirrorwright, Assayer, Roadwarden).
- **Every FOE name matches the curriculum's FOE for its domain** (The Borrowed Hoard, The Menagerie, The Gatehouse Release, The Crooked Foundation, The Thing Behind the Veil, The Heart Locked Twice, The Loom of Many Hands, The Suspended Hour, The Hoard Beneath, The Treaty of Two Tongues, The Second Tongue, The Cell Without Windows, The Quicksilver Trial, The Root of the Tree, The Mirror Turned Inward, The Assay of All Things, The Road to Everywhere) — 17 of 17.
- **Every failure class from the concept design is assigned.** All 15 rows of `GAME-DESIGN.md`'s mapping table appear in §3.2's ledger with at least one primary floor; no class remains palette-only. Primaries total 17 (one per FOE): 13 classes primary once; classes #1 and #5 primary twice, explicitly as second-order recurrences (Floors 16 and 15), per §3.3 item 2.
- **Every FOE has a codex principle and loot.** Each of the 17 floor specs in §4 states a named codex stone with its principle and why-it-works, and a named artifact. 17 stones, 17 artifacts.
- **Loot reuse is accounted for.** Each spec's "wielded later" field was checked against the tracker's equipment inventory (`MAP.md`): all 17 rows agree. Reverse check: every "wielded in" destination in the inventory is honoured in the destination floor's spec — the Interner's Hoard is Floor 3's specimen, Floor 9's rebuild, Floor 14's string discipline; the Anchorite's Cell is Floor 14's chassis and Floor 17's custom-spec build; the Gatehouse Key is Floor 17's publication subject; the Plumb Line is invoked by name on Floors 5, 9, 10, 13, 14; the Assayer's Touchstone's retroactive wielding is mechanized in §5.5 as the re-grade. No artifact is won and never used; no floor requires an artifact before its source FOE can have fallen (checked against §5.2's prerequisites — e.g., Floor 8 wields the Lockheart Pair, and Floor 6 is a stated prerequisite of Floor 8).
- **Prerequisites and stairs match the record.** §5.2's table reproduces `MAP.md`'s governing edge list, including its two chamber-depth refinements (Floor 6 ← Floor 5 through Level 3; Floor 16's Level 4 ← Floor 7). §5.6's stairs gates reproduce the curriculum's three stairs texts and the Niflhel seal condition; the one elaboration (Stratum I's lantern clause) is a presentation of the curriculum's clippy-clean stairs condition, flagged in place as such.
- **Niflhel is the win state, not an eighteenth domain.** It appears in §4 only as a status note, has no domain, class, chambers, FOE, or loot anywhere in this document, and its full design is the endgame of §5.10. The floor count remains 17.
- **No exercise solutions.** §4's chambers describe in-game challenges that teach the levels' mechanisms as physics; the curriculum's exercise specs (names, pass checks, gate commands) are referenced as the syllabus twin but never reproduced, solved, or walked through.
- **MAP.md untouched.** This pass changed no status in the tracker: design work is not an expedition gate, and every cell remains exactly as it was — UNCHARTED. A square fills on a verified gate run; there has not been one, and this document does not pretend otherwise.

### 9.2 Dialect and readiness audit of CURRICULUM.md

The curriculum was audited against §2's standard (Tiger Style Rust; edition fluency) as part of this pass. Outcomes:

**Fixed directly in `CURRICULUM.md` during this pass:**

1. **The "Dialect and readiness" note** was added to the curriculum's front matter (immediately after its "Standards" paragraph), stating the Tiger Style standard, the standing gates (`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` for every FOE crate), the exact toolchain pin (`rust-toolchain.toml`), the edition policy (current-edition 2024 idioms; no deprecated or edition-fragile constructs; unstable/nightly-only features fenced to Domains 15 and 17), and the placement of MSRV/semver/deprecation in Domain 3 and of edition migration + RFC/release-note reading in Domain 17.
2. **Eleven FOE gate lines gained the missing clippy step** (`cargo clippy --all-targets -- -D warnings`): the FOEs of Domains 5, 6, 7, 9, 10, 12, 13, 14, 15, 16, and 17. (Domains 15 and 16 carry it scoped to the lint crate itself and the battery's own crates respectively.) Verification after the fix: all 17 FOE gate specifications in the curriculum now name clippy — Domains 1–3 in their command blocks, Domains 4–17 inline.
3. **rustfmt** was named as a standing gate in the note (item 1) rather than by editing 17 gate lists — a deliberate, recorded choice: the standing-gates declaration covers every FOE crate, and per-FOE repetition would be noise. Recorded here so the choice is auditable.

**Named amendments for a later curriculum revision** (gaps where the level/exercise specs do not yet cash out the stated policy; the design pass did not rewrite exercise specs — A-1…A-4 were applied by the amendment pass D-811, and each entry below records its application):

- **A-1 — Domain 3 (MSRV, semver, deprecation).** The dialect note assigns `rust-version` declaration, semver, and deprecation discipline to Domain 3, and this design teaches them at Floor 3 — but no Domain 3 level or FOE criterion currently grades them. *Amendment:* add an MSRV stamp and a deprecation exercise to Domain 3's Level 5 audit (or a named FOE pass item). **Applied (D-811):** Domain 3's Level 5 audit now closes with the `rust-version` stamp (proven on the declared-oldest toolchain) and a performed deprecation rite, both graded in its pass criteria; Floor 3's FOE proof grades the oldest-road stamp and one amendment rite on the log-slate.
- **A-2 — Domain 17 (edition migration; RFC habit).** The note assigns edition migration (`cargo fix --edition`, edition guides) and RFC/release-note reading to Domain 17, and Floor 17's fourth chamber is built around the edition rite — but no Domain 17 level spec covers them. *Amendment:* add an edition-migration exercise (a candidate home is beside Level 1's feature work or as a sixth pass item in the FOE drill) and name the reading habit in the domain header. **Applied (D-811):** Domain 17's header now names the reading habit (edition guides, the RFC book, release notes), and the FOE drill carries the migration leg — `cargo fix --edition` on an old-edition fixture, its suite re-run, every diff hunk accounted for; Floor 17's FOE proof grades the crossing re-performed for the departure record.
- **A-3 — Domain 13, Level 4 (portable SIMD fencing).** Level 4 teaches portable SIMD, which is nightly-only and unstable, outside the Domains 15/17 fence the policy now states. *Amendment:* either fence Level 4's portable-SIMD half as nightly-subject study, or rest the exercise on stable `core::arch` intrinsics with portable SIMD as the comparison. (Floor 13's fourth chamber in this design already takes the second path.) **Applied (D-811):** the second path is now taken in both documents — Domain 13's Level 4 rests on stable `core::arch` intrinsics with portable SIMD as the named comparison, its depth routed to Domains 15 and 17 by cross-reference; Floor 13's fourth chamber already complied and is unchanged.
- **A-4 — Domain 14 (nightly build machinery, unfenced).** The kernel build route depends in practice on nightly build-std/custom-target machinery inherited from Domain 17's remit; the curriculum does not say so. *Amendment:* state in Domain 14's header that its nightly dependence is toolchain infrastructure (Domain 17's subject, used under its fence), while the domain's taught language content remains stable. **Applied (D-811):** Domain 14's header carries the toolchain note (custom target, `-Zbuild-std`, boot image = nightly infrastructure, Domain 17's subject used under its fence; taught language stable), and Floor 14's prerequisites carry the same statement in the design's terms.
- **A-5 — Domain 15 (pin mechanism).** The FOE spec says "the estate's pinned nightly" without naming the mechanism; the dialect note now fixes the pin as an exact date in `rust-toolchain.toml`. No further amendment needed — recorded as closed by the note.

**Audited and found aligned (no gap):** typed, hand-rolled error taxonomies (Domain 3, against broad error types); bounded queues with documented full-behaviour (Domain 7); stated budgets where work is bounded (Domain 16's 300-second fuzz budget; Domain 9's stated operation counts); determinism as a gate property (Domain 8's event-for-event interleaving, Domain 13's output-hash equality); assertions as the contract mechanism (Domain 4's Plumb Line discipline); minimal-dependency posture (the curriculum hand-builds what ecosystems would provide, throughout); `forbid(unsafe_code)` where unsafe is not the subject (Domain 1).

### 9.3 Standing of this document

This full design pass decides everything §8.10 does not reserve to the Owner. It changes the concept design only as recorded in §3.3, the curriculum only as recorded in §9.2, and the tracker not at all. If this document and the concept design ever disagree, §3.3 is the reconciliation of record; if this document and the curriculum disagree on a syllabus fact (a prerequisite, a gate, a loot destination), the curriculum and the tracker govern, and this document is to be amended — the game is the syllabus drawn at walking scale, and the drawing does not outrank the survey.

*— End of the full design pass.*
