//! Authored Floor 1 content: every word and number the floor
//! presents, kept as data so the state machine and the validators
//! stay honest, testable machinery. Nothing in this module decides
//! anything; it only says what the Lending Vaults contain.
//!
//! Floor 1 — The Lending Vaults. Domain 1: Ownership, borrowing &
//! lifetimes. Keeper: the Oathwarden. FOE: The Borrowed Hoard.

/// The floor's name.
pub const FLOOR_NAME: &str = "The Lending Vaults";
/// The domain this floor teaches.
pub const DOMAIN_NAME: &str = "Ownership, borrowing & lifetimes";
/// The floor's keeper.
pub const KEEPER_NAME: &str = "the Oathwarden";

/// The floor inscription, read at the camp stone.
pub const FLOOR_INTRO: &str = "Floor 1 — The Lending Vaults. The Oathwarden keeps a lending library of tools, charters, and beasts, and one rule is carved over its door: one thing, one holder. Five chambers teach the rule's five faces — moves and copies, borrowing, lifetimes, the steward's unmaking, and the hard cases. At the floor's heart the Borrowed Hoard, a hoard-engine grown fat on promises, issues the vault's treasures twice and calls it generosity. Prove its plan broken, write the Hoard that should replace it, and strike.";

/// The Plank Stone's principle (the Principle Codex entry).
pub const PLANK_STONE: &str = "The Plank Stone";
/// The Plank Stone's text.
pub const PLANK_STONE_TEXT: &str = "Ownership: one value, one owner.";
/// The floor's loot artifact.
pub const INTERNERS_HOARD: &str = "The Interner's Hoard";
/// The loot's codex text.
pub const INTERNERS_HOARD_TEXT: &str = "A ledger-chest that keeps itself: every name stored once, every thing in one column, nothing promised twice. The inventory backbone of the descent — wielded on Floors 3, 9, and 14.";

/// One chamber's authored framing.
pub struct ChamberMeta {
    /// Short name, used on doors and the map.
    pub name: &'static str,
    /// The room inside the chamber.
    pub room: &'static str,
    /// The inscription stone's title.
    pub inscription_title: &'static str,
    /// The inscription stone's text (the rule, carved).
    pub inscription: &'static str,
    /// The demonstration: the mechanism shown working correctly,
    /// replayable at any time (the design's chamber opening).
    pub demonstration: &'static str,
    /// What the proof panel's slot asks for, by kind.
    pub slot_wants: &'static str,
    /// The label on the chamber's prove/commit action.
    pub prove_label: &'static str,
}

/// The five chambers, in design order.
pub const CHAMBERS: [ChamberMeta; 5] = [
    ChamberMeta {
        name: "Moves and Copies",
        room: "The Issue Desk",
        inscription_title: "The Vault Rule",
        inscription: "One thing, one holder. What may be copied, copy freely; what may not, promise once. A plan that hands the same saw to two porters is not a generous plan. It is a broken one.",
        demonstration: "The keeper packs for one porter while you watch: the saw goes in once, under one name; the tally sticks are copied for every satchel, because copies of a copiable thing cost the vault nothing. The plan balances because every uncopiable thing stands in exactly one column.",
        slot_wants: "a packing list with exactly one holder per uncopiable item, and the toll of tallies reconciled",
        prove_label: "File the plan",
    },
    ChamberMeta {
        name: "Borrowing",
        room: "The Reading Room",
        inscription_title: "The Reading Rule",
        inscription: "Many eyes may read one chart at once. The annotator's pen may write only when no eye holds the chart — and while the pen writes, no eye may take it up. Shared reading is welcome; writing is exclusive.",
        demonstration: "Three readers share the star chart through the morning, passing it hand to hand, and the room is calm. At noon the readers lay it down; only then does the annotator uncap her pen. The chart is never written on while it is held, and never held while it is written on.",
        slot_wants: "a day-plan with no overlap between the annotator's hours and any reader's",
        prove_label: "Present the day-plan",
    },
    ChamberMeta {
        name: "Lifetimes as Contracts",
        room: "The Long Lease",
        inscription_title: "The Lease Rule",
        inscription: "A charter may be sub-lent only within its term, and that lease sub-lent again only within its own. A lease that steps outside its bracket is not a lease. It is a trespass, and the vault will draw you the brackets to prove it.",
        demonstration: "The keeper draws the charter's term as one long bracket, the first sub-lease nested inside it, the second inside the first — three brackets, each wholly held by the one before. Nothing sticks out. That is what a clean chain looks like.",
        slot_wants: "a lease chain whose brackets nest cleanly, each lease inside the one before",
        prove_label: "Present the chain",
    },
    ChamberMeta {
        name: "Drop and Scope Guards",
        room: "The Faithful Steward",
        inscription_title: "The Steward's Rule",
        inscription: "When an expedition leaves a room, the steward unmakes every temporary arrangement — unless it stands committed in the book. What the book holds, stands. What it does not, is restored, as faithfully as if it had never been.",
        demonstration: "A delivery crosses two rooms. In the first, its arrangements were committed in the book, and they stand. In the second, nothing was committed, and the steward restores the room behind the expedition — rope back on its peg, chalk washed away. Both halves arrive exactly as the rule promises.",
        slot_wants: "both records exhibited: what the book kept, and what the room restored",
        prove_label: "Exhibit the records",
    },
    ChamberMeta {
        name: "Reborrows and the Hard Cases",
        room: "The Gauntlet Yard",
        inscription_title: "The Yard Rule",
        inscription: "Ten placards hang in the yard. Judge each by the rule, not by its face — some honest-looking lendings break the vault, and some strange ones keep it. The yard's gate judges as you judge, and names the rule wherever you stray.",
        demonstration: "The yard-keeper judges two placards aloud, slowly: this lending keeps one holder — legal; this one promises a chart to two expeditions — barred, and she names the rule it breaks. Then she steps back. The remaining ten are yours.",
        slot_wants: "ten correct judgements, each barred gate naming the rule",
        prove_label: "Face the yard's gate",
    },
];

// ---------------------------------------------------------------------------
// Chamber 1 — Moves and Copies: the packing list and the Toll of Tallies.
// ---------------------------------------------------------------------------

/// The three porters of the Issue Desk.
pub const PORTERS: [&str; 3] = ["Anke", "Boro", "Cato"];

/// One packable item in Chamber 1.
pub struct PackItem {
    /// The item's name.
    pub name: &'static str,
    /// True when the item may be copied freely.
    pub copiable: bool,
    /// The flawed plan's holders, as porter indices — the starting
    /// state the player must repair.
    pub initial_holders: &'static [usize],
}

/// Chamber 1's items, in panel order.
pub const PACK_ITEMS: [PackItem; 8] = [
    PackItem {
        name: "Saw",
        copiable: false,
        initial_holders: &[0, 1],
    },
    PackItem {
        name: "Heartstone Chart",
        copiable: false,
        initial_holders: &[0],
    },
    PackItem {
        name: "Tally Sticks",
        copiable: true,
        initial_holders: &[0, 1],
    },
    PackItem {
        name: "Rope Ladder",
        copiable: false,
        initial_holders: &[],
    },
    PackItem {
        name: "Star Lantern",
        copiable: false,
        initial_holders: &[2],
    },
    PackItem {
        name: "Ship's Kettle",
        copiable: false,
        initial_holders: &[1],
    },
    PackItem {
        name: "Wax Seals",
        copiable: true,
        initial_holders: &[2],
    },
    PackItem {
        name: "Blank Tallies",
        copiable: true,
        initial_holders: &[1],
    },
];

/// The Toll of Tallies: Keeper Ansel's crate list (14 names).
pub const TOLL_KEEPER_A: [&str; 14] = [
    "Anke", "Boro", "Cato", "Dain", "Edda", "Fenn", "Gull", "Halla", "Ivo", "Jora", "Kettil",
    "Linna", "Miro", "Nessa",
];
/// The Toll of Tallies: Keeper Brin's crate list (11 names).
pub const TOLL_KEEPER_B: [&str; 11] = [
    "Cato", "Edda", "Gull", "Ivo", "Kettil", "Miro", "Osric", "Pell", "Ruta", "Sanna", "Tove",
];
/// The counts the player may file for the toll. Exactly one is the
/// reconciled count: |A ∪ B| = 14 + 11 − 6 = 19.
pub const TOLL_CANDIDATES: [u32; 4] = [25, 19, 17, 14];

// ---------------------------------------------------------------------------
// Chamber 2 — Borrowing: the Reading Room day-plan.
// ---------------------------------------------------------------------------

/// One person in the Reading Room, with their two candidate blocks.
/// Blocks are `[start, end)` in whole hours.
pub struct Reader {
    /// The person's name and role.
    pub name: &'static str,
    /// True for the annotator, whose writing must be exclusive.
    pub annotator: bool,
    /// The two candidate blocks; the player picks one per person.
    pub blocks: [(u32, u32); 2],
}

/// The Reading Room's five people, annotator first.
pub const READERS: [Reader; 5] = [
    Reader {
        name: "Annotator Sefa",
        annotator: true,
        blocks: [(6, 10), (14, 18)],
    },
    Reader {
        name: "Reader Ansel",
        annotator: false,
        blocks: [(6, 9), (10, 13)],
    },
    Reader {
        name: "Reader Brin",
        annotator: false,
        blocks: [(8, 11), (13, 16)],
    },
    Reader {
        name: "Reader Corin",
        annotator: false,
        blocks: [(9, 12), (14, 17)],
    },
    Reader {
        name: "Reader Disa",
        annotator: false,
        blocks: [(6, 8), (15, 18)],
    },
];

// ---------------------------------------------------------------------------
// Chamber 3 — Lifetimes as Contracts: the Long Lease.
// ---------------------------------------------------------------------------

/// The charter's term, in months: `[start, end]`.
pub const CHARTER: (i32, i32) = (0, 36);
/// The three sub-leases as the flawed chain stands: the second
/// lease steps outside the first.
pub const LEASES_INITIAL: [(i32, i32); 3] = [(3, 30), (6, 33), (9, 24)];
/// The stepper's stride, in months.
pub const LEASE_STEP: i32 = 3;

// ---------------------------------------------------------------------------
// Chamber 4 — Drop and Scope Guards: the Faithful Steward's delivery.
// ---------------------------------------------------------------------------

/// What the delivery carries, committed in the book.
pub const DELIVERY_COMMITTED: &str = "the sealed letter and the chart tube";
/// What the delivery arranges for the room only, never committed.
pub const DELIVERY_TEMPORARY: &str = "the rope coil and the chalk-marked crate";
/// The record the book keeps for a committed room.
pub const RECORD_KEPT: &str = "The book kept: the sealed letter and the chart tube stand delivered, entered under the expedition's name.";
/// The record for a room whose temporary half was also committed.
pub const RECORD_KEPT_SECOND: &str = "The book kept (second room): the rope coil and the chalk marks were committed too, and stand as the book's own.";
/// The record the room gives back when the steward restores it.
pub const RECORD_RESTORED: &str = "The room restored: the rope coil hangs on its peg and the chalk marks are washed away, as if never arranged.";

// ---------------------------------------------------------------------------
// Chamber 5 — Reborrows and the Hard Cases: the Gauntlet Yard.
// ---------------------------------------------------------------------------

/// One placard in the Gauntlet Yard.
pub struct Placard {
    /// The lending situation, as written on the placard.
    pub text: &'static str,
    /// The yard's judgement: true when the lending is legal.
    pub legal: bool,
    /// The rule that decides it, named when the player strays.
    pub rule: &'static str,
}

/// The ten placards: five legal, five barred.
pub const PLACARDS: [Placard; 10] = [
    Placard {
        text: "A porter lends her own saw to a second porter for one afternoon, and takes it back at dusk.",
        legal: true,
        rule: "one holder at a time, and the thing returns — a clean move of a loan",
    },
    Placard {
        text: "Two porters sign for the same saw on the same day, each in a different column.",
        legal: false,
        rule: "the vault rule — an uncopiable thing promised to two holders at once",
    },
    Placard {
        text: "The keeper copies the tally sticks for every expedition that asks.",
        legal: true,
        rule: "what may be copied, copy freely — copies of a copiable thing break nothing",
    },
    Placard {
        text: "A reader annotates the chart while two readers still hold it open.",
        legal: false,
        rule: "the reading rule — the pen writes only when no eye holds the chart",
    },
    Placard {
        text: "Three readers share one chart through the whole morning.",
        legal: true,
        rule: "the reading rule — shared reading is welcome; only writing is exclusive",
    },
    Placard {
        text: "A sub-lease runs two months past the end of its charter's term.",
        legal: false,
        rule: "the lease rule — a lease that steps outside its bracket is a trespass",
    },
    Placard {
        text: "A charter is sub-lent inside its term, and that lease sub-lent again inside its own.",
        legal: true,
        rule: "the lease rule — brackets that nest cleanly are a clean chain",
    },
    Placard {
        text: "The steward packs away a room's temporary ropes, though the expedition never committed them in the book.",
        legal: true,
        rule: "the steward's rule — what the book does not hold, the steward unmakes",
    },
    Placard {
        text: "The steward restores the committed chart to the shelf as if it were temporary.",
        legal: false,
        rule: "the steward's rule — what the book holds, stands; the steward may not unmake it",
    },
    Placard {
        text: "An expedition leaves its uncommitted arrangements standing in a room, 'just in case'.",
        legal: false,
        rule: "the steward's rule — uncommitted arrangements are unmade when the expedition leaves",
    },
];

// ---------------------------------------------------------------------------
// The FOE — The Borrowed Hoard: the master plan and the strike.
// ---------------------------------------------------------------------------

/// The FOE's name.
pub const FOE_NAME: &str = "The Borrowed Hoard";
/// The FOE chamber's inscription.
pub const FOE_INSCRIPTION: &str = "The Hoard does not steal. It promises. Every treasure in its ledger is lent with a smile — and the Heartstone Chart is lent twice. Audit the master plan, enumerate its uncopiables, count their holders. What can be found by census need never be guessed.";
/// The FOE's demonstration: the engine shown at its work.
pub const FOE_DEMONSTRATION: &str = "The hoard-engine turns its great ledger toward you, page by page: charts, saws, pens, charters, crates — every lending the vault has made, in one master plan. It is proud of the plan. It has never once counted the holders.";
/// The master plan, in `item | holder | kind` lines. The Heartstone
/// Chart stands in two expedition columns; everything else is
/// consistent. This is the plan the census is taken over.
pub const MASTER_PLAN: &str = "\
# The Hoard's master plan
Heartstone Chart | Expedition A | unique
Saw | Expedition A | unique
Tally Sticks | Expedition A | copiable
Star Chart | Expedition C | unique
Annotator's Pen | The Reading Room | unique
Long Charter | The Leasing Office | unique
Heartstone Chart | Expedition B | unique
Delivery Crate | Expedition C | unique
Rope Ladder | Expedition B | unique
Wax Seals | Expedition A | copiable
Tally Sticks | Expedition B | copiable
Star Lantern | Expedition C | unique
Ship's Kettle | Expedition B | unique
Blank Tallies | Expedition B | copiable
";

// ---------------------------------------------------------------------------
// The hint ladder: four rungs per challenge, free to climb. Rung 4
// names the seam and forfeits the challenge's facet.
// ---------------------------------------------------------------------------

/// Which challenge a hint belongs to: chambers 0–4, or the FOE (5).
pub const FOE_CHALLENGE: usize = 5;

/// The four rungs for a challenge. Index 0 is the Keeper's Remark,
/// 3 is The Seam Named.
pub fn hints(challenge: usize) -> [&'static str; 4] {
    match challenge {
        0 => [
            "Keeper's Remark: The plan on the desk is the plan that failed at the vault door. Read it as the door read it.",
            "Codex Nudge: Count holders, item by item. Uncopiable things want exactly one; copiable things want at least one copy out.",
            "Worked Parallel: The keeper's own packing balanced because the saw stood in one column and the tallies in every satchel. Compare, item by item.",
            "The Seam Named: The saw stands under two porters, and the rope ladder under none. Set the saw to one holder, give the ladder one, and reconcile the toll: 14 + 11 − 6 names counted twice = 19.",
        ],
        1 => [
            "Keeper's Remark: The chart is one object. Reading shares it; writing cannot be shared.",
            "Codex Nudge: Only the annotator's hours are dangerous. Check her block against each reader's, pair by pair.",
            "Worked Parallel: At noon in the demonstration, every reader had laid the chart down before the pen came out. Make the day-plan rhyme with noon.",
            "The Seam Named: Give the annotator the late block, 14 to 18 — or the early one, 6 to 10 — and give every reader a block that never touches hers.",
        ],
        2 => [
            "Keeper's Remark: Draw the brackets. The vault always draws the brackets.",
            "Codex Nudge: A chain is clean when each lease starts no earlier and ends no later than the lease that holds it.",
            "Worked Parallel: In the demonstration, three brackets nested like bowls. Here, one bowl's rim sticks out past the one that should hold it.",
            "The Seam Named: The second lease ends at 33, outside the first lease's end of 30. Pull its end in to 30 or earlier, and keep the third lease inside the second.",
        ],
        3 => [
            "Keeper's Remark: Two rooms, two fates. The book decides which is which.",
            "Codex Nudge: One slot wants what commitment preserved; the other wants what the steward's faithfulness restored. Do not let the records swap coats.",
            "Worked Parallel: In the demonstration the letter stood and the chalk vanished. The records on the desk say exactly that, in the steward's own hand.",
            "The Seam Named: Exhibit 'The book kept' into the kept slot and 'The room restored' into the restored slot — and if you committed the second room too, reset and run it uncommitted.",
        ],
        4 => [
            "Keeper's Remark: Judge by the rule carved over the yard, not by whether the lending sounds kind.",
            "Codex Nudge: Five of the ten keep the vault; five break it. The breaks are: a double promise, a writing while read, a lease outside its bracket, and two betrayals of the steward.",
            "Worked Parallel: The yard-keeper barred the lending that promised one chart to two expeditions, and passed the lending that moved one saw between two porters in turn. Sequence is everything.",
            "The Seam Named: Legal: 1, 3, 5, 7, 8. Barred: 2, 4, 6, 9, 10. Judge them so, and the gate will agree.",
        ],
        _ => [
            "Keeper's Remark: The Hoard is proud of its ledger. Pride is not a defence; it is an invitation to audit.",
            "Codex Nudge: Verify every uncopiable row — one holder each — and the census itself will point at the row that lies.",
            "Worked Parallel: Chamber 1's census found one chart in two columns at a desk. The master plan is the same crime at floor scale.",
            "The Seam Named: The Heartstone Chart stands under Expedition A and Expedition B. Exhibit it, verify the other eight uncopiables, then write the Hoard: dedup, lookup, and lines that borrow their source.",
        ],
    }
}

