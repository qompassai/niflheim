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
    ADMISSION_SERVICES, BAR_REASONS, BLIND_RUN_PERFORMANCE, CHARTER, COLLAR_ROUTES,
    COLLISION_READINGS, DELIVERIES, FOE_ADMISSIONS, PACK_ITEMS, PLACARDS, PORTERS, READERS,
    RECORD_KEPT, RECORD_KEPT_SECOND, RECORD_RESTORED, REGISTRY_BEASTS, RegistryBeast,
    STABLE_SERVICES, STABLE_SIGILS, TOLL_KEEPER_A, TOLL_KEEPER_B, TRICK_CHARTER, TRICK_PERFORMERS,
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

// ---------------------------------------------------------------------------
// Floor 2, Chamber 1 — Traits and impls.
// ---------------------------------------------------------------------------

/// Whether `sigil` charters the Anvil Call: the sigil-book declares
/// the trick under exactly one sigil.
pub fn trick_charter_ok(sigil: &str) -> bool {
    sigil == TRICK_CHARTER
}

/// Which taught beast ran the blind run, judged by behaviour
/// alone: the index into `TRICK_PERFORMERS` whose performance text
/// is the blind run's text.
pub fn blind_run_performer() -> Option<usize> {
    TRICK_PERFORMERS
        .iter()
        .position(|performer| performer.performance == BLIND_RUN_PERFORMANCE)
}

/// Whether the player's identification names the body that ran.
pub fn identification_ok(choice: usize) -> bool {
    blind_run_performer() == Some(choice)
}

// ---------------------------------------------------------------------------
// Floor 2, Chamber 2 — Generics and bounds.
// ---------------------------------------------------------------------------

/// The sigils a bound demands that the service's work never asks
/// for, in board order — the Sigilwright's refusal grounds. An
/// out-of-range service has no work to judge against and yields
/// silence, never a pass.
pub fn bound_unjustified(service: usize, mask: u8) -> Vec<&'static str> {
    let Some(stable) = STABLE_SERVICES.get(service) else {
        return Vec::new();
    };
    STABLE_SIGILS
        .iter()
        .enumerate()
        .filter(|(bit, _)| mask & (1 << bit) != 0 && stable.required & (1 << bit) == 0)
        .map(|(_, sigil)| *sigil)
        .collect()
}

/// The sigils the service's work requires that the bound fails to
/// demand, in board order.
pub fn bound_missing(service: usize, mask: u8) -> Vec<&'static str> {
    let Some(stable) = STABLE_SERVICES.get(service) else {
        return Vec::new();
    };
    STABLE_SIGILS
        .iter()
        .enumerate()
        .filter(|(bit, _)| stable.required & (1 << bit) != 0 && mask & (1 << bit) == 0)
        .map(|(_, sigil)| *sigil)
        .collect()
}

/// Whether the bound demands exactly what the service's work
/// requires — nothing unjustified, nothing missing. Bits beyond
/// the stable's four sigils make any bound inexact.
pub fn bound_exact(service: usize, mask: u8) -> bool {
    let Some(stable) = STABLE_SERVICES.get(service) else {
        return false;
    };
    mask & !0b1111 == 0
        && bound_unjustified(service, mask).is_empty()
        && bound_missing(service, mask).is_empty()
        && mask == stable.required
}

// ---------------------------------------------------------------------------
// Floor 2, Chamber 3 — Associated types.
// ---------------------------------------------------------------------------

/// The errand that runs contorted from the kennel that chooses
/// its own kind, judged from the delivery records alone.
pub fn contorted_errand() -> Option<usize> {
    DELIVERIES.first()?.iter().position(|run| run.contorted)
}

/// Whether the player's finding names the contorted errand.
pub fn errand_choice_ok(choice: usize) -> bool {
    contorted_errand() == Some(choice)
}

/// Whether `kennel` is a kennel from which the contorted errand
/// runs naturally: not the choosing kennel itself, and its record
/// for that errand is not a contorted run.
pub fn kennel_choice_ok(kennel: usize) -> bool {
    let Some(errand) = contorted_errand() else {
        return false;
    };
    kennel != 0
        && DELIVERIES
            .get(kennel)
            .and_then(|runs| runs.get(errand))
            .is_some_and(|run| !run.contorted)
}

// ---------------------------------------------------------------------------
// Floor 2, Chamber 4 — Trait objects and the Wide Door.
// ---------------------------------------------------------------------------

/// Whether `label` names the bar that keeps `beast` from the Wide
/// Door. A beast that passes has no bar, and no label names one
/// for it; out-of-range beasts and labels never hold.
pub fn bar_label_ok(beast: usize, label: usize) -> bool {
    REGISTRY_BEASTS.get(beast).and_then(|entry| entry.bar) == Some(label)
        && label < BAR_REASONS.len()
}

/// Whether the player's naming assigns every barred beast its own
/// bar. `choices` is keyed by the barred beasts in book order;
/// a short, long, or partly empty naming never holds.
pub fn registry_bars_ok(choices: &[Option<usize>]) -> bool {
    let barred: Vec<&RegistryBeast> = REGISTRY_BEASTS
        .iter()
        .filter(|entry| !entry.passes)
        .collect();
    choices.len() == barred.len()
        && barred
            .iter()
            .zip(choices.iter())
            .all(|(entry, choice)| *choice == entry.bar)
}

// ---------------------------------------------------------------------------
// Floor 2, Chamber 5 — Coherence.
// ---------------------------------------------------------------------------

/// Whether the joining route the player chose is the lawful one:
/// the guild collar of the player's own making.
pub fn collar_route_ok(choice: usize) -> bool {
    COLLAR_ROUTES.get(choice).is_some_and(|route| route.legal)
}

/// Whether the reading of the staged collision the player holds
/// to is the one that holds.
pub fn collision_reading_ok(choice: usize) -> bool {
    COLLISION_READINGS
        .get(choice)
        .is_some_and(|reading| reading.ok)
}

// ---------------------------------------------------------------------------
// Floor 2, the FOE — the admission audit.
// ---------------------------------------------------------------------------

/// Whether the desk's slip for audit row `row` disagrees with the
/// beast's own sigils: the post the slip claims asks a sigil the
/// beast does not carry. Judged from the table alone — slips and
/// sigil lists, never names or coats. An out-of-range row has no
/// slip to judge and never disagrees.
pub fn foe2_row_disagrees(row: usize) -> bool {
    let Some(admission) = FOE_ADMISSIONS.get(row) else {
        return false;
    };
    let Some(service) = ADMISSION_SERVICES.get(admission.service) else {
        return false;
    };
    !service
        .bound
        .iter()
        .all(|sigil| admission.sigils.contains(sigil))
}

/// The audit table's one disagreeing row, if the table stands as
/// authored — the seam the audit exists to find.
pub fn foe2_disagreement_row() -> Option<usize> {
    FOE_ADMISSIONS
        .iter()
        .enumerate()
        .find(|(row, _)| foe2_row_disagrees(*row))
        .map(|(row, _)| row)
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

    /// Validation: Forgehand charters the Anvil Call, and the blind
    /// run resolves by behaviour to the variant performer, Soot.
    #[test]
    fn anvil_call_charter_and_blind_run() {
        use crate::content::{BEASTS, CHARTER_CANDIDATES};
        assert!(trick_charter_ok("Forgehand"));
        assert!(CHARTER_CANDIDATES.contains(&"Forgehand"));
        let runner = blind_run_performer().expect("the blind run has a body");
        assert_eq!(BEASTS[TRICK_PERFORMERS[runner].beast].name, "Soot");
        assert!(identification_ok(runner));
    }

    /// Adversarial: the sigils the performers wear do not charter
    /// the call, the house performer is not the blind runner, and
    /// an out-of-range identification never holds.
    #[test]
    fn anvil_call_refuses_resemblance_and_wrong_bodies() {
        assert!(!trick_charter_ok("FireWard"));
        assert!(!trick_charter_ok("Swift"));
        assert!(!trick_charter_ok(""));
        assert!(!identification_ok(0));
        assert!(!identification_ok(usize::MAX));
    }

    /// Validation: each stable service's own required bound is
    /// exact, and the posted bound — every sigil demanded — is
    /// exact nowhere.
    #[test]
    fn stable_bounds_judge_exactness() {
        use crate::content::POSTED_BOUND;
        for (service, stable) in STABLE_SERVICES.iter().enumerate() {
            assert!(bound_exact(service, stable.required));
            assert!(bound_unjustified(service, stable.required).is_empty());
            assert!(bound_missing(service, stable.required).is_empty());
            assert!(!bound_exact(service, POSTED_BOUND));
        }
    }

    /// Adversarial: an unjustified sigil and a missing sigil are
    /// named separately, a bound of nothing staffs nothing, and
    /// out-of-range services and stray bits never judge exact.
    #[test]
    fn stable_bounds_name_unjustified_and_missing() {
        assert_eq!(bound_unjustified(0, 0b0011), ["MistSense"]);
        assert_eq!(bound_missing(0, 0b0010), ["FireWard"]);
        assert_eq!(bound_missing(2, 0), ["KeenNose"]);
        assert!(!bound_exact(0, u8::MAX));
        assert!(!bound_exact(usize::MAX, 0));
        assert!(bound_unjustified(usize::MAX, u8::MAX).is_empty());
    }

    /// Validation: the delivery records hold exactly one contorted
    /// run — the River Parcel from the Hearth Kennel — and the
    /// finding that names it and the Open Kennel holds.
    #[test]
    fn kennel_records_hold_one_contorted_run() {
        assert_eq!(contorted_errand(), Some(1));
        assert!(errand_choice_ok(1));
        assert!(kennel_choice_ok(1));
    }

    /// Validation: the registry's barred beasts carry the three
    /// bars in book order, and the naming that assigns each its
    /// own bar holds.
    #[test]
    fn registry_bars_are_named_as_authored() {
        use crate::content::REGISTRY_BEASTS;
        let barred: Vec<usize> = REGISTRY_BEASTS
            .iter()
            .enumerate()
            .filter(|(_, entry)| !entry.passes)
            .map(|(index, _)| index)
            .collect();
        assert_eq!(barred, [3, 4, 5]);
        for (position, beast) in barred.iter().enumerate() {
            assert!(bar_label_ok(*beast, position));
        }
        assert!(registry_bars_ok(&[Some(0), Some(1), Some(2)]));
    }

    /// Adversarial: a passing beast has no bar to name, swapped
    /// bars fail, a partly empty naming fails, and out-of-range
    /// beasts and labels never hold.
    #[test]
    fn registry_bars_refuse_swaps_and_partial_namings() {
        assert!(!bar_label_ok(0, 0));
        assert!(!bar_label_ok(usize::MAX, 0));
        assert!(!bar_label_ok(3, usize::MAX));
        assert!(!registry_bars_ok(&[Some(1), Some(0), Some(2)]));
        assert!(!registry_bars_ok(&[Some(0), None, Some(2)]));
        assert!(!registry_bars_ok(&[Some(0), Some(1)]));
        assert!(!registry_bars_ok(&[]));
    }

    /// Validation: exactly one joining route is lawful — the
    /// guild collar — and exactly one reading of the collision
    /// holds — one claim per beast.
    #[test]
    fn coherence_judges_hold_as_authored() {
        assert!(collar_route_ok(1));
        assert!(collision_reading_ok(2));
    }

    /// Adversarial: stamping directly and the blanket joining are
    /// both unlawful, the priority readings do not hold, and
    /// out-of-range choices never hold.
    #[test]
    fn coherence_refuses_foreign_marks_and_priority_readings() {
        assert!(!collar_route_ok(0));
        assert!(!collar_route_ok(2));
        assert!(!collar_route_ok(usize::MAX));
        assert!(!collision_reading_ok(0));
        assert!(!collision_reading_ok(1));
        assert!(!collision_reading_ok(usize::MAX));
    }
    /// Validation: exactly one admission on the desk's table
    /// disagrees with its beast's sigils — the mist-hound's
    /// fire-watch slip — and the judge finds it from the table.
    #[test]
    fn foe_audit_finds_the_one_disagreement() {
        assert_eq!(foe2_disagreement_row(), Some(3));
        assert!(foe2_row_disagrees(3));
        let disagreeing: Vec<usize> = (0..crate::content::FOE_ADMISSIONS.len())
            .filter(|row| foe2_row_disagrees(*row))
            .collect();
        assert_eq!(disagreeing, vec![3]);
    }

    /// Adversarial: sound admissions never disagree — each roster
    /// beast's sigils satisfy its post's bound — and forged rows
    /// never hold.
    #[test]
    fn foe_audit_refuses_sound_rows_and_forgeries() {
        for row in [0, 1, 2, 4, 5, 6] {
            assert!(!foe2_row_disagrees(row), "row {row} stands sound");
        }
        assert!(!foe2_row_disagrees(usize::MAX));
    }
    /// Adversarial: the smooth errands are not the finding, the
    /// choosing kennel cannot be its own remedy, and out-of-range
    /// choices never hold.
    #[test]
    fn kennel_finding_refuses_smooth_errands_and_the_choosing_kennel() {
        assert!(!errand_choice_ok(0));
        assert!(!errand_choice_ok(2));
        assert!(!kennel_choice_ok(0));
        assert!(!errand_choice_ok(usize::MAX));
        assert!(!kennel_choice_ok(usize::MAX));
    }
}

/// Floor 3 Chamber 1's judge: a report is sorted when the pair
/// chosen for it is its own. The generic Broken pair (index 4)
/// is no report's pair, so choosing it never sorts — the
/// desk's refusal is the table itself.
pub fn failure_pair_ok(report: usize, choice: usize) -> bool {
    crate::content::FAILURE_REPORTS
        .get(report)
        .is_some_and(|entry| entry.pair == choice)
}

/// Floor 3 Chamber 2's judge: run the player's three stations
/// as one real iterator chain over the belt, in station order —
/// the portioner takes, the joiner settles duplicates, the gate
/// decides the cracked piece — then poll the chain past its
/// end. Returns what came off the belt and whether the chain
/// stayed silent past its end (fused).
pub fn conveyor_run(portion: usize, dedup: usize, halt: usize) -> (Vec<&'static str>, bool) {
    let take = match portion {
        0 => 2,
        1 => 3,
        _ => usize::MAX,
    };
    let belt = crate::content::CONVEYOR_BELT.iter().copied().take(take);
    let deduped: Box<dyn Iterator<Item = crate::content::BeltItem>> = match dedup {
        0 => {
            let mut last: Option<crate::content::BeltItem> = None;
            Box::new(belt.filter(move |item| {
                if last == Some(*item) {
                    false
                } else {
                    last = Some(*item);
                    true
                }
            }))
        }
        1 => {
            let mut seen: Vec<crate::content::BeltItem> = Vec::new();
            Box::new(belt.filter(move |item| {
                if seen.contains(item) {
                    false
                } else {
                    seen.push(*item);
                    true
                }
            }))
        }
        _ => Box::new(belt),
    };
    let mut chain: Box<dyn Iterator<Item = &'static str>> = match halt {
        0 => Box::new(
            deduped
                .take_while(|item| !matches!(item, crate::content::BeltItem::Spoiled(_)))
                .map(|item| item.name()),
        ),
        1 => Box::new(
            deduped
                .filter(|item| !matches!(item, crate::content::BeltItem::Spoiled(_)))
                .map(|item| item.name()),
        ),
        _ => Box::new(deduped.map(|item| item.name())),
    };
    let mut out = Vec::new();
    for name in chain.by_ref() {
        out.push(name);
    }
    let fused = chain.next().is_none() && chain.next().is_none();
    (out, fused)
}

