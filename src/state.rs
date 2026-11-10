//! The campaign state machine and the save format.
//!
//! One `Expedition` value holds the whole descent: Floor 1's five
//! chambers, the FOE, the codex, the pack, the log, the player's
//! editor draft, and their settings. Every transition validates
//! before it commits — a refused transition returns the vault's
//! wording and leaves the published state untouched. The save is a
//! single portable JSON file (magic + version header, fixed field
//! order): a save's records are indistinguishable by platform, and
//! a save that breaks an invariant of the design is refused as
//! corrupt rather than repaired by guessing.
//!
//! Resolution model (design §5.4): chambers move UNWORKED →
//! DIAGNOSED → REPAIRED on assembled proof; the FOE moves STANDING
//! → EXPOSED → FALLEN. The strike is a deliberate two-step act and
//! is impossible without complete proof.

use serde::{Deserialize, Serialize};

use crate::content::{
    ADMISSION_SERVICES, BEASTS, FOE_CHALLENGE, GATE_SCAFFOLD, INTERNERS_HOARD, LEASES_INITIAL,
    PACK_ITEMS, PLACARDS, PLANK_STONE, ROSTER_GROUPS,
};
use crate::core::{SAVE_FORMAT_VERSION, SAVE_MAGIC, SaveError, Settings, TextScale};
use crate::puzzles::{
    RecordKind, bound_exact, bound_missing, bound_unjustified, day_plan_overlap, errand_choice_ok,
    foe_census, foe_uncopiable_rows, identification_ok, judge_placards, kennel_choice_ok,
    lease_chain_problem, packing_problems, reconciled_toll, record_fits_slot, run_delivery,
    trick_charter_ok,
};

/// Most bytes of editor draft a save will carry.
pub const MAX_EDITOR_BYTES: usize = 128 * 1024;

/// A chamber's resolution state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChamberState {
    /// No proof assembled yet.
    Unworked,
    /// Proof assembled and filed; the repair may be struck.
    Diagnosed,
    /// Repaired. The chamber's square fills on the chart.
    Repaired,
}

/// The FOE's state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FoeState {
    /// Standing: unaudited, or the audit incomplete.
    Standing,
    /// Exposed: the census proof is complete; the seam is shown.
    Exposed,
    /// Fallen: the strike landed; stone and loot are awarded.
    Fallen,
}

/// The two-step strike's phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StrikePhase {
    /// The seal sleeps.
    Idle,
    /// The seal is pressed; one deliberate confirmation remains.
    Armed,
}

/// A refused transition, worded for the player. The state that was
/// refused against is never mutated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal(pub String);

impl Refusal {
    /// The refusal's wording.
    pub fn message(&self) -> String {
        self.0.clone()
    }
}

/// A chamber's work-in-progress: the player's current attempt plus
/// which proof slots stand filled. Plain data, saved verbatim.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChamberWork {
    /// C1: holder bitmask per item (bit per porter).
    pub assignments: Vec<u8>,
    /// C1: the toll candidate chosen, as an index.
    pub toll_choice: Option<usize>,
    /// C1: the packing slot stands filled.
    pub packing_ok: bool,
    /// C1: the toll slot stands filled.
    pub toll_ok: bool,
    /// C2: candidate block index per person.
    pub picks: Vec<u8>,
    /// C3: the three leases as `[start, end]` pairs.
    pub leases: Vec<[i32; 2]>,
    /// C4: the delivery has been run.
    pub ran: bool,
    /// C4: the second room was committed too.
    pub commit_second: bool,
    /// C4: record kinds produced (0 kept, 1 kept-second, 2 restored).
    pub records: Vec<u8>,
    /// C4: which record fills the kept slot.
    pub slot_kept: Option<usize>,
    /// C4: which record fills the restored slot.
    pub slot_restored: Option<usize>,
    /// C5: judgement per placard (`true` = judged legal).
    pub judgements: Vec<Option<bool>>,
    /// C5: placards the yard's gate marked wrong at the last facing.
    pub wrong_marks: Vec<usize>,
}

impl ChamberWork {
    /// The starting work for a chamber, as the flawed plans stand.
    fn initial(chamber: usize) -> Self {
        let mut work = ChamberWork {
            assignments: Vec::new(),
            toll_choice: None,
            packing_ok: false,
            toll_ok: false,
            picks: Vec::new(),
            leases: Vec::new(),
            ran: false,
            commit_second: false,
            records: Vec::new(),
            slot_kept: None,
            slot_restored: None,
            judgements: Vec::new(),
            wrong_marks: Vec::new(),
        };
        match chamber {
            0 => {
                work.assignments = PACK_ITEMS
                    .iter()
                    .map(|item| {
                        item.initial_holders
                            .iter()
                            .fold(0u8, |mask, p| mask | (1 << p))
                    })
                    .collect();
            }
            1 => work.picks = vec![0; 5],
            2 => {
                work.leases = LEASES_INITIAL.iter().map(|(s, e)| [*s, *e]).collect();
            }
            4 => work.judgements = vec![None; PLACARDS.len()],
            _ => {}
        }
        work
    }
}

/// A Floor 2 chamber's work-in-progress: the player's current
/// attempt. Plain data, saved verbatim; fields arrive chamber by
/// chamber as their steps land, each serde-defaulted so saves
/// written before a field existed load whole.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor2Work {
    /// C1: the charter candidate chosen, as an index into
    /// `CHARTER_CANDIDATES`.
    pub charter_choice: Option<usize>,
    /// C1: the charter slot stands filled.
    pub charter_ok: bool,
    /// C1: both performances have been watched.
    pub watched: bool,
    /// C1: the beast identified as the blind run's body, as an
    /// index into `TRICK_PERFORMERS`.
    pub run_choice: Option<usize>,
    /// C1: the identification slot stands filled.
    pub run_ok: bool,
    /// C2: the player's revised bounds, per stable service, as
    /// bitmasks over `STABLE_SIGILS`; a missing entry stands at
    /// the stable's posted (over-strict) bound.
    #[serde(default)]
    pub bound_masks: Vec<u8>,
    /// C2: every service's bound stands exact.
    #[serde(default)]
    pub bounds_ok: bool,
    /// C3: both kennels' deliveries have been run and read.
    #[serde(default)]
    pub ran_deliveries: bool,
    /// C3: the errand named as the one that runs contorted at the
    /// kennel that chooses its own kind, as an index into
    /// `ERRANDS`.
    #[serde(default)]
    pub errand_choice: Option<usize>,
    /// C3: the errand slot stands filled.
    #[serde(default)]
    pub errand_ok: bool,
    /// C3: the kennel named as running that errand naturally, as
    /// an index into `KENNELS`.
    #[serde(default)]
    pub kennel_choice: Option<usize>,
    /// C3: the kennel slot stands filled.
    #[serde(default)]
    pub kennel_ok: bool,
}

/// One Floor 2 chamber's progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor2ChamberProgress {
    /// Resolution state.
    pub state: ChamberState,
    /// The deepest hint rung viewed for this chamber (0 = none).
    pub hints_used: u8,
    /// The facet: earned by a repair at full standard (rung ≤ 3).
    pub facet: bool,
    /// The work in progress.
    #[serde(default)]
    pub work: Floor2Work,
}

/// The Floor 2 FOE's progress: the audit arc.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor2FoeProgress {
    /// Resolution state.
    pub state: FoeState,
    /// Per admission service: the audit has read this service's bound.
    pub audited: Vec<bool>,
    /// The Lens stands calibrated against the roster.
    pub calibrated: bool,
    /// The roster crate has passed the cargo gate.
    pub gate_passed: bool,
    /// The strike's phase.
    pub strike: StrikePhase,
    /// The date inscribed when the FOE fell (`YYYY-MM-DD`).
    pub fallen_date: Option<String>,
}

/// Floor 2's whole progress record: five chambers, the FOE, the
/// admissions ledger, and the roster board's view state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor2Progress {
    /// The five chambers, in design order.
    pub chambers: Vec<Floor2ChamberProgress>,
    /// The FOE.
    pub foe: Floor2FoeProgress,
    /// The deepest hint rung viewed for the FOE.
    pub foe_hints_used: u8,
    /// Per beast on the roster: admitted past its service's bound.
    pub admissions: Vec<bool>,
    /// Per roster group: the board shows it collapsed.
    pub roster_collapsed: Vec<bool>,
}