// ---------------------------------------------------------------------------
// The FOE gate: the Interner's Hoard, built by the player's hand.
// The game assembles the player's editor text into this project,
// adds the hidden test suite, and runs cargo test + clippy.
// ---------------------------------------------------------------------------

/// The scratch project's manifest. An empty `[workspace]` table
/// keeps the project self-contained wherever it is assembled.
pub const GATE_CARGO_TOML: &str = "\
[package]
name = \"the-interners-hoard\"
version = \"0.1.0\"
edition = \"2024\"

[workspace]

[lib]
name = \"the_interners_hoard\"
path = \"src/lib.rs\"
";

/// The scaffold the editor opens with: the Hoard's contract, its
/// public API, and unfinished bodies. It compiles; its tests fail
/// until the player writes the machinery. The vault prepends
/// `#![forbid(unsafe_code)]` at assembly time.
pub const GATE_SCAFFOLD: &str = r##"//! The Interner's Hoard — the ledger-chest at the heart of the
//! Lending Vaults, written by the delver's own hand.
//!
//! The contract, from the vault rule: every distinct string is
//! stored exactly once and handed out as a small symbol; a symbol
//! always names the same string; lookups never store anything; and
//! the line index borrows its lines from the source text — it
//! copies nothing.
//!
//! The vault assembles this file with `#![forbid(unsafe_code)]`
//! prepended, then runs its gate: cargo test (half validation,
//! half adversarial) and cargo clippy at -D warnings. Both must
//! pass — and the lanterns judge style as well as sense: a `new`
//! with no `Default` beside it burns red, as does anything else
//! clippy names. Keep the public API exactly as given; the
//! vault's tests are written against it.

use std::collections::HashMap;

/// Stores each distinct string once and hands out symbols.
///
/// Symbols are stable for the life of the interner: interning the
/// same text always returns the same symbol, and distinct texts
/// never share one.
#[allow(dead_code)]
pub struct Interner {
    entries: Vec<Box<str>>,
    by_text: HashMap<Box<str>, u32>,
}

impl Interner {
    /// A new, empty interner.
    pub fn new() -> Self {
        todo!("an interner starts empty")
    }

    /// Store `text` if it is new; return its symbol either way.
    pub fn intern(&mut self, text: &str) -> u32 {
        todo!("store each distinct string exactly once")
    }

    /// The symbol for `text`, if it was ever interned. Never stores.
    pub fn lookup(&self, text: &str) -> Option<u32> {
        todo!("a lookup is a question, not a promise")
    }

    /// The string a symbol names; `None` for a symbol this
    /// interner never issued.
    pub fn text(&self, symbol: u32) -> Option<&str> {
        todo!("a symbol names the string it was minted for")
    }

    /// How many distinct strings are stored.
    pub fn len(&self) -> usize {
        todo!("count what is held")
    }

    /// Whether nothing is stored.
    pub fn is_empty(&self) -> bool {
        todo!("empty means empty")
    }
}

/// A zero-copy index of the lines of a source text: every line is
/// a slice borrowed from the source, never a copy. Line semantics
/// match `str::lines` — a trailing newline mints no phantom line.
#[allow(dead_code)]
pub struct LineIndex<'a> {
    lines: Vec<&'a str>,
}

impl<'a> LineIndex<'a> {
    /// Index `source` without copying any of it.
    pub fn new(source: &'a str) -> Self {
        todo!("borrow the lines; copy nothing")
    }

    /// The line at `line` (0-based); `None` past the end.
    pub fn get(&self, line: usize) -> Option<&'a str> {
        todo!("lines are counted from zero")
    }

    /// How many lines the source has.
    pub fn len(&self) -> usize {
        todo!("count the lines")
    }

    /// Whether the source has no lines.
    pub fn is_empty(&self) -> bool {
        todo!("empty means empty")
    }
}
"##;

/// The vault's hidden gate suite: half validation, half
/// adversarial, per the house rule. Written against the scaffold's
/// public API only.
pub const GATE_TESTS: &str = r##"//! The vault's gate for the Interner's Hoard.
//! Half of these tests are validation; half are adversarial.

use the_interners_hoard::{Interner, LineIndex};

// ------------------------------------------------------------------
// Validation
// ------------------------------------------------------------------

#[test]
fn dedup_repeated_inserts_return_one_symbol() {
    let mut interner = Interner::new();
    let first = interner.intern("Heartstone Chart");
    let second = interner.intern("Heartstone Chart");
    let third = interner.intern("Heartstone Chart");
    assert_eq!(first, second);
    assert_eq!(second, third);
    assert_eq!(interner.len(), 1);
}

#[test]
fn distinct_strings_get_distinct_symbols() {
    let mut interner = Interner::new();
    let saw = interner.intern("Saw");
    let chart = interner.intern("Heartstone Chart");
    let rope = interner.intern("Rope Ladder");
    assert_ne!(saw, chart);
    assert_ne!(chart, rope);
    assert_ne!(saw, rope);
    assert_eq!(interner.len(), 3);
    assert!(!interner.is_empty());
}

#[test]
fn lookup_finds_what_was_interned() {
    let mut interner = Interner::new();
    let chart = interner.intern("Heartstone Chart");
    interner.intern("Saw");
    assert_eq!(interner.lookup("Heartstone Chart"), Some(chart));
}

#[test]
fn text_round_trips_for_every_symbol() {
    let mut interner = Interner::new();
    let names = ["Saw", "Heartstone Chart", "Tally Sticks", "Annotator's Pen"];
    let symbols: Vec<u32> = names.iter().map(|name| interner.intern(name)).collect();
    for (symbol, name) in symbols.iter().zip(names.iter()) {
        assert_eq!(interner.text(*symbol), Some(*name));
    }
}

#[test]
fn line_index_agrees_with_naive_lines() {
    let source = "the vault rule\none thing, one holder\npromise once\n";
    let index = LineIndex::new(source);
    let naive: Vec<&str> = source.lines().collect();
    assert_eq!(index.len(), naive.len());
    for (i, line) in naive.iter().enumerate() {
        assert_eq!(index.get(i), Some(*line));
    }
}

#[test]
fn line_index_reports_its_length() {
    let index = LineIndex::new("one\ntwo\nthree\nfour");
    assert_eq!(index.len(), 4);
    assert!(!index.is_empty());
    assert_eq!(index.get(3), Some("four"));
}

// ------------------------------------------------------------------
// Adversarial
// ------------------------------------------------------------------

#[test]
fn empty_string_interns_and_round_trips() {
    let mut interner = Interner::new();
    let empty = interner.intern("");
    assert_eq!(interner.text(empty), Some(""));
    assert_eq!(interner.lookup(""), Some(empty));
    assert_eq!(interner.len(), 1);
}

#[test]
fn duplicate_inserts_under_load_stay_deduped() {
    let mut interner = Interner::new();
    let names = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
    let mut first_symbols = Vec::new();
    for round in 0..100 {
        for (i, name) in names.iter().enumerate() {
            let symbol = interner.intern(name);
            if round == 0 {
                first_symbols.push(symbol);
            } else {
                assert_eq!(symbol, first_symbols[i], "symbols must be stable");
            }
        }
    }
    assert_eq!(interner.len(), 10);
}

#[test]
fn lookups_for_absent_keys_are_none() {
    let mut interner = Interner::new();
    assert_eq!(interner.lookup("never stored"), None);
    interner.intern("Saw");
    assert_eq!(interner.lookup("saw"), None, "lookup is case-sensitive");
    assert_eq!(interner.lookup("Saws"), None);
    assert_eq!(interner.lookup(""), None);
    assert_eq!(interner.len(), 1, "lookups must never store");
}

#[test]
fn text_for_unissued_symbols_is_none() {
    let mut interner = Interner::new();
    interner.intern("Saw");
    assert_eq!(interner.text(9_999), None);
    assert_eq!(interner.text(u32::MAX), None);
}

#[test]
fn line_index_without_trailing_newline() {
    let index = LineIndex::new("first\nsecond\nlast has no newline");
    assert_eq!(index.len(), 3);
    assert_eq!(index.get(2), Some("last has no newline"));
    assert_eq!(index.get(3), None);
}

#[test]
fn line_index_over_multibyte_text() {
    let source = "café\n🦀 ferrugo\nnaïve façade";
    let index = LineIndex::new(source);
    assert_eq!(index.len(), 3);
    assert_eq!(index.get(0), Some("café"));
    assert_eq!(index.get(1), Some("🦀 ferrugo"));
    assert_eq!(index.get(2), Some("naïve façade"));
}

#[test]
fn line_index_of_empty_source_is_empty() {
    let index = LineIndex::new("");
    assert_eq!(index.len(), 0);
    assert!(index.is_empty());
    assert_eq!(index.get(0), None);
}

#[test]
fn line_index_of_single_newline() {
    let index = LineIndex::new("\n");
    assert_eq!(index.len(), 1);
    assert_eq!(index.get(0), Some(""));
    assert_eq!(index.get(1), None);
}
"##;

// ---------------------------------------------------------------------------
// The Floor 2 FOE gate: the Admission Desk, re-posted by the
// player's hand. The game assembles the player's editor text into
// this project, adds the hidden test suite, and runs cargo test +
// clippy — the same bounded gate as Floor 1's, a second lantern.
// ---------------------------------------------------------------------------

/// The Floor 2 scratch project's manifest.
pub const GATE2_CARGO_TOML: &str = "\
[package]
name = \"the-admission-desk\"
version = \"0.1.0\"
edition = \"2024\"

[workspace]

[lib]
name = \"the_admission_desk\"
path = \"src/lib.rs\"
";

/// The scaffold the Floor 2 editor opens with: the desk's shape,
/// unfinished. Its staffing and its predicate are `todo!()`, so
/// the gate refuses it — as it should, until the player re-posts
/// the bound.
pub const GATE2_SCAFFOLD: &str = r##"//! The Admission Desk — the Menagerie's charter, re-posted in
//! your own hand.
//!
//! The desk admitted by resemblance: a grey coat stood for a fire
//! sigil, and the slip was stamped all the same. The guild's book
//! of failures names the seam — failure class #6, the
//! Cartographer's Blind Eye: a fine instrument used outside the
//! domain it was made for.
//!
//! The repair is a bound, re-posted. Write the desk so a beast is
//! admitted by its sigils and nothing else: the two watches
//! staffed by bound, so the wrong kind cannot be posted at all,
//! and the desk's own predicate for the roster board — which
//! refuses, above all, an admission that names no domain.

/// The fire sigil: the bearer can stand against flame.
pub trait FireWard {}

/// The mist sigil: the bearer can read the fog.
pub trait MistSense {}

/// The swiftness sigil: the bearer runs the valley roads.
pub trait Swift {}

/// An ash-hound of the fire-watch: grey as ash, sigilled for flame.
pub struct AshHound {
    /// The hound's name, as the roster writes it.
    pub name: &'static str,
}

/// A mist-hound of the mist-watch: grey as fog, sigilled for mist.
pub struct MistHound {
    /// The hound's name, as the roster writes it.
    pub name: &'static str,
}

impl FireWard for AshHound {}

impl MistSense for MistHound {}

/// The fire-watch, re-posted: only a beast bearing the FireWard
/// sigil can be staffed to it — the bound is the admission.
pub fn staff_fire_watch<B: FireWard>(beast: B) -> B {
    todo!()
}

/// The mist-watch, re-posted: only a beast bearing the MistSense
/// sigil can be staffed to it.
pub fn staff_mist_watch<B: MistSense>(beast: B) -> B {
    todo!()
}

/// The desk's predicate, for the roster board: does this beast's
/// sigil list satisfy this post's bound? Every sigil the bound
/// names must stand in the beast's list. A bound that names
/// nothing is no post at all — the desk refuses domain-less
/// admissions, so an empty bound admits no one.
pub fn admitted(sigils: &[&str], bound: &[&str]) -> bool {
    todo!()
}
"##;

/// The hidden suite for the Admission Desk: half validation,
/// half adversarial, in the gate's standing proportion.
pub const GATE2_TESTS: &str = r##"//! The vault's gate for the Admission Desk.

use the_admission_desk::{AshHound, MistHound, admitted, staff_fire_watch, staff_mist_watch};

// ------------------------------------------------------------------
// Validation
// ------------------------------------------------------------------

#[test]
fn fire_watch_staffs_the_ash_hound() {
    let posted = staff_fire_watch(AshHound { name: "Soot" });
    assert_eq!(posted.name, "Soot");
}

#[test]
fn mist_watch_staffs_the_mist_hound() {
    let posted = staff_mist_watch(MistHound { name: "Mistveil" });
    assert_eq!(posted.name, "Mistveil");
}

#[test]
fn the_predicate_reads_the_roster() {
    // Ember stands the Hearth Post; Mistveil the Mist Post;
    // Drizzle the Valley Post.
    assert!(admitted(&["FireWard", "Swift"], &["FireWard"]));
    assert!(admitted(&["MistSense", "DeepSight"], &["MistSense"]));
    assert!(admitted(&["MistSense", "Swift"], &["Swift"]));
}

#[test]
fn extra_sigils_and_bound_order_do_not_bar() {
    assert!(admitted(&["Swift", "FireWard"], &["FireWard"]));
    assert!(admitted(
        &["MistSense", "Swift", "KeenNose"],
        &["Swift", "MistSense"]
    ));
}

// ------------------------------------------------------------------
// Adversarial
// ------------------------------------------------------------------

#[test]
fn the_mist_hound_is_refused_the_fire_watch() {
    // The seam itself: the hound's sigils are true, and they are
    // not the fire-watch's.
    assert!(!admitted(&["MistSense", "DeepSight"], &["FireWard"]));
}

#[test]
fn a_grey_coat_is_not_a_sigil() {
    assert!(!admitted(&["GreyCoat"], &["FireWard"]));
    assert!(!admitted(&[], &["FireWard"]));
    assert!(!admitted(&["AshHound"], &["FireWard"]));
}

#[test]
fn domain_less_admissions_are_refused() {
    assert!(!admitted(&["FireWard"], &[]));
    assert!(!admitted(&[], &[]));
}

#[test]
fn every_named_sigil_is_required() {
    assert!(!admitted(&["FireWard"], &["FireWard", "Swift"]));
    assert!(!admitted(&["Swift"], &["FireWard", "Swift"]));
}

#[test]
fn a_duplicated_bound_sigil_still_admits() {
    assert!(admitted(&["FireWard"], &["FireWard", "FireWard"]));
}
"##;

// ---------------------------------------------------------------------------
// The Ferronia chart: all seventeen floors, by stratum. Only Floor 1
// is built in this build; the rest stand on the chart as the design
// prescribes — known, and unclaimed.
// ---------------------------------------------------------------------------

/// One charted floor: domain name and class.
pub struct ChartFloor {
    /// The domain.
    pub domain: &'static str,
    /// The class that keeps it.
    pub class: &'static str,
}

/// The seventeen floors, in descent order.
pub const FLOOR_CHART: [ChartFloor; 17] = [
    ChartFloor {
        domain: "Ownership, borrowing & lifetimes",
        class: "Oathwarden",
    },
    ChartFloor {
        domain: "Traits & the type system",
        class: "Sigilwright",
    },
    ChartFloor {
        domain: "Idiomatic API design",
        class: "Gatewright",
    },
    ChartFloor {
        domain: "Memory layout",
        class: "Plumbmason",
    },
    ChartFloor {
        domain: "Unsafe Rust & the aliasing model",
        class: "Veilwalker",
    },
    ChartFloor {
        domain: "Interior mutability",
        class: "Lockheart",
    },
    ChartFloor {
        domain: "Concurrency & atomics",
        class: "Threadweaver",
    },
    ChartFloor {
        domain: "Async & Pin",
        class: "Pinwright",
    },
    ChartFloor {
        domain: "Collections & allocators",
        class: "Heapwarden",
    },
    ChartFloor {
        domain: "FFI & ABI",
        class: "Marchenvoy",
    },
    ChartFloor {
        domain: "Macros",
        class: "Runesmith",
    },
    ChartFloor {
        domain: "no_std & embedded",
        class: "Anchorite",
    },
    ChartFloor {
        domain: "Performance",
        class: "Quicksilver Adept",
    },
    ChartFloor {
        domain: "Kernel & OS",
        class: "Rootwarden",
    },
    ChartFloor {
        domain: "Compiler internals",
        class: "Mirrorwright",
    },
    ChartFloor {
        domain: "Verification",
        class: "Assayer",
    },
    ChartFloor {
        domain: "Toolchain internals",
        class: "Roadwarden",
    },
];