/// The belt is set when its chain delivers exactly the desk's
/// slate, fused past its end.
pub fn conveyor_diagnosed(portion: usize, dedup: usize, halt: usize) -> bool {
    let (out, fused) = conveyor_run(portion, dedup, halt);
    fused && out == crate::content::CONVEYOR_REQUIRED
}

#[cfg(test)]
mod floor3_tests {
    use super::*;

    /// The taxonomy judge: each report sorts only under its own
    /// pair, and the Broken pair sorts nothing.
    #[test]
    fn failure_pairs_judge_each_report() {
        for (report, entry) in crate::content::FAILURE_REPORTS.iter().enumerate() {
            assert!(failure_pair_ok(report, entry.pair));
            assert!(!failure_pair_ok(report, 4), "the Broken bin sorts nothing");
            assert!(!failure_pair_ok(report, (entry.pair + 1) % 4));
        }
        assert!(!failure_pair_ok(99, 0));
    }

    /// The conveyor's correct setting is unique among its
    /// neighbours: whole belt, first sightings, lift the cracked
    /// piece — and every station off by one fails the slate.
    #[test]
    fn conveyor_correct_setting_is_unique() {
        assert!(conveyor_diagnosed(2, 1, 1));
        for portion in 0..3 {
            for dedup in 0..3 {
                for halt in 0..3 {
                    if (portion, dedup, halt) != (2, 1, 1) {
                        assert!(
                            !conveyor_diagnosed(portion, dedup, halt),
                            "({portion}, {dedup}, {halt}) must not deliver the slate"
                        );
                    }
                }
            }
        }
        let (out, fused) = conveyor_run(2, 1, 1);
        assert_eq!(out, crate::content::CONVEYOR_REQUIRED);
        assert!(fused);
        // Halting at the cracked piece starves the slate.
        let (out, _) = conveyor_run(2, 1, 0);
        assert_eq!(out, ["gear", "spring"]);
        // Joining runs lets the second spring through.
        let (out, _) = conveyor_run(2, 0, 1);
        assert_eq!(out, ["gear", "spring", "spring", "cam", "bell"]);
    }
}

/// Floor 3 Chamber 3's judge: a ledger entry is sorted when
/// its verdict matches its kind — a traveller-reachable failure
/// under "named refusal", an invariant-only failure under
/// "the shout stands" with its own invariant named.
pub fn shout_case_ok(case: usize, verdict: Option<usize>, invariant: Option<usize>) -> bool {
    let Some(entry) = crate::content::SHOUT_CASES.get(case) else {
        return false;
    };
    match (entry.shout, verdict) {
        (false, Some(0)) => true,
        (true, Some(1)) => invariant == entry.invariant,
        _ => false,
    }
}

/// Floor 3 Chamber 4's judge: the rail is complete when both
/// honest slips stand sealed and the body-less slip stands in
/// the returns tray.
pub fn rail_complete(stages: &[u8; 3]) -> bool {
    *stages == [3, 3, 4]
}

#[cfg(test)]
mod floor3_step3_tests {
    use super::*;

    /// The audit judge: refusals sort under refusal, shouts
    /// under their own invariant only — a decoy invariant or a
    /// swapped verdict fails the entry.
    #[test]
    fn shout_cases_judge_verdict_and_invariant() {
        assert!(shout_case_ok(0, Some(0), None));
        assert!(!shout_case_ok(0, Some(1), Some(2)));
        assert!(shout_case_ok(1, Some(1), Some(0)));
        assert!(
            !shout_case_ok(1, Some(1), Some(1)),
            "the pot is not the belt"
        );
        assert!(
            !shout_case_ok(1, Some(0), None),
            "the house's failure is no refusal"
        );
        assert!(shout_case_ok(3, Some(1), Some(1)));
        assert!(!shout_case_ok(2, Some(1), Some(3)));
        assert!(!shout_case_ok(1, None, Some(0)), "unsorted is unsorted");
        assert!(!shout_case_ok(99, Some(0), None));
    }

    /// The rail judge: only the one true end state completes.
    #[test]
    fn rail_judges_its_end_state() {
        assert!(rail_complete(&[3, 3, 4]));
        assert!(!rail_complete(&[3, 3, 3]), "the body-less slip cannot seal");
        assert!(!rail_complete(&[3, 2, 4]));
        assert!(!rail_complete(&[0, 1, 1]));
    }
}

/// Floor 3 Chamber 5's judges: a checklist line is answered
/// when its verdict is the Hoard's true standing; the
/// not-applicable line also needs its true reason; the stamp
/// is the oldest road.
pub fn inspection_item_ok(item: usize, verdict: Option<usize>) -> bool {
    crate::content::INSPECTION_ITEMS
        .get(item)
        .is_some_and(|entry| verdict == Some(entry.verdict))
}

/// The not-applicable reason is the true one (index 0).
pub fn inspection_reason_ok(reason: Option<usize>) -> bool {
    reason == Some(0)
}

/// The oldest-road stamp is truly chosen (index 0).
pub fn msrv_ok(choice: Option<usize>) -> bool {
    choice == Some(0)
}

/// The Release's slate judge: an entry is reconciled when
/// its verdict matches whether the pass truly ran
/// (0 = it ran, 1 = it never ran).
pub fn slate_entry_ok(entry: usize, verdict: Option<usize>) -> bool {
    crate::content::LOG_SLATE
        .get(entry)
        .is_some_and(|log| verdict == Some(if log.ran { 0 } else { 1 }))
}

// ---------------------------------------------------------------------------
// Floor 4 — memory layout, judged by measurement.

/// Floor 4 Chamber 1: a committed prediction graded against
/// the bench's own measure of the real type.
pub fn bench_prediction_ok(index: usize, pred_size: u32, pred_align: u32) -> bool {
    crate::content::bench_stone_facts(index) == (pred_size, pred_align)
}

/// Floor 4 Chamber 1: whether a measured stone's slate
/// missed (and so owes an annotation).
pub fn bench_missed(index: usize, pred_size: Option<u32>, pred_align: Option<u32>) -> bool {
    match (pred_size, pred_align) {
        (Some(size), Some(align)) => !bench_prediction_ok(index, size, align),
        _ => false,
    }
}

/// The one layout algorithm of the descent: seat (size,
/// align) stones in the given order; each stone's offset is
/// aligned up to its alignment, the wall's span is the last
/// stone's end rounded up to the widest alignment. Returns
/// (total span, void bytes) — the voids are every padding
/// byte, counted one by one.
pub fn wall_layout(stones: &[(u32, u32)]) -> (u32, u32) {
    let mut offset = 0u32;
    let mut voids = 0u32;
    let mut widest = 1u32;
    for &(size, align) in stones {
        widest = widest.max(align);
        let aligned = offset.div_ceil(align) * align;
        voids += aligned - offset;
        offset = aligned + size;
    }
    let total = offset.div_ceil(widest) * widest;
    voids += total - offset;
    (total, voids)
}

/// The smallest span the same stones allow, by the same
/// algorithm: seat them in falling alignment (widest law
/// first, then the heavier stone). A unit test brute-forces
/// every seating of the chamber's stones and finds no
/// smaller span, so this is a measured minimum, not a claim.
pub fn wall_minimum(stones: &[(u32, u32)]) -> u32 {
    let mut order: Vec<(u32, u32)> = stones.to_vec();
    order.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));
    wall_layout(&order).0
}

#[cfg(test)]
mod floor3_step4_tests {
    use super::*;

    /// The inspection judges: every line under its true
    /// verdict only, the reason and the stamp at their truths.
    #[test]
    fn inspection_judges_lines_reason_and_stamp() {
        for (item, entry) in crate::content::INSPECTION_ITEMS.iter().enumerate() {
            assert!(inspection_item_ok(item, Some(entry.verdict)));
            assert!(!inspection_item_ok(item, Some((entry.verdict + 1) % 3)));
            assert!(!inspection_item_ok(item, None));
        }
        assert!(inspection_reason_ok(Some(0)));
        assert!(!inspection_reason_ok(Some(1)));
        assert!(msrv_ok(Some(0)));
        assert!(!msrv_ok(Some(1)));
    }

    /// The slate judges: the fresh-marked test pass ran; the
    /// old-paper passes never did.
    #[test]
    fn slate_judges_the_old_paper() {
        assert!(slate_entry_ok(0, Some(1)));
        assert!(slate_entry_ok(1, Some(1)));
        assert!(slate_entry_ok(2, Some(0)));
        assert!(!slate_entry_ok(0, Some(0)));
        assert!(!slate_entry_ok(2, None));
    }
}

/// Floor 4 Chamber 3's judge helpers: the size of the
/// smallest encoding of `crest`, measured from the real
/// types, and whether a choice is that smallest.
pub fn crest_best(crest: usize) -> usize {
    let mut best = 0usize;
    for candidate in 1..3 {
        let (size, _) = crate::content::crest_candidate_facts(crest, candidate);
        let (best_size, _) = crate::content::crest_candidate_facts(crest, best);
        if size < best_size {
            best = candidate;
        }
    }
    best
}

/// Whether the crest's chosen encoding is a smallest one.
pub fn crest_choice_ok(crest: usize, choice: Option<usize>) -> bool {
    choice == Some(crest_best(crest))
}

/// The offsets the language itself promises a seating
/// under the treaty rules: each stone's offset aligned up
/// to its own alignment, in order. (Chamber 2's
/// `wall_layout` runs the same accumulation and rounds the
/// span; the treaty needs the offsets themselves.)
pub fn c_offsets(stones: &[(u32, u32)]) -> Vec<u32> {
    let mut offsets = Vec::with_capacity(stones.len());
    let mut cursor: u32 = 0;
    for &(size, align) in stones {
        let align = align.max(1);
        cursor = cursor.div_ceil(align) * align;
        offsets.push(cursor);
        cursor += size;
    }
    offsets
}

/// Whether one dimension-book entry's figure disagrees with
/// the stone it names — the re-survey's whole audit, in one
/// comparison the book cannot make about itself.
pub fn book_disagrees(index: usize) -> bool {
    crate::content::DIMENSION_BOOK[index].1 != crate::content::dimension_book_true(index)
}

/// The bits a hollow stone's cavity currently holds: the
/// number's own bits, or the ratio's, reinterpreted honestly
/// — never a union read. `None` while the cavity is empty
/// (filling 0).
pub fn hollow_bits(filling: u8, number: u32, ratio: f32) -> Option<u32> {
    match filling {
        1 => Some(number),
        2 => Some(ratio.to_bits()),
        _ => None,
    }
}

/// A cavity's bits read as a ratio: `f32::from_bits`, the
/// honest reinterpretation the gallery demonstrates.
pub fn hollow_as_ratio(bits: u32) -> f32 {
    f32::from_bits(bits)
}

#[cfg(test)]
mod floor4_step2_tests {
    use super::*;

    /// Floor 4: the bench's facts are the real types' facts —
    /// the table the whole floor stands on, asserted outright.
    #[test]
    fn floor4_bench_facts_are_the_real_types() {
        let expected = [(1, 1), (8, 4), (16, 8), (2, 1), (8, 8), (24, 8)];
        for (index, facts) in expected.iter().enumerate() {
            assert_eq!(
                crate::content::bench_stone_facts(index),
                *facts,
                "stone {index}"
            );
        }
        assert!(bench_prediction_ok(1, 8, 4));
        assert!(
            !bench_prediction_ok(1, 5, 4),
            "the naive sum is not the size"
        );
        assert!(bench_missed(4, Some(16), Some(8)));
        assert!(!bench_missed(4, Some(8), Some(8)));
        assert!(!bench_missed(0, None, None), "uncommitted is not a miss");
    }

    /// Floor 4: the wall algorithm counts the bench order's
    /// voids byte by byte, and its claimed minimum survives a
    /// brute-force pass over every seating.
    #[test]
    fn floor4_wall_minimum_survives_brute_force() {
        let stones: Vec<(u32, u32)> = crate::content::WALL_STONES
            .iter()
            .map(|(_, size, align)| (*size, *align))
            .collect();
        assert_eq!(wall_layout(&stones), (24, 8), "the bench order's wall");
        assert_eq!(wall_minimum(&stones), 16, "the least span");
        // Every permutation of five stones, judged by the
        // same algorithm: none beats the claimed minimum.
        fn permutations(items: &[(u32, u32)]) -> Vec<Vec<(u32, u32)>> {
            if items.len() <= 1 {
                return vec![items.to_vec()];
            }
            let mut out = Vec::new();
            for i in 0..items.len() {
                let mut rest = items.to_vec();
                let head = rest.remove(i);
                for mut tail in permutations(&rest) {
                    let mut row = vec![head];
                    row.append(&mut tail);
                    out.push(row);
                }
            }
            out
        }
        let all = permutations(&stones);
        assert_eq!(all.len(), 120);
        for seating in &all {
            assert!(
                wall_layout(seating).0 >= 16,
                "a seating beat the minimum: {seating:?}"
            );
        }
    }
}

#[cfg(test)]
mod floor4_step3_tests {
    use super::*;

    /// Floor 4 Step 3: the crests' prices are the real types'
    /// prices, the niche wins each crest, and the treaty's
    /// true offsets differ from the book in exactly one seat.
    #[test]
    fn floor4_crest_facts_and_treaty_offsets() {
        let expect: [[(u32, u32); 3]; 3] = [
            [(2, 1), (2, 1), (1, 1)],
            [(24, 8), (16, 8), (24, 8)],
            [(16, 8), (16, 8), (8, 8)],
        ];
        for (crest, row) in expect.iter().enumerate() {
            for (candidate, wanted) in row.iter().enumerate() {
                assert_eq!(
                    crate::content::crest_candidate_facts(crest, candidate),
                    *wanted,
                    "crest {crest} candidate {candidate}"
                );
            }
        }
        assert_eq!(crest_best(0), 2);
        assert_eq!(crest_best(1), 1);
        assert_eq!(crest_best(2), 2);
        assert!(crest_choice_ok(0, Some(2)));
        assert!(!crest_choice_ok(0, Some(1)));
        assert!(!crest_choice_ok(1, None));
        let dims: Vec<(u32, u32)> = crate::content::TREATY_STONES[..4]
            .iter()
            .map(|&(_, size, align)| (size, align))
            .collect();
        let offsets = c_offsets(&dims);
        assert_eq!(offsets, vec![0, 3, 8, 16]);
        let promised = crate::content::TREATY_PROMISES;
        let mut wrong = 0;
        for seat in 0..4 {
            if promised[seat] != offsets[seat] {
                wrong += 1;
                assert_eq!(seat, 3, "only the final promise is careless");
                assert_eq!(offsets[seat] - promised[seat], 6);
            }
        }
        assert_eq!(wrong, 1);
    }
}