impl Floor2Progress {
    /// A fresh Floor 2: nothing repaired, nothing admitted, the
    /// roster standing open.
    pub fn new() -> Self {
        Self {
            chambers: (0..5)
                .map(|_| Floor2ChamberProgress {
                    state: ChamberState::Unworked,
                    hints_used: 0,
                    facet: false,
                    work: Floor2Work::default(),
                })
                .collect(),
            foe: Floor2FoeProgress {
                state: FoeState::Standing,
                audited: vec![false; ADMISSION_SERVICES.len()],
                calibrated: false,
                gate_passed: false,
                strike: StrikePhase::Idle,
                fallen_date: None,
            },
            foe_hints_used: 0,
            admissions: vec![false; BEASTS.len()],
            roster_collapsed: vec![false; ROSTER_GROUPS.len()],
        }
    }
}

impl Default for Floor2Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// One chamber's progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChamberProgress {
    /// Resolution state.
    pub state: ChamberState,
    /// The deepest hint rung viewed for this chamber (0 = none).
    pub hints_used: u8,
    /// The facet: earned by a repair at full standard (rung ≤ 3).
    pub facet: bool,
    /// The work in progress.
    pub work: ChamberWork,
}

/// The FOE's progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FoeProgress {
    /// Resolution state.
    pub state: FoeState,
    /// The master plan has been enumerated into a census.
    pub enumerated: bool,
    /// Per census row: the player has verified this row's holders.
    pub verified: Vec<bool>,
    /// The double-promised item stands exhibited in the proof slot.
    pub exhibited: bool,
    /// The cargo gate has passed on the player's Hoard.
    pub gate_passed: bool,
    /// The strike's phase.
    pub strike: StrikePhase,
    /// The date inscribed when the FOE fell (`YYYY-MM-DD`).
    pub fallen_date: Option<String>,
}

/// Settings, in save form (the text scale travels by name).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct SaveSettings {
    text_scale: String,
    high_contrast: bool,
    reduced_motion: bool,
}

/// The save file itself: fixed field order, unknown fields refused.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveFile {
    magic: String,
    version: u32,
    ambient_seed: u64,
    settings: SaveSettings,
    log: Vec<String>,
    chambers: Vec<ChamberProgress>,
    foe: FoeProgress,
    foe_hints_used: u8,
    codex: Vec<String>,
    pack: Vec<String>,
    editor_code: String,
    gate_record: Option<String>,
    #[serde(default)]
    floor2: Floor2Progress,
}

/// The whole expedition: the campaign's one source of truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expedition {
    /// Player settings (travel inside the save).
    pub settings: Settings,
    /// The ambient seed, carried for later floors' dressing.
    pub ambient_seed: u64,
    /// The expedition log, oldest first, bounded.
    pub log: Vec<String>,
    /// The five chambers, in design order.
    pub chambers: Vec<ChamberProgress>,
    /// The FOE.
    pub foe: FoeProgress,
    /// The deepest hint rung viewed for the FOE.
    pub foe_hints_used: u8,
    /// Principle stones earned, by name.
    pub codex: Vec<String>,
    /// Loot artifacts carried, by name.
    pub pack: Vec<String>,
    /// The player's Interner's Hoard draft, verbatim.
    pub editor_code: String,
    /// The last gate verdict's summary, for the gate record.
    pub gate_record: Option<String>,
    /// Floor 2's progress record.
    pub floor2: Floor2Progress,
}

impl Default for Expedition {
    fn default() -> Self {
        Self::new()
    }
}

impl Expedition {
    pub fn new() -> Self {
        let census_rows = foe_census().rows.len();
        Self {
            settings: Settings::default(),
            ambient_seed: crate::core::AMBIENT_SEED_DEFAULT,
            log: Vec::new(),
            chambers: (0..5)
                .map(|index| ChamberProgress {
                    state: ChamberState::Unworked,
                    hints_used: 0,
                    facet: false,
                    work: ChamberWork::initial(index),
                })
                .collect(),
            foe: FoeProgress {
                state: FoeState::Standing,
                enumerated: false,
                verified: vec![false; census_rows],
                exhibited: false,
                gate_passed: false,
                strike: StrikePhase::Idle,
                fallen_date: None,
            },
            foe_hints_used: 0,
            codex: Vec::new(),
            pack: Vec::new(),
            editor_code: GATE_SCAFFOLD.to_string(),
            gate_record: None,
            floor2: Floor2Progress::new(),
        }
    }

    /// Append a line to the log, bounded: the oldest lines fall
    /// away past [`crate::core::MAX_LOG_LINES`].
    pub fn log_push(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > crate::core::MAX_LOG_LINES {
            let excess = self.log.len() - crate::core::MAX_LOG_LINES;
            self.log.drain(0..excess);
        }
    }

    /// Whether Floor 2 stands open to the expedition: Floor 1's
    /// FOE has fallen.
    pub fn floor2_reachable(&self) -> bool {
        self.foe.state == FoeState::Fallen
    }

    /// How many Floor 2 chambers stand repaired.
    pub fn floor2_chambers_repaired(&self) -> usize {
        self.floor2
            .chambers
            .iter()
            .filter(|c| c.state == ChamberState::Repaired)
            .count()
    }

    /// Whether Floor 2 is fully charted (all chambers + the FOE).
    pub fn floor2_complete(&self) -> bool {
        self.floor2_chambers_repaired() == 5 && self.floor2.foe.state == FoeState::Fallen
    }

    /// How many chambers stand repaired.
    pub fn chambers_repaired(&self) -> usize {
        self.chambers
            .iter()
            .filter(|c| c.state == ChamberState::Repaired)
            .count()
    }

    /// Whether Floor 1 is fully charted (all chambers + the FOE).
    pub fn floor_complete(&self) -> bool {
        self.chambers_repaired() == 5 && self.foe.state == FoeState::Fallen
    }

    /// View a hint rung for a challenge (0–4 chambers, 5 = FOE):
    /// records the deepest rung viewed and returns the rung now
    /// shown (1-based, capped at 4). Viewing is free; rung 4
    /// forfeits the challenge's facet, by the design.
    pub fn view_hint(&mut self, challenge: usize) -> u8 {
        let slot = if challenge == FOE_CHALLENGE {
            &mut self.foe_hints_used
        } else if let Some(chamber) = self.chambers.get_mut(challenge) {
            &mut chamber.hints_used
        } else {
            return 0;
        };
        *slot = (*slot + 1).min(4);
        *slot
    }

    /// The deepest hint rung recorded for a challenge.
    pub fn hints_used(&self, challenge: usize) -> u8 {
        if challenge == FOE_CHALLENGE {
            self.foe_hints_used
        } else {
            self.chambers.get(challenge).map_or(0, |c| c.hints_used)
        }
    }

    /// Reset one chamber's work to its authored starting state.
    /// The resolution returns to UNWORKED and any facet is given
    /// back; hint usage stays on the record. This control always
    /// exists, so no chamber can be painted into a corner.
    pub fn reset_chamber(&mut self, chamber: usize) {
        if let Some(progress) = self.chambers.get_mut(chamber) {
            progress.state = ChamberState::Unworked;
            progress.facet = false;
            progress.work = ChamberWork::initial(chamber);
        }
    }