/// Stratum boundaries as (name, first floor index, floor count).
pub const STRATA: [(&str, usize, usize); 4] = [
    ("Stratum I — The language as a system", 0, 3),
    ("Stratum II — The machine model", 3, 5),
    ("Stratum III — Systems", 8, 5),
    ("Stratum IV — The depths", 13, 4),
];

// ---------------------------------------------------------------------------
// Floor 2 — The Menagerie Grounds. Domain 2: Traits & the type
// system. Keeper: the Sigilwright. FOE: The Menagerie.

/// The floor's name.
pub const FLOOR2_NAME: &str = "The Menagerie Grounds";
/// The domain this floor teaches.
pub const FLOOR2_DOMAIN: &str = "Traits & the type system";
/// The floor's keeper.
pub const FLOOR2_KEEPER: &str = "the Sigilwright";

/// The floor inscription, read at the camp stone.
pub const FLOOR2_INTRO: &str = "Floor 2 — The Menagerie Grounds. The Sigilwright keeps a menagerie where every beast is described by what it can do. Five chambers teach the description's five faces — traits and impls, generics and bounds, associated types, trait objects and the Wide Door, and coherence. At the grounds' heart stands the admission desk of the Menagerie — and the desk has begun admitting beasts by resemblance, not by sigil.";

/// The Lens Stone's principle (the Principle Codex entry).
pub const LENS_STONE: &str = "The Lens Stone";
/// The Lens Stone's text.
pub const LENS_STONE_TEXT: &str = "Bounds are admissions.";
/// The floor's loot artifact.
pub const PLUGIN_MENAGERIE: &str = "The Plugin Menagerie";
/// The loot's codex text.
pub const PLUGIN_MENAGERIE_TEXT: &str = "The two-door host: a travelling menagerie that can field a beast by its kind, through the Narrow Door, or by its sigils, through the Wide Door — carried in the pack, and wielded wherever a service must be staffed by kind or by sigil.";

/// One beast on the Menagerie roster.
pub struct Beast {
    /// The beast's name.
    pub name: &'static str,
    /// The kind of beast (e.g., "a salamander of the banked coals").
    pub kind: &'static str,
    /// The sigils this beast carries.
    pub sigils: &'static [&'static str],
    /// The group this beast belongs to.
    pub group: &'static str,
}

/// The roster's two watches, in board order.
pub const ROSTER_GROUPS: [&str; 2] = ["The Fire Watch", "The Mist Watch"];

/// The roster: six beasts, three to a watch.
pub const BEASTS: [Beast; 6] = [
    Beast {
        name: "Ember",
        kind: "a salamander of the banked coals",
        sigils: &["FireWard", "Swift"],
        group: "The Fire Watch",
    },
    Beast {
        name: "Cinder",
        kind: "a fox of the smoldering ashes",
        sigils: &["FireWard", "KeenNose"],
        group: "The Fire Watch",
    },
    Beast {
        name: "Soot",
        kind: "a raven of the chimney's height",
        sigils: &["FireWard", "SureFoot"],
        group: "The Fire Watch",
    },
    Beast {
        name: "Mistveil",
        kind: "a wolf of the shifting fog",
        sigils: &["MistSense", "DeepSight"],
        group: "The Mist Watch",
    },
    Beast {
        name: "Drizzle",
        kind: "a hare of the dew-laden meadow",
        sigils: &["MistSense", "Swift"],
        group: "The Mist Watch",
    },
    Beast {
        name: "Vapor",
        kind: "a hawk of the cloud's edge",
        sigils: &["MistSense", "KeenNose"],
        group: "The Mist Watch",
    },
];

/// A post the roster serves, and the bound it asks of a beast.
pub struct AdmissionService {
    /// The name of the service post.
    pub name: &'static str,
    /// The bounds required to enter this service.
    pub bound: &'static [&'static str],
}

/// The three admission services, in audit order.
pub const ADMISSION_SERVICES: [AdmissionService; 3] = [
    AdmissionService {
        name: "The Hearth Post",
        bound: &["FireWard"],
    },
    AdmissionService {
        name: "The Mist Post",
        bound: &["MistSense"],
    },
    AdmissionService {
        name: "The Valley Post",
        bound: &["Swift"],
    },
];

/// The five chambers, in design order.
pub const MENAGERIE_CHAMBERS: [ChamberMeta; 5] = [
    ChamberMeta {
        name: "Traits and impls",
        room: "The forge where every beast is marked",
        inscription_title: "The Marking",
        inscription: "A beast's sigils are its deeds. The forge stamps them true.",
        demonstration: "A cub enters unmarked; it performs, and the anvil rings with its proof.",
        slot_wants: "a type that can do the deed",
        prove_label: "Strike the sigil",
    },
    ChamberMeta {
        name: "Generics and bounds",
        room: "The hall of many doors",
        inscription_title: "The Door's Rule",
        inscription: "A door admits only those who bear the sigil. The hall's doors test each beast.",
        demonstration: "A fire-ward beast passes the hearth door; a mist-sensed beast is turned away.",
        slot_wants: "a type parameter with a trait bound",
        prove_label: "Test the door",
    },
    ChamberMeta {
        name: "Associated types",
        room: "The chamber of hidden forms",
        inscription_title: "The Hidden Form",
        inscription: "Some sigils name not what a beast is, but what it becomes. The chamber reveals them.",
        demonstration: "A beast enters as one kind and leaves as another, its hidden form now plain.",
        slot_wants: "a trait with an associated type",
        prove_label: "Reveal the form",
    },
    ChamberMeta {
        name: "Trait objects and the Wide Door",
        room: "The wide hall of many paths",
        inscription_title: "The Wide Door",
        inscription: "A wide door admits any beast that bears any sigil. The hall's paths are many.",
        demonstration: "A fire-ward beast and a mist-sensed beast both pass through the wide door.",
        slot_wants: "a trait object",
        prove_label: "Open the wide door",
    },
    ChamberMeta {
        name: "Coherence",
        room: "The chamber of the single truth",
        inscription_title: "The Single Truth",
        inscription: "A sigil may be stamped only once. The chamber enforces this rule.",
        demonstration: "Two forges attempt to stamp the same sigil; only one succeeds.",
        slot_wants: "a coherent impl",
        prove_label: "Enforce the rule",
    },
];

/// The FOE's name.
pub const FOE2_NAME: &str = "The Menagerie";
/// The FOE's inscription: the rule it breaks, carved.
pub const FOE2_INSCRIPTION: &str = "The desk admits by resemblance. A grey coat is not a fire sigil, yet the slip is stamped all the same. Audit every admission against its charter: exhibit each working beast's sigils against the bound of the post it serves. What can be found by audit need never be guessed. The guild's book of failures names the seam: failure class #6, the Cartographer's Blind Eye — a fine instrument used outside the domain it was made for.";
/// The FOE demonstrated: the mechanism at its worst.
pub const FOE2_DEMONSTRATION: &str = "The mist-hound stands its fire-watch in a grey coat the colour of the ash-hounds. Set before a lit brazier — the valley's one known landmark — it lowers its nose, breathes deep, and reports fog. It is not lying. It was never sigilled for the post.";

/// One admission on the FOE's own audit table: a beast, the post
/// its slip assigns it to, and the grounds the slip gives. The
/// FOE's cast — the ash-hounds and the mist-hound among them —
/// stands in this table and nowhere else: the roster (`BEASTS`)
/// is never resized for the audit.
pub struct FoeAdmission {
    /// The beast's name, as the slip writes it.
    pub name: &'static str,
    /// The kind of beast.
    pub kind: &'static str,
    /// The sigils this beast actually carries.
    pub sigils: &'static [&'static str],
    /// The post the slip assigns it to: an index into
    /// [`ADMISSION_SERVICES`].
    pub service: usize,
    /// What the desk's slip claims: the post, and its grounds.
    pub slip: &'static str,
}

/// The desk's admission table: seven slips, one for every beast
/// standing a post. Six agree with the sigils they name. One was
/// stamped on resemblance alone.
pub const FOE_ADMISSIONS: [FoeAdmission; 7] = [
    FoeAdmission {
        name: "Ember",
        kind: "a salamander of the banked coals",
        sigils: &["FireWard", "Swift"],
        service: 0,
        slip: "fire-watch at the Hearth Post — sigilled and standing",
    },
    FoeAdmission {
        name: "Cinder",
        kind: "a fox of the smoldering ashes",
        sigils: &["FireWard", "KeenNose"],
        service: 0,
        slip: "fire-watch at the Hearth Post — sigilled and standing",
    },
    FoeAdmission {
        name: "Soot",
        kind: "a raven of the chimney's height",
        sigils: &["FireWard", "SureFoot"],
        service: 0,
        slip: "fire-watch at the Hearth Post — sigilled and standing",
    },
    FoeAdmission {
        name: "The Mist-Hound",
        kind: "a hound of the shifting fog",
        sigils: &["MistSense", "DeepSight"],
        service: 0,
        slip: "fire-watch at the Hearth Post — grey as the ash-hounds, \
               and quiet at the post",
    },
    FoeAdmission {
        name: "Mistveil",
        kind: "a wolf of the shifting fog",
        sigils: &["MistSense", "DeepSight"],
        service: 1,
        slip: "mist-watch at the Mist Post — sigilled and standing",
    },
    FoeAdmission {
        name: "Vapor",
        kind: "a hawk of the cloud's edge",
        sigils: &["MistSense", "KeenNose"],
        service: 1,
        slip: "mist-watch at the Mist Post — sigilled and standing",
    },
    FoeAdmission {
        name: "Drizzle",
        kind: "a hare of the dew-laden meadow",
        sigils: &["MistSense", "Swift"],
        service: 2,
        slip: "the Valley Post — sigilled and standing",
    },
];

/// The calibration record: the suspect run against the valley's
/// one known landmark. Written when the brazier test is run and
/// kept with the audit as the proof's second half.
pub const FOE2_CALIBRATION: &str = "The calibration record: set before the lit \
    brazier, burning true, the mist-hound reported fog, thick and honest. \
    Its senses did not fail — they were never made for this post. The Lens \
    stands calibrated.";

// ---------------------------------------------------------------------------
// Floor 2, Chamber 1 — Traits and impls: the Anvil Call.
// ---------------------------------------------------------------------------

/// The house trick of the forge: a call chartered in the sigil-book.
pub const TRICK_NAME: &str = "the Anvil Call";

/// The sigil that charters the Anvil Call: the sigil-book declares
/// the trick under this sigil, and no other.
pub const TRICK_CHARTER: &str = "Forgehand";

/// The sigils the proof slot offers as candidate charters. Two are
/// traps of resemblance: FireWard is what both performing beasts
/// wear to the hearth, and Swift is what one of them wears — a
/// charter is the sigil that declares the trick, not a sigil the
/// body happens to wear.
pub const CHARTER_CANDIDATES: [&str; 3] = ["FireWard", "Forgehand", "Swift"];

/// The house performance: the trick exactly as the sigil-book
/// writes it — the default body of the call.
pub const TRICK_HOUSE_PERFORMANCE: &str = "strikes the anvil once — a single square \
    note, exactly as the sigil-book writes it, neither hurried nor held.";

/// A variant performance: the same call, its ending made the
/// performer's own — an overriding body of the trick.
pub const TRICK_VARIANT_PERFORMANCE: &str = "strikes twice — the written note, then \
    a second, softer one that falls like a feather: an ending of its own.";

/// One beast that has learned the Anvil Call at the forge.
pub struct TrickPerformer {
    /// Index into [`BEASTS`].
    pub beast: usize,
    /// How this body performs the call (the behaviour text).
    pub performance: &'static str,
}

/// The two beasts taught at the forge: Ember keeps the house body
/// of the trick; Soot performs its own variant.
pub const TRICK_PERFORMERS: [TrickPerformer; 2] = [
    TrickPerformer {
        beast: 0,
        performance: TRICK_HOUSE_PERFORMANCE,
    },
    TrickPerformer {
        beast: 2,
        performance: TRICK_VARIANT_PERFORMANCE,
    },
];

/// The blind run: behind the forge screen the call is sounded once
/// more, unlabelled. Which body ran is knowable only by behaviour.
pub const BLIND_RUN_PERFORMANCE: &str = TRICK_VARIANT_PERFORMANCE;

// ---------------------------------------------------------------------------
// Floor 2, Chamber 2 — Generics and bounds: the Generic Stable.
// ---------------------------------------------------------------------------

/// The sigils the stable's bounds may name, in board order.
pub const STABLE_SIGILS: [&str; 4] = ["FireWard", "MistSense", "Swift", "KeenNose"];

/// The bound the stable posts before the player revises it: every
/// sigil demanded of every beast, over-strict by the house's own
/// admission. A bitmask over `STABLE_SIGILS`.
pub const POSTED_BOUND: u8 = 0b1111;

/// One service the stable staffs, and the sigils its work — stated
/// plainly in its posting — actually requires of a beast.
pub struct StableService {
    /// The service's name.
    pub name: &'static str,
    /// The work, as the stable posts it; the required sigils are
    /// readable from it, never guessed.
    pub work: &'static str,
    /// The sigils the work requires, as a bitmask over
    /// `STABLE_SIGILS`.
    pub required: u8,
}

/// The three stable services, in board order.
pub const STABLE_SERVICES: [StableService; 3] = [
    StableService {
        name: "The Hearth Post",
        work: "Stand the hearth through the night and feel heat through \
               stone, raising the alarm before any flame shows.",
        required: 0b0001,
    },
    StableService {
        name: "The Letter Run",
        work: "Carry the sealed letters through the morning mist to the \
               far houses: the road is fog-blind, and the run is won on \
               speed, and on sensing the way when eyes fail.",
        required: 0b0110,
    },
    StableService {
        name: "The Stores Audit",
        work: "Walk the stores and find by scent alone which casks have \
               turned — no mark, no label, only the nose.",
        required: 0b1000,
    },
];

// ---------------------------------------------------------------------------
// Floor 2, Chamber 3 — Associated types: the Two Kennels.
// ---------------------------------------------------------------------------

/// One kennel of the courier service, and who chooses the kind
/// that runs its errands.
pub struct Kennel {
    /// The kennel's name.
    pub name: &'static str,
    /// The house rule: who chooses the beast-kind.
    pub rule: &'static str,
}

/// The two kennels, in board order: the kennel that chooses its
/// own kind (an associated type, fixed by the house), and the
/// kennel where the customer chooses (a generic kind, fixed at
/// each call).
pub const KENNELS: [Kennel; 2] = [
    Kennel {
        name: "The Hearth Kennel",
        rule: "the kennel chooses the kind — every errand runs on the \
               ash-hound, sure-footed and deep-chested, and no swimmer.",
    },
    Kennel {
        name: "The Open Kennel",
        rule: "the customer chooses the kind, named at the desk for \
               each errand, from any beast on the roster.",
    },
];

/// One courier errand the kennels run.
pub struct Errand {
    /// The errand's name.
    pub name: &'static str,
    /// What the errand needs, posted at the desk.
    pub need: &'static str,
}

/// The three errands, in board order.
pub const ERRANDS: [Errand; 3] = [
    Errand {
        name: "The Valley Letters",
        need: "Sealed letters for the valley post, by the high road. \
               Any sound beast delivers them.",
    },
    Errand {
        name: "The River Parcel",
        need: "A parcel for the far bank. The road crosses the river \
               at the deep ford — the carrier must swim.",
    },
    Errand {
        name: "The Night Medicines",
        need: "Medicines for the night post, wanted before dawn along \
               the long ridge road. Any sound beast delivers them.",
    },
];

/// The record of one delivery: how an errand ran from a kennel,
/// and whether the running was contorted — the errand fighting
/// the house's choosing rule.
pub struct Delivery {
    /// How the run went, as the kennel's record states it.
    pub text: &'static str,
    /// Whether the errand ran contorted from this kennel.
    pub contorted: bool,
}