#[cfg(test)]
mod floor4_step4_tests {
    use super::*;

    /// Floor 4 Step 4: the dimension book is false in exactly
    /// the three entries the shadow touched, each by exactly
    /// the shadow's offset — and the book's internal
    /// consistency never enters into it.
    #[test]
    fn the_book_is_false_only_where_the_shadow_fell() {
        let wrong: Vec<usize> = (0..crate::content::DIMENSION_BOOK.len())
            .filter(|&entry| book_disagrees(entry))
            .collect();
        assert_eq!(wrong, vec![1, 3, 4], "three entries of five are false");
        for entry in wrong {
            assert_eq!(
                crate::content::DIMENSION_BOOK[entry].1
                    - crate::content::dimension_book_true(entry),
                crate::content::SHADOW_OFFSET,
                "every false figure stands off by the shadow's offset"
            );
        }
    }

    /// The hollow stones: bits read as the tag claims, in
    /// both directions — a number's bits are next to nothing
    /// as a ratio, and a ratio's bits are a count in the
    /// millions as a number.
    #[test]
    fn hollow_bits_read_as_the_tag_claims() {
        assert_eq!(hollow_bits(1, 7, 2.5), Some(7));
        assert_eq!(hollow_bits(2, 7, 2.5), Some(2.5f32.to_bits()));
        assert_eq!(hollow_bits(0, 7, 2.5), None);
        let as_ratio = hollow_as_ratio(7);
        assert!(
            as_ratio > 0.0 && as_ratio < 1.0e-40,
            "7 as a ratio: {as_ratio}"
        );
        assert_eq!(2.5f32.to_bits(), 1_075_838_976);
        assert_eq!(hollow_as_ratio(2.5f32.to_bits()), 2.5);
        assert_eq!(0.75f32.to_bits(), 1_061_158_912);
    }

    /// The re-cut's prices: the foreign spec's promised
    /// offsets, the one dishonest stencil, and the free
    /// set's budget proven the minimum by brute force over
    /// every seating.
    #[test]
    fn the_recut_sets_price_themselves() {
        let dims: Vec<(u32, u32)> = crate::content::RECUT_FOREIGN[..4]
            .iter()
            .map(|&(_, size, align)| (size, align))
            .collect();
        assert_eq!(c_offsets(&dims), vec![0, 4, 8, 16]);
        for stone in 0..4 {
            assert_eq!(
                crate::content::RECUT_FOREIGN_STENCIL[stone],
                crate::content::RECUT_FOREIGN[stone].1,
                "stone {stone}'s stencil is honest"
            );
        }
        assert_ne!(
            crate::content::RECUT_FOREIGN_STENCIL[4],
            crate::content::RECUT_FOREIGN[4].1,
            "the shadow-cut beam's stencil repeats the book"
        );
        let pairs: Vec<(u32, u32)> = crate::content::RECUT_FREE
            .iter()
            .map(|(_, size, align)| (*size, *align))
            .collect();
        assert_eq!(wall_minimum(&pairs), 16);
        assert_eq!(wall_layout(&pairs).0, 24, "the tray order overruns");
        let mut best = u32::MAX;
        for a in 0..4usize {
            for b in 0..4usize {
                for c in 0..4usize {
                    for d in 0..4usize {
                        let order = [a, b, c, d];
                        let mut seen = [false; 4];
                        let mut distinct = true;
                        for index in order {
                            if seen[index] {
                                distinct = false;
                            }
                            seen[index] = true;
                        }
                        if !distinct {
                            continue;
                        }
                        let seated: Vec<(u32, u32)> =
                            order.iter().map(|&index| pairs[index]).collect();
                        best = best.min(wall_layout(&seated).0);
                    }
                }
            }
        }
        assert_eq!(best, 16, "no seating of the free four beats the budget");
    }
}

// ---------------------------------------------------------------------------
// Floor 5 — the Veilworks: the Pointer Walks and the Two
// Tenants. The judges are pure over the recorded work: the
// walk is six stones resolved under their cards, one of them
// refused; the tenancies are eight calls the referee stamps,
// with every miss annotated by its deciding rule.
// ---------------------------------------------------------------------------

/// Whether a Pointer Walks step's stone is cracked (the
/// footing fails its card).
pub fn step_cracked(step: usize) -> bool {
    crate::content::STEP_CARDS
        .get(step)
        .map(|(_, _, cracked)| *cracked)
        .unwrap_or(false)
}

/// How many of the eight tenancy calls are true.
pub fn tenancy_correct(predictions: &[u8; 8]) -> usize {
    crate::content::TENANCIES
        .iter()
        .zip(predictions.iter())
        .filter(|((_, sound, _), called)| **called != 0 && (**called == 1) == *sound)
        .count()
}

/// Whether a tenancy call was made and missed.
pub fn tenancy_missed(predictions: &[u8; 8], index: usize) -> bool {
    let Some((_, sound, _)) = crate::content::TENANCIES.get(index) else {
        return false;
    };
    let called = predictions.get(index).copied().unwrap_or(0);
    called != 0 && (called == 1) != *sound
}

// ---------------------------------------------------------------------------
// Floor 5, Step 3 — the Contract Cards and the Split Beam.
// The cards judge purely over the chosen phrasings against
// the table's key; the beam is a real buffer, and its seam
// judges over the split the player chose.
// ---------------------------------------------------------------------------

/// One UB Hunt station's verdict read against the station
/// table: sound stations want 1, faulted stations want 2 —
/// any other entry (0, unentered, among them) is not right.
pub fn station_verdict_right(station: usize, verdict: u8) -> bool {
    match crate::content::STATIONS.get(station) {
        Some(meta) => verdict == if meta.fault.is_some() { 2 } else { 1 },
        None => false,
    }
}

/// The UB Hunt's proof, whole: every station under the lens,
/// every verdict the true one, and every faulted station's
/// card showing its true contract.
pub fn hunt_complete(lensed: &[bool; 6], verdicts: &[u8; 6], cards: &[u8; 6]) -> bool {
    lensed.iter().all(|seen| *seen)
        && crate::content::STATIONS
            .iter()
            .enumerate()
            .all(|(station, meta)| {
                station_verdict_right(station, verdicts[station])
                    && match meta.fault {
                        Some(_) => cards[station] == meta.true_contract,
                        None => true,
                    }
            })
}

/// The Thing's repair card, judged: the beam's card stands
/// rewritten when all three lines show their checkable
/// truths (the key rides with the card's table).
pub fn foe_card_repaired(card: &[u8; 3]) -> bool {
    *card == crate::content::FOE5_CARD_KEY
}

/// A modelled beam under the repair card's discipline: a
/// length, a stated capacity, loans out, and which cells
/// stand filled. The referee's five cases are run against
/// this model, never merely asserted.
#[derive(Clone, Debug)]
struct BeamModel {
    /// Cells the beam currently holds.
    len: usize,
    /// The stated capacity.
    cap: usize,
    /// Shared loans currently out.
    loans: usize,
    /// Which cells stand filled (one entry per cell).
    filled: Vec<bool>,
}

impl BeamModel {
    /// Grow by one cell: refused while any loan is out, and
    /// refused at the stated capacity.
    fn grow(&mut self) -> Result<(), ()> {
        if self.loans > 0 || self.len >= self.cap {
            return Err(());
        }
        self.len += 1;
        self.filled.push(false);
        Ok(())
    }

    /// Read one cell: refused past the end, and refused for
    /// an unfilled cell.
    fn access(&self, index: usize) -> Result<(), ()> {
        if index >= self.len || !self.filled[index] {
            return Err(());
        }
        Ok(())
    }

    /// Drop every cell, last first; the order it fell in.
    fn drop_all(&mut self) -> Vec<usize> {
        let order: Vec<usize> = (0..self.len).rev().collect();
        self.len = 0;
        self.filled.clear();
        order
    }
}

/// The referee's five standing-gate cases, run against the
/// modelled beam: true when the beam behaves exactly as the
/// repair card promises, per case.
pub fn beam_case_holds(case: usize) -> bool {
    match case {
        // Growth while a loan is out: refused.
        0 => {
            let mut beam = BeamModel {
                len: 6,
                cap: 8,
                loans: 1,
                filled: vec![true; 6],
            };
            beam.grow().is_err()
        }
        // Access one past the last cell: refused, not UB.
        1 => {
            let beam = BeamModel {
                len: 6,
                cap: 8,
                loans: 0,
                filled: vec![true; 6],
            };
            beam.access(6).is_err()
        }
        // The drop order: every cell exactly once — none
        // twice, none leaked.
        2 => {
            let mut beam = BeamModel {
                len: 3,
                cap: 8,
                loans: 0,
                filled: vec![true; 3],
            };
            let dropped = beam.drop_all();
            let mut seen = [false; 3];
            dropped.iter().all(|&cell| {
                let fresh = !seen[cell];
                seen[cell] = true;
                fresh
            }) && seen.iter().all(|dropped_cell| *dropped_cell)
        }
        // The empty element: access refused, growth still
        // allowed beside it.
        3 => {
            let mut beam = BeamModel {
                len: 6,
                cap: 8,
                loans: 0,
                filled: vec![true, true, true, true, true, false],
            };
            beam.access(5).is_err() && beam.grow().is_ok()
        }
        // Overflow at the stated capacity: refused.
        4 => {
            let mut beam = BeamModel {
                len: 8,
                cap: 8,
                loans: 0,
                filled: vec![true; 8],
            };
            beam.grow().is_err()
        }
        _ => false,
    }
}
/// Whether a chosen card-line phrasing is the checkable one
/// — the phrasing a sceptic can test before the call.
pub fn card_line_checkable(tool: usize, line: usize, phrasing: u8) -> bool {
    crate::content::CARD_TOOLS
        .get(tool)
        .and_then(|(_, _, key)| key.get(line))
        .map(|key| *key == phrasing)
        .unwrap_or(false)
}

/// How many of the twelve card lines stand checkable.
pub fn card_lines_checkable(card_lines: &[[u8; 3]; 4]) -> usize {
    card_lines
        .iter()
        .enumerate()
        .flat_map(|(tool, lines)| {
            lines
                .iter()
                .enumerate()
                .map(move |(line, phrasing)| (tool, line, *phrasing))
        })
        .filter(|(tool, line, phrasing)| card_line_checkable(*tool, *line, *phrasing))
        .count()
}

/// Whether every line of every card stands checkable.
pub fn cards_all_checkable(card_lines: &[[u8; 3]; 4]) -> bool {
    card_lines_checkable(card_lines) == 12
}

/// The Split Beam itself: the real buffer the chamber works.
/// Its size is the Plumb Line's fact — six words, 24 bytes —
/// and the words are the record's own.
pub const BEAM: [u32; 6] = [2, 5, 1, 4, 2, 3];

/// Whether a split seams the beam soundly: strictly inside,
/// so both halves are non-empty.
pub fn beam_sound_split(split: u8) -> bool {
    (1..=5).contains(&split)
}

/// The beam's two halves at a split; a split at an end
/// leaves the short side empty, which is why the face
/// refuses to seam there.
pub fn beam_halves(split: u8) -> (&'static [u32], &'static [u32]) {
    let at = (split as usize).min(BEAM.len());
    BEAM.split_at(at)
}

#[cfg(test)]
mod floor5_tests {
    use super::*;

    /// Validation: exactly one phrasing per line is checkable,
    /// and it is the key of record.
    #[test]
    fn card_key_is_exactly_one_per_line() {
        for (tool, (_, _, key)) in crate::content::CARD_TOOLS.iter().enumerate() {
            for (line, key_phrasing) in key.iter().enumerate() {
                let checkable: Vec<u8> = (0..3)
                    .filter(|phrasing| card_line_checkable(tool, line, *phrasing))
                    .collect();
                assert_eq!(checkable, vec![*key_phrasing], "tool {tool} line {line}");
            }
        }
    }

    /// Validation: the key itself, set on every line, scores
    /// twelve and proves the rack.
    #[test]
    fn card_key_scores_twelve() {
        let mut card_lines = [[0u8; 3]; 4];
        for (tool, (_, _, key)) in crate::content::CARD_TOOLS.iter().enumerate() {
            card_lines[tool] = *key;
        }
        assert_eq!(card_lines_checkable(&card_lines), 12);
        assert!(cards_all_checkable(&card_lines));
    }

    /// Adversarial: the rack as found — every line at its
    /// first phrasing — does not prove; vapour stands on it.
    #[test]
    fn card_default_rack_is_not_checkable() {
        let card_lines = [[0u8; 3]; 4];
        assert!(!cards_all_checkable(&card_lines));
        assert!(card_lines_checkable(&card_lines) < 12);
        assert!(!card_line_checkable(9, 0, 0), "no ninth tool");
        assert!(!card_line_checkable(0, 9, 0), "no ninth line");
    }

    /// Adversarial: one vapour line anywhere unmakes the
    /// whole rack — the sceptic reads cards, not averages.
    #[test]
    fn card_one_vapour_line_unmakes_it() {
        let mut card_lines = [[0u8; 3]; 4];
        for (tool, (_, _, key)) in crate::content::CARD_TOOLS.iter().enumerate() {
            card_lines[tool] = *key;
        }
        card_lines[2][0] = (card_lines[2][0] + 1) % 3;
        assert_eq!(card_lines_checkable(&card_lines), 11);
        assert!(!cards_all_checkable(&card_lines));
    }

    /// Validation: the beam is the beam of record, and the
    /// Plumb Line's fact about it is real.
    #[test]
    fn beam_facts_are_real() {
        assert_eq!(BEAM, [2, 5, 1, 4, 2, 3]);
        assert_eq!(std::mem::size_of_val(&BEAM), 24);
        let (left, right) = beam_halves(3);
        assert_eq!(left, &[2, 5, 1]);
        assert_eq!(right, &[4, 2, 3]);
    }

