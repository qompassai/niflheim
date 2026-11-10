//! The five chambers' proof validators, as pure functions over
//! content + player state. No state lives here: each function reads
//! the player's current work and returns either silence (the proof
//! holds) or the refusal, worded as the vault would word it —
//! because in this design a refusal is a lesson, and failure must
//! return specific knowledge.
//!
//! The state's transitions live in `state`; this module only judges.

use crate::content::MASTER_PLAN;
use crate::content::{
    CHARTER, PACK_ITEMS, PLACARDS, PORTERS, READERS, RECORD_KEPT, RECORD_KEPT_SECOND,
    RECORD_RESTORED, TOLL_KEEPER_A, TOLL_KEEPER_B,
};
use crate::core::{Census, census_of, parse_plan};

// ---------------------------------------------------------------------------
// Chamber 1 — Moves and Copies.
// ---------------------------------------------------------------------------

/// The reconciled toll: how many distinct crates the two tally
/// keepers counted between them, overlaps counted once.
pub fn reconciled_toll() -> u32 {
    let mut names: Vec<&str> = TOLL_KEEPER_A.to_vec();
    for name in TOLL_KEEPER_B {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    u32::try_from(names.len()).unwrap_or(0)
}

/// The names both tally keepers counted (the overlap), in Keeper
/// A's order — the names behind the reconciliation.
pub fn toll_overlap() -> Vec<&'static str> {
    TOLL_KEEPER_A
        .iter()
        .copied()
        .filter(|name| TOLL_KEEPER_B.contains(name))
        .collect()
}

