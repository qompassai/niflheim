//! The bounded text machinery the game is built on: a string
//! interner (each distinct string stored once, referenced by a
//! small symbol), a zero-copy line index, the lending-plan census,
//! player settings, and the typed errors every layer shares.
//!
//! Everything here is deterministic and bounded: collections carry
//! named caps, external text is validated before it is indexed, and
//! no function panics on data that came from a file, a plan, or a
//! player. This module is also the game's first lesson made real —
//! the Interner's Hoard the player builds at Floor 1's FOE is this
//! machinery, rebuilt by the player's own hand.

use std::collections::HashMap;
use std::sync::Arc;

/// Save-file magic, written as the first field of every save.
pub const SAVE_MAGIC: &str = "NIFLHEIM-SAVE";
/// The save format version this build reads and writes.
pub const SAVE_FORMAT_VERSION: u32 = 8;
/// Most lines a lending plan may carry.
pub const MAX_LEDGER_LINES: usize = 96;
/// Most distinct strings the interner will hold.
pub const MAX_DISTINCT_STRINGS: usize = 64;
/// Most holders one plan entry set may name for a single item.
pub const MAX_HOLDERS_PER_ITEM: usize = 8;
/// Most lines kept in the expedition log.
pub const MAX_LOG_LINES: usize = 256;
/// The ambient seed a fresh expedition starts with.
pub const AMBIENT_SEED_DEFAULT: u64 = 0x4e49_464c_4845_494d;

/// The name of the legitimate lesser chart the vault lends in place
/// of a struck claim.
pub const LENT_COPY_NAME: &str = "Lent Copy of a Lesser Chart";

/// A handle to an interned string. Small, copyable, and meaningless
/// without the interner that minted it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Symbol(u32);

impl Symbol {
    /// The symbol's index inside its interner.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Errors from the core text machinery. Expected failures are
/// values, never panics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreError {
    /// The interner is at its bound.
    TooManyStrings,
    /// A plan carried more lines than the bound allows.
    LedgerTooLong,
    /// A plan line did not parse.
    LedgerParse {
        /// 1-based line number in the plan text.
        line: usize,
    },
    /// A symbol that this interner never minted.
    UnknownSymbol,
}

impl CoreError {
    /// A one-line, player-readable account of the failure.
    pub fn message(&self) -> String {
        match self {
            CoreError::TooManyStrings => {
                format!("the ledger holds at most {MAX_DISTINCT_STRINGS} distinct names")
            }
            CoreError::LedgerTooLong => {
                format!("a plan holds at most {MAX_LEDGER_LINES} lines")
            }
            CoreError::LedgerParse { line } => format!("plan line {line} does not parse"),
            CoreError::UnknownSymbol => "a name this ledger never minted".to_string(),
        }
    }
}

/// The string interner: stores each distinct string exactly once
/// and hands out symbols. Bounded at [`MAX_DISTINCT_STRINGS`].
#[derive(Clone, Debug, Default)]
pub struct Interner {
    entries: Vec<Arc<str>>,
    by_text: HashMap<Arc<str>, Symbol>,
}

impl Interner {
    /// An empty interner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Intern `text`, returning the existing symbol when the string
    /// is already held. Fails at the bound rather than growing.
    pub fn intern(&mut self, text: &str) -> Result<Symbol, CoreError> {
        if let Some(symbol) = self.by_text.get(text) {
            return Ok(*symbol);
        }
        if self.entries.len() >= MAX_DISTINCT_STRINGS {
            return Err(CoreError::TooManyStrings);
        }
        let owned: Arc<str> = Arc::from(text);
        let symbol = Symbol(u32::try_from(self.entries.len()).unwrap_or(u32::MAX));
        self.entries.push(Arc::clone(&owned));
        self.by_text.insert(owned, symbol);
        Ok(symbol)
    }

    /// The text a symbol names.
    pub fn text(&self, symbol: Symbol) -> Result<&str, CoreError> {
        self.entries
            .get(symbol.index())
            .map(|entry| &**entry)
            .ok_or(CoreError::UnknownSymbol)
    }

    /// The symbol for a string, if it was ever interned.
    pub fn lookup(&self, text: &str) -> Option<Symbol> {
        self.by_text.get(text).copied()
    }

    /// How many distinct strings are held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is held.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A zero-copy index of the lines of a borrowed source text: each
/// line is a slice of the source, never a copy. Line semantics match
/// `str::lines` (a trailing newline does not mint a phantom line).
pub struct LineIndex<'a> {
    lines: Vec<&'a str>,
}

impl<'a> LineIndex<'a> {
    /// Index `source` without copying any of it.
    pub fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines().collect(),
        }
    }

    /// The line at `index` (0-based), if it exists.
    pub fn get(&self, index: usize) -> Option<&'a str> {
        self.lines.get(index).copied()
    }

    /// How many lines the source has.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Whether the source has no lines.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// One line of a lending plan: an item, its holder, and whether the