    /// Adversarial: the beam's ends are not seams — a seam
    /// there leaves a half with nothing in it.
    #[test]
    fn beam_ends_are_not_seams() {
        assert!(!beam_sound_split(0));
        assert!(!beam_sound_split(6));
        assert!((1..=5).all(beam_sound_split));
        assert!(beam_halves(0).0.is_empty());
        assert!(beam_halves(6).1.is_empty());
    }
    /// Validation: exactly one of the six stones is cracked,
    /// and it is the fourth.
    #[test]
    fn exactly_one_step_is_cracked() {
        let cracked: Vec<usize> = (0..6).filter(|step| step_cracked(*step)).collect();
        assert_eq!(cracked, vec![3]);
        assert!(!step_cracked(6), "no seventh step");
    }

    /// Validation: a perfect set of calls scores eight.
    #[test]
    fn tenancy_perfect_calls_score_eight() {
        let mut predictions = [0u8; 8];
        for (index, (_, sound, _)) in crate::content::TENANCIES.iter().enumerate() {
            predictions[index] = if *sound { 1 } else { 2 };
        }
        assert_eq!(tenancy_correct(&predictions), 8);
        assert!((0..8).all(|index| !tenancy_missed(&predictions, index)));
    }

    /// Adversarial: uncalled placards score nothing and miss
    /// nothing — the referee cannot be rushed by silence.
    #[test]
    fn tenancy_uncalled_scores_zero() {
        let predictions = [0u8; 8];
        assert_eq!(tenancy_correct(&predictions), 0);
        assert!((0..8).all(|index| !tenancy_missed(&predictions, index)));
    }

    /// Adversarial: an inverted call is a miss, and only the
    /// inverted placard misses.
    #[test]
    fn tenancy_inverted_call_misses_alone() {
        let mut predictions = [0u8; 8];
        for (index, (_, sound, _)) in crate::content::TENANCIES.iter().enumerate() {
            predictions[index] = if *sound { 1 } else { 2 };
        }
        predictions[1] = 1; // the repainting writer, called sound
        assert_eq!(tenancy_correct(&predictions), 7);
        assert!(tenancy_missed(&predictions, 1));
        assert!(!tenancy_missed(&predictions, 0));
    }

    /// Validation: exactly three stations are faulted, and
    /// each station's one right verdict agrees with whether
    /// it carries a fault; the verdict 0 is right nowhere.
    #[test]
    fn station_truth_table_is_exact() {
        let mut faulted = 0;
        for (station, meta) in crate::content::STATIONS.iter().enumerate() {
            if meta.fault.is_some() {
                faulted += 1;
                assert!(station_verdict_right(station, 2));
                assert!(!station_verdict_right(station, 1));
            } else {
                assert!(station_verdict_right(station, 1));
                assert!(!station_verdict_right(station, 2));
            }
            assert!(!station_verdict_right(station, 0));
        }
        assert_eq!(faulted, 3);
        assert!(!station_verdict_right(6, 1));
    }

    /// Adversarial: the hunt is unmade by any single omission
    /// — an unlensed station, one flipped verdict, one false
    /// contract on a faulted station's card.
    #[test]
    fn hunt_unmade_by_any_single_omission() {
        let lensed = [true; 6];
        let verdicts = [1, 2, 2, 1, 2, 1];
        let cards = [0, 2, 0, 0, 1, 0];
        assert!(hunt_complete(&lensed, &verdicts, &cards));
        let mut dim = lensed;
        dim[3] = false;
        assert!(!hunt_complete(&dim, &verdicts, &cards));
        let mut wrong = verdicts;
        wrong[0] = 2;
        assert!(!hunt_complete(&lensed, &wrong, &cards));
        let mut unentered = verdicts;
        unentered[5] = 0;
        assert!(!hunt_complete(&lensed, &unentered, &cards));
        for station in [1, 2, 4] {
            let mut bent = cards;
            bent[station] = (bent[station] + 1) % 3;
            assert!(
                !hunt_complete(&lensed, &verdicts, &bent),
                "station {station}'s false contract must unmake the hunt"
            );
        }
    }

    /// Validation: the beam's card as found is not its
    /// repair; the key repairs it.
    #[test]
    fn foe_card_found_is_not_repaired() {
        assert!(!foe_card_repaired(&[0, 0, 0]));
        assert!(foe_card_repaired(&crate::content::FOE5_CARD_KEY));
        assert!(!foe_card_repaired(&[1, 2, 0]));
    }

    /// Validation: all five of the referee's cases hold
    /// against the modelled beam.
    #[test]
    fn referee_cases_all_hold() {
        for case in 0..5 {
            assert!(beam_case_holds(case), "case {case} must hold");
        }
        assert!(!beam_case_holds(5));
    }
}

// ---------------------------------------------------------------------------
// Floor 6 — the Lockheart: the Cell Heart and the Borrow-Flag
// Brazier. Both chambers run on real machines, not tables:
// the flag machine (Free / Shared(n) / Exclusive) decides
// every opening and closing, and the brazier is a lazy cell
// that kindles once no matter how many call. The census on
// Floor 6 and the FOE's broken staff reuse the flag machine.
// ---------------------------------------------------------------------------

/// The flag machine's state: one heart, one occupant rule.
/// Shared carries its reader count — the count is part of
/// the state, never a side tally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagState {
    /// No hand on the heart.
    Free,
    /// This many readers inside.
    Shared(u8),
    /// One hand, alone.
    Exclusive,
}

/// One operation on the flag machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagOp {
    /// Open shared: join the readers.
    OpenShared,
    /// Close shared: a reader leaves.
    CloseShared,
    /// Open exclusive: take the heart alone — lawful only
    /// from Free.
    OpenExclusive,
    /// Close exclusive: the lone hand leaves.
    CloseExclusive,
}

/// Decode a content operation code (see CELL_DAY).
pub fn flag_op(code: u8) -> Option<FlagOp> {
    match code {
        0 => Some(FlagOp::OpenShared),
        1 => Some(FlagOp::CloseShared),
        2 => Some(FlagOp::OpenExclusive),
        3 => Some(FlagOp::CloseExclusive),
        _ => None,
    }
}

/// One transition of the flag machine. A refused operation
/// returns None and the state stands unchanged — the
/// machine never half-moves.
pub fn flag_step(state: FlagState, op: FlagOp) -> Option<FlagState> {
    match (state, op) {
        (FlagState::Free, FlagOp::OpenShared) => Some(FlagState::Shared(1)),
        (FlagState::Shared(n), FlagOp::OpenShared) => Some(FlagState::Shared(n + 1)),
        (FlagState::Shared(1), FlagOp::CloseShared) => Some(FlagState::Free),
        (FlagState::Shared(n), FlagOp::CloseShared) => Some(FlagState::Shared(n - 1)),
        (FlagState::Free, FlagOp::OpenExclusive) => Some(FlagState::Exclusive),
        (FlagState::Exclusive, FlagOp::CloseExclusive) => Some(FlagState::Free),
        _ => None,
    }
}

/// Replay the Cell Heart's day over the walked prefix:
/// returns the state the heart stands in and the index of
/// a walked operation the machine refused, if any. The day
/// is fixed in content; only the walked flags vary.
pub fn cell_day_replay(walked: &[bool; 6]) -> (FlagState, Option<usize>) {
    let mut state = FlagState::Free;
    let mut refused = None;
    for (index, (_, _, code)) in crate::content::CELL_DAY.iter().enumerate() {
        if !walked[index] {
            continue;
        }
        let Some(op) = flag_op(*code) else {
            continue;
        };
        match flag_step(state, op) {
            Some(next) => state = next,
            None => {
                if refused.is_none() {
                    refused = Some(index);
                }
            }
        }
    }
    (state, refused)
}

/// The words for a flag state, for the chamber face and the
/// day's account.
pub fn flag_state_name(state: FlagState) -> &'static str {
    match state {
        FlagState::Free => "free — no hand on it",
        FlagState::Shared(1) => "shared — one reader inside",
        FlagState::Shared(_) => "shared — readers inside",
        FlagState::Exclusive => "exclusive — one hand, alone",
    }
}

/// Replay the brazier's calls over the walked prefix: the
/// brazier is a lazy cell — the first call for the flame
/// kindles it and raises the light-count to one; every
/// later call, for the flame or to read, finds it lit and
/// kindles nothing. Returns whether it stands lit and how
/// many times it has been kindled. Calls 0, 1, and 3 call
/// for the flame; calls 2 and 4 only read.
pub fn brazier_replay(calls: &[bool; 5]) -> (bool, u32) {
    let mut lit = false;
    let mut light_count: u32 = 0;
    for (index, walked) in calls.iter().enumerate() {
        if !walked {
            continue;
        }
        let calls_for_flame = matches!(index, 0 | 1 | 3);
        if calls_for_flame && !lit {
            lit = true;
            light_count += 1;
        }
    }
    (lit, light_count)
}

#[cfg(test)]
mod floor6_tests {
    use super::*;

    /// Validation: the machine's lawful transitions.
    #[test]
    fn flag_machine_lawful_transitions() {
        assert_eq!(
            flag_step(FlagState::Free, FlagOp::OpenShared),
            Some(FlagState::Shared(1))
        );
        assert_eq!(
            flag_step(FlagState::Shared(1), FlagOp::OpenShared),
            Some(FlagState::Shared(2))
        );
        assert_eq!(
            flag_step(FlagState::Shared(2), FlagOp::CloseShared),
            Some(FlagState::Shared(1))
        );
        assert_eq!(
            flag_step(FlagState::Shared(1), FlagOp::CloseShared),
            Some(FlagState::Free)
        );
        assert_eq!(
            flag_step(FlagState::Free, FlagOp::OpenExclusive),
            Some(FlagState::Exclusive)
        );
        assert_eq!(
            flag_step(FlagState::Exclusive, FlagOp::CloseExclusive),
            Some(FlagState::Free)
        );
    }

    /// Adversarial: every unlawful transition is refused and
    /// the state stands unchanged — most of all, exclusive
    /// never opens over a held share.
    #[test]
    fn flag_machine_refusals() {
        for state in [
            FlagState::Shared(1),
            FlagState::Shared(3),
            FlagState::Exclusive,
        ] {
            assert_eq!(flag_step(state, FlagOp::OpenExclusive), None);
        }
        assert_eq!(flag_step(FlagState::Free, FlagOp::CloseShared), None);
        assert_eq!(flag_step(FlagState::Free, FlagOp::CloseExclusive), None);
        assert_eq!(flag_step(FlagState::Exclusive, FlagOp::OpenShared), None);
        assert_eq!(flag_step(FlagState::Exclusive, FlagOp::CloseShared), None);
        assert_eq!(
            flag_step(FlagState::Shared(2), FlagOp::CloseExclusive),
            None
        );
    }

    /// Validation: the full day ends exclusive, and the one
    /// refusal is the fourth operation.
    #[test]
    fn cell_day_full_walk() {
        let (state, refused) = cell_day_replay(&[true; 6]);
        assert_eq!(state, FlagState::Exclusive);
        assert_eq!(refused, Some(3));
    }

    /// Adversarial: a partial day refuses nothing early —
    /// the first three operations are all lawful.
    #[test]
    fn cell_day_partial_walk_refuses_nothing() {
        let (state, refused) = cell_day_replay(&[true, true, true, false, false, false]);
        assert_eq!(state, FlagState::Shared(1));
        assert_eq!(refused, None);
        let (state, refused) = cell_day_replay(&[false; 6]);
        assert_eq!(state, FlagState::Free);
        assert_eq!(refused, None);
    }

    /// Validation: five calls, one kindling — the lazy cell
    /// lights once no matter how many call.
    #[test]
    fn brazier_five_calls_one_light() {
        assert_eq!(brazier_replay(&[true; 5]), (true, 1));
    }

    /// Adversarial: reads alone never kindle, and no calls
    /// leave the brazier dark.
    #[test]
    fn brazier_reads_never_kindle() {
        assert_eq!(
            brazier_replay(&[false, false, true, false, true]),
            (false, 0)
        );
        assert_eq!(brazier_replay(&[false; 5]), (false, 0));
        assert_eq!(
            brazier_replay(&[true, false, false, false, false]),
            (true, 1)
        );
    }
}

// ---------------------------------------------------------------------------
// Floor 6 — the Lockheart: the Spin Heart and the Day
// Census. The spin heart is a bounded loop against a flag
// the world provides; the census is a reachable-state
// enumeration over the flag machine — turn by turn, every
// ordering of a turn's events explored, refused operations
// standing as self-loops. The judge's truth is computed
// here, never written in content.
// ---------------------------------------------------------------------------

/// Run the spin heart: attempt after attempt, up to the
/// bound, watching for the flag. The flag's state is the
/// world's to give — in this chamber the setter is another
/// porter's work and never comes, so an honest heart gives
/// up at its bound. Returns the attempts made and whether
/// the flag was acquired.
pub fn spin_heart_run(bound: u8, flag_set: bool) -> (u8, bool) {
    let mut attempts: u8 = 0;
    while attempts < bound {
        attempts += 1;
        if flag_set {
            return (attempts, true);
        }
    }
    (attempts, false)
}

/// Add a state to a census set if it is not already there.
fn push_state(set: &mut Vec<FlagState>, state: FlagState) {
    if !set.contains(&state) {
        set.push(state);
    }
}

/// Walk one turn from one standing state: the turn's events
/// happen in every order; every state stood in joins the
/// visited set, and every state the turn can end in joins
/// the finals. Refused operations are self-loops — the day
/// stands where it stood.
fn walk_turn(
    state: FlagState,
    events: &[FlagOp],
    used: &mut [bool],
    left: usize,
    visited: &mut Vec<FlagState>,
    finals: &mut Vec<FlagState>,
) {
    if left == 0 {
        push_state(finals, state);
        return;
    }
    for index in 0..events.len() {
        if used[index] {
            continue;
        }
        used[index] = true;
        let next = flag_step(state, events[index]).unwrap_or(state);
        push_state(visited, next);
        walk_turn(next, events, used, left - 1, visited, finals);
        used[index] = false;
    }
}

/// Enumerate a day of turns from Free: the census of
/// distinct reachable states, [free, shared, exclusive],
/// counting each distinct Shared(n) as its own state.
fn census_of_turns(turns: &[Vec<FlagOp>]) -> [u8; 3] {
    let mut visited = vec![FlagState::Free];
    let mut frontier = vec![FlagState::Free];
    for turn in turns {
        let mut finals = Vec::new();
        for start in &frontier {
            let mut used = vec![false; turn.len()];
            walk_turn(
                *start,
                turn,
                &mut used,
                turn.len(),
                &mut visited,
                &mut finals,
            );
        }
        frontier = finals;
    }
    let mut census = [0u8; 3];
    for state in &visited {
        match state {
            FlagState::Free => census[0] += 1,
            FlagState::Shared(_) => census[1] += 1,
            FlagState::Exclusive => census[2] += 1,
        }
    }
    census
}