/// The delivery records, `DELIVERIES[kennel][errand]`. One errand
/// runs contorted from the kennel that chooses its own kind: the
/// River Parcel, whose crossing the ash-hound cannot make.
pub const DELIVERIES: [[Delivery; 3]; 2] = [
    [
        Delivery {
            text: "the ash-hound trots the high road; the letters are \
                   at the valley post by noon, the kind suiting the \
                   errand exactly.",
            contorted: false,
        },
        Delivery {
            text: "the ash-hound paws at the deep ford and can go no \
                   further; the parcel comes back on its own collar, \
                   and crosses at last by boat, a day late.",
            contorted: true,
        },
        Delivery {
            text: "the ash-hound takes the ridge road at its steady \
                   pace; the medicines arrive before dawn.",
            contorted: false,
        },
    ],
    [
        Delivery {
            text: "the customer names a kind at the desk for an errand \
                   any beast could run; the letters go, after the \
                   choosing is done.",
            contorted: false,
        },
        Delivery {
            text: "the customer names the otter-kind; the parcel \
                   crosses the deep ford dry, straight over.",
            contorted: false,
        },
        Delivery {
            text: "the customer names a swift beast for the ridge \
                   road; the medicines arrive before dawn.",
            contorted: false,
        },
    ],
];

// ---------------------------------------------------------------------------
// Floor 2, Chamber 4 — Trait objects and the Wide Door: the registry.
// ---------------------------------------------------------------------------

/// The one service routine the registry drives through the Wide
/// Door, stated once for every beast alike.
pub const WIDE_DOOR_SERVICE: &str = "The Dawn Round: one routine, driven through the \
    Wide Door, calls on every beast in the registry alike. The door admits a beast \
    only if its charter can be answered without knowing the beast's kind.";

/// The three bars the Wide Door sets, in placard order. A barred
/// beast's placard states the shape of its charter; the bar is the
/// rule that shape breaks.
pub const BAR_REASONS: [&str; 3] = [
    "Its charter returns its own body, whole — a method returning Self cannot be \
     called through the door, which does not know the body's size",
    "Its charter method is generic over the load — a method that names a new \
     kind at every call cannot be answered by one routine for all beasts",
    "Its rite names no self — a charter struck with no beast present cannot be \
     driven through a door that admits bodies",
];

/// The short forms of the three bars, for the naming chips.
pub const BAR_REASON_LABELS: [&str; 3] = ["Returns Self", "Generic method", "No self"];

/// One beast standing in the registry, awaiting the Wide Door.
pub struct RegistryBeast {
    /// The beast's name.
    pub name: &'static str,
    /// The kind of beast.
    pub kind: &'static str,
    /// What its charter method looks like, as the registry's book states it.
    pub charter: &'static str,
    /// Whether this beast may pass the Wide Door.
    pub passes: bool,
    /// For a barred beast, the index into `BAR_REASONS` of its bar.
    pub bar: Option<usize>,
}

/// The registry's six beasts, in book order: three may pass the
/// Wide Door; three are barred, each for a different named reason.
pub const REGISTRY_BEASTS: [RegistryBeast; 6] = [
    RegistryBeast {
        name: "Ember",
        kind: "a salamander of the banked coals",
        charter: "the beast answers the round in its own body, and the round asks \
                  nothing the body cannot give",
        passes: true,
        bar: None,
    },
    RegistryBeast {
        name: "Drizzle",
        kind: "a hare of the dew-laden meadow",
        charter: "the beast answers the round in its own body, and the round asks \
                  nothing the body cannot give",
        passes: true,
        bar: None,
    },
    RegistryBeast {
        name: "Vapor",
        kind: "a hawk of the cloud's edge",
        charter: "the beast answers the round in its own body, and the round asks \
                  nothing the body cannot give",
        passes: true,
        bar: None,
    },
    RegistryBeast {
        name: "Brindle",
        kind: "a hound of the coppice",
        charter: "its charter method hands the caller back the beast's own body, \
                  whole",
        passes: false,
        bar: Some(0),
    },
    RegistryBeast {
        name: "Tessell",
        kind: "a tortoise of the river road",
        charter: "its charter method carries a load of any kind, the kind named \
                  fresh by the caller at every call",
        passes: false,
        bar: Some(1),
    },
    RegistryBeast {
        name: "The Unmounted Bell",
        kind: "a bell that hangs in the registry door, beast to no house",
        charter: "its rite is struck at dawn with no beast present; the charter \
                  names no self",
        passes: false,
        bar: Some(2),
    },
];

// ---------------------------------------------------------------------------
// Floor 2, Chamber 5 — Coherence: Foreign Marks.
// ---------------------------------------------------------------------------

/// The foreign beast, come up the valley with the floods.
pub const FOREIGN_BEAST_NAME: &str = "Auzi";

/// What the foreign beast is.
pub const FOREIGN_BEAST_KIND: &str = "a river-horse of the Silt House";

/// The foreign sigil the player would join to the foreign beast.
pub const FOREIGN_SIGIL: &str = "TideLedger";

/// The chamber's situation, as the Sigilwright states it.
pub const FOREIGN_MARK_STORY: &str = "Auzi, a river-horse of the Silt House, stands in the \
    grounds with the floods' mud on her legs. Her own house's sigil, TideLedger, \
    would let her work the Menagerie's water-ledgers — but the beast is foreign and \
    the sigil is foreign, and this house owns neither. The joining, if it is to be \
    made here, must be made lawfully.";

/// One route by which the player might join the foreign sigil
/// to the foreign beast.
pub struct CollarRoute {
    /// The route's name.
    pub name: &'static str,
    /// What the route does, plainly.
    pub detail: &'static str,
    /// Whether the house's law allows this route.
    pub legal: bool,
    /// Why the Sigilwright refuses it, when she does.
    pub refusal: &'static str,
}

/// The three joining routes, in desk order. Exactly one is lawful.
pub const COLLAR_ROUTES: [CollarRoute; 3] = [
    CollarRoute {
        name: "Stamp the sigil on the beast directly",
        detail: "Work TideLedger onto Auzi's own hide, here at the Menagerie's forge.",
        legal: false,
        refusal: "Neither the beast nor the sigil is this house's own. A house may \
                  stamp only what it owns — its own sigils on any beast, or any sigil on \
                  its own beasts. Foreign marks on foreign hides are refused at the forge.",
    },
    CollarRoute {
        name: "Forge a guild collar for the beast",
        detail: "Make a collar of your own forging — a beast-shape of this house's \
                 making that wears Auzi within it — and stamp the sigil on the collar.",
        legal: true,
        refusal: "",
    },
    CollarRoute {
        name: "Ask the door for a blanket joining",
        detail: "Persuade the registry to treat any foreign beast as joined to any \
                 sigil it is shown beside.",
        legal: false,
        refusal: "A blanket joining would claim the sigil for every foreign beast at \
                  once — and collide with every other house's own stampings, present and \
                  future. Coherence is not waived at the door.",
    },
];

/// The staged collision, as the desk presents it.
pub const COLLISION_TEXT: &str = "Two collars now claim TideLedger for Auzi: the Reed Collar of \
    your own forging, and the Brass Collar another keeper laid on her last spring. \
    Both are honest work; neither keeper erred. Read what their collision specifies.";

/// One reading of the staged collision.
pub struct CollisionReading {
    /// The reading, as the desk offers it.
    pub text: &'static str,
    /// Whether this reading holds.
    pub ok: bool,
}

/// The three readings of the collision. Exactly one holds.
pub const COLLISION_READINGS: [CollisionReading; 3] = [
    CollisionReading {
        text: "Both collars stand; the door takes whichever was forged first.",
        ok: false,
    },
    CollisionReading {
        text: "The newer collar replaces the older, as a new roof replaces the old.",
        ok: false,
    },
    CollisionReading {
        text: "A sigil may be claimed once for a beast: the collision is the rule \
               stating itself, and the houses must agree a single collar.",
        ok: true,
    },
];

/// The short forms of the three readings, for the holding chips.
pub const COLLISION_READING_LABELS: [&str; 3] = [
    "First forged stands",
    "Newer replaces older",
    "One claim per beast",
];

// ---------------------------------------------------------------------------
// Floor 2 — the Herald beat: sealed letters arrive through the floor.
// ---------------------------------------------------------------------------

/// One sealed letter the Herald leaves at the grounds.
pub struct HeraldLetter {
    /// The letter's announcement, read at the desk.
    pub text: &'static str,
    /// Whether any beast of the grounds offers the announced service.
    pub offered: bool,
}

/// The Herald's three letters, in arrival order. One announces a
/// service no beast has ever offered.
pub const HERALD_LETTERS: [HeraldLetter; 3] = [
    HeraldLetter {
        text: "For the Hearth Post: a beast is wanted that feels heat through stone, \
               to stand the night watch.",
        offered: true,
    },
    HeraldLetter {
        text: "For the Frost Post: a beast is wanted that breathes winter and stills \
               the ponds, to keep the ice road.",
        offered: false,
    },
    HeraldLetter {
        text: "For the Valley Post: a swift beast is wanted, to carry the morning \
               letters to the far houses.",
        offered: true,
    },
];

/// Acting on a true letter.
pub const HERALD_FILED: &str = "The letter is filed with the post it names. The service is real, \
    and the roster hears of it.";

/// Acting on the false letter: a free, miniature expedition, wasted.
pub const HERALD_WRONG_ROOM: &str = "You ride out at dawn for the Frost Post — and find no post, no \
    keeper, no frost sigil anywhere in the grounds. The letter announced a service no \
    beast has ever offered. The expedition is free, and miniature, and entirely wasted: \
    the floor's first wrong room. The letter is filed, marked as what it was.";
/// The Floor 2 hint ladders, one per chamber, four rungs each —
/// the same law as Floor 1's: viewing is free, rung 4 names the
/// seam and forfeits the facet. Chambers beyond the first are
/// carved by their own steps; until then their ladders carry the
/// Sigilwright's placeholder line.
pub fn floor2_hints(chamber: usize) -> [&'static str; 4] {
    match chamber {
        0 => [
            "Watch both performances again. The call is one; the bodies are two — \
             the difference lives in the ending.",
            "A sigil charters a trick when the sigil-book declares the trick under \
             it — not when a beast merely wears the sigil for other work.",
            "The house body strikes once. The variant strikes twice, the second \
             note falling soft. Which ending did the blind run have — and which \
             sigil declares the call, whatever the beasts wear?",
            "The seam is plain: the Anvil Call is chartered by Forgehand, and the \
             blind run struck twice, feather-soft at the end — Soot, performing \
             its own variant of the house trick.",
        ],
        1 => [
            "The stable posts its bound the way a nervous house posts rules: \
             everything, against everyone. Read each service's work again — \
             the bound should ask only for what the work names.",
            "A bound is the full price of admission. Every sigil it demands \
             excludes every beast that lacks it — whether the service needs \
             that sigil or not.",
            "Consider the Hearth Post: its work is feeling heat through \
             stone. Which single sigil does that work name? The other three \
             are the nervous house talking.",
            "The seam is plain: the Hearth Post wants FireWard alone; the \
             Letter Run wants MistSense and Swift; the Stores Audit wants \
             KeenNose alone. Every other sigil in the posted bound is \
             unjustified.",
        ],
        2 => [
            "Two kennels, one service. In one house the kennel's own kind \
             runs every errand; in the other, the customer names the kind \
             at the desk. Run the deliveries and read what actually \
             happened.",
            "An associated type is a choice made once, by the house. A \
             generic kind is a choice made at every call, by the customer. \
             Each errand feels the difference in a different place.",
            "Watch the parcel that must cross the river. From one kennel \
             it comes back on its own collar, a day late; from the other \
             it crosses dry. That errand is the finding.",
            "The seam is plain: the River Parcel runs contorted from the \
             Hearth Kennel, whose ash-hound cannot swim — and naturally \
             from the Open Kennel, where the customer names the \
             otter-kind.",
        ],
        3 => [
            "The Wide Door is one routine for every beast alike. It can only \
             call what a charter promises without knowing the beast's kind — \
             drive the beasts whose charters keep that promise.",
            "Three beasts in the registry cannot pass, and each placard states \
             the shape of its charter. Read the charters, not the coats.",
            "A charter that hands back the beast's own body, a charter that \
             names a new kind at every call, a rite with no self to drive — \
             none of these can be answered through one door for all.",
            "The seam is plain: Ember, Drizzle, and Vapor pass. Brindle is \
             barred for returning Self, Tessell for a generic method, and the \
             Unmounted Bell for a rite that names no self.",
        ],
        4 => [
            "A foreign beast and a foreign sigil: the house owns neither. \
             What a house may stamp is bounded by what it owns — its sigils, \
             its beasts, or a thing of its own making.",
            "The collar is the house's own making. What is stamped on the \
             collar is stamped on a beast-shape this house owns, though the \
             river-horse inside it stays foreign.",
            "For the collision: two honest collars, one sigil, one beast. The \
             question is not which keeper erred — read the collision as the \
             rule stating its own shape.",
            "The seam is plain: forge the guild collar and stamp the sigil on \
             it. And the two collars specify the coherence rule — a sigil may \
             be claimed once for a beast, so the houses must agree a single \
             collar.",
        ],
        _ => [
            "The Sigilwright has not carved this ladder yet.",
            "The Sigilwright has not carved this ladder yet.",
            "The Sigilwright has not carved this ladder yet.",
            "The Sigilwright has not carved this ladder yet.",
        ],
    }
}

/// The FOE's hint ladder: four rungs, the same law as every
/// ladder on the floor — viewing is free, and the fourth names
/// the seam outright. The FOE carries no facet, so nothing is
/// forfeit but the guessing.
pub fn floor2_foe_hints() -> [&'static str; 4] {
    [
        "Take the audit at the desk, then exhibit each admission's sigils \
         against its post's bound. Seven slips stand on the table; the desk \
         stamped every one.",
        "A slip and a sigil list are two different writings. Where they \
         agree, the admission stands. Read for the one place they do not: a \
         post whose bound names a sigil the beast does not carry.",
        "Consider the fire-watch. Three of its four slips name beasts \
         sigilled FireWard. The fourth is grey, and quiet, and was admitted \
         for it. Grey is a colour. FireWard is a sigil.",
        "The seam is named: failure class #6, the Cartographer's Blind Eye. \
         The mist-hound stands the fire-watch on resemblance — its sigils \
         are MistSense and DeepSight, and its slip claims the Hearth Post. \
         Exhibit every admission, name that row, and run the hound against \
         the lit brazier: it will report fog, honestly, because the mist is \
         the domain it was made for.",
    ]
}

// ---------------------------------------------------------------------------
// Floor 3 — The Gatehouse. Domain 3: Idiomatic API design.
// Keeper: the Gatewright. FOE: The Gatehouse Release.

/// The floor's name.
pub const FLOOR3_NAME: &str = "The Gatehouse";
/// The domain this floor teaches.
pub const FLOOR3_DOMAIN: &str = "Idiomatic API design";
/// The floor's keeper.
pub const FLOOR3_KEEPER: &str = "the Gatewright";

/// The floor inscription, read at the camp stone.
pub const FLOOR3_INTRO: &str = "Floor 3 — The Gatehouse. The Gatewright keeps the valley's release office, where everything the guild sends into the world is inspected, documented, and released. Five chambers teach the promise a public surface makes — failures named by kind, belts that keep their word, shouts reserved for broken walls, paper that cannot be sealed early, and a checklist graded in the open. Above the gate hangs the season's great release, its lantern already green — and the keeper's chair is empty.";

/// The Lantern Stone's principle (the Principle Codex entry).
pub const LANTERN_STONE: &str = "The Lantern Stone";
/// The Lantern Stone's text.
pub const LANTERN_STONE_TEXT: &str = "A result unexamined is a gate unrun.";
/// The floor's loot artifact.
pub const GATEHOUSE_KEY: &str = "The Gatehouse Key";
/// The Gatehouse Key's card text.
pub const GATEHOUSE_KEY_TEXT: &str = "The Hoard raised to release grade: documented, checklisted, stamped with its oldest road — the standing example of done for a public surface, and the subject of the publication rite at the descent's end.";

/// The five Gatehouse chambers' carved furniture.
pub const GATEHOUSE_CHAMBERS: [ChamberMeta; 5] = [
    ChamberMeta {
        name: "Error taxonomy",
        room: "The Named Failures desk",
        inscription_title: "The Naming",
        inscription: "A failure unnamed is a failure unowned. The desk names every broken charter by its kind, and the kind prescribes the remedy.",
        demonstration: "A traveller's charter snaps at the seal; the desk names the break, and the remedy already stands written beside the name.",
        slot_wants: "a failure named by its kind",
        prove_label: "Name the failure",
    },
    ChamberMeta {
        name: "Iterator internals",
        room: "The Conveyor",
        inscription_title: "The Belt's Contract",
        inscription: "The belt promises one portion at a time — and, once empty, empty forever. A belt that changes its answer is a liar with a motor.",
        demonstration: "Crates ride one pull at a time; past the last crate the belt answers empty, and empty again.",
        slot_wants: "a chain that stays lazy and stays fused",
        prove_label: "Run the belt",
    },
    ChamberMeta {
        name: "Panic discipline",
        room: "The Audit of Shouts",
        inscription_title: "The Wall of Invariants",
        inscription: "A shout is for the wall that cannot hold, never for the traveller at the desk. Every failure a traveller can reach gets a name and a refusal.",
        demonstration: "The clerk shouts at a torn form; the audit re-trains the office, and the same form earns a named refusal instead.",
        slot_wants: "refusals for travellers, invariants for shouts",
        prove_label: "Hush the clerk",
    },
    ChamberMeta {
        name: "Typestate builders",
        room: "The Sealed Request",
        inscription_title: "The Stamped Paper",
        inscription: "Paper is addressed, then bodied, then sealed — in that order, and the press knows it. A seal on unstamped paper is not early; it is impossible.",
        demonstration: "An unaddressed petition meets the press; the press does not close.",
        slot_wants: "a request built through its states",
        prove_label: "Work the press",
    },
    ChamberMeta {
        name: "The Guidelines audit",
        room: "The Inspection",
        inscription_title: "The Public Checklist",
        inscription: "A surface is a promise read by strangers. The checklist is the promise, item by item — and the oldest road is stamped where travellers can read it.",
        demonstration: "The Hoard's papers are read against the wall, item by item; one line is missing, and the seal waits.",
        slot_wants: "a checklist honestly graded",
        prove_label: "Run the checklist",
    },
];