/// item may be copied freely.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanEntry {
    /// The item's name.
    pub item: String,
    /// Who holds it under this plan.
    pub holder: String,
    /// True when the item is copiable (tally sticks, seals, blanks).
    pub copiable: bool,
}

/// Parse plan text of the form `item | holder | kind`, where kind is
/// `unique` or `copiable`. Blank lines and `#` comments are skipped.
/// Bounded at [`MAX_LEDGER_LINES`] entry lines.
pub fn parse_plan(text: &str) -> Result<Vec<PlanEntry>, CoreError> {
    let mut entries = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if entries.len() >= MAX_LEDGER_LINES {
            return Err(CoreError::LedgerTooLong);
        }
        let mut parts = line.split('|');
        let item = parts.next().map(str::trim).unwrap_or("");
        let holder = parts.next().map(str::trim).unwrap_or("");
        let kind = parts.next().map(str::trim).unwrap_or("");
        if item.is_empty() || holder.is_empty() || parts.next().is_some() {
            return Err(CoreError::LedgerParse { line: index + 1 });
        }
        let copiable = match kind {
            "copiable" => true,
            "unique" => false,
            _ => return Err(CoreError::LedgerParse { line: index + 1 }),
        };
        entries.push(PlanEntry {
            item: item.to_string(),
            holder: holder.to_string(),
            copiable,
        });
    }
    Ok(entries)
}

/// One row of a plan census: an item, its kind, and everyone the
/// plan gives it to, in first-appearance order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CensusRow {
    /// The item's name.
    pub item: String,
    /// Whether the item may be copied freely.
    pub copiable: bool,
    /// Distinct holders, in the order the plan first names them.
    pub holders: Vec<String>,
}

impl CensusRow {
    /// The vault rule broken in one line: an uncopiable item in two
    /// or more columns.
    pub fn double_promised(&self) -> bool {
        !self.copiable && self.holders.len() > 1
    }
}

/// A census of a plan: one row per distinct item, in
/// first-appearance order. Deterministic by construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Census {
    /// The rows, in plan order.
    pub rows: Vec<CensusRow>,
}

/// Take the census of a parsed plan.
pub fn census_of(entries: &[PlanEntry]) -> Census {
    let mut rows: Vec<CensusRow> = Vec::new();
    for entry in entries {
        match rows.iter_mut().find(|row| row.item == entry.item) {
            Some(row) => {
                if !row.holders.contains(&entry.holder) && row.holders.len() < MAX_HOLDERS_PER_ITEM
                {
                    row.holders.push(entry.holder.clone());
                }
            }
            None => rows.push(CensusRow {
                item: entry.item.clone(),
                copiable: entry.copiable,
                holders: vec![entry.holder.clone()],
            }),
        }
    }
    Census { rows }
}

/// The census after a repair strike: the struck entry is replaced
/// by a loan of the vault's lesser chart to the struck holder, so
/// the plan balances — every uncopiable item in exactly one column.
pub fn census_after_repair(entries: &[PlanEntry], struck: usize) -> Census {
    let mut repaired: Vec<PlanEntry> = entries.to_vec();
    if let Some(entry) = repaired.get_mut(struck) {
        entry.item = LENT_COPY_NAME.to_string();
        entry.copiable = true;
    }
    census_of(&repaired)
}

/// Player-facing text size. The design's legibility floor is the
/// Standard factor; larger scales only grow from there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextScale {
    /// 1.0× — the legibility floor.
    Standard,
    /// 1.25×.
    Large,
    /// 1.5×.
    Largest,
}

impl TextScale {
    /// The multiplier applied to body text sizes.
    pub fn factor(self) -> f32 {
        match self {
            TextScale::Standard => 1.0,
            TextScale::Large => 1.25,
            TextScale::Largest => 1.5,
        }
    }

    /// The next scale in the cycle.
    pub fn next(self) -> Self {
        match self {
            TextScale::Standard => TextScale::Large,
            TextScale::Large => TextScale::Largest,
            TextScale::Largest => TextScale::Standard,
        }
    }

    /// The scale's settings label.
    pub fn label(self) -> &'static str {
        match self {
            TextScale::Standard => "Standard",
            TextScale::Large => "Large",
            TextScale::Largest => "Largest",
        }
    }

    /// The scale's save-file name.
    pub fn as_save_str(self) -> &'static str {
        match self {
            TextScale::Standard => "standard",
            TextScale::Large => "large",
            TextScale::Largest => "largest",
        }
    }

    /// Read a scale back from its save-file name.
    pub fn from_save_str(text: &str) -> Option<Self> {
        match text {
            "standard" => Some(TextScale::Standard),
            "large" => Some(TextScale::Large),
            "largest" => Some(TextScale::Largest),
            _ => None,
        }
    }
}

/// The player's settings. They travel inside the save file, so a
/// save's records are the same on every platform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    /// Body text scale.
    pub text_scale: TextScale,
    /// High-contrast palette.
    pub high_contrast: bool,
    /// Reduced motion: demonstrations step, transitions are wipes.
    pub reduced_motion: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            text_scale: TextScale::Standard,
            high_contrast: false,
            reduced_motion: false,
        }
    }
}