/// The Day Census's truth: the authored day (content's
/// CENSUS_EVENTS, grouped by turn) enumerated over the
/// flag machine.
pub fn day_census() -> [u8; 3] {
    let mut turns: Vec<Vec<FlagOp>> = Vec::new();
    for (turn, _, code) in crate::content::CENSUS_EVENTS {
        let index = (turn as usize).saturating_sub(1);
        while turns.len() <= index {
            turns.push(Vec::new());
        }
        if let Some(op) = flag_op(code) {
            turns[index].push(op);
        }
    }
    census_of_turns(&turns)
}

#[cfg(test)]
mod floor6_census_tests {
    use super::*;

    /// Validation: an unset flag is never acquired, and the
    /// attempts stop exactly at the bound — the give-up is
    /// honest and counted.
    #[test]
    fn spin_heart_gives_up_at_the_bound() {
        assert_eq!(spin_heart_run(9, false), (9, false));
        assert_eq!(spin_heart_run(1, false), (1, false));
        assert_eq!(spin_heart_run(0, false), (0, false));
    }

    /// Validation: a set flag is acquired on the first
    /// attempt — the loop is a real check, not a fixed
    /// refusal.
    #[test]
    fn spin_heart_acquires_a_set_flag() {
        assert_eq!(spin_heart_run(9, true), (1, true));
    }

    /// Validation: the authored day's census, enumerated —
    /// free once, shared at one, two, and three readers,
    /// exclusive once.
    #[test]
    fn day_census_enumerates_the_authored_day() {
        assert_eq!(day_census(), [1, 3, 1]);
    }

    /// Adversarial: a refused operation is a self-loop, not
    /// a door — a day of one barred closing reaches nothing
    /// but Free, and an exclusive opening from Free is a
    /// real reach.
    #[test]
    fn census_refusals_are_self_loops() {
        assert_eq!(census_of_turns(&[vec![FlagOp::CloseShared]]), [1, 0, 0]);
        assert_eq!(census_of_turns(&[vec![FlagOp::OpenExclusive]]), [1, 0, 1]);
        assert_eq!(
            census_of_turns(&[vec![FlagOp::OpenShared], vec![FlagOp::OpenExclusive]]),
            [1, 1, 0],
            "exclusive never opens over a held share, in any order"
        );
    }
}

// ---------------------------------------------------------------------------
// Floor 6 — Choosing the Cell and the Heart Locked Twice.
// The chooser's judge reads content's keys. The FOE runs on
// the BROKEN staff: a flag machine whose exclusive opens
// over standing readers and whose reader count drifted off
// the state — producing the fused state, a heart shared and
// exclusive at once. The staff's census is enumerated, its
// worked day replayed, its torn read computed, and the
// model-loom judges four interleavings by running them.
// ---------------------------------------------------------------------------

/// Whether the three data stand housed true: each datum in
/// its true kind, each door carrying its true sentence.
/// Choices and invariants are recorded as index + 1, with
/// 0 unrecorded.
pub fn choosing_complete(choices: &[u8; 3], invariants: &[u8; 3]) -> bool {
    crate::content::CELL_DATA
        .iter()
        .enumerate()
        .all(|(index, datum)| {
            choices[index] == datum.true_kind + 1 && invariants[index] == datum.true_sentence + 1
        })
}

/// The broken staff's state: the flag machine's three
/// lawful states, and the fused fourth — exclusive held
/// while readers stand inside — that the flaw admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaffState {
    /// No hand on the heart.
    Free,
    /// This many readers inside.
    Shared(u8),
    /// One hand, alone.
    Exclusive,
    /// The flaw's child: a writer inside over this many
    /// readers.
    Fused(u8),
}

/// The reader cap the staff's census enumerates to — the
/// warren fields three porters, and the census counts the
/// staff's own reach, not an infinity.
pub const STAFF_READER_CAP: u8 = 3;

/// One transition of the broken staff. The staff never
/// refuses — that is the flaw's shape: every operation
/// "succeeds", and exclusive over standing readers lands
/// in the fused state instead of at the face.
pub fn staff_step(state: StaffState, op: FlagOp) -> StaffState {
    match (state, op) {
        (StaffState::Free, FlagOp::OpenShared) => StaffState::Shared(1),
        (StaffState::Shared(n), FlagOp::OpenShared) => StaffState::Shared(n + 1),
        (StaffState::Exclusive, FlagOp::OpenShared) => StaffState::Fused(1),
        (StaffState::Fused(n), FlagOp::OpenShared) => StaffState::Fused(n + 1),
        // THE FLAW: exclusive, issued over standing readers.
        (StaffState::Shared(n), FlagOp::OpenExclusive) => StaffState::Fused(n),
        (StaffState::Free, FlagOp::OpenExclusive) => StaffState::Exclusive,
        (StaffState::Exclusive, FlagOp::OpenExclusive) => StaffState::Exclusive,
        (StaffState::Fused(n), FlagOp::OpenExclusive) => StaffState::Fused(n),
        (StaffState::Shared(1), FlagOp::CloseShared) => StaffState::Free,
        (StaffState::Shared(n), FlagOp::CloseShared) => StaffState::Shared(n - 1),
        (StaffState::Fused(1), FlagOp::CloseShared) => StaffState::Exclusive,
        (StaffState::Fused(n), FlagOp::CloseShared) => StaffState::Fused(n - 1),
        (StaffState::Exclusive, FlagOp::CloseExclusive) => StaffState::Free,
        (StaffState::Fused(n), FlagOp::CloseExclusive) => StaffState::Shared(n),
        _ => state,
    }
}

/// The words for a staff state, for the exhibits' accounts.
pub fn staff_state_name(state: StaffState) -> String {
    match state {
        StaffState::Free => "free".to_string(),
        StaffState::Shared(n) => format!("shared({n})"),
        StaffState::Exclusive => "exclusive".to_string(),
        StaffState::Fused(n) => format!("FUSED({n})"),
    }
}

/// The staff's census: every state the broken staff can
/// reach from Free, breadth-first to the fixpoint, reader
/// counts capped at the warren's three porters. Counted
/// [free, shared, exclusive, fused] over distinct states.
pub fn staff_census() -> [u8; 4] {
    let ops = [
        FlagOp::OpenShared,
        FlagOp::CloseShared,
        FlagOp::OpenExclusive,
        FlagOp::CloseExclusive,
    ];
    let mut visited = vec![StaffState::Free];
    let mut frontier = vec![StaffState::Free];
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for state in &frontier {
            for op in ops {
                let stepped = staff_step(*state, op);
                let over_cap = match stepped {
                    StaffState::Shared(n) | StaffState::Fused(n) => n > STAFF_READER_CAP,
                    _ => false,
                };
                if !over_cap && !visited.contains(&stepped) {
                    visited.push(stepped);
                    next.push(stepped);
                }
            }
        }
        frontier = next;
    }
    let mut census = [0u8; 4];
    for state in &visited {
        match state {
            StaffState::Free => census[0] += 1,
            StaffState::Shared(_) => census[1] += 1,
            StaffState::Exclusive => census[2] += 1,
            StaffState::Fused(_) => census[3] += 1,
        }
    }
    census
}

/// The worked day: two shared openings, then the writer
/// comes — replayed on the broken staff, which lets her
/// in. The day ends in the fused state.
pub fn staff_worked_day() -> StaffState {
    let mut state = StaffState::Free;
    for op in [
        FlagOp::OpenShared,
        FlagOp::OpenShared,
        FlagOp::OpenExclusive,
    ] {
        state = staff_step(state, op);
    }
    state
}

/// The torn reading, computed word by word: the heart's
/// value is two words of one generation. The writer lands
/// the new day's first word; the reader reads both words
/// before the second lands. Returns the pair the reader
/// actually saw — two generations in one reading.
pub fn staff_torn_read() -> (u32, u32) {
    let mut words = (1u32, 1u32);
    words.0 = 2;
    let seen = (words.0, words.1);
    words.1 = 2;
    let _finished_too_late = words;
    seen
}

/// One step on the model-loom: a porter writes one word of
/// the heart's two-word value, or reads one word of it.
#[derive(Clone, Copy, Debug)]
enum LoomStep {
    /// The writer lands the value's first word, of a day.
    WriteFirst(u32),
    /// The writer lands the value's second word, of a day.
    WriteSecond(u32),
    /// The reader takes the value's first word.
    ReadFirst,
    /// The reader takes the value's second word.
    ReadSecond,
}

/// The loom's four cases, as step tables (the case names
/// and expected verdicts live in content's FOE6_CASES; the
/// loom computes, the table is checked against it).
const LOOM_CASES: [&[LoomStep]; 4] = [
    // The Courteous Pair: both words land, then the read.
    &[
        LoomStep::WriteFirst(2),
        LoomStep::WriteSecond(2),
        LoomStep::ReadFirst,
        LoomStep::ReadSecond,
    ],
    // The Torn Letter: the read straddles the two words.
    &[
        LoomStep::WriteFirst(2),
        LoomStep::ReadFirst,
        LoomStep::ReadSecond,
        LoomStep::WriteSecond(2),
    ],
    // The Patient Porter: the read completes first.
    &[
        LoomStep::ReadFirst,
        LoomStep::ReadSecond,
        LoomStep::WriteFirst(2),
        LoomStep::WriteSecond(2),
    ],
    // The Second Reader: the second word turns first, and
    // the read straddles it the other way.
    &[
        LoomStep::WriteSecond(2),
        LoomStep::ReadFirst,
        LoomStep::WriteFirst(2),
        LoomStep::ReadSecond,
    ],
];

/// Run one loom case: the heart starts at day one, both
/// words; the steps land in order; the reader's pair is
/// judged torn (its two words of different days) or sound.
/// None for no such case.
pub fn loom_case_torn(case: usize) -> Option<bool> {
    let steps = LOOM_CASES.get(case)?;
    let mut words = (1u32, 1u32);
    let mut read = (0u32, 0u32);
    for step in *steps {
        match step {
            LoomStep::WriteFirst(day) => words.0 = *day,
            LoomStep::WriteSecond(day) => words.1 = *day,
            LoomStep::ReadFirst => read.0 = words.0,
            LoomStep::ReadSecond => read.1 = words.1,
        }
    }
    Some(read.0 != read.1)
}

/// Whether the staff's rewritten card shows its three
/// checkable truths.
pub fn staff_card_repaired(lines: &[u8; 3]) -> bool {
    *lines == crate::content::FOE6_CARD_KEY
}

#[cfg(test)]
mod floor6_foe_tests {
    use super::*;

    /// Validation: the chooser's key — the charter in the
    /// many-readers heart, the tally in the borrow-flag
    /// heart, the log in the plain cell, each under its
    /// true sentence.
    #[test]
    fn choosing_key_completes() {
        assert!(choosing_complete(&[2, 3, 1], &[3, 2, 2]));
    }

    /// Adversarial: one wrong housing or one wrong sentence
    /// fails the whole table — and the tempting swap (the
    /// charter and the log exchanged) fails twice over.
    #[test]
    fn choosing_misses_fail() {
        assert!(!choosing_complete(&[2, 3, 1], &[3, 2, 1]));
        assert!(!choosing_complete(&[1, 3, 2], &[3, 2, 2]));
        assert!(!choosing_complete(&[0, 3, 1], &[3, 2, 2]));
    }

    /// Validation: the broken staff's census — eight states,
    /// three of them fused, a state no lawful staff admits.
    #[test]
    fn staff_census_counts_the_fused() {
        assert_eq!(staff_census(), [1, 3, 1, 3]);
    }

    /// Validation: the worked day ends fused — the writer
    /// let in over two standing readers.
    #[test]
    fn staff_worked_day_ends_fused() {
        assert_eq!(staff_worked_day(), StaffState::Fused(2));
        // And the lawful machine refuses the same day at the
        // same door: the difference is the staff, not the day.
        let mut state = FlagState::Free;
        for op in [
            FlagOp::OpenShared,
            FlagOp::OpenShared,
            FlagOp::OpenExclusive,
        ] {
            state = flag_step(state, op).unwrap_or(state);
        }
        assert_eq!(state, FlagState::Shared(2));
    }

    /// Validation: the torn reading mixes two days in one
    /// pair — computed, not asserted into existence.
    #[test]
    fn staff_torn_read_mixes_days() {
        assert_eq!(staff_torn_read(), (2, 1));
    }

    /// Validation, fail-closed: the loom's computed verdict
    /// for every case equals the verdict content records —
    /// if a step table and its record ever part, this fails.
    #[test]
    fn loom_verdicts_match_their_records() {
        for (case, (_, expected, _)) in crate::content::FOE6_CASES.iter().enumerate() {
            assert_eq!(loom_case_torn(case), Some(*expected), "case {case}");
        }
        assert_eq!(loom_case_torn(4), None);
    }

    /// Validation: the card's key repairs; a near-miss does
    /// not.
    #[test]
    fn foe_card_key_judged() {
        assert!(staff_card_repaired(&crate::content::FOE6_CARD_KEY));
        assert!(!staff_card_repaired(&[1, 2, 1]));
        assert!(!staff_card_repaired(&[0, 0, 0]));
    }
}

// ---------------------------------------------------------------------------
// The model loom (Floor 7's system, frozen for Part B):
// authored thread programs as op lists, explored over every
// interleaving in deterministic, lexicographic-by-thread-
// choice order, up to a stated cap. Deterministic, bounded,
// no RNG. Consumed by this floor's chambers and FOE, and by
// Floor 16's whole-cloth assay.
// ---------------------------------------------------------------------------

/// The model loom: thread programs, interleavings, deadlock.
pub mod loom {
    use std::collections::BTreeSet;