/// Holders of one item as a reading: "Anke and Boro", "no porter".
pub fn holders_reading(mask: u8) -> String {
    let names: Vec<&str> = PORTERS
        .iter()
        .enumerate()
        .filter(|(index, _)| mask & (1 << index) != 0)
        .map(|(_, name)| *name)
        .collect();
    match names.len() {
        0 => "no porter".to_string(),
        1 => names[0].to_string(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => format!("{}, {} and {}", names[0], names[1], names[2]),
    }
}

/// Judge a packing list. `assignments` is one holder-bitmask per
/// item in [`PACK_ITEMS`] order. Returns every problem found, in
/// item order; an empty list means the packing proof holds.
pub fn packing_problems(assignments: &[u8]) -> Vec<String> {
    let mut problems = Vec::new();
    for (index, item) in PACK_ITEMS.iter().enumerate() {
        let mask = assignments.get(index).copied().unwrap_or(0);
        let count = mask.count_ones();
        if item.copiable {
            if count == 0 {
                problems.push(format!(
                    "the {} stand under no porter — a copiable thing still has to be carried",
                    item.name
                ));
            }
        } else if count == 0 {
            problems.push(format!(
                "the {} stands under no porter — the plan leaves it behind",
                item.name
            ));
        } else if count > 1 {
            problems.push(format!(
                "the {} stands under {} ({}) — an uncopiable thing promised more than once",
                item.name,
                count,
                holders_reading(mask)
            ));
        }
    }
    problems
}

// ---------------------------------------------------------------------------
// Chamber 2 — Borrowing.
// ---------------------------------------------------------------------------

/// Judge a day-plan: `picks` is one candidate index per person in
/// [`READERS`] order (annotator first). Returns the first overlap
/// between the annotator's block and a reader's, worded, or `None`
/// when the plan is clean.
pub fn day_plan_overlap(picks: &[u8]) -> Option<String> {
    let annotator = READERS.first()?;
    let a_pick = picks.first().copied().unwrap_or(0) as usize;
    let (a_start, a_end) = annotator.blocks[a_pick.min(1)];
    for (index, reader) in READERS.iter().enumerate().skip(1) {
        let pick = picks.get(index).copied().unwrap_or(0) as usize;
        let (r_start, r_end) = reader.blocks[pick.min(1)];
        if a_start < r_end && r_start < a_end {
            let start = a_start.max(r_start);
            let end = a_end.min(r_end);
            return Some(format!(
                "{} holds the chart from {r_start} to {r_end}, and the annotator writes from \
                 {a_start} to {a_end} — they share {start} to {end}, when the pen must be alone",
                reader.name
            ));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Chamber 3 — Lifetimes as Contracts.
// ---------------------------------------------------------------------------

/// Judge a lease chain against the charter. Returns the first
/// defect, worded with the brackets named, or `None` when the
/// chain nests cleanly.
pub fn lease_chain_problem(leases: &[(i32, i32); 3]) -> Option<String> {
    let ordinals = ["first", "second", "third"];
    for (index, (start, end)) in leases.iter().enumerate() {
        if start >= end {
            return Some(format!(
                "the {} lease is empty — it begins at month {start} and ends at month {end}",
                ordinals[index]
            ));
        }
    }
    let (c_start, c_end) = CHARTER;
    let (first_start, first_end) = leases[0];
    if first_start < c_start || first_end > c_end {
        return Some(format!(
            "the first lease runs {first_start} to {first_end}, outside the charter's \
             {c_start} to {c_end}"
        ));
    }
    for index in 1..3 {
        let (outer_start, outer_end) = leases[index - 1];
        let (start, end) = leases[index];
        if start < outer_start || end > outer_end {
            return Some(format!(
                "the {} lease runs {start} to {end}, stepping outside the {} lease, \
                 which runs {outer_start} to {outer_end}",
                ordinals[index],
                ordinals[index - 1]
            ));
        }
    }
    None
}

/// Draw the chain as bracket lines, one per term, scaled so the
/// charter spans the full width. Deterministic text, drawn the
/// same way the vault draws it — a refused chain is shown, not
/// just described.
pub fn bracket_lines(leases: &[(i32, i32); 3]) -> Vec<String> {
    let width = (CHARTER.1 - CHARTER.0).max(1) as usize;
    let draw = |label: &str, (start, end): (i32, i32)| -> String {
        let mut bar = String::with_capacity(width);
        for month in CHARTER.0..CHARTER.1 {
            bar.push(if month >= start && month < end {
                '='
            } else {
                ' '
            });
        }
        format!("{label:<10} [{bar}] {start}..{end}")
    };
    let mut lines = vec![draw("charter", CHARTER)];
    let ordinals = ["lease 1", "lease 2", "lease 3"];
    for (index, lease) in leases.iter().enumerate() {
        lines.push(draw(ordinals[index], *lease));
    }
    lines
}

// ---------------------------------------------------------------------------
// Chamber 4 — Drop and Scope Guards.
// ---------------------------------------------------------------------------

/// Which kind of record a delivery run produced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordKind {
    /// The book kept the room's arrangements.
    Kept,
    /// The book kept the second room's (also committed) arrangements.
    KeptSecond,
    /// The steward restored the room.
    Restored,
}

/// One record card from a delivery run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryRecord {
    /// Which kind of record this is.
    pub kind: RecordKind,
    /// The record's text, in the steward's hand.
    pub text: &'static str,
}

/// Run the delivery through both rooms. The first room's
/// arrangements stand committed in the book; the second room's
/// fate follows `commit_second`. The two records are the chamber's
/// proof material.
pub fn run_delivery(commit_second: bool) -> Vec<DeliveryRecord> {
    let second = if commit_second {
        DeliveryRecord {
            kind: RecordKind::KeptSecond,
            text: RECORD_KEPT_SECOND,
        }
    } else {
        DeliveryRecord {
            kind: RecordKind::Restored,
            text: RECORD_RESTORED,
        }
    };
    vec![
        DeliveryRecord {
            kind: RecordKind::Kept,
            text: RECORD_KEPT,
        },
        second,
    ]
}

/// Judge placing `record` into `slot` (0 = what the book kept,
/// 1 = what the room restored). `Ok(())` when the record belongs
/// in the slot; the refusal otherwise.
pub fn record_fits_slot(kind: RecordKind, slot: usize) -> Result<(), String> {
    match (kind, slot) {
        (RecordKind::Kept, 0) | (RecordKind::KeptSecond, 0) | (RecordKind::Restored, 1) => Ok(()),
        (RecordKind::Kept, _) => Err(
            "that record is what the book kept — it does not belong in the restored slot"
                .to_string(),
        ),
        (RecordKind::Restored, _) => Err(
            "that record is what the room restored — it does not belong in the kept slot"
                .to_string(),
        ),
        (RecordKind::KeptSecond, _) => Err(
            "the second room was committed too — nothing was restored there. The steward \
             cannot restore what the book holds; reset the chamber and run the delivery \
             uncommitted"
                .to_string(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Chamber 5 — Reborrows and the Hard Cases.
// ---------------------------------------------------------------------------

/// Judge the placards. `judgements` is one `Option<bool>` per
/// placard (`true` = judged legal). Returns the unjudged count and
/// every wrong judgement with the rule that decides it.
pub fn judge_placards(judgements: &[Option<bool>]) -> (usize, Vec<(usize, &'static str)>) {
    let mut unjudged = 0;
    let mut wrong = Vec::new();
    for (index, placard) in PLACARDS.iter().enumerate() {
        match judgements.get(index).copied().flatten() {
            None => unjudged += 1,
            Some(judged) if judged != placard.legal => wrong.push((index, placard.rule)),
            Some(_) => {}
        }
    }
    (unjudged, wrong)
}

// ---------------------------------------------------------------------------
// The FOE — the master census.
// ---------------------------------------------------------------------------

/// Take the census of the FOE's master plan. The plan is authored
/// content and parses by construction; a parse failure would be a
/// build defect, so it comes back as a census of nothing rather
/// than a panic.
pub fn foe_census() -> Census {
    match parse_plan(MASTER_PLAN) {
        Ok(entries) => census_of(&entries),
        Err(_) => Census { rows: Vec::new() },
    }
}

/// The census rows that are uncopiable, as indices into
/// [`foe_census`]'s rows — the rows the player must verify.
pub fn foe_uncopiable_rows() -> Vec<usize> {
    foe_census()
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| !row.copiable)
        .map(|(index, _)| index)
        .collect()
}

/// The census row index of the double-promised item, if the plan
/// still stands as authored.
pub fn foe_double_row() -> Option<usize> {
    foe_census()
        .rows
        .iter()
        .position(|row| row.double_promised())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{LEASES_INITIAL, TOLL_CANDIDATES};

    /// Validation: the authored flawed packing plan fails on the
    /// saw and the ladder, and a repaired plan passes.
    #[test]
    fn packing_judges_the_flawed_and_repaired_plans() {
        let initial: Vec<u8> = PACK_ITEMS
            .iter()
            .map(|item| {
                item.initial_holders
                    .iter()
                    .fold(0u8, |mask, p| mask | (1 << p))
            })
            .collect();
        let problems = packing_problems(&initial);
        assert_eq!(problems.len(), 2);
        assert!(problems[0].contains("Saw"));
        assert!(problems[1].contains("Rope Ladder"));
        let repaired = vec![0b001, 0b001, 0b011, 0b010, 0b100, 0b010, 0b100, 0b010];
        assert!(packing_problems(&repaired).is_empty());
    }

    /// Validation: the toll reconciles to the union count, which
    /// is one of the candidates.
    #[test]
    fn toll_reconciles_by_name() {
        assert_eq!(reconciled_toll(), 19);
        assert!(TOLL_CANDIDATES.contains(&19));
        assert_eq!(toll_overlap().len(), 6);
    }

    /// Validation: a clean day-plan exists and the initial plan
    /// is tangled.
    #[test]
    fn day_plan_clean_and_tangled() {
        let tangled = [0u8; 5];
        assert!(day_plan_overlap(&tangled).is_some());
        let clean = [1u8, 0, 0, 0, 0];
        assert_eq!(day_plan_overlap(&clean), None);
        let clean_early = [0u8, 1, 1, 1, 1];
        assert_eq!(day_plan_overlap(&clean_early), None);
    }

    /// Validation: the authored lease chain fails on lease 2; a
    /// nested chain passes.
    #[test]
    fn lease_chain_judgements() {
        let problem = lease_chain_problem(&LEASES_INITIAL).expect("flawed chain");
        assert!(problem.contains("second lease"));
        assert_eq!(lease_chain_problem(&[(3, 30), (6, 30), (9, 24)]), None);
    }

    /// Validation: both delivery fates produce fittable records.
    #[test]
    fn delivery_records_fit_their_slots() {
        let records = run_delivery(false);
        assert_eq!(record_fits_slot(records[0].kind, 0), Ok(()));
        assert_eq!(record_fits_slot(records[1].kind, 1), Ok(()));
    }

    /// Validation: ten correct judgements pass the yard.
    #[test]
    fn placards_all_correct_passes() {
        let judgements: Vec<Option<bool>> = PLACARDS.iter().map(|p| Some(p.legal)).collect();
        let (unjudged, wrong) = judge_placards(&judgements);
        assert_eq!(unjudged, 0);
        assert!(wrong.is_empty());
    }

    /// Adversarial: a copiable item with no porter still fails —
    /// copies must be carried by someone.
    #[test]
    fn packing_refuses_uncarried_copiables() {
        let mut plan = vec![0b001, 0b001, 0b011, 0b010, 0b100, 0b010, 0b100, 0b010];
        plan[2] = 0;
        let problems = packing_problems(&plan);
        assert_eq!(problems.len(), 1);
        assert!(problems[0].contains("Tally Sticks"));
    }

    /// Adversarial: an empty lease is named before nesting is judged.
    #[test]
    fn lease_chain_names_empty_lease() {
        let problem = lease_chain_problem(&[(3, 30), (12, 12), (9, 24)]).expect("empty lease");
        assert!(problem.contains("empty"));
    }

    /// Adversarial: swapped records are refused with the reason.
    #[test]
    fn delivery_records_refuse_swapped_slots() {
        assert!(record_fits_slot(RecordKind::Kept, 1).is_err());
        assert!(record_fits_slot(RecordKind::Restored, 0).is_err());
        let committed = run_delivery(true);
        assert!(record_fits_slot(committed[1].kind, 1).is_err());
    }

    /// Adversarial: a wrong placard judgement comes back with its rule.
    #[test]
    fn placards_name_the_rule_for_wrong_judgements() {
        let mut judgements: Vec<Option<bool>> = PLACARDS.iter().map(|p| Some(p.legal)).collect();
        judgements[1] = Some(true);
        let (_, wrong) = judge_placards(&judgements);
        assert_eq!(wrong.len(), 1);
        assert_eq!(wrong[0].0, 1);
        assert!(wrong[0].1.contains("vault rule"));
    }

    /// Adversarial: the FOE census has exactly one double promise,
    /// the Chart, among nine uncopiable rows.
    #[test]
    fn foe_census_shape_is_as_authored() {
        let census = foe_census();
        let doubles: Vec<&str> = census
            .rows
            .iter()
            .filter(|row| row.double_promised())
            .map(|row| row.item.as_str())
            .collect();
        assert_eq!(doubles, ["Heartstone Chart"]);
        assert_eq!(foe_uncopiable_rows().len(), 9);
        assert!(foe_double_row().is_some());
    }
}
