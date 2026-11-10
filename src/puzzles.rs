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