    /// One operation in an authored thread program, over
    /// named (indexed) channels and claimables.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum LoomOp {
        /// Put one message on a channel. Never blocks.
        Send(u8),
        /// Take one message from a channel. Blocks while
        /// the channel stands empty.
        Recv(u8),
        /// Take a claimable. Blocks while another thread
        /// holds it.
        Claim(u8),
        /// Give a claimable back. Never blocks; releasing
        /// what the thread does not hold is a no-op.
        Release(u8),
        /// Wait for a thread to finish. Blocks until the
        /// named thread's program has run out.
        Join(u8),
    }

    /// The loom's report on one exploration.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct LoomReport {
        /// Terminal paths visited (completed or
        /// deadlocked), never more than the cap.
        pub interleavings_tested: u64,
        /// The first deadlocking interleaving found, as
        /// the thread-choice trace that reached it, in
        /// exploration order. None if no deadlock was
        /// reached within the cap.
        pub deadlock: Option<Vec<u8>>,
        /// The final channel-state signatures of the
        /// completed interleavings: one count per channel,
        /// in channel order.
        pub outcomes: BTreeSet<Vec<u8>>,
    }

    /// The loom's working state for one interleaving.
    #[derive(Clone)]
    struct LoomState {
        /// Each thread's next operation index.
        pcs: Vec<usize>,
        /// Messages standing on each channel.
        channels: Vec<u8>,
        /// Each claimable's holder, if any.
        holders: Vec<Option<usize>>,
    }

    /// Whether one thread can step from this state, and
    /// the state after its step. None: the thread is
    /// finished, or its next operation blocks here.
    fn step(programs: &[Vec<LoomOp>], state: &LoomState, thread: usize) -> Option<LoomState> {
        let pc = state.pcs[thread];
        let op = programs.get(thread)?.get(pc)?;
        let mut next = state.clone();
        match *op {
            LoomOp::Send(channel) => {
                next.channels[channel as usize] = next.channels[channel as usize].saturating_add(1);
            }
            LoomOp::Recv(channel) => {
                if next.channels[channel as usize] == 0 {
                    return None;
                }
                next.channels[channel as usize] -= 1;
            }
            LoomOp::Claim(claimable) => match next.holders[claimable as usize] {
                None => next.holders[claimable as usize] = Some(thread),
                Some(holder) if holder == thread => {}
                Some(_) => return None,
            },
            LoomOp::Release(claimable) => {
                if next.holders[claimable as usize] == Some(thread) {
                    next.holders[claimable as usize] = None;
                }
            }
            LoomOp::Join(other) => {
                let other = other as usize;
                let finished = programs
                    .get(other)
                    .is_some_and(|program| state.pcs[other] >= program.len());
                if !finished {
                    return None;
                }
            }
        }
        next.pcs[thread] += 1;
        Some(next)
    }

    /// Depth-first exploration, thread choices in index
    /// order — the lexicographic order of record.
    #[allow(clippy::too_many_arguments)]
    fn walk(
        programs: &[Vec<LoomOp>],
        state: &LoomState,
        trace: &mut Vec<u8>,
        cap: u64,
        tested: &mut u64,
        deadlock: &mut Option<Vec<u8>>,
        outcomes: &mut BTreeSet<Vec<u8>>,
    ) {
        if *tested >= cap {
            return;
        }
        let mut moved = false;
        let mut finished_all = true;
        for thread in 0..programs.len() {
            if state.pcs[thread] < programs[thread].len() {
                finished_all = false;
            }
            if let Some(next) = step(programs, state, thread) {
                moved = true;
                trace.push(thread as u8);
                walk(programs, &next, trace, cap, tested, deadlock, outcomes);
                trace.pop();
                if *tested >= cap {
                    return;
                }
            }
        }
        if finished_all {
            *tested += 1;
            outcomes.insert(state.channels.clone());
        } else if !moved {
            *tested += 1;
            if deadlock.is_none() {
                *deadlock = Some(trace.clone());
            }
        }
    }

    /// Explore the interleavings of the given thread
    /// programs, in deterministic order, visiting at most
    /// `cap` terminal paths.
    pub fn explore(programs: &[Vec<LoomOp>], cap: u64) -> LoomReport {
        let mut channels = 0usize;
        let mut claimables = 0usize;
        for program in programs {
            for op in program {
                match *op {
                    LoomOp::Send(channel) | LoomOp::Recv(channel) => {
                        channels = channels.max(channel as usize + 1);
                    }
                    LoomOp::Claim(claimable) | LoomOp::Release(claimable) => {
                        claimables = claimables.max(claimable as usize + 1);
                    }
                    LoomOp::Join(_) => {}
                }
            }
        }
        let state = LoomState {
            pcs: vec![0; programs.len()],
            channels: vec![0; channels],
            holders: vec![None; claimables],
        };
        let mut tested = 0u64;
        let mut deadlock = None;
        let mut outcomes = BTreeSet::new();
        let mut trace = Vec::new();
        if cap > 0 {
            walk(
                programs,
                &state,
                &mut trace,
                cap,
                &mut tested,
                &mut deadlock,
                &mut outcomes,
            );
        }
        LoomReport {
            interleavings_tested: tested,
            deadlock,
            outcomes,
        }
    }

    #[cfg(test)]
    mod loom_tests {
        use super::*;

        /// Validation: no threads is one empty
        /// interleaving, completing with no channels.
        #[test]
        fn empty_programs_complete_once() {
            let report = explore(&[], 16);
            assert_eq!(report.interleavings_tested, 1);
            assert_eq!(report.deadlock, None);
            assert_eq!(report.outcomes.len(), 1);
            assert!(report.outcomes.contains(&Vec::<u8>::new()));
        }

        /// Validation: two senders interleave two ways;
        /// both complete with two messages standing.
        #[test]
        fn two_senders_two_interleavings() {
            let programs = vec![vec![LoomOp::Send(0)], vec![LoomOp::Send(0)]];
            let report = explore(&programs, 16);
            assert_eq!(report.interleavings_tested, 2);
            assert_eq!(report.deadlock, None);
            assert_eq!(report.outcomes.len(), 1);
            assert!(report.outcomes.contains(&vec![2]));
        }

        /// Adversarial: two porters claiming in opposite
        /// orders, neither ever releasing, can only
        /// deadlock: every interleaving strands someone.
        /// The first trace in the loom's order is thread
        /// 0 taking both stairs while thread 1 waits.
        #[test]
        fn opposite_claims_deadlock() {
            let programs = vec![
                vec![LoomOp::Claim(0), LoomOp::Claim(1)],
                vec![LoomOp::Claim(1), LoomOp::Claim(0)],
            ];
            let report = explore(&programs, 64);
            assert_eq!(report.interleavings_tested, 4);
            assert_eq!(report.deadlock, Some(vec![0, 0]));
            assert!(report.outcomes.is_empty(), "no order completes");
            let again = explore(&programs, 64);
            assert_eq!(report, again, "the loom is deterministic");
        }

        /// Adversarial: with releases, opposite orders
        /// still deadlock on some interleaving, though
        /// courteous orders complete — the cycle is in
        /// the shape, not in every day.
        #[test]
        fn opposite_orders_with_releases_still_deadlock() {
            let programs = vec![
                vec![
                    LoomOp::Claim(0),
                    LoomOp::Claim(1),
                    LoomOp::Release(1),
                    LoomOp::Release(0),
                ],
                vec![
                    LoomOp::Claim(1),
                    LoomOp::Claim(0),
                    LoomOp::Release(0),
                    LoomOp::Release(1),
                ],
            ];
            let report = explore(&programs, 4096);
            assert!(
                report.deadlock.is_some(),
                "some interleaving closes the cycle"
            );
            assert!(
                !report.outcomes.is_empty(),
                "courteous interleavings complete"
            );
        }

        /// Validation: one total claim order admits no
        /// deadlock — the shape the FOE's repair posts.
        #[test]
        fn one_order_never_deadlocks() {
            let programs = vec![
                vec![
                    LoomOp::Claim(0),
                    LoomOp::Claim(1),
                    LoomOp::Release(1),
                    LoomOp::Release(0),
                ],
                vec![
                    LoomOp::Claim(0),
                    LoomOp::Claim(1),
                    LoomOp::Release(1),
                    LoomOp::Release(0),
                ],
            ];
            let report = explore(&programs, 1024);
            assert_eq!(report.deadlock, None);
            assert!(report.interleavings_tested > 1);
        }

        /// Adversarial: a receive on an empty channel
        /// with no sender deadlocks at the empty trace.
        #[test]
        fn lonely_receive_deadlocks() {
            let report = explore(&[vec![LoomOp::Recv(0)]], 16);
            assert_eq!(report.interleavings_tested, 1);
            assert_eq!(report.deadlock, Some(vec![]));
            assert!(report.outcomes.is_empty());
        }

        /// Validation: a join waits for its thread, then
        /// completes — exactly one interleaving survives.
        #[test]
        fn join_waits_for_its_thread() {
            let programs = vec![vec![LoomOp::Join(1)], vec![LoomOp::Send(0)]];
            let report = explore(&programs, 16);
            assert_eq!(report.interleavings_tested, 1);
            assert_eq!(report.deadlock, None);
            assert!(report.outcomes.contains(&vec![1]));
        }

        /// Adversarial: the cap bounds the exploration,
        /// deterministically — four senders have 24
        /// interleavings; a cap of 3 tests exactly 3.
        #[test]
        fn cap_bounds_the_exploration() {
            let programs = vec![
                vec![LoomOp::Send(0)],
                vec![LoomOp::Send(0)],
                vec![LoomOp::Send(0)],
                vec![LoomOp::Send(0)],
            ];
            let full = explore(&programs, 10_000);
            assert_eq!(full.interleavings_tested, 24);
            let capped = explore(&programs, 3);
            assert_eq!(capped.interleavings_tested, 3);
            let again = explore(&programs, 3);
            assert_eq!(capped, again, "the loom is deterministic");
        }
    }
}

// ---------------------------------------------------------------------------
// Floor 7 — the Loom of Many Hands: the Scoped Loom and the
// Border Seals. Both judges are pure checks over the
// authored tables in content: the join order against the
// threads' lives, the seals against the tools' truths.
// ---------------------------------------------------------------------------

/// The Scoped Loom's first fault, if the posted order has
/// one — spoken by the judge press, in the day's terms.
pub fn scoped_order_fault(order: &[u8; 6]) -> Option<&'static str> {
    if order.contains(&0) {
        return Some("a thread stands unassigned: every thread takes a slot");
    }
    for slot in 1..=6u8 {
        if order.iter().filter(|taken| **taken == slot).count() > 1 {
            return Some("two threads stand in one slot: a slot joins one thread");
        }
    }
    for (index, thread) in crate::content::SCOPED_THREADS.iter().enumerate() {
        if order[index] > thread.expiry {
            return Some(if index == 1 {
                "Bryn's thread is joined after its life ran out: it ends by slot 2"
            } else {
                "Elin's thread is joined after its life ran out: it ends by slot 3"
            });
        }
    }
    None
}

/// Whether the posted join order is a valid scoped order:
/// a permutation of slots 1..=6 with every early thread
/// joined before its expiry.
pub fn scoped_order_valid(order: &[u8; 6]) -> bool {
    scoped_order_fault(order).is_none()
}

/// Whether the six seals stand true: every tool sealed as
/// it truly is (recorded value = the true seal + 1, with
/// 0 unsealed).
pub fn seals_complete(seals: &[u8; 6]) -> bool {
    crate::content::SEAL_TOOLS
        .iter()
        .enumerate()
        .all(|(index, tool)| seals[index] == tool.true_seal + 1)
}

#[cfg(test)]
mod floor7_first_tests {
    use super::*;

    /// Validation: an order that homes the early threads
    /// in time — Bryn in slot 2, Elin in slot 3 — proves.
    #[test]
    fn scoped_order_homes_early_threads() {
        assert!(scoped_order_valid(&[4, 2, 5, 6, 3, 1]));
        assert!(scoped_order_valid(&[1, 2, 4, 5, 3, 6]));
    }

    /// Adversarial: a tidy-looking permutation that
    /// strands Bryn in the last slot is no scoped order.
    #[test]
    fn scoped_order_strands_the_early() {
        assert!(!scoped_order_valid(&[1, 6, 2, 3, 4, 5]));
        assert_eq!(
            scoped_order_fault(&[1, 6, 2, 3, 4, 5]),
            Some("Bryn's thread is joined after its life ran out: it ends by slot 2")
        );
        assert!(!scoped_order_valid(&[1, 2, 3, 4, 6, 5]));
    }

    /// Adversarial: unassigned and doubled slots fail,
    /// each with its own named fault.
    #[test]
    fn scoped_order_faults_are_named() {
        assert_eq!(
            scoped_order_fault(&[0, 2, 3, 4, 5, 6]),
            Some("a thread stands unassigned: every thread takes a slot")
        );
        assert_eq!(
            scoped_order_fault(&[1, 1, 3, 4, 5, 6]),
            Some("two threads stand in one slot: a slot joins one thread")
        );
        assert_eq!(scoped_order_fault(&[4, 2, 5, 6, 3, 1]), None);
    }

    /// Validation: the tools' true seals complete the
    /// chamber — shuttle and hearts as their cards state.
    #[test]
    fn seals_key_completes() {
        assert!(seals_complete(&[1, 2, 1, 2, 3, 1]));
    }

    /// Adversarial: the phantom-sealed kit under either
    /// boasted seal fails — the raw pointer inside is
    /// covered by neither — and an unsealed tool fails.
    #[test]
    fn phantom_seal_fails_both_ways() {
        assert!(
            !seals_complete(&[1, 2, 1, 2, 1, 1]),
            "SEND does not cover it"
        );
        assert!(
            !seals_complete(&[1, 2, 1, 2, 2, 1]),
            "SYNC does not cover it"
        );
        assert!(
            !seals_complete(&[1, 2, 1, 2, 0, 1]),
            "unsealed is not a seal"
        );
        assert!(
            !seals_complete(&[2, 2, 1, 2, 3, 1]),
            "the shuttle is no wall board"
        );
    }
}

// ---------------------------------------------------------------------------
// Floor 7 — the Charter Wall and Arc and Weak Echoes. The
// wall's judge checks the posted edges, the marked claim,
// and the found tide against the wall's tables. The book's
// referee is a REAL strong/weak counter simulation: the
// script's operations run on a counted model, and the
// trace the player must match is the model's own output.
// ---------------------------------------------------------------------------

/// The Charter Wall's first fault, if the grading has
/// one — spoken by the grade press, in the wall's terms.
pub fn wall_fault(edges: &[bool; 5], weak_marked: u8, stale_found: bool) -> Option<&'static str> {
    if *edges != crate::content::WALL_EDGE_KEY {
        return Some(
            "the edges are not the true set: four of the five chalked pairs are \
             edges, and one runs backwards",
        );
    }
    if weak_marked != crate::content::WALL_WEAK_INDEX + 1 {
        return Some(
            "the weakened claim is not the one marked: one claim leans on \
             Relaxed for an edge Relaxed never makes",
        );
    }
    if !stale_found {
        return Some(
            "the stale tide stands unfound: carry the shuttle's token, not \
             its dawn note",
        );
    }
    None
}

/// Whether the wall stands graded true: the true edges,
/// the weakened claim marked, the stale tide found.
pub fn wall_graded(edges: &[bool; 5], weak_marked: u8, stale_found: bool) -> bool {
    wall_fault(edges, weak_marked, stale_found).is_none()
}

/// The counted strong/weak model: strong holds keep the
/// book open; weak echoes point at it. Raising an echo
/// succeeds only while a strong hold stands — and
/// consumes the echo into a hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArcModel {
    /// Strong holds standing.
    strong: u8,
    /// Weak echoes standing.
    weak: u8,
}