/// Ways a save file can refuse to load. Each carries the wording the
/// game shows; a refused load never mutates live state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SaveError {
    /// The text is not a save at all (wrong or missing magic).
    NotASave,
    /// The save was written by a newer format than this build reads.
    FutureVersion {
        /// The version found in the file.
        found: u32,
    },
    /// The save's version predates the supported floor.
    UnsupportedVersion {
        /// The version found in the file.
        found: u32,
    },
    /// The save parses but breaks an invariant of the design.
    Corrupt {
        /// What was wrong, in plain words.
        reason: String,
    },
}

impl SaveError {
    /// A one-line, player-readable account of the refusal.
    pub fn message(&self) -> String {
        match self {
            SaveError::NotASave => "that is not a Niflheim save".to_string(),
            SaveError::FutureVersion { found } => {
                format!("the save is format {found}, newer than this build reads")
            }
            SaveError::UnsupportedVersion { found } => {
                format!("the save is format {found}, older than this build reads")
            }
            SaveError::Corrupt { reason } => format!("the save is corrupt: {reason}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAN: &str = "\
# The issue desk plan
Heartstone Chart | Expedition A | unique
Saw | Expedition A | unique
Tally Sticks | Expedition A | copiable
Heartstone Chart | Expedition B | unique
Tally Sticks | Expedition B | copiable
Rope Ladder | Expedition C | unique
";

    /// Validation: repeated interns dedup to one symbol.
    #[test]
    fn interner_dedups_repeated_inserts() {
        let mut interner = Interner::new();
        let first = interner.intern("Heartstone Chart").expect("intern");
        let second = interner.intern("Heartstone Chart").expect("intern");
        assert_eq!(first, second);
        assert_eq!(interner.len(), 1);
        assert_eq!(interner.text(first).expect("text"), "Heartstone Chart");
    }

    /// Validation: the line index agrees with a naive reading, and
    /// its slices borrow the source (zero copy).
    #[test]
    fn line_index_matches_naive_lines() {
        let source = "one\ntwo\nthree";
        let index = LineIndex::new(source);
        assert_eq!(index.len(), 3);
        assert_eq!(index.get(0), Some("one"));
        assert_eq!(index.get(2), Some("three"));
        assert_eq!(index.get(3), None);
        let first = index.get(0).expect("line");
        assert!(std::ptr::eq(first.as_ptr(), source.as_ptr()));
    }

    /// Validation: the census finds the double promise and only it.
    #[test]
    fn census_finds_the_double_promise() {
        let entries = parse_plan(PLAN).expect("plan parses");
        let census = census_of(&entries);
        let doubles: Vec<&str> = census
            .rows
            .iter()
            .filter(|row| row.double_promised())
            .map(|row| row.item.as_str())
            .collect();
        assert_eq!(doubles, ["Heartstone Chart"]);
    }

    /// Validation: after the strike the census balances.
    #[test]
    fn census_balances_after_repair() {
        let entries = parse_plan(PLAN).expect("plan parses");
        let struck = entries
            .iter()
            .rposition(|e| e.item == "Heartstone Chart")
            .expect("second chart entry");
        let census = census_after_repair(&entries, struck);
        assert!(census.rows.iter().all(|row| !row.double_promised()));
    }

    /// Adversarial: the interner refuses past its bound.
    #[test]
    fn interner_refuses_past_its_bound() {
        let mut interner = Interner::new();
        for n in 0..MAX_DISTINCT_STRINGS {
            interner.intern(&format!("name-{n}")).expect("within bound");
        }
        assert_eq!(
            interner.intern("one too many"),
            Err(CoreError::TooManyStrings)
        );
    }

    /// Adversarial: a repeated holder is not a double promise.
    #[test]
    fn census_same_holder_twice_is_not_double() {
        let entries = parse_plan("Saw | Expedition A | unique\nSaw | Expedition A | unique\n")
            .expect("plan parses");
        let census = census_of(&entries);
        assert_eq!(census.rows.len(), 1);
        assert!(!census.rows[0].double_promised());
    }

    /// Adversarial: malformed plan lines are refused with the line.
    #[test]
    fn parse_plan_refuses_malformed_lines() {
        assert_eq!(
            parse_plan("Saw | Expedition A\n"),
            Err(CoreError::LedgerParse { line: 1 })
        );
        assert_eq!(
            parse_plan("Saw | Expedition A | heavy\n"),
            Err(CoreError::LedgerParse { line: 1 })
        );
    }

    /// Adversarial: line indexing over text with no trailing
    /// newline and multi-byte characters slices on boundaries.
    #[test]
    fn line_index_handles_multibyte_without_trailing_newline() {
        let source = "café\n🦀 rust\nfin";
        let index = LineIndex::new(source);
        assert_eq!(index.len(), 3);
        assert_eq!(index.get(0), Some("café"));
        assert_eq!(index.get(1), Some("🦀 rust"));
        assert_eq!(index.get(2), Some("fin"));
    }
}