    /// File a chamber's proof (the prove action). On success the
    /// chamber is DIAGNOSED and the narration says what was filed;
    /// on refusal nothing is committed except knowledge — C1's two
    /// slots fill independently, and C5's wrong placards are marked.
    pub fn file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        if progress.state == ChamberState::Repaired {
            return Err(Refusal("this chamber already stands repaired".to_string()));
        }
        match chamber {
            0 => {
                // Both slots are re-judged on every filing: the
                // flags record the last judgement of the work as it
                // stands now, never a stale one.
                let problems = packing_problems(&progress.work.assignments);
                progress.work.packing_ok = problems.is_empty();
                let candidates = crate::content::TOLL_CANDIDATES;
                progress.work.toll_ok = matches!(
                    progress.work.toll_choice,
                    Some(choice) if candidates.get(choice).copied() == Some(reconciled_toll())
                );
                if progress.work.packing_ok && progress.work.toll_ok {
                    progress.state = ChamberState::Diagnosed;
                    Ok(
                        "The plan is filed: one holder per uncopiable item, and the toll \
                        reconciled by name. The vault door swings open."
                            .to_string(),
                    )
                } else if !problems.is_empty() {
                    Err(Refusal(format!(
                        "The vault door refuses the plan: {}",
                        problems[0]
                    )))
                } else if !progress.work.packing_ok {
                    Err(Refusal(
                        "The packing is not filed yet — the plan does not balance".to_string(),
                    ))
                } else {
                    Err(Refusal(
                        "The toll does not reconcile. Count the names, not the tallies: \
                         the keepers share names, and shared names are counted once"
                            .to_string(),
                    ))
                }
            }
            1 => match day_plan_overlap(&progress.work.picks) {
                Some(overlap) => Err(Refusal(format!("The room refuses the day-plan: {overlap}"))),
                None => {
                    progress.state = ChamberState::Diagnosed;
                    Ok(
                        "The day-plan is filed: the pen writes alone, and every reader \
                        reads in peace."
                            .to_string(),
                    )
                }
            },
            2 => {
                let leases: [(i32, i32); 3] = [
                    lease_at(&progress.work.leases, 0),
                    lease_at(&progress.work.leases, 1),
                    lease_at(&progress.work.leases, 2),
                ];
                match lease_chain_problem(&leases) {
                    Some(problem) => Err(Refusal(format!(
                        "The vault refuses the chain, and draws the brackets: {problem}"
                    ))),
                    None => {
                        progress.state = ChamberState::Diagnosed;
                        Ok(
                            "The chain is filed: three brackets, each wholly held by the \
                            one before."
                                .to_string(),
                        )
                    }
                }
            }
            3 => {
                if progress.work.slot_kept.is_some() && progress.work.slot_restored.is_some() {
                    progress.state = ChamberState::Diagnosed;
                    Ok(
                        "Both records stand exhibited: what the book kept, and what the \
                        room restored."
                            .to_string(),
                    )
                } else if !progress.work.ran {
                    Err(Refusal(
                        "There are no records yet — run the delivery through both rooms"
                            .to_string(),
                    ))
                } else {
                    Err(Refusal(
                        "Both slots must be filled: one wants what the book kept, the \
                         other what the room restored"
                            .to_string(),
                    ))
                }
            }
            4 => {
                let (unjudged, wrong) = judge_placards(&progress.work.judgements);
                progress.work.wrong_marks = wrong.iter().map(|(index, _)| *index).collect();
                if unjudged > 0 {
                    Err(Refusal(format!(
                        "{unjudged} placard{} still hang{} unjudged",
                        if unjudged == 1 { "" } else { "s" },
                        if unjudged == 1 { "s" } else { "" }
                    )))
                } else if let Some((_, rule)) = wrong.first() {
                    Err(Refusal(format!(
                        "The yard's gate bars {} judgement{} — the first breaks on {rule}",
                        wrong.len(),
                        if wrong.len() == 1 { "" } else { "s" }
                    )))
                } else {
                    progress.state = ChamberState::Diagnosed;
                    Ok("The yard's gate swings wide: ten judgements, ten agreements.".to_string())
                }
            }
            _ => Err(Refusal("no such chamber".to_string())),
        }
    }

    /// Repair a diagnosed chamber (the deliberate act after proof).
    /// A repair at full standard — hint rung 4 never viewed — earns
    /// the chamber's facet for the codex.
    pub fn repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        match progress.state {
            ChamberState::Repaired => {
                Err(Refusal("this chamber already stands repaired".to_string()))
            }
            ChamberState::Unworked => Err(Refusal(
                "there is nothing to repair yet — assemble the proof first".to_string(),
            )),
            ChamberState::Diagnosed => {
                progress.state = ChamberState::Repaired;
                if progress.hints_used <= 3 {
                    progress.facet = true;
                    Ok(
                        "Repaired at full standard. A facet of the Plank Stone settles \
                        into the codex."
                            .to_string(),
                    )
                } else {
                    Ok(
                        "Repaired — though the seam was named, and the record keeps it. \
                        No facet, but the floor still completes."
                            .to_string(),
                    )
                }
            }
        }
    }

    /// View a hint rung for a Floor 2 chamber: Floor 1's ladder
    /// law, kept on Floor 2's own record.
    pub fn floor2_view_hint(&mut self, chamber: usize) -> u8 {
        let Some(progress) = self.floor2.chambers.get_mut(chamber) else {
            return 0;
        };
        progress.hints_used = (progress.hints_used + 1).min(4);
        progress.hints_used
    }

    /// The deepest hint rung recorded for a Floor 2 chamber.
    pub fn floor2_hints_used(&self, chamber: usize) -> u8 {
        self.floor2
            .chambers
            .get(chamber)
            .map_or(0, |c| c.hints_used)
    }

    /// Reset one Floor 2 chamber's work to its starting state, as
    /// `reset_chamber` does for Floor 1: hint usage stays on the
    /// record.
    pub fn floor2_reset_chamber(&mut self, chamber: usize) {
        if let Some(progress) = self.floor2.chambers.get_mut(chamber) {
            progress.state = ChamberState::Unworked;
            progress.facet = false;
            progress.work = Floor2Work::default();
        }
    }

    /// File a Floor 2 chamber's proof (the prove action). Each
    /// landed chamber judges all of its slots on every filing, as
    /// Floor 1's first chamber does: the flags record the last
    /// judgement of the work as it stands now, never a stale one.
    pub fn floor2_file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .floor2
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        if progress.state == ChamberState::Repaired {
            return Err(Refusal("this chamber already stands repaired".to_string()));
        }
        match chamber {
            0 => {
                let (charter_ok, run_ok, watched, charter_choice, run_choice) = {
                    let work = &mut progress.work;
                    work.charter_ok = matches!(
                        work.charter_choice,
                        Some(choice) if crate::content::CHARTER_CANDIDATES
                            .get(choice)
                            .is_some_and(|sigil| trick_charter_ok(sigil))
                    );
                    work.run_ok = work.watched
                        && matches!(work.run_choice, Some(choice) if identification_ok(choice));
                    (
                        work.charter_ok,
                        work.run_ok,
                        work.watched,
                        work.charter_choice,
                        work.run_choice,
                    )
                };
                if charter_ok && run_ok {
                    progress.state = ChamberState::Diagnosed;
                    Ok("The sigil is struck and rings true: the Anvil Call stands \
                         chartered by Forgehand, and the body behind the screen is \
                         named by its behaviour — Soot, and its own ending. The \
                         forge door swings open."
                        .to_string())
                } else if !watched {
                    Err(Refusal(
                        "The forge asks nothing blind: watch both beasts perform \
                         the call first — the desk replays it as often as you ask"
                            .to_string(),
                    ))
                } else if charter_choice.is_none() {
                    Err(Refusal(
                        "The slot stands empty: strike the sigil that charters \
                         the Anvil Call"
                            .to_string(),
                    ))
                } else if !charter_ok {
                    let chosen = charter_choice
                        .and_then(|choice| crate::content::CHARTER_CANDIDATES.get(choice))
                        .copied()
                        .unwrap_or("That sigil");
                    Err(Refusal(format!(
                        "{chosen} does not charter the call. A charter is the sigil \
                         that declares the trick in the sigil-book — not a sigil \
                         the performing bodies happen to wear"
                    )))
                } else if run_choice.is_none() {
                    Err(Refusal(
                        "Name the body that ran behind the screen — by its \
                         behaviour, not by any label"
                            .to_string(),
                    ))
                } else {
                    let named = run_choice
                        .and_then(|choice| crate::content::TRICK_PERFORMERS.get(choice))
                        .map(|performer| crate::content::BEASTS[performer.beast].name)
                        .unwrap_or("that beast");
                    Err(Refusal(format!(
                        "That was not {named}'s running. Watch the ending: the \
                         house body strikes once; a second, softer note is a \
                         variant's signature"
                    )))
                }
            }
            1 => {
                let (bounds_ok, unjustified, missing) = {
                    let work = &mut progress.work;
                    let mut all_exact = true;
                    let mut unjustified: Option<(usize, &str)> = None;
                    let mut missing: Option<(usize, &str)> = None;
                    for service in 0..crate::content::STABLE_SERVICES.len() {
                        let mask = work
                            .bound_masks
                            .get(service)
                            .copied()
                            .unwrap_or(crate::content::POSTED_BOUND);
                        if !bound_exact(service, mask) {
                            all_exact = false;
                        }
                        if unjustified.is_none() {
                            unjustified = bound_unjustified(service, mask)
                                .first()
                                .copied()
                                .map(|sigil| (service, sigil));
                        }
                        if missing.is_none() {
                            missing = bound_missing(service, mask)
                                .first()
                                .copied()
                                .map(|sigil| (service, sigil));
                        }
                    }
                    work.bounds_ok = all_exact;
                    (all_exact, unjustified, missing)
                };
                if bounds_ok {
                    progress.state = ChamberState::Diagnosed;
                    Ok("The bounds stand exact: each post now demands the \
                         sigils its work names, and no others. The \
                         Sigilwright walks the stalls, reads every posting, \
                         and nods — the stable door swings open."
                        .to_string())
                } else if let Some((service, sigil)) = unjustified {
                    let name = crate::content::STABLE_SERVICES[service].name;
                    Err(Refusal(format!(
                        "{sigil} is demanded of every beast at {name}, but \
                         nothing in its work asks for it. A bound may demand \
                         only what the service needs — no more"
                    )))
                } else if let Some((service, sigil)) = missing {
                    let name = crate::content::STABLE_SERVICES[service].name;
                    Err(Refusal(format!(
                        "The bound at {name} no longer asks {sigil}; a beast \
                         admitted without it cannot do the work the post \
                         describes"
                    )))
                } else {
                    Err(Refusal(
                        "the stable's bounds do not yet stand exact".to_string(),
                    ))
                }
            }
            2 => {
                let (errand_ok, kennel_ok, ran, errand_choice, kennel_choice) = {
                    let work = &mut progress.work;
                    work.errand_ok = work.ran_deliveries
                        && matches!(
                            work.errand_choice,
                            Some(choice) if errand_choice_ok(choice)
                        );
                    work.kennel_ok = work.ran_deliveries
                        && matches!(
                            work.kennel_choice,
                            Some(kennel) if kennel_choice_ok(kennel)
                        );
                    (
                        work.errand_ok,
                        work.kennel_ok,
                        work.ran_deliveries,
                        work.errand_choice,
                        work.kennel_choice,
                    )
                };
                if errand_ok && kennel_ok {
                    progress.state = ChamberState::Diagnosed;
                    Ok("The finding stands: one errand runs contorted where \
                         the kennel chooses, and true where the customer \
                         does. The Sigilwright chalks both records on the \
                         kennel wall — the door swings open."
                        .to_string())
                } else if !ran {
                    Err(Refusal(
                        "The kennels ask nothing unrun: run the deliveries \
                         from both houses first — the record is the evidence"
                            .to_string(),
                    ))
                } else if errand_choice.is_none() {
                    Err(Refusal(
                        "Name the errand that ran contorted at the Hearth \
                         Kennel — the house that chooses its own kind"
                            .to_string(),
                    ))
                } else if !errand_ok {
                    let named = errand_choice
                        .and_then(|choice| crate::content::ERRANDS.get(choice))
                        .map(|errand| errand.name)
                        .unwrap_or("That errand");
                    Err(Refusal(format!(
                        "{named} did not run contorted from the Hearth \
                         Kennel — the ash-hound's kind carried it. Read the \
                         kennel's records again"
                    )))
                } else if kennel_choice.is_none() {
                    Err(Refusal(
                        "Name the kennel from which that errand runs \
                         naturally"
                            .to_string(),
                    ))
                } else {
                    Err(Refusal(
                        "From the Hearth Kennel that errand still runs on \
                         the ash-hound, and the ford still stops it. Name \
                         the kennel where the kind is the customer's to \
                         choose"
                            .to_string(),
                    ))
                }
            }
            _ => Err(Refusal(
                "this chamber's proof is carved in a later step".to_string(),
            )),
        }
    }

    /// Repair a diagnosed Floor 2 chamber, under Floor 1's law: a
    /// repair at full standard — hint rung 4 never viewed — earns
    /// the chamber's facet for the codex.
    pub fn floor2_repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .floor2
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        match progress.state {
            ChamberState::Repaired => {
                Err(Refusal("this chamber already stands repaired".to_string()))
            }
            ChamberState::Unworked => Err(Refusal(
                "there is nothing to repair yet — assemble the proof first".to_string(),
            )),
            ChamberState::Diagnosed => {
                progress.state = ChamberState::Repaired;
                if progress.hints_used <= 3 {
                    progress.facet = true;
                    Ok(
                        "Repaired at full standard. A facet of the Lens Stone settles \
                        into the codex."
                            .to_string(),
                    )
                } else {
                    Ok(
                        "Repaired — though the seam was named, and the record keeps it. \
                        No facet, but the floor still completes."
                            .to_string(),
                    )
                }
            }
        }
    }

    /// C4's exhibit act: place record `record` into `slot`
    /// (0 = kept, 1 = restored). Fills the slot on success.
    pub fn exhibit_record(
        &mut self,
        chamber: usize,
        record: usize,
        slot: usize,
    ) -> Result<String, Refusal> {
        let progress = self
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        let kind_raw = progress
            .work
            .records
            .get(record)
            .copied()
            .ok_or_else(|| Refusal("no such record".to_string()))?;
        let kind = match kind_raw {
            0 => RecordKind::Kept,
            1 => RecordKind::KeptSecond,
            _ => RecordKind::Restored,
        };
        record_fits_slot(kind, slot).map_err(Refusal)?;
        match slot {
            0 => progress.work.slot_kept = Some(record),
            _ => progress.work.slot_restored = Some(record),
        }
        Ok("The record stands exhibited.".to_string())
    }

    /// C4's run: perform the delivery, producing the two records.
    pub fn run_delivery(&mut self, chamber: usize) -> Result<(), Refusal> {
        let progress = self
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        let records = run_delivery(progress.work.commit_second);
        progress.work.records = records
            .iter()
            .map(|record| match record.kind {
                RecordKind::Kept => 0,
                RecordKind::KeptSecond => 1,
                RecordKind::Restored => 2,
            })
            .collect();
        progress.work.ran = true;
        progress.work.slot_kept = None;
        progress.work.slot_restored = None;
        Ok(())
    }

    // ------------------------------------------------------------------
    // The FOE.
    // ------------------------------------------------------------------

    /// Enumerate the master plan into its census.
    pub fn foe_enumerate(&mut self) {
        self.foe.enumerated = true;
    }

    /// Verify one census row's holders (one-way: a verified row
    /// stays verified until the proof is reset).
    pub fn foe_verify_row(&mut self, row: usize) -> Result<(), Refusal> {
        if !self.foe.enumerated {
            return Err(Refusal(
                "there is no census yet — enumerate the master plan first".to_string(),
            ));
        }
        let census = foe_census();
        let entry = census
            .rows
            .get(row)
            .ok_or_else(|| Refusal("no such census row".to_string()))?;
        if entry.copiable {
            return Err(Refusal(format!(
                "the {} may be copied freely — copies need no verification",
                entry.item
            )));
        }
        if let Some(slot) = self.foe.verified.get_mut(row) {
            *slot = true;
        }
        self.foe_maybe_expose();
        Ok(())
    }

    /// Exhibit a census row into the proof slot. Only the
    /// double-promised row belongs there; anything else is refused
    /// with the reason.
    pub fn foe_exhibit_row(&mut self, row: usize) -> Result<String, Refusal> {
        if !self.foe.enumerated {
            return Err(Refusal(
                "there is no census yet — enumerate the master plan first".to_string(),
            ));
        }
        let census = foe_census();
        let entry = census
            .rows
            .get(row)
            .ok_or_else(|| Refusal("no such census row".to_string()))?;
        if entry.copiable {
            return Err(Refusal(format!(
                "the {} may be copied freely — the slot wants an uncopiable thing \
                 promised twice",
                entry.item
            )));
        }
        if !entry.double_promised() {
            return Err(Refusal(format!(
                "the {} stands in one column only — the slot wants the plan's one \
                 double promise",
                entry.item
            )));
        }
        self.foe.exhibited = true;
        self.foe_maybe_expose();
        Ok(format!(
            "Exhibited: the {}, promised to {} — both columns shown.",
            entry.item,
            entry.holders.join(" and ")
        ))
    }

    /// Whether the census proof is complete: every uncopiable row
    /// verified, and the double promise exhibited.
    pub fn foe_census_complete(&self) -> bool {
        if !self.foe.exhibited {
            return false;
        }
        foe_uncopiable_rows()
            .iter()
            .all(|row| self.foe.verified.get(*row).copied().unwrap_or(false))
    }

    /// The moment the census completes, the FOE stands exposed.
    fn foe_maybe_expose(&mut self) {
        if self.foe.state == FoeState::Standing && self.foe_census_complete() {
            self.foe.state = FoeState::Exposed;
        }
    }

    /// Record a passed cargo gate. The gate itself is run by the
    /// shell; this only commits its verdict.
    pub fn foe_mark_gate_passed(&mut self) {
        self.foe.gate_passed = true;
    }

    /// Reset the FOE's proof surface: census verifications and the
    /// exhibit are given back; the player's gate verdict and editor
    /// draft are untouched. Always available.
    pub fn foe_reset_proof(&mut self) {
        self.foe.verified.fill(false);
        self.foe.exhibited = false;
        self.foe.strike = StrikePhase::Idle;
        if self.foe.state == FoeState::Exposed {
            self.foe.state = FoeState::Standing;
        }
    }

    /// Arm the repair strike. Impossible without complete proof:
    /// the FOE exposed by its census, and the gate passed.
    pub fn foe_arm_strike(&mut self) -> Result<(), Refusal> {
        match self.foe.state {
            FoeState::Fallen => Err(Refusal("the Hoard has already fallen".to_string())),
            FoeState::Standing => Err(Refusal(
                "the seal sleeps — the census proof is incomplete".to_string(),
            )),
            FoeState::Exposed => {
                if !self.foe.gate_passed {
                    return Err(Refusal(
                        "the Hoard first — write the Interner's Hoard and pass the \
                         vault's gate; then the strike may be armed"
                            .to_string(),
                    ));
                }
                self.foe.strike = StrikePhase::Armed;
                Ok(())
            }
        }
    }

    /// Stand the strike down.
    pub fn foe_disarm_strike(&mut self) {
        self.foe.strike = StrikePhase::Idle;
    }

    /// Confirm the strike: the second expedition's claim is struck
    /// and re-routed through a lent copy of a lesser chart. Awards
    /// the Plank Stone and the Interner's Hoard, once.
    pub fn foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.foe.state == FoeState::Fallen {
            return Err(Refusal("the Hoard has already fallen".to_string()));
        }
        if self.foe.strike != StrikePhase::Armed {
            return Err(Refusal(
                "the strike is not armed — press the seal first".to_string(),
            ));
        }
        self.foe.strike = StrikePhase::Idle;
        self.foe.state = FoeState::Fallen;
        self.foe.fallen_date = Some(date.to_string());
        if !self.codex.iter().any(|stone| stone == PLANK_STONE) {
            self.codex.push(PLANK_STONE.to_string());
        }
        if !self.pack.iter().any(|loot| loot == INTERNERS_HOARD) {
            self.pack.push(INTERNERS_HOARD.to_string());
        }
        Ok(
            "The strike lands — not confiscation, but re-planning: the second \
            expedition's claim is struck, and it is re-routed through a lent copy \
            of a lesser chart the vault legitimately holds."
                .to_string(),
        )
    }

    /// The Stratum I stairs clauses, each with its standing. The
    /// stairs out of Stratum I need more than this floor can give;
    /// the unmet clauses say so plainly.
    pub fn stairs_clauses(&self) -> [(bool, String); 3] {
        [
            (
                self.foe.state == FoeState::Fallen,
                "FOE 1, the Borrowed Hoard, has fallen".to_string(),
            ),
            (
                self.floor2.foe.state == FoeState::Fallen,
                if self.floor2.foe.state == FoeState::Fallen {
                    "FOE 2, the Menagerie, has fallen".to_string()
                } else {
                    "FOE 2 or FOE 3 has fallen — Floor 3 lies beyond this build".to_string()
                },
            ),
            (
                false,
                if self.foe.gate_passed {
                    "every Stratum I boss crate clippy-clean — Floor 1's lantern is lit; \
                     Floors 2 and 3 have no crates yet"
                        .to_string()
                } else {
                    "every Stratum I boss crate clippy-clean — Floor 1's lantern waits \
                     on the gate"
                        .to_string()
                },
            ),
        ]
    }

    // ------------------------------------------------------------------
    // The save.
    // ------------------------------------------------------------------

    /// Serialize the expedition into its portable save text.
    /// Deterministic: fixed field order, ordered collections, no
    /// hash-map iteration anywhere on the path.
    pub fn save_json(&self) -> Result<String, SaveError> {
        let file = SaveFile {
            magic: SAVE_MAGIC.to_string(),
            version: SAVE_FORMAT_VERSION,
            ambient_seed: self.ambient_seed,
            settings: SaveSettings {
                text_scale: self.settings.text_scale.as_save_str().to_string(),
                high_contrast: self.settings.high_contrast,
                reduced_motion: self.settings.reduced_motion,
            },
            log: self.log.clone(),
            chambers: self.chambers.clone(),
            foe: self.foe.clone(),
            foe_hints_used: self.foe_hints_used,
            codex: self.codex.clone(),
            pack: self.pack.clone(),
            editor_code: self.editor_code.clone(),
            gate_record: self.gate_record.clone(),
            floor2: self.floor2.clone(),
        };
        serde_json::to_string(&file).map_err(|error| SaveError::Corrupt {
            reason: error.to_string(),
        })
    }

    /// Load an expedition from save text. Validate-then-commit: a
    /// refused load changes nothing, and a loaded save is checked
    /// against the design's invariants before it is believed.
    pub fn load_json(text: &str) -> Result<Self, SaveError> {
        // The magic and version are judged before the typed parse,
        // so a foreign file is "not a save" (never "corrupt"), and
        // a newer save is refused by its version, not by its shape.
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|_| SaveError::NotASave)?;
        if value.get("magic").and_then(serde_json::Value::as_str) != Some(SAVE_MAGIC) {
            return Err(SaveError::NotASave);
        }
        if let Some(found) = value
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
        {
            if found > SAVE_FORMAT_VERSION {
                return Err(SaveError::FutureVersion { found });
            }
            if found < 1 {
                return Err(SaveError::UnsupportedVersion { found });
            }
        }
        let file: SaveFile = serde_json::from_value(value).map_err(|error| SaveError::Corrupt {
            reason: error.to_string(),
        })?;
        if file.version > SAVE_FORMAT_VERSION {
            return Err(SaveError::FutureVersion {
                found: file.version,
            });
        }
        if file.version < 1 {
            return Err(SaveError::UnsupportedVersion {
                found: file.version,
            });
        }
        let text_scale = TextScale::from_save_str(&file.settings.text_scale).ok_or_else(|| {
            SaveError::Corrupt {
                reason: format!("unknown text scale {}", file.settings.text_scale),
            }
        })?;
        let expedition = Self {
            settings: Settings {
                text_scale,
                high_contrast: file.settings.high_contrast,
                reduced_motion: file.settings.reduced_motion,
            },
            ambient_seed: file.ambient_seed,
            log: file.log,
            chambers: file.chambers,
            foe: file.foe,
            foe_hints_used: file.foe_hints_used,
            codex: file.codex,
            pack: file.pack,
            editor_code: file.editor_code,
            gate_record: file.gate_record,
            floor2: file.floor2,
        };
        expedition.validate_invariants()?;
        Ok(expedition)
    }

    /// The design's invariants, checked on every load: a repaired
    /// chamber's proof must actually hold, a fallen FOE's awards
    /// must be on the record, facets must have been earned at full
    /// standard, and every work vector must be its authored length.
    fn validate_invariants(&self) -> Result<(), SaveError> {
        let corrupt = |reason: &str| SaveError::Corrupt {
            reason: reason.to_string(),
        };
        if self.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five chambers"));
        }
        if self.editor_code.len() > MAX_EDITOR_BYTES {
            return Err(corrupt("the editor draft exceeds its bound"));
        }
        let census_rows = foe_census().rows.len();
        if self.foe.verified.len() != census_rows {
            return Err(corrupt("the census record is the wrong size"));
        }
        if self.floor2.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five Floor 2 chambers"));
        }
        if self.floor2.foe.audited.len() != ADMISSION_SERVICES.len() {
            return Err(corrupt("the audit record is the wrong size"));
        }
        if self.floor2.admissions.len() != BEASTS.len() {
            return Err(corrupt("the roster record is the wrong size"));
        }
        if self.floor2.roster_collapsed.len() != ROSTER_GROUPS.len() {
            return Err(corrupt("the roster view record is the wrong size"));
        }
        for chamber in &self.floor2.chambers {
            if chamber.facet && (chamber.state != ChamberState::Repaired || chamber.hints_used > 3)
            {
                return Err(corrupt(
                    "a Floor 2 facet stands without a full-standard repair",
                ));
            }
        }
        // Floor 2 chambers answer to the same law: no state beyond
        // Unworked without a proof that recomputes.
        for (index, chamber) in self.floor2.chambers.iter().enumerate() {
            if chamber.state != ChamberState::Unworked && !floor2_proof_complete(self, index) {
                return Err(corrupt(
                    "a Floor 2 chamber stands diagnosed without its proof",
                ));
            }
        }
        for (index, chamber) in self.chambers.iter().enumerate() {
            let work = &chamber.work;
            let lengths_ok = match index {
                0 => work.assignments.len() == PACK_ITEMS.len(),
                1 => work.picks.len() == 5,
                2 => work.leases.len() == 3,
                4 => work.judgements.len() == PLACARDS.len(),
                _ => true,
            };
            if !lengths_ok {
                return Err(corrupt("a chamber's work is the wrong size"));
            }
            if chamber.facet && (chamber.state != ChamberState::Repaired || chamber.hints_used > 3)
            {
                return Err(corrupt("a facet stands without a full-standard repair"));
            }
            if chamber.state != ChamberState::Unworked && !proof_complete(self, index) {
                return Err(corrupt("a chamber stands diagnosed without its proof"));
            }
        }
        match self.foe.state {
            FoeState::Exposed | FoeState::Fallen => {
                if !self.foe_census_complete() {
                    return Err(corrupt("the FOE stands exposed without its census"));
                }
            }
            FoeState::Standing => {}
        }
        if self.foe.state == FoeState::Fallen {
            if !self.foe.gate_passed {
                return Err(corrupt("the FOE fell without the gate"));
            }
            if !self.codex.iter().any(|stone| stone == PLANK_STONE)
                || !self.pack.iter().any(|loot| loot == INTERNERS_HOARD)
            {
                return Err(corrupt("the FOE fell without its awards"));
            }
            if self.foe.fallen_date.is_none() {
                return Err(corrupt("the FOE fell without a date on the chart"));
            }
        }
        Ok(())
    }
}