/// Floor 3 Chamber 1's desk charter, read at the watch.
pub const FAILURE_CHARTER: &str = "Every failure leaves this house by a named \
    door, with its remedy written under the name. A failure with no name is \
    carried to the Broken bin, and the Broken bin is emptied into the river \
    at night — which is to say, it is lost, and the house falls down one \
    unnamed failure at a time. Sort the reports on the wall: each takes the \
    pair that is truly its own — its kind, and the remedy its kind carries. \
    The Broken pair is on the desk because clerks reach for it. Reaching \
    for it is the one sorting this desk refuses.";

/// The failure pairs a report can be sorted under: a kind and the
/// remedy the kind carries. The last is the generic bin the desk
/// refuses — it is a candidate only so the refusal has something
/// to refuse.
pub const FAILURE_PAIRS: [(&str, &str); 5] = [
    (
        "NotFound",
        "Say what was sought; the caller decides whether to go on.",
    ),
    (
        "Malformed",
        "Name the bad figure and the slate it came from.",
    ),
    (
        "Refused",
        "Carry the refusal upward with the door's name on it.",
    ),
    ("Mismatch", "Show both faces of the pair that disagree."),
    ("Broken", "Mend it somehow."),
];

/// One failure report on Chamber 1's evidence wall.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FailureReport {
    /// The report as it reached the desk.
    pub text: &'static str,
    /// Its true pair in `FAILURE_PAIRS`.
    pub pair: usize,
}

/// The reports on Chamber 1's wall.
pub const FAILURE_REPORTS: [FailureReport; 4] = [
    FailureReport {
        text: "The night ledger is not on its shelf, nor in the returns cart.",
        pair: 0,
    },
    FailureReport {
        text: "The third figure of the tally slate is written in no hand \
               the desk knows.",
        pair: 1,
    },
    FailureReport {
        text: "The sluice gate will not answer its crank; the keeper's \
               mark is on the refusal.",
        pair: 2,
    },
    FailureReport {
        text: "The seal on the request does not match the seal on its letter.",
        pair: 3,
    },
];

/// Floor 3 Chamber 2's belt charter, read at the watch.
pub const CONVEYOR_CHARTER: &str = "The belt carries the release's pieces \
    past three stations, in this order: the portioner, which takes its \
    share from the raw belt; the joiner, which settles duplicates; and \
    the gate, which decides what a cracked piece does to the run. The \
    desk's slate names what the office must receive. Set the three \
    stations so the belt delivers exactly the slate — the desk runs \
    your chain itself, polls it past its end to prove it fused, and \
    compares what comes off the belt.";

/// One piece on Chamber 2's belt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BeltItem {
    /// A sound piece, by name.
    Sound(&'static str),
    /// A cracked piece, by name.
    Spoiled(&'static str),
}

impl BeltItem {
    /// The piece's name, sound or cracked.
    pub fn name(&self) -> &'static str {
        match self {
            BeltItem::Sound(name) | BeltItem::Spoiled(name) => name,
        }
    }
}

/// The belt's load, in order.
pub const CONVEYOR_BELT: [BeltItem; 8] = [
    BeltItem::Sound("gear"),
    BeltItem::Sound("gear"),
    BeltItem::Sound("spring"),
    BeltItem::Spoiled("cracked gear"),
    BeltItem::Sound("spring"),
    BeltItem::Sound("cam"),
    BeltItem::Sound("cam"),
    BeltItem::Sound("bell"),
];

/// What the office must receive, in order.
pub const CONVEYOR_REQUIRED: [&str; 4] = ["gear", "spring", "cam", "bell"];

/// The portioner's settings.
pub const PORTION_OPTIONS: [&str; 3] = [
    "Take two from the belt",
    "Take three from the belt",
    "Take the whole belt",
];

/// The joiner's settings.
pub const DEDUP_OPTIONS: [&str; 3] = [
    "Join each run of alike pieces into one",
    "Keep only the first sighting of each piece",
    "Keep every piece as it comes",
];

/// The gate's settings.
pub const HALT_OPTIONS: [&str; 3] = [
    "Halt the belt at the cracked piece",
    "Lift the cracked piece off and carry on",
    "Send the cracked piece through",
];

/// The five chambers' reading texts (the Read verb).
pub const FLOOR3_CHAMBER_INTROS: [&str; 5] = [
    "The Named Failures desk. Reports come up from every floor of the \
     house and are pinned to the wall unsorted; below them stands one \
     great bin chalked BROKEN, and it is fuller than all the named \
     doors together.",
    "The Conveyor. A belt of release pieces runs the length of the room \
     past three stations, and a slate by the gate names what the office \
     must receive.",
    "The Audit of Shouts. A soundproof booth, a ledger of everything \
     ever shouted in the house, and two chairs: one for the failures a \
     traveller can cause, one for the ones only the house can.",
    "The Sealed Request. A press, a wax pot, and a rail of requests in \
     three trays: unaddressed, addressed, bodied. The seal sits at the \
     end of the rail, and the rail is the point.",
    "The Inspection. The Guidelines wall, the Interner's Hoard open on \
     the table under audit, the amendment slate, and — smallest and \
     last — the Stranger's Anvil in its box.",
];

/// The Gatehouse's hint ladders, four rungs each (FULL-DESIGN
/// §5.4). Rung 4 names the chamber facet's location — a FOE has
/// no facet, but a chamber's ladder closes on its own seal.
pub fn floor3_chamber_hints(chamber: usize) -> [&'static str; 4] {
    match chamber {
        0 => [
            "Four reports, five pairs. Read the charter first — it \
             tells you what a sorting is.",
            "Each report's own words carry its kind: what is absent, \
             what is misshapen, what refused, what disagrees.",
            "The remedy is not chosen separately — it rides with the \
             kind. Find the kind and the pair is whole.",
            "The Broken pair fits nothing, and the desk refuses it \
             for all four. The pairs, in the wall's order, begin: \
             NotFound, Malformed…",
        ],
        1 => [
            "The stations act in order: portion, join, gate. Run the \
             belt in your head exactly as the desk will run it.",
            "The slate asks for a bell — the last piece on the belt. \
             Any portion smaller than the whole belt never reaches it.",
            "The second spring must not arrive: it is a duplicate, \
             but not of its neighbour. Joining runs is not enough.",
            "Whole belt, first sightings only, and the cracked gear \
             lifted off — the slate comes off the belt exactly, and \
             the chain stays silent past its end.",
        ],
        2 => [
            "Four shouts. Ask of each: could a traveller have caused \
             this, by handing the house something wrong?",
            "What a traveller causes leaves by a named door. What only \
             the house can cause may stand — but never naked.",
            "Two entries are the house breaking its own promises: the \
             belt's tally, and the pot's oath. A standing shout needs \
             its promise named.",
            "Refusals for the address-less request and the ninth piece; \
             the tally's invariant (candidate 1) under the belt's shout, \
             the pot's (candidate 2) under the press's.",
        ],
        3 => [
            "The order is the machine: unaddressed, addressed, bodied, \
             sealed. The press knows it even if a clerk forgets.",
            "Work the first slip down the whole rail. The press refuses \
             it twice before it may fall — that refusal is the lesson \
             working.",
            "One slip's body page is missing. It cannot be bodied, and \
             the rail will not pretend; there is a tray for exactly this.",
            "Key and tally sealed (the tally starts addressed — one \
             advance fewer); the stranger's slip returned. The rail \
             reads sealed, sealed, returned.",
        ],
        4 => [
            "Five lines over one Hoard. Judge the Hoard as it stands \
             today — with everything the floors below have done to it.",
            "Two lines it keeps as written. Two it broke and the work \
             below repaired. One governs something the Hoard is not.",
            "The seal line is the stranger: the Hoard seals nothing — \
             that is the reason, and it wants naming, not waiving.",
            "Repaired, conforms, not-applicable (it lends and holds), \
             conforms, repaired — and the stamp is the oldest road: \
             1.85, the first candidate.",
        ],
        _ => [
            "This desk is not open yet.",
            "This desk is not open yet.",
            "This desk is not open yet.",
            "This desk is not open yet.",
        ],
    }
}

/// Floor 3 Chamber 3's booth charter, read at the watch.
pub const SHOUT_CHARTER: &str = "A shout is the house's last \
    resort, not its first answer. Two kinds of failure reach \
    this booth. A failure a traveller can cause — a request \
    with no address, a belt asked for more than it carries — \
    must leave by a named door: it is the traveller's to \
    correct, and shouting at them teaches nothing. A failure \
    only the house can cause — its own machinery breaking a \
    promise the house itself made — may stand as a shout, \
    with the broken promise, its invariant, written under it. \
    A shout without its invariant is just noise in the hall.";

/// One entry in Chamber 3's ledger of shouts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShoutCase {
    /// The ledger entry, as recorded.
    pub text: &'static str,
    /// True when the shout stands (invariant-only); false when
    /// it should have been a named refusal.
    pub shout: bool,
    /// The invariant's index in `INVARIANT_CANDIDATES`, when
    /// the shout stands.
    pub invariant: Option<usize>,
}

/// The ledger of shouts on Chamber 3's wall.
pub const SHOUT_CASES: [ShoutCase; 4] = [
    ShoutCase {
        text: "A traveller hands in a request bearing no address; \
               the clerk SHOUTS, and the hall empties.",
        shout: false,
        invariant: None,
    },
    ShoutCase {
        text: "The belt reaches the gate with two cracked pieces \
               in a row; the run's own tally had promised one at \
               most. The station SHOUTS.",
        shout: true,
        invariant: Some(0),
    },
    ShoutCase {
        text: "A traveller asks the portioner for a ninth piece \
               from an eight-piece belt; the station SHOUTS.",
        shout: false,
        invariant: None,
    },
    ShoutCase {
        text: "The seal press fires over a cold wax pot; the \
               pot's keeper had sworn the pot is lit before the \
               press is ever armed. The press SHOUTS.",
        shout: true,
        invariant: Some(1),
    },
];

/// The invariants a standing shout can carry. Two are sworn by
/// the house; two are travellers' premises wearing the house's
/// clothes.
pub const INVARIANT_CANDIDATES: [&str; 4] = [
    "The run's tally bounds cracked pieces to one in a row.",
    "The press is armed only over a lit pot.",
    "A traveller's request always bears an address.",
    "The belt never carries a ninth piece.",
];

/// Floor 3 Chamber 4's rail charter, read at the watch.
pub const RAIL_CHARTER: &str = "A request is built in its \
    order, and no other: unaddressed, addressed, bodied, \
    sealed. The rail enforces what the order means — the seal \
    press cannot fall on a request the rail has not brought \
    bodied beneath it, not from policy but from iron: the \
    press and the rail are one machine. A slip whose body page \
    is missing cannot be bodied honestly; it goes to the \
    returns tray, and the rail keeps moving.";

/// One slip on Chamber 4's rail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestSlip {
    /// The slip's name.
    pub name: &'static str,
    /// Its stage on arrival: 0 unaddressed, 1 addressed,
    /// 2 bodied.
    pub start: u8,
    /// True when its body page is missing from the tray.
    pub body_missing: bool,
}

/// The slips on Chamber 4's rail. Stages: 0 unaddressed,
/// 1 addressed, 2 bodied, 3 sealed, 4 in the returns tray.
pub const REQUEST_SLIPS: [RequestSlip; 3] = [
    RequestSlip {
        name: "A request for a second key",
        start: 0,
        body_missing: false,
    },
    RequestSlip {
        name: "A request for the winter tally",
        start: 1,
        body_missing: false,
    },
    RequestSlip {
        name: "A request in a stranger's hand",
        start: 1,
        body_missing: true,
    },
];

/// A rail stage's name.
pub fn stage_name(stage: u8) -> &'static str {
    match stage {
        0 => "unaddressed",
        1 => "addressed",
        2 => "bodied",
        3 => "sealed",
        _ => "in the returns tray",
    }
}

/// Floor 3 Chamber 5's wall: the Guidelines, the oldest road,
/// and the amendment rite — read at the watch.
pub const INSPECTION_CHARTER: &str = "The Guidelines hang in \
    five lines, and the Hoard from the first floor stands open \
    under them for audit. Answer every line honestly: conforms, \
    where the Hoard as it stands keeps the line; repaired, \
    where it did not and the work below has mended it; \
    not-applicable, where the line governs something the Hoard \
    is not — with the reason named. Below the lines, the oldest \
    road: the gate honors every road from 1.85 onward, and the \
    release's stamp must say so. At the side desk, one amendment \
    waits in its rite: read it, second it, enter it in the book.";

/// One line of the Guidelines, with the Hoard's true standing
/// under it (0 conforms, 1 repaired, 2 not applicable).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InspectionItem {
    /// The guideline line.
    pub line: &'static str,
    /// The Hoard's true verdict under it.
    pub verdict: usize,
}

/// The five lines over the Hoard.
pub const INSPECTION_ITEMS: [InspectionItem; 5] = [
    InspectionItem {
        line: "Every admission names its bound.",
        verdict: 1,
    },
    InspectionItem {
        line: "The keeper's name stands on the door.",
        verdict: 0,
    },
    InspectionItem {
        line: "No surface lends past its seal.",
        verdict: 2,
    },
    InspectionItem {
        line: "The oldest road is walked at every release.",
        verdict: 0,
    },
    InspectionItem {
        line: "A stranger's tool is weighed before it is wielded.",
        verdict: 1,
    },
];

/// The verdicts a checklist line can carry.
pub const INSPECTION_VERDICTS: [&str; 3] = ["Conforms", "Repaired", "Not applicable"];

/// The reasons the not-applicable line can carry; the true one
/// is the first.
pub const NA_REASONS: [&str; 3] = [
    "The Hoard seals nothing; it lends and holds.",
    "The Hoard predates the Guidelines wall.",
    "The Hoard's keeper waived the line.",
];

/// The oldest road's stamp, as candidates; the true stamp is
/// the first.
pub const MSRV_CANDIDATES: [&str; 3] = [
    "1.85 — the oldest road the gate still honors",
    "1.99 — the pin the house builds under tonight",
    "2.00 — the road not yet laid",
];

/// The amendment waiting at the side desk.
pub const AMENDMENT_TEXT: &str = "Add to the Guidelines: a \
    lantern lit by schedule is no lantern. A gate is run, or \
    the release does not leave.";

/// The amendment rite's three acts, in their order.
pub const RITE_STAGES: [&str; 3] = [
    "Read the proposal aloud",
    "Second the proposal",
    "Enter it in the book",
];

/// The Gatehouse Release: the floor's FOE (FULL-DESIGN §4,
/// Floor 3). Failure class #5, the Lantern That Lights
/// Itself, is its seam.
pub const FOE3_NAME: &str = "The Gatehouse Release";

/// The Release's introduction, spoken at its door.
pub const FOE3_INTRO: &str = "The Gatehouse Release stands at \
    the great door with its lantern already burning — green, \
    steady, and lit before its keeper came in. Its log-slate \
    claims every pass was run. The Gatewright says nothing, \
    and hands you the slate.";

/// The Release's inscription: failure class #5, named in the
/// fiction as FULL-DESIGN requires of every FOE.
pub const FOE3_INSCRIPTION: &str = "Here is kept the record \
    of the fifth failure class: THE LANTERN THAT LIGHTS \
    ITSELF — a gate believed run because its lantern burned. \
    The lantern was wired to the clock, not to the work: it \
    lit at the release hour whether the passes ran or not, \
    and the house shipped on the strength of a light. The \
    cure is not a brighter lantern. The cure is a lantern with \
    no wire to the clock: lit by the doing, doused by the \
    next touch of the draft.";

/// One entry on the Release's log-slate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogEntry {
    /// The entry as written.
    pub text: &'static str,
    /// Whether the pass truly ran.
    pub ran: bool,
}

/// The log-slate's entries. Two are in an old hand on old
/// paper; one carries the draft's own fresh mark.
pub const LOG_SLATE: [LogEntry; 3] = [
    LogEntry {
        text: "Format pass — entered in a careful hand, on paper \
               older than the draft it names.",
        ran: false,
    },
    LogEntry {
        text: "Lint pass — the same hand, the same old paper, the \
               same hour struck twice.",
        ran: false,
    },
    LogEntry {
        text: "Test pass — entered fresh, the draft's own mark \
               beside the count.",
        ran: true,
    },
];