impl ArcModel {
    /// Run one scripted operation; for a raising, return
    /// whether it held.
    fn run(&mut self, op: u8) -> bool {
        match op {
            0 => self.strong += 1,
            1 | 2 => self.weak += 1,
            3 => self.strong = self.strong.saturating_sub(1),
            4 => {
                if self.strong > 0 && self.weak > 0 {
                    self.weak -= 1;
                    self.strong += 1;
                    return true;
                }
                return false;
            }
            _ => {}
        }
        false
    }
}

/// The referee's trace over the authored eight-step
/// script: per step, the foretold quantity — the strong
/// count, the weak count, or the raising's outcome
/// (1 = it holds, 2 = it fails) — as the model ran it.
pub fn arc_trace() -> [u8; 8] {
    let mut model = ArcModel { strong: 1, weak: 0 };
    let mut trace = [0u8; 8];
    for (index, step) in crate::content::ARC_SCRIPT.iter().enumerate() {
        let held = model.run(step.op);
        trace[index] = match step.kind {
            0 => model.strong,
            1 => model.weak,
            _ => {
                if held {
                    1
                } else {
                    2
                }
            }
        };
    }
    trace
}

/// Whether the eight foretellings match the model's
/// trace, step for step.
pub fn arc_foretold(predictions: &[u8; 8]) -> bool {
    *predictions == arc_trace()
}

#[cfg(test)]
mod floor7_second_tests {
    use super::*;

    /// Validation: the wall's true grading — the true
    /// edges, the Relaxed claim marked, the tide found.
    #[test]
    fn wall_key_grades_true() {
        assert!(wall_graded(&crate::content::WALL_EDGE_KEY, 3, true));
    }

    /// Adversarial: the backwards pair chalked as an
    /// edge fails the set, and each missing piece is
    /// named in its own order.
    #[test]
    fn wall_faults_are_named() {
        let mut edges = crate::content::WALL_EDGE_KEY;
        edges[3] = true;
        assert!(!wall_graded(&edges, 3, true));
        assert!(wall_fault(&crate::content::WALL_EDGE_KEY, 0, true).is_some());
        assert!(wall_fault(&crate::content::WALL_EDGE_KEY, 1, true).is_some());
        assert!(wall_fault(&crate::content::WALL_EDGE_KEY, 3, false).is_some());
        assert_eq!(wall_fault(&crate::content::WALL_EDGE_KEY, 3, true), None);
    }

    /// Validation: the model's trace over the script —
    /// holds growing and letting go, echoes made and
    /// raised, the last raising refused by an empty book.
    #[test]
    fn arc_trace_is_the_models_own() {
        assert_eq!(arc_trace(), [2, 1, 2, 1, 1, 1, 0, 2]);
    }

    /// Adversarial: one optimistic foretelling — the
    /// last raising holds, in the player's hope — fails
    /// the whole script; the counts are the model's, not
    /// the hope's.
    #[test]
    fn arc_hope_is_not_the_trace() {
        let mut foretold = arc_trace();
        assert!(arc_foretold(&foretold));
        foretold[7] = 1;
        assert!(!arc_foretold(&foretold));
        foretold = arc_trace();
        foretold[0] = 3;
        assert!(!arc_foretold(&foretold));
    }
}

// ---------------------------------------------------------------------------
// Floor 7 — the Lock-Free Stack and the Loom of Many Hands.
// The stack's referee is a real linearizability search:
// every sequential order the histories' turn intervals
// allow is tried against a sequential stack. The FOE runs
// on the model loom itself, and its gate on the bounded
// queue model: producers, a bounded queue, back-pressure
// when full — the received set reconciled exactly.
// ---------------------------------------------------------------------------

use crate::content::StackOp;

/// Whether operation `a` must precede operation `b` in any
/// sequential account: a ended before b began.
fn must_precede(a: &StackOp, b: &StackOp) -> bool {
    a.end < b.start
}

/// Try the remaining operations in every allowed order:
/// an operation may be placed next when nothing unplaced
/// must precede it. The sequential stack judges each
/// placement — a pop must return the stack's top, or
/// nothing from an empty stack.
fn order_search(ops: &[StackOp], placed: &mut [bool], stack: &mut Vec<u8>, left: usize) -> bool {
    if left == 0 {
        return true;
    }
    for index in 0..ops.len() {
        if placed[index] {
            continue;
        }
        let blocked = (0..ops.len()).any(|other| {
            !placed[other] && other != index && must_precede(&ops[other], &ops[index])
        });
        if blocked {
            continue;
        }
        let op = &ops[index];
        if op.push {
            placed[index] = true;
            stack.push(op.value);
            if order_search(ops, placed, stack, left - 1) {
                return true;
            }
            stack.pop();
            placed[index] = false;
        } else if stack.is_empty() {
            if op.value != crate::content::STACK_EMPTY {
                continue;
            }
            placed[index] = true;
            if order_search(ops, placed, stack, left - 1) {
                return true;
            }
            placed[index] = false;
        } else {
            if *stack.last().expect("non-empty") != op.value {
                continue;
            }
            placed[index] = true;
            let value = stack.pop().expect("non-empty");
            if order_search(ops, placed, stack, left - 1) {
                return true;
            }
            stack.push(value);
            placed[index] = false;
        }
    }
    false
}

/// Whether one recorded history is linearizable: some
/// sequential order, respecting the turn intervals,
/// produces exactly its results.
pub fn history_linearizable(ops: &[StackOp]) -> bool {
    let mut placed = vec![false; ops.len()];
    let mut stack = Vec::new();
    order_search(ops, &mut placed, &mut stack, ops.len())
}

/// Whether the six verdicts stand true against the
/// referee's search (recorded 1 = linearizable,
/// 2 = not).
pub fn stack_verdicts_true(verdicts: &[u8; 6]) -> bool {
    crate::content::STACK_HISTORIES
        .iter()
        .enumerate()
        .all(|(index, history)| {
            verdicts[index] == if history_linearizable(history) { 1 } else { 2 }
        })
}

/// The bounded many-handed queue's report on one load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueueReport {
    /// Messages the producers sent.
    pub sent: u64,
    /// Messages the consumer received.
    pub received: u64,
    /// Whether the received set is exactly the sent set:
    /// every message once, none lost, none invented.
    pub reconciled: bool,
}

/// Run one load on the bounded queue: the producers take
/// turns in fixed order, each sending one message per
/// turn while the queue has room — at capacity a producer
/// WAITS (its turn passes; nothing is dropped, nothing
/// spins in place) — and the consumer takes one message
/// per turn. Message ids are dense (producer-major), so
/// the received set is checked exactly, id by id.
pub fn queue_run(producers: u32, per_producer: u32, capacity: usize) -> QueueReport {
    let total = (producers * per_producer) as usize;
    let mut queue: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
    let mut seen = vec![false; total];
    let mut next = vec![0u32; producers as usize];
    let mut sent = 0u64;
    let mut received = 0u64;
    let mut exact = true;
    while (received as usize) < total {
        for (producer, next_id) in next.iter_mut().enumerate() {
            if *next_id < per_producer && queue.len() < capacity.max(1) {
                let id = producer as u32 * per_producer + *next_id;
                *next_id += 1;
                queue.push_back(id);
                sent += 1;
            }
        }
        if let Some(id) = queue.pop_front() {
            if seen[id as usize] {
                exact = false;
            }
            seen[id as usize] = true;
            received += 1;
        }
    }
    QueueReport {
        sent,
        received,
        reconciled: exact && seen.iter().all(|held| *held) && sent == received,
    }
}

/// The at-capacity demonstration: fill the queue to its
/// stated capacity; the next send finds no room and
/// WAITS — it stands pending, neither dropped nor spun.
/// Returns the standing count and whether the next send
/// pends.
pub fn queue_at_capacity(capacity: usize) -> (usize, bool) {
    let mut queue: std::collections::VecDeque<u32> = std::collections::VecDeque::new();
    for id in 0..capacity as u32 {
        queue.push_back(id);
    }
    let next_waits = queue.len() >= capacity;
    (queue.len(), next_waits)
}

/// The FOE's two servants as loom programs over the two
/// stairs (claimables 0 and 1), in their standing —
/// opposite — claim orders.
pub fn foe_servant_programs() -> Vec<Vec<crate::puzzles::loom::LoomOp>> {
    use crate::puzzles::loom::LoomOp;
    vec![
        vec![
            LoomOp::Claim(0),
            LoomOp::Claim(1),
            LoomOp::Release(1),
            LoomOp::Release(0),
        ],
        vec![
            LoomOp::Claim(1),
            LoomOp::Claim(0),
            LoomOp::Release(0),
            LoomOp::Release(1),
        ],
    ]
}

/// The servants re-cut to a posted total order: both
/// claim the stairs in the order's stair sequence (the
/// order's other claimables are house furniture — the
/// servants' stairs are what the cycle turns on).
pub fn foe_recut_programs(order: &[u8; 4]) -> Vec<Vec<crate::puzzles::loom::LoomOp>> {
    use crate::puzzles::loom::LoomOp;
    let mut stairs: Vec<u8> = (0..4u8).filter(|claimable| *claimable < 2).collect();
    stairs.sort_by_key(|claimable| order[*claimable as usize]);
    let program = |stairs: &[u8]| -> Vec<LoomOp> {
        let mut ops: Vec<LoomOp> = stairs.iter().map(|stair| LoomOp::Claim(*stair)).collect();
        ops.extend(stairs.iter().rev().map(|stair| LoomOp::Release(*stair)));
        ops
    };
    vec![program(&stairs), program(&stairs)]
}

/// Whether a posted order is the repair's truth: a valid
/// total order of the four claimables (each rank 1..=4
/// exactly once) under which the loom — run on the
/// re-cut servants — finds no deadlock.
pub fn posted_order_holds(order: &[u8; 4]) -> bool {
    let mut ranks = [0u8; 4];
    for rank in order {
        if *rank == 0 || *rank > 4 {
            return false;
        }
        ranks[(*rank - 1) as usize] += 1;
    }
    if ranks != [1, 1, 1, 1] {
        return false;
    }
    let report = crate::puzzles::loom::explore(&foe_recut_programs(order), 4096);
    report.deadlock.is_none()
}

#[cfg(test)]
mod floor7_third_tests {
    use super::*;

    /// Validation: five of the six histories order; the
    /// referee's search finds their sequential accounts.
    #[test]
    fn five_histories_order() {
        for (index, history) in crate::content::STACK_HISTORIES.iter().enumerate() {
            assert_eq!(history_linearizable(history), index != 2, "history {index}");
        }
        assert!(stack_verdicts_true(&[1, 1, 2, 1, 1, 1]));
    }

    /// Adversarial: calling the buried-A history sound —
    /// or any honest history torn — fails the set; and a
    /// hand-made history whose pop returns a value never
    /// pushed is refused by the same search.
    #[test]
    fn verdict_lies_fail() {
        assert!(!stack_verdicts_true(&[1, 1, 1, 1, 1, 1]));
        assert!(!stack_verdicts_true(&[1, 2, 2, 1, 1, 1]));
        assert!(!stack_verdicts_true(&[0, 1, 2, 1, 1, 1]));
        let invented = [StackOp {
            push: false,
            value: 1,
            start: 1,
            end: 2,
        }];
        assert!(!history_linearizable(&invented));
    }

    /// Validation: the FOE's standing orders deadlock on
    /// the loom, and every posted total order — stairs
    /// first or last — clears the cycle when both
    /// servants are re-cut to it.
    #[test]
    fn foe_orders_on_the_loom() {
        let report = crate::puzzles::loom::explore(&foe_servant_programs(), 4096);
        assert!(report.deadlock.is_some());
        assert!(posted_order_holds(&[1, 2, 3, 4]));
        assert!(posted_order_holds(&[4, 3, 2, 1]));
        assert!(posted_order_holds(&[2, 1, 4, 3]));
    }

    /// Adversarial: a posted order with a doubled or
    /// missing rank is no total order, whatever the loom
    /// would say of it.
    #[test]
    fn posted_order_must_be_total() {
        assert!(!posted_order_holds(&[1, 1, 2, 3]));
        assert!(!posted_order_holds(&[0, 1, 2, 3]));
        assert!(!posted_order_holds(&[1, 2, 3, 5]));
    }

    /// Validation: the queue carries its stated loads —
    /// the evening count among them — with the received
    /// set reconciling exactly, every time.
    #[test]
    fn queue_loads_reconcile() {
        for (producers, per_producer) in [(2, 60), (3, 45), (4, 250)] {
            let report = queue_run(producers, per_producer, 8);
            assert!(report.reconciled);
            assert_eq!(report.sent, report.received);
            assert_eq!(report.sent, (producers * per_producer) as u64);
        }
    }

    /// The million-message exact-set proof (DS-13): a
    /// real run of the queue model at scale — ten hands,
    /// a hundred thousand messages each — and the
    /// received set is the sent set, exactly.
    #[test]
    fn queue_million_messages_exact() {
        let report = queue_run(10, 100_000, 64);
        assert_eq!(report.sent, 1_000_000);
        assert_eq!(report.received, 1_000_000);
        assert!(report.reconciled);
    }

    /// Validation: at capacity the next send waits — the
    /// queue stands full and the pending send is a state,
    /// not a drop and not a spin.
    #[test]
    fn queue_full_waits() {
        assert_eq!(queue_at_capacity(8), (8, true));
        assert_eq!(queue_at_capacity(1), (1, true));
    }
}

/// Frozen turn-stepped executor for Floor 8 and Part B.
pub mod executor;
pub mod hour;

/// The authored state-box drive: loading and preparation are Pending;
/// only the final state is Ready. Reports are computed from the executor.
pub fn hour_event_kind(event: &executor::Event) -> u8 {
    use executor::Event;
    match event {
        Event::Poll { .. } => 1,
        Event::Sleep { .. } => 2,
        Event::Wake { .. } => 3,
        Event::TimerFired { .. } => 4,
        Event::Completed { .. } => 5,
        Event::CancelledWithCleanup { .. } => 6,
        Event::Panicked { .. } => 7,
        Event::StaleRefused { .. } => 8,
        Event::Accounting { .. } => 9,
    }
}

pub fn hour_sleep_evidence() -> (usize, usize, usize, usize) {
    let sleep = executor::run(&crate::content::hour_sleep_roster(false));
    let spin = executor::run(&crate::content::hour_sleep_roster(true));
    (
        sleep
            .iter()
            .filter(|e| matches!(e, executor::Event::Sleep { .. }))
            .count(),
        sleep
            .iter()
            .filter(|e| matches!(e, executor::Event::Wake { .. }))
            .count(),
        sleep
            .iter()
            .filter(|e| matches!(e, executor::Event::Poll { .. }))
            .count(),
        spin.iter()
            .filter(|e| matches!(e, executor::Event::Poll { .. }))
            .count(),
    )
}