/// Read lease `index` from saved work, defaulting to an empty
/// lease (which the chain judge will name) if absent.
fn lease_at(leases: &[[i32; 2]], index: usize) -> (i32, i32) {
    leases
        .get(index)
        .map(|pair| (pair[0], pair[1]))
        .unwrap_or((0, 0))
}

/// Whether a chamber's proof currently holds, recomputed from its
/// work — used by the load invariants.
fn proof_complete(expedition: &Expedition, chamber: usize) -> bool {
    let work = &expedition.chambers[chamber].work;
    match chamber {
        0 => {
            let candidates = crate::content::TOLL_CANDIDATES;
            packing_problems(&work.assignments).is_empty()
                && matches!(
                    work.toll_choice,
                    Some(choice) if candidates.get(choice).copied() == Some(reconciled_toll())
                )
        }
        1 => day_plan_overlap(&work.picks).is_none(),
        2 => {
            let leases = [
                lease_at(&work.leases, 0),
                lease_at(&work.leases, 1),
                lease_at(&work.leases, 2),
            ];
            lease_chain_problem(&leases).is_none()
        }
        3 => work.slot_kept.is_some() && work.slot_restored.is_some(),
        4 => {
            let (unjudged, wrong) = judge_placards(&work.judgements);
            unjudged == 0 && wrong.is_empty()
        }
        _ => false,
    }
}