/// The Release's hint ladder.
pub fn floor3_foe_hints() -> [&'static str; 4] {
    [
        "Begin with the slate. It claims three passes; paper and \
         hands testify about two of them.",
        "A verdict per entry: ran, or never ran. Then reconcile — \
         write the true slate over the claimed one.",
        "The stamp is the oldest road, and the rite at the slate's \
         side desk is done in its order: read, second, enter.",
        "Three lanterns over the door, none wired to the clock: \
         format the draft and the first lights; lint it and the \
         second; run the gate and the third. Touch the draft and \
         all three go out — that is the lantern honest.",
    ]
}

/// The third gate's manifest: the Release judged as a crate.
pub const GATE3_CARGO_TOML: &str = "[package]\n\
     name = \"the_gatehouse_release\"\n\
     version = \"0.1.0\"\n\
     edition = \"2024\"\n\n\
     [dependencies]\n";

/// The third gate's scaffold, as the clerk left it.
pub const GATE3_SCAFFOLD: &str = "// The Gatehouse Release, as the clerk left it: two claims\n\
     // the release must make, neither yet made. The inscription\n\
     // over the office names the oldest road; the lanterns over\n\
     // the door say what ready means.\n\
     \n\
     /// The oldest road the gate still honors, as the stamp reads.\n\
     pub fn oldest_road() -> &'static str {\n\
     \x20   todo!()\n\
     }\n\
     \n\
     /// Whether the release may leave: every lantern lit by doing.\n\
     pub fn release_ready(fmt_done: bool, lint_done: bool, tests_done: bool) -> bool {\n\
     \x20   todo!()\n\
     }\n";

/// The third gate's reference release, as the keeper keeps it.
pub const GATE3_REFERENCE: &str = "pub fn oldest_road() -> &'static str {\n\
     \x20   \"1.85\"\n\
     }\n\
     \n\
     pub fn release_ready(fmt_done: bool, lint_done: bool, tests_done: bool) -> bool {\n\
     \x20   fmt_done && lint_done && tests_done\n\
     }\n";

/// The third gate's hidden suite: the stamp must name the
/// oldest road, and ready must mean every lantern.
pub const GATE3_TESTS: &str = "use the_gatehouse_release::{oldest_road, release_ready};\n\
     \n\
     #[test]\n\
     fn the_stamp_names_the_oldest_road() {\n\
     \x20   assert_eq!(oldest_road(), \"1.85\");\n\
     }\n\
     \n\
     #[test]\n\
     fn ready_means_every_lantern() {\n\
     \x20   assert!(release_ready(true, true, true));\n\
     }\n\
     \n\
     #[test]\n\
     fn an_unformatted_release_does_not_leave() {\n\
     \x20   assert!(!release_ready(false, true, true));\n\
     }\n\
     \n\
     #[test]\n\
     fn an_unlinted_release_does_not_leave() {\n\
     \x20   assert!(!release_ready(true, false, true));\n\
     }\n\
     \n\
     #[test]\n\
     fn an_untested_release_does_not_leave() {\n\
     \x20   assert!(!release_ready(true, true, false));\n\
     }\n\
     \n\
     #[test]\n\
     fn a_dark_release_does_not_leave() {\n\
     \x20   assert!(!release_ready(false, false, false));\n\
     }\n";

// ---------------------------------------------------------------------------
// Floor 4 — The Foundation Galleries. Domain 4: Memory layout.
// Keeper: the Plumbmason. FOE: The Crooked Foundation.

/// The floor's name.
pub const FLOOR4_NAME: &str = "The Foundation Galleries";
/// The domain this floor teaches.
pub const FLOOR4_DOMAIN: &str = "Memory layout";
/// The floor's keeper.
pub const FLOOR4_KEEPER: &str = "the Plumbmason";

/// The floor inscription, read at the camp stone.
pub const FLOOR4_INTRO: &str = "Floor 4 — The Foundation Galleries. Below the valley the warrens hold the labyrinth up: galleries of foundation stones, each stencilled with its true dimensions, and a plumb line hanging in every room. The Plumbmason's law is short — layout is measurable fact, proven by assertion and never described in prose, and what the line says outranks what anyone remembers. Five chambers teach the measure: predict before you drop the line, count the voids byte by byte, find the niche that hides a mark for free, keep a treaty's promised offsets, and never trust a tag that lives only in a mason's head. At the galleries' heart the master foundation settles crooked, laid from a celebrated survey no one has checked against the ground.";

/// Floor 4's five chambers, in design order.
pub const FOUNDATION_CHAMBERS: [ChamberMeta; 5] = [
    ChamberMeta {
        name: "Sizes and alignment",
        room: "The Predict-and-Measure bench",
        inscription_title: "The Committed Slate",
        inscription: "Commit the measure before the line drops. A prediction written down can be wrong honestly; a memory of one can only be wrong comfortably.",
        demonstration: "The mason writes a figure on the slate, drops the line, and grades the slate — not the stone — where they disagree.",
        slot_wants: "predictions committed, measured, and annotated",
        prove_label: "Drop the line",
    },
    ChamberMeta {
        name: "Padding and field order",
        room: "The Smallest Wall",
        inscription_title: "The Counted Voids",
        inscription: "The wall is charged for every void it carries. Reorder the same stones and the voids move — count them byte by byte, and let every saved unit name the move that saved it.",
        demonstration: "Five stones stand in bench order, their wall full of gaps; the same five, re-seated by their alignments, close the wall to its least span.",
        slot_wants: "the smallest wall the stones allow",
        prove_label: "Count the wall",
    },
    ChamberMeta {
        name: "Enum encoding and the niche",
        room: "The Seal Stones",
        inscription_title: "The Free Mark",
        inscription: "Some values can never occur, and the place they can never occupy is room for a mark that costs nothing. Find the niche before you pay for the tag.",
        demonstration: "A crest pays a whole word for its absent-mark until the mason points at the number that cannot happen — and the mark moves into the gap, free.",
        slot_wants: "every crest at its proven smallest",
        prove_label: "Prove the crest",
    },
    ChamberMeta {
        name: "The repr family",
        room: "The Treaty Stones",
        inscription_title: "The Promised Offsets",
        inscription: "A treaty stone promises every offset and is held to all of them. A free stone promises nothing and owes nothing — seat one in the other's wall and the wall fails honestly, which is the kindest failure there is.",
        demonstration: "The line walks the treaty wall and reads each offset aloud against the book; at the last stone it stops — the book's sum forgot the padding, and the stone overhangs the plinth by exactly that much.",
        slot_wants: "promised offsets, proven and amended",
        prove_label: "Prove the wall",
    },
    ChamberMeta {
        name: "Unions and uninitialized memory",
        room: "The Hollow Stones",
        inscription_title: "The Mason's Head",
        inscription: "A hollow stone holds one filling of two kinds, and the tag that says which kind lives only in the mason's head. Heads forget. The gallery's referee exists for the day one does — and for the blank stone used before it is full.",
        demonstration: "The mason fills a cavity with a ratio, tags it in his head as a number, and reads it back: the bits answer faithfully, and the answer is nonsense — the referee's whistle is the only honest sound in the room.",
        slot_wants: "tags that match their fillings, blanks used whole",
        prove_label: "Read the stones",
    },
];

/// Floor 4's chamber doors: the line each keeper speaks as
/// the player enters.
pub const FLOOR4_CHAMBER_INTROS: [&str; 5] = [
    "The Predict-and-Measure bench. Six stones wait in a row, and the slate hangs beside them: commit your measure first — size, then alignment — and only then does the line drop. Where the slate and the stone disagree, the surprise is yours to annotate, in your own hand, before the chamber will complete.",
    "The Smallest Wall. Five stones lie in bench order, and the wall they make in that order is full of voids. Seat them yourself, in any order you like — the line counts the wall byte by byte, and it will say what the least span is when you have found it.",
    "The Seal Stones. Three crests, each paying more for its absent-mark than it should. Every crest offers its encodings on the bench; choose, then prove — the bench reads real sizes, and the smallest honest encoding is the one that keeps the mark in a niche, free.",
    "The Treaty Stones. A wall under foreign spec: every offset promised in the book, and a free twin on the tray that promises nothing. Seat the treaty stones, drop the line — and mind the last stone, where the book's own sum was written carelessly.",
    "The Hollow Stones. Two cavities with two possible fillings each, and the tag that says which kind lives only in the mason's head. Beside them a blank stone of four cells, to be filled one by one — the referee is watching, and the referee is patient, and the referee is Miri.",
];

/// Floor 4's chamber Examine texts.
pub const F4_EXAMINE_TEXTS: [&str; 5] = [
    "The bench: a slate, a chalk line, and six stones stencilled with everything except the two numbers that matter.",
    "A low wall in bench order, its voids showing like missing teeth; the same stones stacked loose beside it.",
    "Three crests under glass, each with its encodings priced in whole words — and one price in each set scratched out.",
    "A treaty book open at the wall's page: four promises in a foreign hand, and the fourth written in a different ink.",
    "Two hollow stones with their fillings beside them in bowls, and a blank stone quartered into four empty cells.",
];

/// Chamber 1's stones, in bench order. Their facts are not
/// written here: `bench_stone_facts` measures the real types
/// at runtime, so the bench can never drift from the truth.
pub const BENCH_STONES: [&str; 6] = [
    "a lone byte — u8",
    "a byte and a word in one skin — (u8, u32)",
    "a borrowed word — &str",
    "a byte, or none — Option<u8>",
    "a borrowed word, or none — Option<&u64>",
    "a heap of bytes — Vec<u8>",
];

/// The bench's own measure of a stone: (size, alignment) of
/// the real type, taken with `size_of` / `align_of`. This is
/// the plumb line's whole doctrine in one function — the
/// quantity itself, never a table about the quantity.
pub fn bench_stone_facts(index: usize) -> (u32, u32) {
    match index {
        0 => (
            std::mem::size_of::<u8>() as u32,
            std::mem::align_of::<u8>() as u32,
        ),
        1 => (
            std::mem::size_of::<(u8, u32)>() as u32,
            std::mem::align_of::<(u8, u32)>() as u32,
        ),
        2 => (
            std::mem::size_of::<&str>() as u32,
            std::mem::align_of::<&str>() as u32,
        ),
        3 => (
            std::mem::size_of::<Option<u8>>() as u32,
            std::mem::align_of::<Option<u8>>() as u32,
        ),
        4 => (
            std::mem::size_of::<Option<&u64>>() as u32,
            std::mem::align_of::<Option<&u64>>() as u32,
        ),
        _ => (
            std::mem::size_of::<Vec<u8>>() as u32,
            std::mem::align_of::<Vec<u8>>() as u32,
        ),
    }
}

/// Chamber 1's annotation candidates per stone: why the
/// slate and the stone disagreed. Exactly one per stone is
/// true (`BENCH_TRUE_CAUSE`).
pub const BENCH_CAUSES: [[&str; 3]; 6] = [
    [
        "Padding fills the gaps between the fields.",
        "Nothing hides: a byte is a byte — size one, alignment one.",
        "The absent-mark hides in a value that cannot occur.",
    ],
    [
        "The absent-mark hides in a value that cannot occur.",
        "Padding fills the gaps: the word must sit on its own alignment, and the byte waits before it.",
        "It is two machine words: a pointer and a length.",
    ],
    [
        "Padding fills the gaps between the fields.",
        "It is two machine words: a pointer and a length.",
        "The absent-mark hides in a value that cannot occur.",
    ],
    [
        "The absent-mark hides in a value that cannot occur.",
        "The tag stands beside the byte: every byte value can occur, so no mark is free.",
        "Padding fills the gaps between the fields.",
    ],
    [
        "The tag stands beside the word, paid in full.",
        "The absent-mark hides in the one address that cannot occur: the null address.",
        "Padding fills the gaps between the fields.",
    ],
    [
        "It is two machine words: a pointer and a length.",
        "Padding fills the gaps between the fields.",
        "It is three machine words: pointer, length, and capacity.",
    ],
];

/// The true annotation per bench stone, by index into
/// `BENCH_CAUSES`.
pub const BENCH_TRUE_CAUSE: [usize; 6] = [1, 1, 1, 1, 1, 2];

/// Chamber 2's stones, in bench order: (name, size,
/// alignment). The pairs are authored data — the layout of
/// any order of them is computed by the one algorithm in
/// puzzles.rs, and a unit test cross-checks the bench order
/// and the minimum against a brute-force pass.
pub const WALL_STONES: [(&str, u32, u32); 5] = [
    ("the pin", 1, 1),
    ("the beam", 8, 8),
    ("the plate", 4, 4),
    ("the shim", 1, 1),
    ("the rod", 2, 2),
];

/// Chamber 1's charter, spoken at the watch.
pub const BENCH_CHARTER: &str = "The bench's charter: commit the measure before the line drops — a prediction on the slate can be wrong honestly, and its surprise annotated in your own hand.";

/// Chamber 2's charter, spoken at the watch.
pub const WALL_CHARTER: &str = "The wall's charter: the stones are the same stones in any order — only the voids move. Count them byte by byte, and let every saved unit name the move that saved it.";

/// Floor 4's chamber hint ladders, four rungs each; the
/// fourth names the seam. Chambers 3–5 keep the gallery's
/// general ladder until their own steps land.
pub fn floor4_chamber_hints(index: usize) -> [&'static str; 4] {
    match index {
        0 => [
            "Commit both numbers for a stone before you touch the line: an uncommitted slate cannot be graded, and the bench will refuse the drop.",
            "Most stones are not the size of the thing they name. A borrowed word is two words; a heap is three; a pair pays for the larger partner's alignment.",
            "Where the slate misses, the annotation is the lesson: padding, a tag paid in full, a niche, or plain word-count. Pick the cause that the measured numbers actually show.",
            "The seam: a byte beside a word measures eight, not five — the gap is padding, and it is the commonest surprise on the bench. An Option over a reference measures one word, not two: the null address is the mark, and it is free.",
        ],
        1 => [
            "Seat the stones and watch the count, not the wall: every void is a byte the wall is charged for.",
            "A stone may only sit on its own alignment. A wide-aligned stone seated late drags a field of padding in front of it.",
            "Seat in falling alignment — widest law first — and the voids close. The bench order's wall reads 24; the least span reads 16.",
            "The seam: the beam seated second costs seven bytes of air before it. Nothing about the stones changed; only their order did — the void was never in the stones, it was in the sequence.",
        ],
        2 => [
            "Choose a cut for each crest, then let the line measure it — the price is a fact, not a guess.",
            "An Option of a plain value pays bytes for its tag. Ask what the value itself forbids.",
            "A count that can never be nothing, a word that can never be null: the absent-mark fits inside the forbidden value.",
            "The seam, named: the niche. Cut each crest where its mark rides free in a value that can never occur — the line's sizes will say you found it.",
        ],
        3 => [
            "Seat the four treaty stones in the order the book lists them; leave the free-cut arch on the tray.",
            "Drop the line and read the seats against the book, seat by seat.",
            "Three of the book's promises are true. The fourth was written by adding sizes and forgetting what alignment costs.",
            "The seam, named — failure class #14 in miniature: amend the book's final promise to the offset the line reads, neither the sum nor a guess.",
        ],

        4 => [
            "Fill a cavity, then tag it — the two cycles are separate, and the stone keeps only \
             the filling. The tag lives in the mason's head: yours.",
            "Read a stone as its tag claims before you judge it. A number read as a ratio is \
             next to nothing; a ratio read as a number is a count in the millions — the bits \
             are faithful either way.",
            "The blank stone is used whole or not at all: fill all four cells first. A Use on \
             a part-filled blank earns the referee's whistle, and the catch stays on the \
             record.",
            "The seam: the tag is not in the stone. Where the tag and the filling disagree, \
             the reading is nonsense and the chamber will not complete — set tag and filling \
             to the same kind on both stones.",
        ],
        _ => [
            "The Plumbmason's galleries reward the measured over the remembered: work the room's own instrument before you guess.",
            "Read the inscription again, slowly. The rule of the room is carved on its stone, and the demonstration at the desk shows it kept once.",
            "Every claim in this room can be proven on the spot. If you are asserting from memory, you are doing the thing the floor exists to cure.",
            "The seam is named in the floor's own failure class: somewhere a proxy is being measured instead of the quantity. Find what the instrument actually reads.",
        ],
    }
}

/// Chamber 3's crests, in bench order.
pub const CRESTS: [&str; 3] = [
    "the Count's crest — absent, or a count that is never nothing",
    "the Herald's crest — absent, or a borrowed word",
    "the Surveyor's crest — absent, or a wide count that is never nothing",
];

/// Chamber 3's candidate encodings per crest. Their sizes
/// are not written here: `crest_candidate_facts` measures
/// the real types at runtime.
pub const CREST_CANDIDATES: [[&str; 3]; 3] = [
    [
        "a tag stone seated beside the count",
        "the count as an Option of a plain count",
        "the count that is never nothing, itself",
    ],
    [
        "a flag stitched to the borrowed word",
        "the borrowed word as an Option",
        "word, length, and tag, packed by hand",
    ],
    [
        "a tag stone seated beside the wide count",
        "the wide count as an Option of a plain count",
        "the wide count that is never nothing, itself",
    ],
];

