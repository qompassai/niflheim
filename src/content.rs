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
pub const FOE2_INSCRIPTION: &str = "The desk admits by resemblance. A grey coat is not a fire sigil, yet the slip is stamped all the same. Audit every admission against its charter: exhibit each working beast's sigils against the bound of the post it serves. What can be found by audit need never be guessed.";
/// The FOE demonstrated: the mechanism at its worst.
pub const FOE2_DEMONSTRATION: &str = "The mist-hound stands its fire-watch in a grey coat the colour of the ash-hounds. Set before a lit brazier — the valley's one known landmark — it lowers its nose, breathes deep, and reports fog. It is not lying. It was never sigilled for the post.";

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
        _ => [
            "The Sigilwright has not carved this ladder yet.",
            "The Sigilwright has not carved this ladder yet.",
            "The Sigilwright has not carved this ladder yet.",
            "The Sigilwright has not carved this ladder yet.",
        ],
    }
}