/// Whether a Floor 2 chamber's proof currently holds, recomputed
/// from its work — used by the load invariants, as `proof_complete`
/// is for Floor 1. Chambers whose builds have not landed never
/// hold a proof, so a forged save cannot stand them diagnosed.
fn floor2_proof_complete(expedition: &Expedition, chamber: usize) -> bool {
    let Some(progress) = expedition.floor2.chambers.get(chamber) else {
        return false;
    };
    let work = &progress.work;
    match chamber {
        0 => {
            matches!(work.charter_choice, Some(choice) if crate::content::CHARTER_CANDIDATES.get(choice).is_some_and(|sigil| trick_charter_ok(sigil)))
                && work.watched
                && matches!(work.run_choice, Some(choice) if identification_ok(choice))
        }
        1 => (0..crate::content::STABLE_SERVICES.len()).all(|service| {
            let mask = work
                .bound_masks
                .get(service)
                .copied()
                .unwrap_or(crate::content::POSTED_BOUND);
            bound_exact(service, mask)
        }),
        2 => {
            work.ran_deliveries
                && matches!(work.errand_choice, Some(choice) if errand_choice_ok(choice))
                && matches!(work.kennel_choice, Some(kennel) if kennel_choice_ok(kennel))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Solve chamber 1 in an expedition's work.
    fn solve_c1(expedition: &mut Expedition) {
        expedition.chambers[0].work.assignments =
            vec![0b001, 0b001, 0b011, 0b010, 0b100, 0b010, 0b100, 0b010];
        expedition.chambers[0].work.toll_choice = Some(1);
    }

    /// Validation: chamber 1 files and repairs, and earns its facet.
    #[test]
    fn chamber_one_full_arc() {
        let mut expedition = Expedition::new();
        solve_c1(&mut expedition);
        expedition.file_proof(0).expect("proof files");
        assert_eq!(expedition.chambers[0].state, ChamberState::Diagnosed);
        expedition.repair_chamber(0).expect("repair lands");
        assert_eq!(expedition.chambers[0].state, ChamberState::Repaired);
        assert!(expedition.chambers[0].facet);
    }

    /// Validation: the FOE arc — census, exhibit, gate, strike,
    /// awards — in design order.
    #[test]
    fn foe_full_arc() {
        let mut expedition = Expedition::new();
        expedition.foe_enumerate();
        let double = crate::puzzles::foe_double_row().expect("double row");
        expedition.foe_exhibit_row(double).expect("exhibit");
        for row in foe_uncopiable_rows() {
            expedition.foe_verify_row(row).expect("verify");
        }
        assert_eq!(expedition.foe.state, FoeState::Exposed);
        assert!(expedition.foe_arm_strike().is_err(), "gate first");
        expedition.foe_mark_gate_passed();
        expedition.foe_arm_strike().expect("arm");
        expedition
            .foe_confirm_strike("2026-11-09")
            .expect("strike lands");
        assert_eq!(expedition.foe.state, FoeState::Fallen);
        assert!(expedition.codex.iter().any(|s| s == PLANK_STONE));
        assert!(expedition.pack.iter().any(|s| s == INTERNERS_HOARD));
        assert_eq!(expedition.foe.fallen_date.as_deref(), Some("2026-11-09"));
    }

    /// Validation: a save round-trips byte-identically.
    #[test]
    fn save_round_trips_byte_identically() {
        let mut expedition = Expedition::new();
        solve_c1(&mut expedition);
        expedition.file_proof(0).expect("proof");
        expedition.repair_chamber(0).expect("repair");
        expedition.log_push("Floor 1 entered.");
        let first = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&first).expect("load");
        let second = loaded.save_json().expect("re-save");
        assert_eq!(first, second);
        assert_eq!(loaded, expedition);
    }

    /// Validation: the editor draft survives the round trip.
    #[test]
    fn save_keeps_the_editor_draft() {
        let mut expedition = Expedition::new();
        expedition.editor_code = "pub fn proof() {}".to_string();
        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert_eq!(loaded.editor_code, "pub fn proof() {}");
    }

    /// Adversarial: repairing before proof is refused, state intact.
    #[test]
    fn repair_before_proof_is_refused() {
        let mut expedition = Expedition::new();
        assert!(expedition.repair_chamber(0).is_err());
        assert_eq!(expedition.chambers[0].state, ChamberState::Unworked);
    }

    /// Adversarial: the flawed C1 plan is refused, and only the
    /// toll slot can fill independently when the packing is fixed.
    #[test]
    fn flawed_plan_refused_and_slots_are_independent() {
        let mut expedition = Expedition::new();
        expedition.chambers[0].work.toll_choice = Some(1);
        assert!(expedition.file_proof(0).is_err());
        assert_eq!(expedition.chambers[0].state, ChamberState::Unworked);
        expedition.chambers[0].work.assignments =
            vec![0b001, 0b001, 0b011, 0b010, 0b100, 0b010, 0b100, 0b010];
        expedition.chambers[0].work.toll_choice = Some(0);
        assert!(
            expedition.file_proof(0).is_err(),
            "wrong toll still refuses"
        );
        assert!(expedition.chambers[0].work.packing_ok);
        assert!(!expedition.chambers[0].work.toll_ok);
    }

    /// Adversarial: rung 4 forfeits the facet but not the floor.
    #[test]
    fn seam_named_forfeits_facet_only() {
        let mut expedition = Expedition::new();
        for _ in 0..4 {
            expedition.view_hint(0);
        }
        solve_c1(&mut expedition);
        expedition.file_proof(0).expect("proof");
        expedition.repair_chamber(0).expect("repair");
        assert!(!expedition.chambers[0].facet);
        assert_eq!(expedition.chambers[0].state, ChamberState::Repaired);
    }

    /// Adversarial: exhibiting a single-holder row is refused.
    #[test]
    fn foe_exhibit_refuses_single_holder_rows() {
        let mut expedition = Expedition::new();
        expedition.foe_enumerate();
        let census = foe_census();
        let saw = census
            .rows
            .iter()
            .position(|row| row.item == "Saw")
            .expect("saw row");
        let refusal = expedition.foe_exhibit_row(saw).expect_err("refused");
        assert!(refusal.message().contains("one column only"));
        assert!(!expedition.foe.exhibited);
    }

    /// Adversarial: a future save version is refused by name.
    #[test]
    fn load_refuses_future_versions() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let future = text.replace("\"version\":2", "\"version\":99");
        assert_eq!(
            Expedition::load_json(&future),
            Err(SaveError::FutureVersion { found: 99 })
        );
    }

    /// Adversarial: wrong magic and truncation are refused.
    #[test]
    fn load_refuses_non_saves_and_truncation() {
        assert_eq!(
            Expedition::load_json("{\"hello\": true}"),
            Err(SaveError::NotASave)
        );
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let truncated = &text[..text.len() / 2];
        assert!(Expedition::load_json(truncated).is_err());
    }

    /// Adversarial: a save claiming a repair without its proof is
    /// corrupt, not believed.
    #[test]
    fn load_refuses_repair_without_proof() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let forged = text.replace("\"state\":\"Unworked\"", "\"state\":\"Repaired\"");
        match Expedition::load_json(&forged) {
            Err(SaveError::Corrupt { .. }) => {}
            other => panic!("expected corrupt refusal, got {other:?}"),
        }
    }

    /// Adversarial: reset returns a chamber to its authored start.
    #[test]
    fn reset_restores_the_authored_start() {
        let mut expedition = Expedition::new();
        solve_c1(&mut expedition);
        expedition.file_proof(0).expect("proof");
        expedition.reset_chamber(0);
        assert_eq!(expedition.chambers[0].state, ChamberState::Unworked);
        let fresh = Expedition::new();
        assert_eq!(expedition.chambers[0].work, fresh.chambers[0].work);
    }

    /// Validation: Floor 2 state round-trips through the save.
    #[test]
    fn floor2_state_round_trips() {
        let mut expedition = Expedition::new();
        expedition.floor2.admissions[0] = true;
        expedition.floor2.roster_collapsed[1] = true;
        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert_eq!(loaded, expedition);
        assert!(loaded.floor2.admissions[0]);
        assert!(loaded.floor2.roster_collapsed[1]);
    }

    /// Validation: a version-1 save (no floor2 field at all) loads
    /// with a fresh Floor 2, and re-saves as version 2.
    #[test]
    fn version_one_saves_load_with_default_floor2() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value.as_object_mut().expect("object").remove("floor2");
        value["version"] = serde_json::json!(1);
        let loaded = Expedition::load_json(&value.to_string()).expect("v1 loads");
        assert_eq!(loaded.floor2, Floor2Progress::new());
        let resaved = loaded.save_json().expect("re-save");
        assert!(resaved.contains("\"version\":2"));
    }

    /// Validation: Floor 2 opens only when the Hoard has fallen.
    #[test]
    fn floor2_reachable_follows_the_hoard() {
        let mut expedition = Expedition::new();
        assert!(!expedition.floor2_reachable());
        expedition.foe_enumerate();
        let double = crate::puzzles::foe_double_row().expect("double row");
        expedition.foe_exhibit_row(double).expect("exhibit");
        for row in foe_uncopiable_rows() {
            expedition.foe_verify_row(row).expect("verify");
        }
        expedition.foe_mark_gate_passed();
        expedition.foe_arm_strike().expect("arm");
        expedition.foe_confirm_strike("2026-11-09").expect("strike");
        assert!(expedition.floor2_reachable());
        assert!(!expedition.floor2_complete());
    }

    /// Adversarial: a forged Floor 2 record of the wrong size is
    /// refused as corrupt.
    #[test]
    fn load_refuses_forged_floor2_lengths() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value["floor2"]["chambers"]
            .as_array_mut()
            .expect("chambers array")
            .pop();
        match Expedition::load_json(&value.to_string()) {
            Err(SaveError::Corrupt { .. }) => {}
            other => panic!("expected corrupt refusal, got {other:?}"),
        }
    }

    /// Floor 2 Chamber 1 solved as designed: watched, Forgehand
    /// struck (candidate 1), the variant's body named (performer 1).
    fn solve_f2c1(expedition: &mut Expedition) {
        let work = &mut expedition.floor2.chambers[0].work;
        work.watched = true;
        work.charter_choice = Some(1);
        work.run_choice = Some(1);
    }

    /// Validation: Chamber 1's proof diagnoses, and the repair
    /// lands at full standard with its facet.
    #[test]
    fn floor2_chamber1_proof_diagnoses_and_repairs() {
        let mut expedition = Expedition::new();
        solve_f2c1(&mut expedition);
        expedition.floor2_file_proof(0).expect("proof");
        assert_eq!(expedition.floor2.chambers[0].state, ChamberState::Diagnosed);
        assert!(expedition.floor2.chambers[0].work.charter_ok);
        assert!(expedition.floor2.chambers[0].work.run_ok);
        expedition.floor2_repair_chamber(0).expect("repair");
        assert_eq!(expedition.floor2.chambers[0].state, ChamberState::Repaired);
        assert!(expedition.floor2.chambers[0].facet);
        assert_eq!(expedition.floor2_chambers_repaired(), 1);
    }

    /// Adversarial: filing unwatched is refused even with the right
    /// answers set, and nothing is committed.
    #[test]
    fn floor2_chamber1_unwatched_filing_is_refused() {
        let mut expedition = Expedition::new();
        solve_f2c1(&mut expedition);
        expedition.floor2.chambers[0].work.watched = false;
        let refusal = expedition.floor2_file_proof(0).expect_err("refused");
        assert!(refusal.message().contains("nothing blind"));
        assert_eq!(expedition.floor2.chambers[0].state, ChamberState::Unworked);
        assert!(!expedition.floor2.chambers[0].work.run_ok);
    }

    /// Adversarial: a worn sigil struck as the charter is refused
    /// with the reason, and the house body named for the blind run
    /// is refused too; the repair cannot precede the proof.
    #[test]
    fn floor2_chamber1_resemblance_and_wrong_body_are_refused() {
        let mut expedition = Expedition::new();
        solve_f2c1(&mut expedition);
        expedition.floor2.chambers[0].work.charter_choice = Some(0);
        let refusal = expedition.floor2_file_proof(0).expect_err("refused");
        assert!(refusal.message().contains("FireWard does not charter"));
        assert!(!expedition.floor2.chambers[0].work.charter_ok);
        expedition.floor2.chambers[0].work.charter_choice = Some(1);
        expedition.floor2.chambers[0].work.run_choice = Some(0);
        let refusal = expedition.floor2_file_proof(0).expect_err("refused");
        assert!(refusal.message().contains("not Ember's running"));
        assert_eq!(expedition.floor2.chambers[0].state, ChamberState::Unworked);
        assert!(expedition.floor2_repair_chamber(0).is_err());
    }

    /// Adversarial: a forged out-of-range identification is refused,
    /// never panics, and a rung-4 repair earns no facet.
    #[test]
    fn floor2_chamber1_forged_choice_and_rung4() {
        let mut expedition = Expedition::new();
        solve_f2c1(&mut expedition);
        expedition.floor2.chambers[0].work.run_choice = Some(usize::MAX);
        assert!(expedition.floor2_file_proof(0).is_err());
        expedition.floor2.chambers[0].work.run_choice = Some(1);
        for _ in 0..4 {
            expedition.floor2_view_hint(0);
        }
        assert_eq!(expedition.floor2_hints_used(0), 4);
        expedition.floor2_file_proof(0).expect("proof");
        expedition.floor2_repair_chamber(0).expect("repair");
        assert!(!expedition.floor2.chambers[0].facet);
    }

    /// Validation: Chamber 1's work round-trips through the save,
    /// and a save written before the work field existed loads with
    /// a fresh work record.
    #[test]
    fn floor2_work_round_trips_and_defaults() {
        let mut expedition = Expedition::new();
        solve_f2c1(&mut expedition);
        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert_eq!(
            loaded.floor2.chambers[0].work,
            expedition.floor2.chambers[0].work
        );
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value["floor2"]["chambers"][0]
            .as_object_mut()
            .expect("chamber object")
            .remove("work");
        let loaded = Expedition::load_json(&value.to_string()).expect("load without work");
        assert_eq!(loaded.floor2.chambers[0].work, Floor2Work::default());
    }

    /// Floor 2 Chamber 2 solved as designed: every service's bound
    /// revised to exactly the sigils its work requires.
    fn solve_f2c2(expedition: &mut Expedition) {
        expedition.floor2.chambers[1].work.bound_masks = crate::content::STABLE_SERVICES
            .iter()
            .map(|service| service.required)
            .collect();
    }

    /// Validation: Chamber 2's proof diagnoses, and the repair
    /// lands at full standard with its facet.
    #[test]
    fn floor2_chamber2_proof_diagnoses_and_repairs() {
        let mut expedition = Expedition::new();
        solve_f2c2(&mut expedition);
        expedition.floor2_file_proof(1).expect("proof");
        assert_eq!(expedition.floor2.chambers[1].state, ChamberState::Diagnosed);
        assert!(expedition.floor2.chambers[1].work.bounds_ok);
        expedition.floor2_repair_chamber(1).expect("repair");
        assert_eq!(expedition.floor2.chambers[1].state, ChamberState::Repaired);
        assert!(expedition.floor2.chambers[1].facet);
    }

    /// Adversarial: the posted bound, untouched, is refused with
    /// the first unjustified sigil named; a bound that drops a
    /// required sigil is refused with the missing one named; a
    /// forged all-bits mask is refused without panic.
    #[test]
    fn floor2_chamber2_posted_and_loose_bounds_are_refused() {
        let mut expedition = Expedition::new();
        let refusal = expedition.floor2_file_proof(1).expect_err("refused");
        assert!(refusal.message().contains("MistSense is demanded"));
        assert_eq!(expedition.floor2.chambers[1].state, ChamberState::Unworked);
        assert!(!expedition.floor2.chambers[1].work.bounds_ok);
        solve_f2c2(&mut expedition);
        expedition.floor2.chambers[1].work.bound_masks[0] = 0;
        let refusal = expedition.floor2_file_proof(1).expect_err("refused");
        assert!(refusal.message().contains("no longer asks FireWard"));
        expedition.floor2.chambers[1].work.bound_masks = vec![u8::MAX; 3];
        assert!(expedition.floor2_file_proof(1).is_err());
        assert_eq!(expedition.floor2.chambers[1].state, ChamberState::Unworked);
    }

    /// Floor 2 Chamber 3 solved as designed: the deliveries run,
    /// the River Parcel named (errand 1), the Open Kennel named
    /// (kennel 1).
    fn solve_f2c3(expedition: &mut Expedition) {
        let work = &mut expedition.floor2.chambers[2].work;
        work.ran_deliveries = true;
        work.errand_choice = Some(1);
        work.kennel_choice = Some(1);
    }

    /// Validation: Chamber 3's proof diagnoses, and the repair
    /// lands at full standard with its facet.
    #[test]
    fn floor2_chamber3_proof_diagnoses_and_repairs() {
        let mut expedition = Expedition::new();
        solve_f2c3(&mut expedition);
        expedition.floor2_file_proof(2).expect("proof");
        assert_eq!(expedition.floor2.chambers[2].state, ChamberState::Diagnosed);
        assert!(expedition.floor2.chambers[2].work.errand_ok);
        assert!(expedition.floor2.chambers[2].work.kennel_ok);
        expedition.floor2_repair_chamber(2).expect("repair");
        assert_eq!(expedition.floor2.chambers[2].state, ChamberState::Repaired);
        assert!(expedition.floor2.chambers[2].facet);
    }

    /// Adversarial: filing before the deliveries run is refused
    /// even with the finding set; a smooth errand and the choosing
    /// kennel are each refused with their reasons; a forged
    /// out-of-range errand is refused without panic.
    #[test]
    fn floor2_chamber3_unrun_and_wrong_finding_are_refused() {
        let mut expedition = Expedition::new();
        solve_f2c3(&mut expedition);
        expedition.floor2.chambers[2].work.ran_deliveries = false;
        let refusal = expedition.floor2_file_proof(2).expect_err("refused");
        assert!(refusal.message().contains("nothing unrun"));
        assert!(!expedition.floor2.chambers[2].work.errand_ok);
        expedition.floor2.chambers[2].work.ran_deliveries = true;
        expedition.floor2.chambers[2].work.errand_choice = Some(0);
        let refusal = expedition.floor2_file_proof(2).expect_err("refused");
        assert!(refusal.message().contains("did not run contorted"));
        expedition.floor2.chambers[2].work.errand_choice = Some(1);
        expedition.floor2.chambers[2].work.kennel_choice = Some(0);
        let refusal = expedition.floor2_file_proof(2).expect_err("refused");
        assert!(refusal.message().contains("the ford still stops it"));
        expedition.floor2.chambers[2].work.errand_choice = Some(usize::MAX);
        assert!(expedition.floor2_file_proof(2).is_err());
        assert_eq!(expedition.floor2.chambers[2].state, ChamberState::Unworked);
    }

    /// Validation: the Step 2 work fields round-trip, and a save
    /// written before they existed (a Step 1B save) loads with
    /// them defaulted.
    #[test]
    fn floor2_step2_work_round_trips_and_defaults() {
        let mut expedition = Expedition::new();
        solve_f2c2(&mut expedition);
        solve_f2c3(&mut expedition);
        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert_eq!(
            loaded.floor2.chambers[1].work,
            expedition.floor2.chambers[1].work
        );
        assert_eq!(
            loaded.floor2.chambers[2].work,
            expedition.floor2.chambers[2].work
        );
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        for chamber in [1, 2] {
            let work = value["floor2"]["chambers"][chamber]
                .as_object_mut()
                .expect("chamber object")
                .get_mut("work")
                .and_then(|w| w.as_object_mut())
                .expect("work object");
            for key in [
                "bound_masks",
                "bounds_ok",
                "ran_deliveries",
                "errand_choice",
                "errand_ok",
                "kennel_choice",
                "kennel_ok",
            ] {
                work.remove(key);
            }
        }
        let loaded = Expedition::load_json(&value.to_string()).expect("load old save");
        assert_eq!(loaded.floor2.chambers[1].work, Floor2Work::default());
        assert_eq!(loaded.floor2.chambers[2].work, Floor2Work::default());
    }
    /// Validation: Floor 2 chambers repaired through their proofs survive a
    /// save/load round-trip with states and facets intact — the load invariants
    /// recompute the proofs and hold them.
    #[test]
    fn floor2_repaired_chambers_survive_a_load() {
        let mut expedition = Expedition::new();
        solve_f2c2(&mut expedition);
        expedition.floor2_file_proof(1).expect("proof c2");
        expedition.floor2_repair_chamber(1).expect("repair c2");
        solve_f2c3(&mut expedition);
        expedition.floor2_file_proof(2).expect("proof c3");
        expedition.floor2_repair_chamber(2).expect("repair c3");

        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert_eq!(loaded.floor2.chambers[1].state, ChamberState::Repaired);
        assert!(loaded.floor2.chambers[1].facet);
        assert_eq!(loaded.floor2.chambers[2].state, ChamberState::Repaired);
        assert!(loaded.floor2.chambers[2].facet);
    }

    /// Adversarial: a forged save that stands Floor 2 chambers repaired or
    /// diagnosed on empty work is refused as corrupt — the load invariants
    /// recompute every landed chamber's proof.
    #[test]
    fn load_refuses_forged_floor2_repairs() {
        let expedition = Expedition::new();
        let original_text = expedition.save_json().expect("save");

        for (chamber, state_str) in [(1, "Repaired"), (2, "Diagnosed"), (3, "Repaired")] {
            let mut value: serde_json::Value =
                serde_json::from_str(&original_text).expect("parse json");
            value["floor2"]["chambers"][chamber]["state"] = state_str.into();
            if state_str == "Repaired" {
                value["floor2"]["chambers"][chamber]["facet"] = true.into();
            }

            match Expedition::load_json(&value.to_string()) {
                Err(SaveError::Corrupt { .. }) => {}
                other => panic!("expected corrupt error, got: {:?}", other),
            }
        }
    }
}