/// The bench's measure of one crest encoding: (size,
/// alignment) of the real type behind the candidate, taken
/// with `size_of` / `align_of` — a candidate's price is a
/// measured fact, never a table entry.
pub fn crest_candidate_facts(crest: usize, candidate: usize) -> (u32, u32) {
    match (crest, candidate) {
        (0, 0) => (
            std::mem::size_of::<(u8, u8)>() as u32,
            std::mem::align_of::<(u8, u8)>() as u32,
        ),
        (0, 1) => (
            std::mem::size_of::<Option<u8>>() as u32,
            std::mem::align_of::<Option<u8>>() as u32,
        ),
        (0, _) => (
            std::mem::size_of::<Option<std::num::NonZeroU8>>() as u32,
            std::mem::align_of::<Option<std::num::NonZeroU8>>() as u32,
        ),
        (1, 0) => (
            std::mem::size_of::<(bool, &str)>() as u32,
            std::mem::align_of::<(bool, &str)>() as u32,
        ),
        (1, 1) => (
            std::mem::size_of::<Option<&str>>() as u32,
            std::mem::align_of::<Option<&str>>() as u32,
        ),
        (1, _) => (
            std::mem::size_of::<(u64, u64, u8)>() as u32,
            std::mem::align_of::<(u64, u64, u8)>() as u32,
        ),
        (2, 0) => (
            std::mem::size_of::<(u8, u64)>() as u32,
            std::mem::align_of::<(u8, u64)>() as u32,
        ),
        (2, 1) => (
            std::mem::size_of::<Option<u64>>() as u32,
            std::mem::align_of::<Option<u64>>() as u32,
        ),
        _ => (
            std::mem::size_of::<Option<std::num::NonZeroU64>>() as u32,
            std::mem::align_of::<Option<std::num::NonZeroU64>>() as u32,
        ),
    }
}

/// Chamber 4's stones: the four treaty stones and the free
/// twin of the last, in tray order — (name, size, alignment).
/// The treaty's promised offsets live in `TREATY_PROMISES`;
/// the true offsets are computed by `puzzles::c_offsets`.
pub const TREATY_STONES: [(&str, u32, u32); 5] = [
    ("the threshold", 3, 1),
    ("the jamb", 5, 1),
    ("the lintel", 2, 2),
    ("the arch", 8, 8),
    ("the arch, free-cut", 8, 8),
];

/// The treaty book's promised offset for each seat, in seat
/// order. The first three promises are true; the fourth was
/// written by the naive sum and forgets the arch's padding —
/// failure class #14, the Last Mile, in miniature.
pub const TREATY_PROMISES: [u32; 4] = [0, 3, 8, 10];

/// Chamber 3's charter, spoken at the watch.
pub const CREST_CHARTER: &str = "The Seal Stones' charter: find the niche before you pay for the tag — a value that can never occur is room for a mark that costs nothing, and the bench proves the saving in real sizes.";

/// Chamber 4's charter, spoken at the watch.
pub const TREATY_CHARTER: &str = "The treaty wall's charter: a treaty stone promises every offset and is held to all of them; a free stone promises nothing and owes nothing. Prove each offset with the line — and where the book itself was written carelessly, amend the book, not the stone.";

/// Chamber 5's charter, spoken at the watch.
pub const HOLLOW_CHARTER: &str = "The Hollow Stones' charter: a cavity holds one filling of two \
    kinds, and the tag that says which kind is kept in a head, not in the stone. Read a stone as \
    its tag claims and the bits will answer — faithfully, and without mercy. Fill the blank whole \
    before it is used; the referee is Miri, and the referee counts.";

/// Chamber 5's hollow stones: (name, the whole-number filling,
/// the ratio filling). Each cavity is four bytes: the number is
/// a `u32`, the ratio an `f32`, and the stone holds whichever
/// was poured in — as bare bits.
pub const HOLLOW_STONES: [(&str, u32, f32); 2] = [
    ("the first hollow stone", 7, 2.5),
    ("the second hollow stone", 1_000_000, 0.75),
];

/// The dimension book of the master foundation: five
/// load-bearing entries, (name, the book's figure). The book's
/// arithmetic is flawless; three of its figures were taken
/// from a shadow and are false to the ground by exactly the
/// shadow's offset.
pub const DIMENSION_BOOK: [(&str, u32); 5] = [
    ("the Keystone", 8),
    ("the Long Beam", 22),
    ("the Footing", 24),
    ("the Arch Stone", 14),
    ("the Capstone", 14),
];

/// The shadow's offset: every false figure in the book stands
/// this many units from the stone it names — the noon shadow
/// of the survey's own rod, counted as stone.
pub const SHADOW_OFFSET: u32 = 6;

/// The plumb line's measure of one book entry: the size of
/// the real stone the entry names, taken with `size_of`. The
/// book is audited against the ground, never against itself.
pub fn dimension_book_true(index: usize) -> u32 {
    match index {
        0 => std::mem::size_of::<(u32, u32)>() as u32,
        1 => std::mem::size_of::<&[u8]>() as u32,
        2 => std::mem::size_of::<Vec<u16>>() as u32,
        3 => std::mem::size_of::<(u8, u32)>() as u32,
        _ => std::mem::size_of::<Option<&u64>>() as u32,
    }
}

/// The re-cut's foreign set, in tray order: (name, size,
/// alignment). Four stones belong to the foreign spec; the
/// fifth was cut from the crooked book — its measure matches
/// its honest twin, but its stencil repeats the book.
pub const RECUT_FOREIGN: [(&str, u32, u32); 5] = [
    ("the sill", 2, 2),
    ("the post", 4, 4),
    ("the beam", 8, 8),
    ("the cap", 1, 1),
    ("the beam, shadow-cut", 8, 8),
];

/// What each foreign stone's stencil claims for its size.
/// Four are honest; the shadow-cut beam's stencil claims the
/// book's figure, six more than the stone measures.
pub const RECUT_FOREIGN_STENCIL: [u32; 5] = [2, 4, 8, 1, 14];

/// The re-cut's free set: (name, size, alignment). Four
/// stones under a stated size budget — the budget being the
/// least span the four allow (`puzzles::wall_minimum`); the
/// tray order's own wall is larger.
pub const RECUT_FREE: [(&str, u32, u32); 4] = [
    ("the wedge", 1, 1),
    ("the pier", 8, 8),
    ("the slab", 4, 4),
    ("the rod", 2, 2),
];

/// Floor 4's FOE: the name of the Crooked Foundation.
pub const FOE4_NAME: &str = "The Crooked Foundation";

/// The Crooked Foundation's introduction, spoken at its door.
pub const FOE4_INTRO: &str = "At the galleries' heart the master foundation settles — a finger's \
    width this year, a thumb's the next, always toward the same side. The dimension book hangs \
    in a frame by the door, its survey celebrated, its arithmetic beautiful. Nobody has dropped \
    a line on the stones themselves in a generation.";

/// The Crooked Foundation's inscription: failure class #8,
/// named in the fiction, as FULL-DESIGN requires of every FOE.
pub const FOE4_INSCRIPTION: &str = "Here is kept the record of the eighth failure class: THE \
    ASTRONOMER'S BRASS EYE — a fine instrument, faithfully read, pointed at the wrong thing. \
    The old survey measured this foundation by its shadow at noon and entered the shadow's \
    numbers in the dimension book as the stones' own sizes. The book has been audited a \
    hundred times and found flawless, because an audit compares the book with itself. The \
    ground has never once been asked.";

/// The Crooked Foundation's demonstration, read at its desk.
pub const FOE4_DEMONSTRATION: &str = "The Plumbmason takes the dimension book down from its \
    frame and opens it on the desk: five load-bearing entries in a beautiful hand, every sum \
    correct. Then she hangs the line beside the first stone and says nothing at all. The line \
    says it for her.";

/// The Crooked Foundation's hint ladder.
pub fn floor4_foe_hints() -> [&'static str; 4] {
    [
        "Take the book down and drop the line on every stone it names. The re-survey is the \
         whole diagnosis: five stones, measured at the stone.",
        "Three of the five figures disagree with the line, and all three disagree by the same \
         amount. Exhibit each false entry beside its true value — a disagreement merely \
         noticed is not yet proof.",
        "The re-cut is laid in two sets: four stones to the foreign spec — mind the tray, one \
         stone on it was cut from the crooked book — and four free stones under the budget, \
         which is their least span.",
        "The seam, named — failure class #8, the Astronomer's Brass Eye: the survey measured a \
         shadow and called it a stone. Nothing in the book can acquit the book; only the line, \
         dropped on the ground itself, measures the quantity.",
    ]
}

/// The Crooked Foundation's codex stone.
pub const BRASS_EYE_STONE: &str = "The Brass Eye Stone";

/// The Brass Eye Stone's text in the codex.
pub const BRASS_EYE_STONE_TEXT: &str = "Distinct quantities get distinct types.";

/// The Crooked Foundation's loot.
pub const PLUMB_LINE: &str = "The Plumb Line";

/// The Plumb Line's text in the pack.
pub const PLUMB_LINE_TEXT: &str = "A weight, a cord, and the whole doctrine of the galleries: \
    what it says outranks what anyone remembers. Wielded on Floors 5, 9, 10, 13, and 14 — from \
    this fall onward, prose about sizes is no longer evidence anywhere in the descent.";

// ---------------------------------------------------------------------------
// Floor 5 — The Veilworks. Domain 5: Unsafe Rust & the aliasing model.
// Keeper: the Veilwalker. FOE: The Thing Behind the Veil.
// ---------------------------------------------------------------------------

/// The floor's name.
pub const FLOOR5_NAME: &str = "The Veilworks";
/// The domain this floor teaches.
pub const FLOOR5_DOMAIN: &str = "Unsafe Rust & the aliasing model";
/// The floor's keeper.
pub const FLOOR5_KEEPER: &str = "the Veilwalker";
/// The floor's introduction, spoken on entry.
pub const FLOOR5_INTRO: &str = "The Veilworks: gauze over the machinery, a sharp \
    tool on every bench, and a card beside each one stating what it assumes. \
    The Veilwalker watches hands, not faces — read the card before you lift \
    the tool, and the tools behave exactly as their cards say.";

/// The five Veilworks chambers.
pub const VEIL_CHAMBERS: [ChamberMeta; 5] = [
    ChamberMeta {
        name: "Raw pointers",
        room: "The Pointer Walks",
        inscription_title: "The Read Card",
        inscription: "The pointer points where it pleases and promises nothing. \
            The card beside each step is the whole of the law: read it before \
            the step, or the yard keeps what you drop.",
        demonstration: "The walker reads the first card aloud, steps where it \
            allows, and refuses the step whose card lies — pole grounded, foot \
            unmoved, the refusal spoken as plainly as the steps.",
        slot_wants: "every step walked or refused by its card",
        prove_label: "Walk the path",
    },
    ChamberMeta {
        name: "The aliasing model",
        room: "The Two Tenants",
        inscription_title: "One Writer, or Readers",
        inscription: "A room holds many readers or one writer, never both at \
            once. The promise is over the whole room, not the corner you \
            happened to touch — the referee judges by the room, not the corner.",
        demonstration: "Eight tenancies stand placarded; the Veilwalker opens \
            each one slowly, and the referee's stamp falls where the rule says \
            it falls — the writer's stamp never shares a room with a reader's.",
        slot_wants: "seven verdicts true, and every miss annotated",
        prove_label: "Call the referee",
    },
    ChamberMeta {
        name: "Unsafe contracts",
        room: "The Contract Cards",
        inscription_title: "Checkable, or Vapour",
        inscription: "A card that cannot be checked is a wish. 'Safe because \
            it is safe' has never held a roof up; write what a sceptic can \
            test, line by line, tool by tool.",
        demonstration: "Four tools stand in a rack, their cards beside them; \
            the Veilwalker sharpens each vague line to a checkable one with a \
            pencil, out loud, and lays the false ones face-down.",
        slot_wants: "twelve lines a sceptic can check",
        prove_label: "Present the cards",
    },
    ChamberMeta {
        name: "Safe abstractions",
        room: "The Split Beam",
        inscription_title: "The Seam Contract",
        inscription: "Two halves may be worked at once only because the seam's \
            contract says exactly where each half ends. The face of the beam \
            is safe; the parts are not; the card at the seam is the whole \
            reason why.",
        demonstration: "The Veilwalker saws the beam at a marked seam, sets \
            one crew to each half, and shows the referee the three rulings: \
            the seam that stands, the overlap refused, the drop order kept.",
        slot_wants: "a sound seam, walked by the referee",
        prove_label: "Seam the beam",
    },
    ChamberMeta {
        name: "The UB hunt",
        room: "The Fault Machine",
        inscription_title: "Fix the Card, Not the Symptom",
        inscription: "A fault at the surface is a contract broken underneath. \
            Lens every station, name the faults truly, and rewrite the card \
            that allowed them — a symptom patched is a fault postponed, not \
            a fault cured.",
        demonstration: "The machine turns over slowly under the lens; the \
            Veilwalker stations a card at each whine and ticks the honest \
            ones through, leaving the broken three ticking louder.",
        slot_wants: "three faults found and their cards rewritten",
        prove_label: "Present the hunt",
    },
];

/// Each chamber's spoken introduction.
pub const FLOOR5_CHAMBER_INTROS: [&str; 5] = [
    "The Pointer Walks. Six steps across the yard, a card beside each: the \
     walker steps where the card allows and refuses where it lies — the \
     pole itself is deaf, and trusts no one.",
    "The Two Tenants. Eight tenancies stand placarded; call each one's \
     soundness before the referee opens it, and annotate every miss with \
     the rule that decided it.",
    "The Contract Cards. Four tools, three lines each: cycle every line to \
     the phrasing a sceptic can check, and leave no vapour standing.",
    "The Split Beam. One beam, two halves, a seam to choose: seam it \
     soundly, refuse the overlap, work both halves, and drop them in the \
     order the card prescribes.",
    "The UB hunt. Six stations, three of them faulted underneath: lens each \
     one before you judge it, name the faults truly, and rewrite the cards \
     that allowed them.",
];

/// The Examine texts for Floor 5's five chambers.
pub const F5_EXAMINE_TEXTS: [&str; 5] = [
    "A marked yard, a long pole, and six stepping stones. Beside every \
     stone a card states what the step assumes; the cards are the only \
     promises in the room, and one of them is lying.",
    "Eight small rooms behind gauze, each placarded with a tenancy: who \
     holds it, who reads it, who writes it. The referee's stamp hangs at \
     the door, patient and exact.",
    "A rack of four sharp tools, each with a three-line card. Some lines \
     can be checked by a sceptic with a ruler; others are vapour wearing \
     a card's shape.",
    "A single beam on trestles, chalked with possible seams. A saw, two \
     crew benches, and the referee's chair at the end, where the three \
     rulings are read out in order.",
    "A long machine under gauze, six stations ticking. Three tick wrong — \
     not at the surface, where the eye rests, but underneath, where the \
     contracts were written carelessly.",
];

/// Chamber 1's demonstration, spoken at the watch.
pub const WALK_CHARTER: &str = "The Veilwalker reads the first card aloud — \
    'the stone beyond is whole; step freely' — and steps. At the fourth \
    stone the card claims a footing the yard does not have; the walker \
    grounds the pole, refuses the step out loud, and walks no further \
    until the refusal stands recorded.";

/// Chamber 2's demonstration, spoken at the watch.
pub const TENANCY_CHARTER: &str = "The Veilwalker opens the tenancies one at \
    a time: readers together, stamping SOUND; a reader at her desk while a \
    writer works the same room, stamped UNSOUND — the rule never asks who \
    touched what last, only who holds the room.";

/// Chamber 3's demonstration, spoken at the watch.
pub const CARD_CHARTER: &str = "The Veilwalker takes the pencil to the first \
    card: 'handle with care' becomes 'the caller's slice is non-null, \
    aligned, and valid for reads of the stated length' — a line a sceptic \
    can test. The vapour lines go face-down on the bench.";

/// Chamber 4's demonstration, spoken at the watch.
pub const BEAM_CHARTER: &str = "The Veilwalker chalks the seam at the beam's \
    middle, saws, and sets a crew to each half. When a hand reaches across \
    the seam for the other crew's half, the referee refuses it — aloud, by \
    the card — and the halves are worked apart, then dropped left first.";

/// Chamber 5's demonstration, spoken at the watch.
pub const HUNT_CHARTER: &str = "The Veilwalker runs the machine slowly under \
    the lens, station by station, ticking the honest cards through. Three \
    stations tick wrong underneath; their cards are lifted out and laid on \
    the bench, waiting to be rewritten at the root.";

/// The hint ladders for Floor 5's chambers. At scaffolding only the
/// general ladder stands; each chamber's own ladder lands with it.
pub fn floor5_general_hints() -> [&'static str; 4] {
    [
        "The Veilworks reward the read card over the lifted tool: before any \
         work, read what the mechanism in front of you assumes — the press \
         that refuses names the assumption you skipped.",
        "Work the chamber's own order. The demonstration at the watch is the \
         mechanism shown honestly, by the keeper's own hands; imitate it \
         exactly and the bench has no surprises left.",
        "Every refusal on this floor names the missing piece of the contract \
         out loud. Collect the refusals; they are the specification, told \
         from the other side.",
        "The seam of this floor, in miniature: a borrow is a contract over \
         the whole value, not the part you touched — look for the place \
         where a card, a tenancy, or a seam quietly narrows it.",
    ]
}