pub fn hour_waker_diagnosed(parts: &[u8; 4], trial: bool) -> bool {
    trial
        && parts
            .iter()
            .enumerate()
            .all(|(i, &part)| usize::from(part) == i + 1)
        && hour_sleep_evidence().0 == 2
        && hour_sleep_evidence().1 == 2
}

pub fn hour_attendant_trace() -> Vec<executor::Event> {
    executor::run(&crate::content::hour_attendant_roster())
}

pub fn hour_attendant_diagnosed(roster: &[u8; 12], judged: bool) -> bool {
    judged
        && roster.as_slice()
            == hour_attendant_trace()
                .iter()
                .map(hour_event_kind)
                .collect::<Vec<_>>()
}

pub fn hour_bell_trace() -> Vec<executor::Event> {
    use executor::{ErrandSpec, Outcome, State, Timer};
    executor::run(&[ErrandSpec {
        seal: "Countdown Bell",
        states: crate::content::F8_BELL_STATES
            .iter()
            .enumerate()
            .map(|(i, &name)| State {
                name,
                outcome: if i + 1 == crate::content::F8_BELL_STATES.len() {
                    Outcome::Ready
                } else {
                    Outcome::Pending
                },
                heart: None,
            })
            .collect(),
        drive: (0..crate::content::F8_BELL_STATES.len()).collect(),
        timers: (1..crate::content::F8_BELL_STATES.len())
            .map(|tick| Timer {
                tick: tick as u32,
                generation: 0,
            })
            .collect(),
    }])
}

pub fn hour_bell_reports() -> Vec<u8> {
    hour_bell_trace()
        .iter()
        .filter_map(|event| match event {
            executor::Event::Sleep { .. } => Some(1),
            executor::Event::Completed { .. } => Some(2),
            _ => None,
        })
        .collect()
}

pub fn hour_bell_diagnosed(polls: &[u8; 5], rung: bool) -> bool {
    rung && polls.as_slice() == hour_bell_reports()
}

pub fn hour_tapestry_diagnosed(torn: bool, pinned: bool, moved: bool) -> bool {
    torn && pinned && moved
}

// Floor 9 — the Heapwarden's Trio. R-F9-1..4 govern this shared surface.
/// A real Vec with an explicit modelled shared-loan ledger.
#[derive(Debug)]
pub struct ShelfVec<T> {
    values: Vec<T>,
    loans: u32,
}
#[allow(clippy::len_without_is_empty)] // Frozen surface: len, not an extra API.
impl<T> ShelfVec<T> {
    pub fn new(reserve: usize) -> Self {
        Self {
            values: Vec::with_capacity(reserve),
            loans: 0,
        }
    }
    /// R-F9-2 refuses every push while a shared loan is live.
    pub fn push(&mut self, value: T) -> bool {
        if self.loans > 0 {
            return false;
        }
        self.values.push(value);
        true
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.values.get(index)
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn cap(&self) -> usize {
        self.values.capacity()
    }
    pub fn stamps(&self) -> u32 {
        self.loans
    }
    pub fn take_loan(&mut self) {
        self.loans = self.loans.saturating_add(1);
    }
    pub fn return_loan(&mut self) {
        self.loans = self.loans.saturating_sub(1);
    }
}

/// Deterministic, tombstone-free linear probing; every walk is bounded by slots.
pub struct CellarMap<K, V> {
    slots: Vec<Option<(K, V)>>,
    len: usize,
    probes: u64,
}
#[allow(clippy::len_without_is_empty)] // Frozen surface.
impl<K: Eq + core::hash::Hash, V> CellarMap<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: std::iter::repeat_with(|| None)
                .take(capacity.max(1))
                .collect(),
            len: 0,
            probes: 0,
        }
    }
    fn start(&self, key: &K) -> usize {
        use core::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.slots.len() as u64) as usize
    }
    /// Counts every examined slot, including a full-table refusal's walk.
    pub fn insert(&mut self, key: K, value: V) -> Option<usize> {
        let start = self.start(&key);
        for offset in 0..self.slots.len() {
            let index = (start + offset) % self.slots.len();
            self.probes += 1;
            match &mut self.slots[index] {
                Some((stored, old)) if *stored == key => {
                    *old = value;
                    return Some(offset + 1);
                }
                None => {
                    self.slots[index] = Some((key, value));
                    self.len += 1;
                    return Some(offset + 1);
                }
                _ => {}
            }
        }
        None
    }
    pub fn get(&self, key: &K) -> Option<&V> {
        let start = self.start(key);
        for offset in 0..self.slots.len() {
            match &self.slots[(start + offset) % self.slots.len()] {
                Some((stored, value)) if stored == key => return Some(value),
                None => return None,
                _ => {}
            }
        }
        None
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn probes_total(&self) -> u64 {
        self.probes
    }
}

/// A byte-bounded bump arena issuing sequential block handles, never pointers.
#[derive(Debug)]
pub struct Hoard {
    capacity: u32,
    used: u32,
    blocks: u32,
}
impl Hoard {
    pub fn new(capacity: u32) -> Self {
        Self {
            capacity,
            used: 0,
            blocks: 0,
        }
    }
    pub fn issue(&mut self, bytes: u32) -> Option<u32> {
        if bytes == 0 {
            return None;
        }
        let used = self.used.checked_add(bytes)?;
        if used > self.capacity {
            return None;
        }
        let next = self.blocks.checked_add(1)?;
        let handle = self.blocks;
        self.used = used;
        self.blocks = next;
        Some(handle)
    }
    pub fn reset(&mut self) {
        self.used = 0;
        self.blocks = 0;
    }
    pub fn used(&self) -> u32 {
        self.used
    }
    pub fn blocks_used(&self) -> u32 {
        self.blocks
    }
}

/// A value fused to its census. R-F9-3 permits only checked addition.
/// Operator addition is unavailable, even when the runtime ids match:
/// ```compile_fail
/// use niflheim::puzzles::Stamped;
/// let _ = Stamped::seal(1u32, 7) + Stamped::seal(2u32, 8);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stamped<T> {
    value: T,
    census: u32,
}
impl<T> Stamped<T> {
    pub fn seal(value: T, census: u32) -> Self {
        Self { value, census }
    }
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn census_id(&self) -> u32 {
        self.census
    }
}
impl<T: Copy + core::ops::Add<Output = T>> Stamped<T> {
    pub fn add(&self, other: &Stamped<T>) -> Option<Stamped<T>> {
        (self.census == other.census).then(|| Self::seal(self.value + other.value, self.census))
    }
}

#[cfg(test)]
mod floor9_trio_tests {
    use super::{CellarMap, Hoard, ShelfVec, Stamped};
    use std::collections::{BTreeMap, hash_map::DefaultHasher};
    use std::hash::{Hash, Hasher};

    #[test]
    fn shelf_differential_10000_operations() {
        let mut shelf = ShelfVec::new(3);
        let mut reference = Vec::with_capacity(3);
        let mut loans = 0u32;
        for operation in 0..10_000 {
            match operation % 11 {
                0 => {
                    shelf.take_loan();
                    loans += 1;
                }
                1 | 2 => {
                    shelf.return_loan();
                    loans = loans.saturating_sub(1);
                }
                3 => assert_eq!(
                    shelf.get(operation % (reference.len() + 1)),
                    reference.get(operation % (reference.len() + 1))
                ),
                _ => {
                    assert_eq!(shelf.push(operation), loans == 0);
                    if loans == 0 {
                        reference.push(operation);
                    }
                }
            }
            assert_eq!(
                (shelf.len(), shelf.cap(), shelf.stamps()),
                (reference.len(), reference.capacity(), loans)
            );
            for (index, value) in reference.iter().enumerate() {
                assert_eq!(shelf.get(index), Some(value));
            }
            assert_eq!(shelf.get(reference.len()), None);
        }
    }

    #[test]
    fn shelf_boundary_and_nested_loan_refusals() {
        let mut shelf = ShelfVec::new(2);
        let mut reference = Vec::with_capacity(2);
        let cap = shelf.cap();
        for value in 0..cap {
            assert!(shelf.push(value));
            reference.push(value);
        }
        shelf.take_loan();
        shelf.take_loan();
        assert!(!shelf.push(cap));
        assert_eq!(
            (shelf.len(), shelf.cap()),
            (reference.len(), reference.capacity())
        );
        shelf.return_loan();
        assert!(!shelf.push(cap));
        shelf.return_loan();
        shelf.return_loan();
        assert_eq!(shelf.stamps(), 0);
        assert!(shelf.push(cap));
        reference.push(cap);
        assert_eq!(
            (shelf.len(), shelf.cap()),
            (reference.len(), reference.capacity())
        );
        let mut spare = ShelfVec::new(8);
        spare.take_loan();
        assert!(!spare.push(1));
        assert_eq!(spare.len(), 0);
        let mut empty = ShelfVec::new(0);
        assert!(empty.push(1));
        assert_eq!(empty.get(1), None);
    }

    // Deliberately slow reference: BTreeMap for values and a separate slot ledger
    // scanned in circular order for probe accounting, independent of map methods.
    fn reference_insert(
        slots: &mut [Option<u32>],
        values: &mut BTreeMap<u32, u32>,
        key: u32,
        value: u32,
    ) -> (Option<usize>, u64) {
        let mut hash = DefaultHasher::new();
        key.hash(&mut hash);
        let home = (hash.finish() % slots.len() as u64) as usize;
        let walk: Vec<_> = (home..slots.len()).chain(0..home).collect();
        let landing = walk
            .iter()
            .position(|&index| slots[index].is_none() || slots[index] == Some(key));
        match landing {
            Some(offset) => {
                slots[walk[offset]] = Some(key);
                values.insert(key, value);
                (Some(offset + 1), (offset + 1) as u64)
            }
            None => (None, slots.len() as u64),
        }
    }

    #[test]
    fn cellar_differential_10000_operations() {
        let mut map = CellarMap::new(31);
        let mut slots = vec![None; 31];
        let mut values = BTreeMap::new();
        let mut total = 0;
        for operation in 0u32..10_000 {
            let key = (operation * 17) % 47;
            if operation % 3 == 0 {
                assert_eq!(map.get(&key), values.get(&key));
            } else {
                let (result, probes) = reference_insert(&mut slots, &mut values, key, operation);
                total += probes;
                assert_eq!(map.insert(key, operation), result);
            }
            assert_eq!((map.len(), map.probes_total()), (values.len(), total));
            for key in 0..48 {
                assert_eq!(map.get(&key), values.get(&key));
            }
        }
    }

    #[derive(Eq, PartialEq)]
    struct Collision(u8);
    impl Hash for Collision {
        fn hash<H: Hasher>(&self, state: &mut H) {
            0u8.hash(state);
        }
    }
    #[test]
    fn cellar_collision_flood_full_replacement_and_wrap() {
        let capacity = 7;
        let mut map = CellarMap::new(capacity);
        let mut total = 0;
        for index in 0..capacity {
            let probes = map.insert(Collision(index as u8), index).unwrap();
            // All keys hash identically: every prior occupied slot plus landing.
            let expected = (0..=index).count();
            assert_eq!(probes, expected);
            assert_ne!(probes, expected + 1);
            total += probes as u64;
            assert_eq!(map.probes_total(), total);
        }
        let before = map.probes_total();
        assert_eq!(map.insert(Collision(255), 99), None);
        assert_eq!(map.probes_total() - before, capacity as u64);
        assert_eq!(map.get(&Collision(255)), None);
        let probes = map.insert(Collision(6), 99).unwrap();
        assert_eq!(probes, capacity);
        assert_eq!(map.len(), capacity);
        assert_eq!(map.get(&Collision(6)), Some(&99));
        let mut one = CellarMap::new(0);
        assert!(one.insert("one", String::from("value")).is_some());
        assert!(one.insert("two", String::new()).is_none());
        assert_eq!(one.get(&"one").map(String::as_str), Some("value"));
    }

    #[test]
    fn hoard_differential_10000_operations() {
        let capacity = 137;
        let mut hoard = Hoard::new(capacity);
        let mut blocks = Vec::<u32>::new();
        for operation in 0..10_000u32 {
            if operation % 97 == 0 {
                hoard.reset();
                blocks.clear();
            } else {
                let bytes = (operation * 13) % 41;
                let used: u32 = blocks.iter().sum();
                let expected =
                    if bytes > 0 && u64::from(used) + u64::from(bytes) <= u64::from(capacity) {
                        let handle = blocks.len() as u32;
                        blocks.push(bytes);
                        Some(handle)
                    } else {
                        None
                    };
                assert_eq!(hoard.issue(bytes), expected);
            }
            assert_eq!(hoard.used(), blocks.iter().sum::<u32>());
            assert_eq!(hoard.blocks_used(), blocks.len() as u32);
        }
    }

    #[test]
    fn hoard_exact_capacity_zero_exhaustion_overflow_and_reset() {
        let mut hoard = Hoard::new(u32::MAX);
        assert_eq!(hoard.issue(0), None);
        assert_eq!((hoard.used(), hoard.blocks_used()), (0, 0));
        assert_eq!(hoard.issue(u32::MAX), Some(0));
        let before = (hoard.used(), hoard.blocks_used());
        for bytes in [0, 1, u32::MAX] {
            assert_eq!(hoard.issue(bytes), None);
            assert_eq!((hoard.used(), hoard.blocks_used()), before);
        }
        hoard.reset();
        assert_eq!((hoard.used(), hoard.blocks_used()), (0, 0));
        assert_eq!(hoard.issue(1), Some(0));
        assert_eq!(hoard.issue(1), Some(1));
        let mut empty = Hoard::new(0);
        assert_eq!(empty.issue(1), None);
    }

    #[test]
    fn stamped_same_scroll_sum_preserves_census() {
        let mut shelf = ShelfVec::new(0);
        let mut map = CellarMap::new(3);
        for value in [4u8, 9] {
            assert!(shelf.push(value));
            assert!(map.insert(value, value).is_some());
        }
        let a = Stamped::seal(shelf.len() as u32, 7);
        let b = Stamped::seal(map.len() as u32, 7);
        let sum = a.add(&b).unwrap();
        assert_eq!(*sum.value(), *a.value() + *b.value());
        assert_eq!(sum.census_id(), a.census_id());
    }

    #[test]
    fn stamped_different_scrolls_refuse_even_equal_values() {
        let a = Stamped::seal(Hoard::new(8).blocks_used(), 7);
        let b = Stamped::seal(*a.value(), 8);
        assert_eq!(a.add(&b), None);
        assert_eq!(b.add(&a), None);
        assert_ne!(a.census_id(), b.census_id());
    }
}