/// Floor 5 Chamber 1's six steps: each step's card (the
/// precondition it states), the yard's account of the stone,
/// and whether the stone is cracked. Exactly one stone is
/// cracked — the fourth — and its card, like all the cards,
/// is honest: it is the footing that fails the card.
pub const STEP_CARDS: [(&str, &str, bool); 6] = [
    (
        "Step only where the stone is whole and pinned at both ends.",
        "whole, pinned at both ends",
        false,
    ),
    (
        "The footing must ring true under the pole's tap, never hollow.",
        "rings true under the tap",
        false,
    ),
    (
        "No step on a stone that shifts when the pole leans on it.",
        "steady under the lean",
        false,
    ),
    (
        "Step only where the stone is whole — a cracked stone keeps no one.",
        "CRACKED across the middle",
        true,
    ),
    (
        "The stone must sit flush with its neighbours, lip to lip.",
        "flush with its neighbours",
        false,
    ),
    (
        "The last stone must be dry; a wet sheen slides underfoot.",
        "dry, dusted with chalk",
        false,
    ),
];

/// The aliasing rules the Two Tenants are judged by — the
/// annotation names one of these as the rule that decided a
/// miss.
pub const TENANCY_RULES: [&str; 5] = [
    "Any number of readers may hold a value at once.",
    "A writer holds the value alone: no reader, no second writer.",
    "While a writer holds it, no reader may be inside.",
    "Two writers never hold the same value at once.",
    "A hold ends at its last use; after that, the room is free.",
];

/// Floor 5 Chamber 2's eight tenancies: the placard's text,
/// whether the tenancy is sound, and the rule that decides
/// it (the annotation a miss must name).
pub const TENANCIES: [(&str, bool, usize); 8] = [
    ("Three readers share the ledger, reading in turn.", true, 0),
    (
        "A writer repaints the sign while two readers read it.",
        false,
        2,
    ),
    ("One writer works the room alone, the door barred.", true, 1),
    ("Two writers take opposite ends of the same beam.", false, 3),
    (
        "A reader sets the book down; then a writer enters.",
        true,
        4,
    ),
    (
        "A reader's finger holds the page while a writer rewrites it.",
        false,
        2,
    ),
    ("Four readers pass one map from hand to hand.", true, 0),
    ("Two writers share one page, both pens moving.", false, 3),
];

/// Floor 5 Chamber 3's four tools: each card carries three
/// lines, and each line cycles three phrasings of which
/// exactly one is checkable — a condition a sceptic can test
/// before the call. The third element of each row is the
/// checkable phrasing's index, per line (the key of record:
/// T0 [1,0,1], T1 [0,2,1], T2 [2,0,1], T3 [0,1,2]).
/// One tool's card: its name, its three lines (three
/// phrasings apiece), and the checkable phrasing per line.
pub type CardTool = (&'static str, [[&'static str; 3]; 3], [u8; 3]);

pub const CARD_TOOLS: [CardTool; 4] = [
    (
        "The Length-Reader",
        [
            [
                "It only ever touches what it is given.",
                "The caller's pointer is non-null and aligned for the words it reads.",
                "The pointer is as sound as the hand that passes it.",
            ],
            [
                "The stated length does not exceed the words actually behind the pointer.",
                "Lengths are handled the way lengths always are.",
                "It reads no more than it should.",
            ],
            [
                "Nothing it does can surprise a careful caller.",
                "It writes nothing; every word it returns comes from the stated range.",
                "It is safe because it is a reader, and readers do no harm.",
            ],
        ],
        [1, 0, 1],
    ),
    (
        "The Splitter",
        [
            [
                "The seam index lies strictly inside the buffer, so both halves are non-empty.",
                "The seam is chosen the way seams are chosen.",
                "Any seam a caller names is a seam the tool can use.",
            ],
            [
                "The halves never trouble each other.",
                "Two hands may hold it, as hands may hold anything.",
                "The two halves share no word: the ranges the tool returns are disjoint.",
            ],
            [
                "A bad seam fails politely, in the usual way.",
                "A seam at either end, or past it, is refused with the seam named.",
                "It cannot fail at a seam it was told to use.",
            ],
        ],
        [0, 2, 1],
    ),
    (
        "The Keeper",
        [
            [
                "What is lent is looked after, as lent things are.",
                "The Keeper's care is total and well known.",
                "While a section is lent out, the store does not grow, shrink, or move it.",
            ],
            [
                "Each lent section is returned exactly once; a second return is refused.",
                "Sections come back when they come back.",
                "Nothing lent is ever truly lost.",
            ],
            [
                "Any hand may look in while a section is out.",
                "No hand reads a lent section through the store while the loan stands.",
                "The store knows its sections wherever they are.",
            ],
        ],
        [2, 0, 1],
    ),
    (
        "The Lens",
        [
            [
                "The station it reads is whole and initialised before the reading.",
                "It reads what is there, whatever 'there' means.",
                "A station under the Lens is a station understood.",
            ],
            [
                "Reading changes nothing worth naming.",
                "It writes nothing: the station's words are the same after as before.",
                "It is safe because looking cannot hurt.",
            ],
            [
                "It sees as far as it needs to.",
                "The whole machine is its proper field.",
                "It reads only the one station named on its card, and no other.",
            ],
        ],
        [0, 1, 2],
    ),
];

/// The four tools' short names, for the bench's buttons.
pub const CARD_TOOL_SHORT: [&str; 4] = ["READER", "SPLITTER", "KEEPER", "LENS"];
/// Chamber 1's hint ladder.
pub fn floor5_walk_hints() -> [&'static str; 4] {
    [
        "Read every card before its step: the walker will not move on an \
         unread card, and the yard's account of each stone appears only \
         with its card in hand.",
        "Walk the sound steps in order. The card states the footing the \
         step needs; the yard shows the footing it has — compare them \
         before you commit your weight.",
        "One stone in six fails its card: its card is honest, and the \
         footing is cracked. That step is not walked; it is refused, by \
         name, and stepped around.",
        "The failing step is the fourth stone: its card demands a whole \
         stone and the yard shows a crack across the middle. Read its \
         card, press Refuse the step, and walk on — the refusal is the \
         step.",
    ]
}

/// Chamber 2's hint ladder.
pub fn floor5_tenancy_hints() -> [&'static str; 4] {
    [
        "Call every placard before the referee is called in: SOUND if the \
         tenancy honours the room's law, UNSOUND if it breaks it.",
        "The law has two halves: readers may share freely, and a writer \
         holds the room alone. Every placard turns on one of the two.",
        "After the referee stamps the calls, each miss must be annotated \
         with the rule that decided it — the stamp alone is not the \
         proof; the named rule is.",
        "The misses turn on two rules: a writer among readers breaks \
         'while a writer holds it, no reader may be inside'; two writers \
         together break 'two writers never hold the same value at once'.",
    ]
}

/// Chamber 3's hint ladder.
pub fn floor5_card_hints() -> [&'static str; 4] {
    [
        "Cycle every line: each of the twelve wears three phrasings, and \
         exactly one of them can be checked by a sceptic with a ruler \
         and a question.",
        "A checkable line states a condition the caller can test before \
         the call — a pointer aligned, a length held against a buffer, \
         a refusal that names what it refuses.",
        "Vapour praises the tool and tests nothing: 'safe because it is \
         safe' is a circle, and 'in the usual way' is a shrug wearing a \
         card's shape.",
        "The seam, named on one tool of the four: the Keeper's loan line \
         stands checkable only as 'while a section is lent out, the \
         store does not grow, shrink, or move it'. Judge every line by \
         that standard — could a sceptic test it before the call?",
    ]
}

/// Chamber 4's hint ladder.
pub fn floor5_beam_hints() -> [&'static str; 4] {
    [
        "Choose the seam before the saw: the stepper walks the split \
         from end to end, and only a seam strictly inside the beam \
         leaves two halves a crew can work.",
        "A seam at either end leaves nothing standing on one side, and \
         the face refuses to seam air — the refusal names the empty \
         half.",
        "The referee's walk has three rulings, in order: an access past \
         a half's end, refused by the face; both halves worked at \
         once; and the halves dropped in the card's order.",
        "The seam, named: chalk the seam at the beam's middle — three \
         words to the left, three to the right — reach past a half, \
         work both halves together, and drop the left half before the \
         right.",
    ]
}
/// One UB Hunt station: the work it claims to do, the
/// seeded fault it carries (an index into `FAULT_KINDS`),
/// and — for a faulted station — the three contracts its
/// card may show, exactly one of them true.
pub struct Station {
    /// The station's name.
    pub name: &'static str,
    /// The work it claims to do, as the room placards it.
    pub duty: &'static str,
    /// The seeded fault's index into `FAULT_KINDS`, if any.
    pub fault: Option<usize>,
    /// The three contracts offered for a faulted station;
    /// blank for sound ones.
    pub contracts: [&'static str; 3],
    /// The true contract's index (faulted stations only).
    pub true_contract: u8,
}

/// The three seeded fault shapes, at miniature size: the
/// tenth class's roots, one per faulted station.
pub const FAULT_KINDS: [&str; 3] = [
    "two writers fused into one tenancy",
    "a room read after it was emptied",
    "a read of an unfilled blank",
];

/// The UB Hunt's six stations (FULL-DESIGN §19 / Floor 5
/// brief, Chamber 5). Three are sound; three carry one
/// seeded fault each — the Twin Pens (two writers fused),
/// the Second Reading (read after the emptying), the Blank
/// Ledger (read of an unfilled blank).
pub const STATIONS: [Station; 6] = [
    Station {
        name: "The Night Store",
        duty: "It keeps lent sections ready overnight and returns each \
               at dawn, to its keeper only.",
        fault: None,
        contracts: ["", "", ""],
        true_contract: 0,
    },
    Station {
        name: "The Twin Pens",
        duty: "Two pens keep one room's diary together, trading it \
               back and forth through the day.",
        fault: Some(0),
        contracts: [
            "Two hands may write one room whenever their turns are \
             called in order.",
            "Writers sharing a room must simply be careful.",
            "One writer at a time holds the room; a second writer \
             waits until the first's hold is spent.",
        ],
        true_contract: 2,
    },
    Station {
        name: "The Second Reading",
        duty: "When a room is emptied at dusk, the reader reads it \
               once more, to be sure the emptying took.",
        fault: Some(1),
        contracts: [
            "What has been emptied is gone: no reading of it stands \
             after the emptying.",
            "An emptied room may be read once more, to be sure it is \
             empty.",
            "A careful reader may read emptied rooms at need.",
        ],
        true_contract: 0,
    },
    Station {
        name: "The Fresh Press",
        duty: "It prints each page once, in order, and hands it to \
               no other press.",
        fault: None,
        contracts: ["", "", ""],
        true_contract: 0,
    },
    Station {
        name: "The Blank Ledger",
        duty: "A new page is read at intake to learn its markings \
               before anything is written on it.",
        fault: Some(2),
        contracts: [
            "A blank page reads as whatever the reader expects.",
            "An unfilled page is not read: the face refuses the \
             reading until a first write has landed.",
            "Blank pages are read as zero, by long custom.",
        ],
        true_contract: 1,
    },
    Station {
        name: "The Quiet Shelf",
        duty: "One writer or many readers, by the old law: never \
               both at once.",
        fault: None,
        contracts: ["", "", ""],
        true_contract: 0,
    },
];

/// Chamber 5's hint ladder: the hunt's method — lens first,
/// verdict second, the card to close — and the seam the
/// fourth rung names.
pub fn floor5_hunt_hints() -> [&'static str; 4] {
    [
        "Lens first, verdict second: a verdict entered blind is a \
         guess wearing a badge.",
        "Three of the six stations hunt. Their faults are the old \
         shapes at miniature size: hands fused, a read after the \
         emptying, a read of nothing.",
        "A faulted station is not convicted by its verdict alone: \
         its card must also be set to the one true contract of the \
         three it offers.",
        "The seam: undefined behaviour is not a mystery — it is a \
         contract broken at exactly one place, here at miniature \
         size as the FOE works it at full size: find where the \
         promise and the act part, and the station names itself.",
    ]
}

/// The FOE's name: the tenth failure class with a face.
pub const FOE5_NAME: &str = "The Thing Behind the Veil";

/// The FOE's door speaks this on entry.
pub const FOE5_INTRO: &str = "Behind the last gauze the workshops share one long beam — \
    and something has been borrowing it out of both ends at once.";

/// The FOE chamber's inscription.
pub const FOE5_INSCRIPTION: &str = "Here is kept the record of the tenth failure class: THE \
    MOVING FLOOR — a store that grows, shrinks, or slides while a \
    reference into it stands in another hand, so the reference comes \
    to name a different room, or none. The Thing Behind the Veil is \
    that class with a face: a beam borrowed at both ends, grown in \
    the middle, and read after the growth by a lens that trusted the \
    old address.";

/// The FOE's demonstration.
pub const FOE5_DEMONSTRATION: &str = "The Veilwalker draws the gauze aside. The beam hangs \
    in its frame, lent at both ends, chalked with its six cells — \
    and while you watch, a seventh cell buds in the middle and every \
    chalked address slides one place down.";

/// The codex stone the Thing's fall records.
pub const ICE_STONE: &str = "The Ice Stone";

/// The pack loot the Thing's fall records.
pub const VEILWALKERS_BUFFER: &str = "The Veilwalker's Buffer";

/// The trace's three exhibits, in the referee's order: a
/// (name, spoken account) pair each. The names are what an
/// out-of-order refusal cites as the missing prior sight.
pub const FOE5_EXHIBITS: [(&str, &str); 3] = [
    (
        "a live shared loan",
        "Exhibit, first sight: a shared loan stands live on the \
         beam — the Splitter's end borrowed by the night watch, its \
         address chalked and trusted.",
    ),
    (
        "a growth while the loan was out",
        "Exhibit, second sight: with the loan still out, the beam \
         grows — a cell buds mid-beam and every address past the bud \
         slides. The loan's chalk now names the wrong cell.",
    ),
    (
        "the lens's verdict",
        "Exhibit, third sight: the lens reads the chalked address \
         and finds a stranger's cell wearing the borrowed name. \
         Verdict: the floor moved under a standing reference — the \
         tenth class, in the flesh.",
    ),
];

/// The referee's five standing-gate cases: a (name, walked
/// outcome) pair each, judged for real against the beam's
/// model in `puzzles::beam_case_holds`.
pub const FOE5_CASES: [(&str, &str); 5] = [
    (
        "Growth under a loan",
        "Case one walked: with a loan out, the beam is asked to \
         grow — and refuses at the face. The floor holds.",
    ),
    (
        "Access past the end",
        "Case two walked: a hand reaches one past the last cell — \
         refused, plainly. No undefined step is taken.",
    ),
    (
        "The drop order",
        "Case three walked: the beam is dropped cell by cell — each \
         exactly once, none twice, none leaked.",
    ),
    (
        "The empty element",
        "Case four walked: an unfilled cell refuses access — and \
         the beam still grows beside it. Emptiness is a state, not \
         a hole.",
    ),
    (
        "Overflow at capacity",
        "Case five walked: at its stated capacity the beam refuses \
         one more cell — refused, not wrapped, not spilled.",
    ),
];

/// The beam's contract card, as the repair rewrites it:
/// three lines, three phrasings each, exactly one checkable
/// truth per line — a loan freezes the whole beam; growth is
/// refused while any loan is out; the face exposes no
/// operation that can move the beam under a reference.
pub const FOE5_CARD: [[&str; 3]; 3] = [
    [
        "A loan leans on the beam, and the beam leans back.",
        "A loan freezes the whole beam: while it stands, the beam \
         does not move, grow, or shrink.",
        "A loan covers the part it touches; the rest of the beam is \
         free.",
    ],
    [
        "Growth waits politely until every loan comes home.",
        "It grows fastest when no hand is watching.",
        "Growth is refused while any loan is out — refused at the \
         face, by name.",
    ],
    [
        "A reference, once given, is the caller's own lookout.",
        "No operation the face exposes can move the beam under a \
         standing reference.",
        "The face shows every operation, and the careful may audit \
         them.",
    ],
];

/// The checkable phrasing per line of the beam's card.
pub const FOE5_CARD_KEY: [u8; 3] = [1, 2, 1];

/// The FOE's hint ladder: exhibits in order, the referee's
/// walk, the card rewritten — and the seam the fourth rung
/// names.
pub fn floor5_foe_hints() -> [&'static str; 4] {
    [
        "Three sights, in the referee's order: the loan, the \
         growth, the verdict. The veil opens one layer at a time.",
        "Then the referee's five cases, walked one by one against \
         the beam — growth, reach, drops, the empty cell, the brim.",
        "The strike also wants the beam's card rewritten: three \
         lines, each with exactly one checkable truth, like the rack \
         in Chamber 3.",
        "The seam: a shared loan freezes the whole beam because \
         addresses are the beam's word — growth under a loan breaks \
         the word. The card's truths are the freeze, the refusal, \
         and no hidden mover.",
    ]
}
/// Floor 5's per-chamber hint ladders, dispatched by chamber.
pub fn floor5_chamber_hints(chamber: usize) -> [&'static str; 4] {
    match chamber {
        0 => floor5_walk_hints(),
        1 => floor5_tenancy_hints(),
        2 => floor5_card_hints(),
        3 => floor5_beam_hints(),
        _ => floor5_general_hints(),
    }
}
