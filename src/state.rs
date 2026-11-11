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
    ADMISSION_SERVICES, BEASTS, BRASS_EYE_STONE, FOE_ADMISSIONS, FOE_CHALLENGE, GATE_SCAFFOLD,
    GATE2_SCAFFOLD, GATEHOUSE_KEY, HERALD_LETTERS, ICE_STONE, INTERNERS_HOARD, LANTERN_STONE,
    LEASES_INITIAL, LENS_STONE, LOCKHEART_PAIR, LOCKHEART_STONE, PACK_ITEMS, PLACARDS, PLANK_STONE,
    PLUGIN_MENAGERIE, PLUMB_LINE, ROSTER_GROUPS, VEILWALKERS_BUFFER,
};
use crate::core::{SAVE_FORMAT_VERSION, SAVE_MAGIC, SaveError, Settings, TextScale};
use crate::puzzles::{
    RecordKind, bound_exact, bound_missing, bound_unjustified, collar_route_ok,
    collision_reading_ok, day_plan_overlap, errand_choice_ok, foe_card_repaired, foe_census,
    foe_uncopiable_rows, foe2_disagreement_row, foe2_row_disagrees, hunt_complete,
    identification_ok, judge_placards, kennel_choice_ok, lease_chain_problem, packing_problems,
    reconciled_toll, record_fits_slot, registry_bars_ok, run_delivery, trick_charter_ok,
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
    /// C4: the Dawn Round has been driven through the Wide Door.
    #[serde(default)]
    pub drove_round: bool,
    /// C4: the bar named for each barred beast, keyed by the
    /// barred beasts in book order, as indices into `BAR_REASONS`.
    #[serde(default)]
    pub bar_choices: Vec<Option<usize>>,
    /// C4: every barred beast's bar stands correctly named.
    #[serde(default)]
    pub bars_ok: bool,
    /// C5: the joining route chosen, as an index into
    /// `COLLAR_ROUTES`.
    #[serde(default)]
    pub route_choice: Option<usize>,
    /// C5: the route slot stands filled.
    #[serde(default)]
    pub route_ok: bool,
    /// C5: the reading of the staged collision the player holds
    /// to, as an index into `COLLISION_READINGS`.
    #[serde(default)]
    pub collision_choice: Option<usize>,
    /// C5: the reading slot stands filled.
    #[serde(default)]
    pub collision_ok: bool,
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
    /// The audit has been taken: the desk's table stands open.
    /// Serde-defaulted: saves written before the FOE landed load
    /// with the table closed.
    #[serde(default)]
    pub audit_taken: bool,
    /// Per audit row, on the FOE's own admission table: the
    /// beast's sigils stand exhibited against its post's bound.
    /// Serde-defaulted; a short record reads as rows unexhibited.
    #[serde(default)]
    pub audit_exhibited: Vec<bool>,
    /// The admission the player names as the one whose slip and
    /// sigils disagree. Serde-defaulted.
    #[serde(default)]
    pub disagreement: Option<usize>,
    /// The player's Admission Desk draft. Serde-defaulted.
    #[serde(default)]
    pub editor_code: String,
    /// The Desk draft has been taken up: once true, an empty
    /// draft is an emptied draft, not an untouched one, and the
    /// scaffold never stands back up in its place. Serde-defaulted.
    #[serde(default)]
    pub editor_started: bool,
    /// The last gate verdict on the Desk, one line, kept for the
    /// gate record. Serde-defaulted.
    #[serde(default)]
    pub gate_record: Option<String>,
}

/// One chamber of Floor 3: state and work in progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor3ChamberProgress {
    /// Resolution state.
    pub state: ChamberState,
    /// The deepest hint rung viewed for this chamber (0 = none).
    pub hints_used: u8,
    /// The facet: earned by a repair at full standard (rung ≤ 3).
    pub facet: bool,
    /// The work in progress.
    #[serde(default)]
    pub work: GatehouseWork,
}

/// Floor 3's chamber work record. Fields land with their
/// chambers (FLOOR3-BRIEF steps 2–4), each serde-defaulted so
/// saves written by earlier steps load whole.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatehouseWork {
    /// Chambers 1–2: the desk/belt charter has been read.
    #[serde(default)]
    pub watched: bool,
    /// Chamber 1: the pair chosen for each report, by report.
    #[serde(default)]
    pub pair_choices: [Option<usize>; 4],
    /// Chamber 2: the portioner's setting.
    #[serde(default)]
    pub portion_choice: Option<usize>,
    /// Chamber 2: the joiner's setting.
    #[serde(default)]
    pub dedup_choice: Option<usize>,
    /// Chamber 2: the gate's setting.
    #[serde(default)]
    pub halt_choice: Option<usize>,
    /// Chamber 3: the verdict for each ledger entry
    /// (0 = named refusal, 1 = the shout stands).
    #[serde(default)]
    pub shout_choices: [Option<usize>; 4],
    /// Chamber 3: the invariant named for each entry.
    #[serde(default)]
    pub invariant_choices: [Option<usize>; 4],
    /// Chamber 4: each slip's stage on the rail (0 unaddressed,
    /// 1 addressed, 2 bodied, 3 sealed, 4 returns tray). The
    /// authored starts live in the manual `Default` below.
    #[serde(default)]
    pub slip_stages: [u8; 3],
    /// Chamber 5: the verdict chosen for each Guideline line.
    #[serde(default)]
    pub checklist_choices: [Option<usize>; 5],
    /// Chamber 5: the reason named for the not-applicable line.
    #[serde(default)]
    pub na_reason: Option<usize>,
    /// Chamber 5: the oldest-road stamp chosen.
    #[serde(default)]
    pub msrv_choice: Option<usize>,
    /// Chamber 5: the amendment rite's stage (0–3).
    #[serde(default)]
    pub rite_stage: u8,
}

impl Default for GatehouseWork {
    fn default() -> Self {
        Self {
            watched: false,
            pair_choices: [None; 4],
            portion_choice: None,
            dedup_choice: None,
            halt_choice: None,
            shout_choices: [None; 4],
            invariant_choices: [None; 4],
            // The rail's authored arrivals: the key request
            // unaddressed, the tally and the stranger's slip
            // addressed (content::REQUEST_SLIPS).
            slip_stages: [0, 1, 1],
            checklist_choices: [None; 5],
            na_reason: None,
            msrv_choice: None,
            rite_stage: 0,
        }
    }
}

/// The Floor 3 FOE's progress: the release arc. The slate, the
/// rite, the three lanterns, and the draft land with the FOE
/// step; the gate record stands from the scaffolding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor3FoeProgress {
    /// Resolution state.
    pub state: FoeState,
    /// The release crate has passed the third cargo gate.
    pub gate_passed: bool,
    /// The strike's phase.
    pub strike: StrikePhase,
    /// The date inscribed when the FOE fell (`YYYY-MM-DD`).
    pub fallen_date: Option<String>,
    /// The log-slate has been taken up (and the FOE Exposed).
    #[serde(default)]
    pub slate_taken: bool,
    /// The verdict per slate entry (0 = it ran, 1 = never ran).
    #[serde(default)]
    pub slate_choices: [Option<usize>; 3],
    /// The slate stands reconciled.
    #[serde(default)]
    pub slate_done: bool,
    /// The oldest-road stamp chosen in the Release room.
    #[serde(default)]
    pub stamp_choice: Option<usize>,
    /// The amendment rite's stage in the Release room (0–3).
    #[serde(default)]
    pub rite_stage: u8,
    /// The three lanterns, lit by doing: format, lint, tests.
    #[serde(default)]
    pub lanterns: [bool; 3],
    /// The player's Release draft, once the editor touches it.
    #[serde(default)]
    pub draft: Option<String>,
    /// The editor has been opened (an emptied draft is a
    /// started draft, not an untouched one).
    #[serde(default)]
    pub editor_started: bool,
    /// The third gate's last summary, for the record view.
    #[serde(default)]
    pub gate_record: Option<String>,
}
/// Floor 3's whole progress record: five chambers and the FOE.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor3Progress {
    /// The five chambers, in design order.
    pub chambers: Vec<Floor3ChamberProgress>,
    /// The FOE.
    pub foe: Floor3FoeProgress,
    /// The deepest hint rung viewed for the FOE.
    pub foe_hints_used: u8,
}

impl Floor3Progress {
    /// A fresh Floor 3: chambers unworked, the FOE standing.
    pub fn new() -> Self {
        Self {
            chambers: (0..5)
                .map(|_| Floor3ChamberProgress {
                    state: ChamberState::Unworked,
                    hints_used: 0,
                    facet: false,
                    work: GatehouseWork::default(),
                })
                .collect(),
            foe: Floor3FoeProgress {
                state: FoeState::Standing,
                gate_passed: false,
                strike: StrikePhase::Idle,
                fallen_date: None,
                slate_taken: false,
                slate_choices: [None; 3],
                slate_done: false,
                stamp_choice: None,
                rite_stage: 0,
                lanterns: [false; 3],
                draft: None,
                editor_started: false,
                gate_record: None,
            },
            foe_hints_used: 0,
        }
    }
}

impl Default for Floor3Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// One chamber of Floor 4: state and work in progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor4ChamberProgress {
    /// Resolution state.
    pub state: ChamberState,
    /// The deepest hint rung viewed for this chamber (0 = none).
    pub hints_used: u8,
    /// The facet: earned by a repair at full standard (rung ≤ 3).
    pub facet: bool,
    /// The work in progress.
    #[serde(default)]
    pub work: FoundationWork,
}

/// Floor 4's chamber work record. Fields land with their
/// chambers (FLOOR4-BRIEF steps 2–4), each serde-defaulted so
/// saves written by earlier steps load whole.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FoundationWork {
    /// Chambers 1–5: the chamber's charter has been read.
    #[serde(default)]
    pub watched: bool,
    /// Chamber 1: the size committed to the slate, per stone.
    #[serde(default)]
    pub pred_size: [Option<u32>; 6],
    /// Chamber 1: the alignment committed, per stone.
    #[serde(default)]
    pub pred_align: [Option<u32>; 6],
    /// Chamber 1: the line has dropped on this stone.
    #[serde(default)]
    pub measured: [bool; 6],
    /// Chamber 1: the annotation chosen for a surprise.
    #[serde(default)]
    pub annotate: [Option<usize>; 6],
    /// Chamber 2: the wall's seating, in order.
    #[serde(default)]
    pub wall_order: Vec<u8>,

    /// Chamber 3: the encoding chosen per crest.
    #[serde(default)]
    pub crest_choices: [Option<usize>; 3],
    /// Chamber 3: the line has measured this crest's choice.
    #[serde(default)]
    pub crest_proven: [bool; 3],
    /// Chamber 4: the wall's seating, in order.
    #[serde(default)]
    pub treaty_order: Vec<u8>,
    /// Chamber 4: the line has dropped on the seated wall.
    #[serde(default)]
    pub treaty_proven: bool,
    /// Chamber 4: the final seat's promise as it currently
    /// stands — the book's naive figure until amended.
    #[serde(default = "default_final_promise")]
    pub final_promise: u32,
    /// Chamber 5: each hollow stone's filling (0 empty,
    /// 1 the whole number, 2 the ratio).
    #[serde(default)]
    pub fillings: [u8; 2],
    /// Chamber 5: the mason's tag per stone (0 untagged,
    /// 1 number, 2 ratio) — kept only in the mason's head.
    #[serde(default)]
    pub tags: [u8; 2],
    /// Chamber 5: the blank stone's cells, filled one by one.
    #[serde(default)]
    pub blank_cells: [bool; 4],
    /// Chamber 5: the blank has been used whole.
    #[serde(default)]
    pub blank_used: bool,
    /// Chamber 5: the referee's catches of a part-filled use.
    #[serde(default)]
    pub referee_catches: u32,
}

/// Chamber 4's final promise starts at the treaty book's
/// naive figure; the player amends it at the wall.
fn default_final_promise() -> u32 {
    crate::content::TREATY_PROMISES[3]
}

impl Default for FoundationWork {
    fn default() -> Self {
        FoundationWork {
            watched: false,
            pred_size: [None; 6],
            pred_align: [None; 6],
            measured: [false; 6],
            annotate: [None; 6],
            wall_order: Vec::new(),
            crest_choices: [None; 3],
            crest_proven: [false; 3],
            treaty_order: Vec::new(),
            treaty_proven: false,
            final_promise: default_final_promise(),
            fillings: [0; 2],
            tags: [0; 2],
            blank_cells: [false; 4],
            blank_used: false,
            referee_catches: 0,
        }
    }
}

/// The Floor 4 FOE's progress: the crooked foundation's arc.
/// The book, the re-survey, and the two re-cuts land with the
/// FOE step; the gate record stands from the scaffolding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor4FoeProgress {
    /// Resolution state.
    pub state: FoeState,
    /// The re-cut has been verified by the line — this FOE's
    /// standing-gate mark (its gate is the line itself).
    pub gate_passed: bool,
    /// The strike's phase.
    pub strike: StrikePhase,
    /// The date inscribed when the FOE fell (`YYYY-MM-DD`).
    pub fallen_date: Option<String>,
    /// The dimension book has been taken down (and the
    /// Foundation stands Exposed).
    #[serde(default)]
    pub book_taken: bool,
    /// The line has dropped on this book entry's stone.
    #[serde(default)]
    pub measured: [bool; 5],
    /// This entry stands exhibited beside its true value.
    #[serde(default)]
    pub exhibited: [bool; 5],
    /// The foreign re-cut's seating, in order.
    #[serde(default)]
    pub foreign_order: Vec<u8>,
    /// The line has verified the foreign set.
    #[serde(default)]
    pub foreign_proven: bool,
    /// The free re-cut's seating, in order.
    #[serde(default)]
    pub free_order: Vec<u8>,
    /// The line has verified the free set at its budget.
    #[serde(default)]
    pub free_proven: bool,
}

/// Floor 4's whole progress record: five chambers and the FOE.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor4Progress {
    /// The five chambers, in design order.
    pub chambers: Vec<Floor4ChamberProgress>,
    /// The FOE.
    pub foe: Floor4FoeProgress,
    /// The deepest hint rung viewed for the FOE.
    pub foe_hints_used: u8,
}

impl Floor4Progress {
    /// A fresh Floor 4: chambers unworked, the FOE standing.
    pub fn new() -> Self {
        Self {
            chambers: (0..5)
                .map(|_| Floor4ChamberProgress {
                    state: ChamberState::Unworked,
                    hints_used: 0,
                    facet: false,
                    work: FoundationWork::default(),
                })
                .collect(),
            foe: Floor4FoeProgress {
                state: FoeState::Standing,
                gate_passed: false,
                strike: StrikePhase::Idle,
                fallen_date: None,
                book_taken: false,
                measured: [false; 5],
                exhibited: [false; 5],
                foreign_order: Vec::new(),
                foreign_proven: false,
                free_order: Vec::new(),
                free_proven: false,
            },
            foe_hints_used: 0,
        }
    }
}

impl Default for Floor4Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// The work inside one Lockheart chamber. Fields land with
/// their chambers; the demonstration flag is here from the
/// scaffolding, like its elders'.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LockWork {
    /// The demonstration has been watched.
    #[serde(default)]
    pub watched: bool,
    /// Chamber 1: the scripted day's operations, walked.
    #[serde(default)]
    pub day_walked: [bool; 6],
    /// Chamber 1: the bell struck — the machine refused the
    /// exclusive opening on the record.
    #[serde(default)]
    pub bell_struck: bool,
    /// Chamber 2: the brazier's five calls, made.
    #[serde(default)]
    pub calls: [bool; 5],
    /// Chamber 3: the spin bound (0 = unset, else 1..=9).
    #[serde(default)]
    pub spin_bound: u8,
    /// Chamber 3: the heart gave up honestly at its bound.
    #[serde(default)]
    pub gave_up: bool,
    /// Chamber 3: the recorded ordering per claim (0 =
    /// unrecorded, else the candidate index + 1) —
    /// provisional, judged on Floor 7 (DS-7).
    #[serde(default)]
    pub spin_annotations: [u8; 4],
    /// Chamber 4: the submitted census, [free, shared,
    /// exclusive].
    #[serde(default)]
    pub census: [u8; 3],
    /// Chamber 4: the slate judged the census true.
    #[serde(default)]
    pub census_judged: bool,
    /// Chamber 5: the cell kind chosen per datum (0 =
    /// unhoused, else the kind index + 1).
    #[serde(default)]
    pub choices: [u8; 3],
    /// Chamber 5: the sentence written per datum's door
    /// (0 = unwritten, else the sentence index + 1).
    #[serde(default)]
    pub invariants: [u8; 3],
}

/// Floor 6, one chamber's progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor6ChamberProgress {
    /// Resolution state.
    pub state: ChamberState,
    /// Hint rungs consumed (4 forfeits the facet).
    pub hints_used: u8,
    /// The facet rides with the repair.
    pub facet: bool,
    /// The chamber's work record.
    pub work: LockWork,
}

/// Floor 6, the FOE's progress: the Heart Locked Twice.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor6FoeProgress {
    /// Resolution state.
    pub state: FoeState,
    /// The safety case is assembled.
    pub gate_passed: bool,
    /// The strike's phase.
    pub strike: StrikePhase,
    /// The date inscribed when the FOE fell (`YYYY-MM-DD`).
    pub fallen_date: Option<String>,
    /// The staff's exhibits, in its order: the census, the
    /// worked day, the torn reading.
    #[serde(default)]
    pub exhibits: [bool; 3],
    /// The model-loom's four interleavings, walked.
    #[serde(default)]
    pub loom_walked: [bool; 4],
    /// The staff's rewritten issue-rule card, line by line.
    #[serde(default)]
    pub card_lines: [u8; 3],
}

/// The whole of Floor 6: five chambers and the FOE.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor6Progress {
    /// The five chambers.
    pub chambers: Vec<Floor6ChamberProgress>,
    /// The FOE record.
    pub foe: Floor6FoeProgress,
    /// Hint rungs spent on the FOE.
    pub foe_hints_used: u8,
}

impl Floor6Progress {
    /// A fresh Floor 6: chambers unworked, the FOE standing.
    pub fn new() -> Self {
        Self {
            chambers: (0..5)
                .map(|_| Floor6ChamberProgress {
                    state: ChamberState::Unworked,
                    hints_used: 0,
                    facet: false,
                    work: LockWork::default(),
                })
                .collect(),
            foe: Floor6FoeProgress {
                state: FoeState::Standing,
                gate_passed: false,
                strike: StrikePhase::Idle,
                fallen_date: None,
                exhibits: [false; 3],
                loom_walked: [false; 4],
                card_lines: [0; 3],
            },
            foe_hints_used: 0,
        }
    }
}

impl Default for Floor6Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// The work inside one Loom chamber. Fields land with
/// their chambers; the demonstration flag is here from the
/// scaffolding, like its elders'.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoomWork {
    /// The demonstration has been watched.
    #[serde(default)]
    pub watched: bool,
    /// Chamber 1: each thread's posted join slot
    /// (0 = unassigned, 1..=6).
    #[serde(default)]
    pub join_order: [u8; 6],
    /// Chamber 1: the judge has stamped the order.
    #[serde(default)]
    pub scoped: bool,
    /// Chamber 2: each tool's posted seal
    /// (0 = unsealed, 1 = Send, 2 = Sync, 3 = Neither).
    #[serde(default)]
    pub seals: [u8; 6],
    /// Chambers 2 and 4: the judge has stamped the work.
    #[serde(default)]
    pub judged: bool,
    /// Chamber 3: the five candidate edges, chalked or not.
    #[serde(default)]
    pub edges: [bool; 5],
    /// Chamber 3: the marked weakened claim
    /// (0 = unmarked, 1..=4 = the claim).
    #[serde(default)]
    pub weak_marked: u8,
    /// Chamber 3: the stale tide has been found by its
    /// witness.
    #[serde(default)]
    pub stale_found: bool,
    /// Chamber 3: the keeper has graded the wall.
    #[serde(default)]
    pub graded: bool,
    /// Chamber 4: the eight foretellings of the book's
    /// script (0 = unforetold).
    #[serde(default)]
    pub predictions: [u8; 8],
    /// Chamber 5: the six verdicts (0 = unentered,
    /// 1 = linearizable, 2 = not).
    #[serde(default)]
    pub verdicts: [u8; 6],
    /// Chamber 5: the named witness pair
    /// (0 = unnamed, 1..=3 = the pair).
    #[serde(default)]
    pub witness: u8,
}

/// Floor 7, one chamber's progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor7ChamberProgress {
    /// Resolution state.
    pub state: ChamberState,
    /// Hint rungs consumed (4 forfeits the facet).
    pub hints_used: u8,
    /// The facet rides with the repair.
    pub facet: bool,
    /// The chamber's work record.
    pub work: LoomWork,
}

/// Floor 7, the FOE's progress: the Loom of Many Hands.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor7FoeProgress {
    /// Resolution state.
    pub state: FoeState,
    /// The safety case is assembled.
    pub gate_passed: bool,
    /// The strike's phase.
    pub strike: StrikePhase,
    /// The date inscribed when the FOE fell (`YYYY-MM-DD`).
    pub fallen_date: Option<String>,
    /// Exhibits demonstrated, in her order.
    #[serde(default)]
    pub exhibits: [bool; 3],
    /// The waits-for chart's edges, drawn or not.
    #[serde(default)]
    pub edges: [bool; 4],
    /// The deadlocking interleaving has been run on the
    /// model loom.
    #[serde(default)]
    pub loom_run: bool,
    /// The queue's stated loads, walked or not.
    #[serde(default)]
    pub loads: [bool; 3],
    /// The at-capacity exhibit has been shown.
    #[serde(default)]
    pub pressure: bool,
    /// The posted order: per-claimable rank
    /// (0 = unranked, 1..=4).
    #[serde(default)]
    pub order: [u8; 4],
}

/// The whole of Floor 7: five chambers and the FOE.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor7Progress {
    /// The five chambers.
    pub chambers: Vec<Floor7ChamberProgress>,
    /// The FOE record.
    pub foe: Floor7FoeProgress,
    /// Hint rungs spent on the FOE.
    pub foe_hints_used: u8,
}

impl Floor7Progress {
    /// A fresh Floor 7: chambers unworked, the FOE standing.
    pub fn new() -> Self {
        Self {
            chambers: (0..5)
                .map(|_| Floor7ChamberProgress {
                    state: ChamberState::Unworked,
                    hints_used: 0,
                    facet: false,
                    work: LoomWork::default(),
                })
                .collect(),
            foe: Floor7FoeProgress {
                state: FoeState::Standing,
                gate_passed: false,
                strike: StrikePhase::Idle,
                fallen_date: None,
                exhibits: [false; 3],
                edges: [false; 4],
                loom_run: false,
                loads: [false; 3],
                pressure: false,
                order: [0; 4],
            },
            foe_hints_used: 0,
        }
    }
}

impl Default for Floor7Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// The work inside one Hour chamber. Fields land with
/// their chambers; the demonstration flag is here from the
/// scaffolding, like its elders'.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct HourWork {
    #[serde(default)]
    pub ledger: [u8; 6],
    #[serde(default)]
    pub parts: [u8; 4],
    #[serde(default)]
    pub trial_walked: bool,
    #[serde(default)]
    pub roster: [u8; 12],
    #[serde(default)]
    pub judged: bool,
    #[serde(default)]
    pub polls: [u8; 5],
    #[serde(default)]
    pub rung: bool,
    #[serde(default)]
    pub torn: bool,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub moved: bool,
    /// The demonstration has been watched.
    #[serde(default)]
    pub watched: bool,
}

/// Floor 8, one chamber's progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor8ChamberProgress {
    /// Resolution state.
    pub state: ChamberState,
    /// Hint rungs consumed (4 forfeits the facet).
    pub hints_used: u8,
    /// The facet rides with the repair.
    pub facet: bool,
    /// The chamber's work record.
    pub work: HourWork,
}

/// Floor 8, the FOE's progress: the Suspended Hour.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor8FoeProgress {
    #[serde(default)]
    pub exhibits: [bool; 3],
    #[serde(default)]
    pub predictions: [Vec<u8>; 4],
    #[serde(default)]
    pub unsequenced: Option<usize>,
    #[serde(default)]
    pub policy_seen: bool,
    #[serde(default)]
    pub pressure: u8,
    #[serde(default)]
    pub card_lines: [u8; 3],
    #[serde(default)]
    pub signal_seal: u8,
    #[serde(default)]
    pub signal_generation: u8,
    #[serde(default)]
    pub page: u8,
    #[serde(default)]
    pub selected: u8,
    /// Resolution state.
    pub state: FoeState,
    /// The safety case is assembled.
    pub gate_passed: bool,
    /// The strike's phase.
    pub strike: StrikePhase,
    /// The date inscribed when the FOE fell (`YYYY-MM-DD`).
    pub fallen_date: Option<String>,
}

/// The whole of Floor 8: five chambers and the FOE.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor8Progress {
    /// The five chambers.
    pub chambers: Vec<Floor8ChamberProgress>,
    /// The FOE record.
    pub foe: Floor8FoeProgress,
    /// Hint rungs spent on the FOE.
    pub foe_hints_used: u8,
}

impl Floor8Progress {
    /// A fresh Floor 8: chambers unworked, the FOE standing.
    pub fn new() -> Self {
        Self {
            chambers: (0..5)
                .map(|_| Floor8ChamberProgress {
                    state: ChamberState::Unworked,
                    hints_used: 0,
                    facet: false,
                    work: HourWork::default(),
                })
                .collect(),
            foe: Floor8FoeProgress {
                exhibits: [false; 3],
                predictions: std::array::from_fn(|_| Vec::new()),
                unsequenced: None,
                policy_seen: false,
                pressure: 0,
                card_lines: [0; 3],
                signal_seal: 0,
                signal_generation: 0,
                page: 0,
                selected: 0,
                state: FoeState::Standing,
                gate_passed: false,
                strike: StrikePhase::Idle,
                fallen_date: None,
            },
            foe_hints_used: 0,
        }
    }
}

impl Default for Floor8Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// Floor 9 — The Hoard Beneath. Work fields arrive with their chambers.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct HoardWork {
    #[serde(default)]
    pub watched: bool,
    #[serde(default)]
    pub predictions: [[u8; 2]; 8],
    #[serde(default)]
    pub judged: bool,
    #[serde(default)]
    pub slots: [u8; 6],
    #[serde(default)]
    pub nexts: [u8; 6],
    #[serde(default)]
    pub probes: [u8; 5],
    #[serde(default)]
    pub flood_cost: u8,
    #[serde(default)]
    pub predicted: u8,
    #[serde(default)]
    pub misses_explained: [bool; 10],
    #[serde(default)]
    pub ran: bool,
    #[serde(default)]
    pub steward_missed: bool,
    #[serde(default)]
    pub blocks: [u8; 8],
    #[serde(default)]
    pub suite_walked: bool,
    #[serde(default)]
    pub last_mile: u8,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor9ChamberProgress {
    #[serde(default = "floor9_unworked")]
    pub state: ChamberState,
    #[serde(default)]
    pub hints_used: u8,
    #[serde(default)]
    pub facet: bool,
    #[serde(default)]
    pub work: HoardWork,
}
fn floor9_unworked() -> ChamberState {
    ChamberState::Unworked
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor9FoeProgress {
    #[serde(default)]
    pub exhibits: [bool; 3],
    #[serde(default)]
    pub classes: [u8; 6],
    #[serde(default)]
    pub census: Option<crate::puzzles::Stamped<u32>>,
    #[serde(default)]
    pub trials: [bool; 3],
    #[serde(default)]
    pub adversaries: [bool; 3],
    #[serde(default)]
    pub reissued: [Option<crate::puzzles::Stamped<u32>>; 3],
    #[serde(default)]
    pub stamps: [u8; 3],
    #[serde(default)]
    pub pressure: u8,
    #[serde(default)]
    pub page: u8,
    #[serde(default = "floor9_standing")]
    pub state: FoeState,
    #[serde(default)]
    pub gate_passed: bool,
    #[serde(default = "floor9_idle")]
    pub strike: StrikePhase,
    #[serde(default)]
    pub fallen_date: Option<String>,
}
fn floor9_standing() -> FoeState {
    FoeState::Standing
}
fn floor9_idle() -> StrikePhase {
    StrikePhase::Idle
}
impl Default for Floor9FoeProgress {
    fn default() -> Self {
        Self {
            exhibits: [false; 3],
            classes: [0; 6],
            census: None,
            trials: [false; 3],
            adversaries: [false; 3],
            reissued: std::array::from_fn(|_| None),
            stamps: [0; 3],
            pressure: 0,
            page: 0,
            state: FoeState::Standing,
            gate_passed: false,
            strike: StrikePhase::Idle,
            fallen_date: None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor9Progress {
    #[serde(default = "floor9_chambers")]
    pub chambers: Vec<Floor9ChamberProgress>,
    #[serde(default)]
    pub foe: Floor9FoeProgress,
    #[serde(default)]
    pub foe_hints_used: u8,
}
fn floor9_chambers() -> Vec<Floor9ChamberProgress> {
    (0..5)
        .map(|_| Floor9ChamberProgress {
            state: ChamberState::Unworked,
            hints_used: 0,
            facet: false,
            work: HoardWork::default(),
        })
        .collect()
}
impl Floor9Progress {
    pub fn new() -> Self {
        Self {
            chambers: floor9_chambers(),
            foe: Floor9FoeProgress::default(),
            foe_hints_used: 0,
        }
    }
}
impl Default for Floor9Progress {
    fn default() -> Self {
        Self::new()
    }
}

/// The work inside one Veilworks chamber. Fields land with
/// their chambers; the demonstration flag is here from the
/// scaffolding, like its elders'.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct VeilWork {
    /// The demonstration has been watched.
    #[serde(default)]
    pub watched: bool,
    /// Chamber 1: the six cards have been read.
    #[serde(default)]
    pub cards_read: [bool; 6],
    /// Chamber 1: steps resolved (walked or refused), 0..=6.
    #[serde(default)]
    pub walked: usize,
    /// Chamber 1: the cracked stone has been refused by name.
    #[serde(default)]
    pub refused: bool,
    /// Chamber 2: the eight calls (0 uncalled, 1 sound,
    /// 2 unsound).
    #[serde(default)]
    pub predictions: [u8; 8],
    /// Chamber 2: the referee has stamped the calls.
    #[serde(default)]
    pub judged: bool,
    /// Chamber 2: per-miss annotation (the deciding rule).
    #[serde(default)]
    pub annotations: [Option<usize>; 8],
    /// Chamber 5: this station has been under the lens.
    #[serde(default)]
    pub lensed: [bool; 6],
    /// Chamber 5: the verdict entered per station (0
    /// unentered, 1 sound, 2 faulted).
    #[serde(default)]
    pub verdicts: [u8; 6],
    /// Chamber 5: the contract showing on each station's
    /// card (only a faulted station's card is judged).
    #[serde(default)]
    pub cards: [u8; 6],
    /// Chamber 3: the chosen phrasing per line, per tool.
    #[serde(default)]
    pub card_lines: [[u8; 3]; 4],
    /// Chamber 4: the chosen seam (words from the left end).
    #[serde(default)]
    pub split: u8,
    /// Chamber 4: the beam stands seamed at the split.
    #[serde(default)]
    pub seamed: bool,
    /// Chamber 4: the referee's over-reach was refused.
    #[serde(default)]
    pub access_refused: bool,
    /// Chamber 4: each half has been worked.
    #[serde(default)]
    pub worked: [bool; 2],
    /// Chamber 4: each half has been dropped, in order.
    #[serde(default)]
    pub dropped: [bool; 2],
}

/// Floor 5, one chamber's progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor5ChamberProgress {
    /// Resolution state.
    pub state: ChamberState,
    /// Hint rungs consumed (4 forfeits the facet).
    pub hints_used: u8,
    /// The facet rides with the repair.
    pub facet: bool,
    /// The chamber's work record.
    pub work: VeilWork,
}

/// Floor 5, the FOE's progress: the Thing Behind the Veil.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor5FoeProgress {
    /// Resolution state.
    pub state: FoeState,
    /// The safety case is assembled.
    pub gate_passed: bool,
    /// The strike's phase.
    pub strike: StrikePhase,
    /// The date inscribed when the FOE fell (`YYYY-MM-DD`).
    pub fallen_date: Option<String>,
    /// The trace's exhibits, in the referee's order: the
    /// live loan, the growth, the lens's verdict.
    #[serde(default)]
    pub exhibits: [bool; 3],
    /// The referee's five standing-gate cases, walked.
    #[serde(default)]
    pub walked: [bool; 5],
    /// The beam's rewritten contract card, line by line.
    #[serde(default)]
    pub card: [u8; 3],
}

/// The whole of Floor 5: five chambers and the FOE.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Floor5Progress {
    /// The five chambers.
    pub chambers: Vec<Floor5ChamberProgress>,
    /// The FOE record.
    pub foe: Floor5FoeProgress,
    /// Hint rungs spent on the FOE.
    pub foe_hints_used: u8,
}

impl Floor5Progress {
    /// A fresh Floor 5: chambers unworked, the FOE standing.
    pub fn new() -> Self {
        Self {
            chambers: (0..5)
                .map(|_| Floor5ChamberProgress {
                    state: ChamberState::Unworked,
                    hints_used: 0,
                    facet: false,
                    work: VeilWork::default(),
                })
                .collect(),
            foe: Floor5FoeProgress {
                state: FoeState::Standing,
                gate_passed: false,
                strike: StrikePhase::Idle,
                fallen_date: None,
                exhibits: [false; 3],
                walked: [false; 5],
                card: [0; 3],
            },
            foe_hints_used: 0,
        }
    }
}

impl Default for Floor5Progress {
    fn default() -> Self {
        Self::new()
    }
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
    /// Per Herald letter: acted on and filed. Serde-defaulted so
    /// saves written before the beat landed load whole; a short
    /// record reads as letters unfiled.
    #[serde(default)]
    pub herald_filed: Vec<bool>,
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
                audit_taken: false,
                audit_exhibited: vec![false; FOE_ADMISSIONS.len()],
                disagreement: None,
                editor_code: String::new(),
                editor_started: false,
                gate_record: None,
            },
            foe_hints_used: 0,
            admissions: vec![false; BEASTS.len()],
            roster_collapsed: vec![false; ROSTER_GROUPS.len()],
            herald_filed: vec![false; HERALD_LETTERS.len()],
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
    #[serde(default)]
    floor3: Floor3Progress,
    #[serde(default)]
    floor4: Floor4Progress,
    #[serde(default)]
    floor5: Floor5Progress,
    #[serde(default)]
    floor6: Floor6Progress,
    #[serde(default)]
    floor7: Floor7Progress,
    #[serde(default)]
    floor8: Floor8Progress,
    #[serde(default)]
    floor9: Floor9Progress,
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
    /// Floor 3's progress record.
    pub floor3: Floor3Progress,
    /// Floor 4's progress record.
    pub floor4: Floor4Progress,
    pub floor5: Floor5Progress,
    pub floor6: Floor6Progress,
    pub floor7: Floor7Progress,
    pub floor8: Floor8Progress,
    pub floor9: Floor9Progress,
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
            floor3: Floor3Progress::new(),
            floor4: Floor4Progress::new(),
            floor5: Floor5Progress::new(),
            floor6: Floor6Progress::new(),
            floor7: Floor7Progress::new(),
            floor8: Floor8Progress::new(),
            floor9: Floor9Progress::new(),
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
            3 => {
                let (bars_ok, drove, unnamed, misnamed) = {
                    let work = &mut progress.work;
                    work.bars_ok = work.drove_round && registry_bars_ok(&work.bar_choices);
                    let barred: Vec<&crate::content::RegistryBeast> =
                        crate::content::REGISTRY_BEASTS
                            .iter()
                            .filter(|entry| !entry.passes)
                            .collect();
                    let unnamed = barred
                        .iter()
                        .enumerate()
                        .find(|(slot, _)| work.bar_choices.get(*slot).copied().flatten().is_none())
                        .map(|(_, entry)| entry.name);
                    let misnamed = barred.iter().enumerate().find_map(|(slot, entry)| {
                        match work.bar_choices.get(slot).copied().flatten() {
                            Some(label) if Some(label) != entry.bar => Some((entry.name, label)),
                            _ => None,
                        }
                    });
                    (work.bars_ok, work.drove_round, unnamed, misnamed)
                };
                if bars_ok {
                    progress.state = ChamberState::Diagnosed;
                    Ok("The registry stands read: three beasts answer the Dawn \
                         Round in their own bodies and pass, and the three the \
                         door refuses are named with their bars — a body \
                         returned whole, a kind named fresh at every call, a \
                         rite with no self to drive. The registry door swings \
                         open."
                        .to_string())
                } else if !drove {
                    Err(Refusal(
                        "The registry asks nothing undriven: drive the Dawn \
                         Round through the Wide Door first, and watch which \
                         beasts the door admits and which it refuses"
                            .to_string(),
                    ))
                } else if let Some(name) = unnamed {
                    Err(Refusal(format!(
                        "{name} stands barred and unnamed: name the bar on \
                         every refused beast's placard"
                    )))
                } else if let Some((name, label)) = misnamed {
                    let said = crate::content::BAR_REASON_LABELS
                        .get(label)
                        .copied()
                        .unwrap_or("that bar");
                    Err(Refusal(format!(
                        "{name}'s placard does not say {said}. Read the shape \
                         of its charter again"
                    )))
                } else {
                    Err(Refusal(
                        "the registry's bars do not yet stand named".to_string(),
                    ))
                }
            }
            4 => {
                let (route_ok, collision_ok, route_choice, collision_choice) = {
                    let work = &mut progress.work;
                    work.route_ok = matches!(
                        work.route_choice,
                        Some(choice) if collar_route_ok(choice)
                    );
                    work.collision_ok = matches!(
                        work.collision_choice,
                        Some(choice) if collision_reading_ok(choice)
                    );
                    (
                        work.route_ok,
                        work.collision_ok,
                        work.route_choice,
                        work.collision_choice,
                    )
                };
                if route_ok && collision_ok {
                    progress.state = ChamberState::Diagnosed;
                    Ok("The joining stands lawful: the foreign sigil rides on \
                         a collar of your own forging, and the collision reads \
                         as the rule itself — a sigil claimed once for a beast, \
                         or it is not claimed lawfully at all. The chamber \
                         door swings open."
                        .to_string())
                } else if route_choice.is_none() {
                    Err(Refusal(
                        "Choose the joining first: stamp the beast, forge \
                         the collar, or ask the door for a blanket joining"
                            .to_string(),
                    ))
                } else if !route_ok {
                    let refusal = route_choice
                        .and_then(|choice| crate::content::COLLAR_ROUTES.get(choice))
                        .map(|route| route.refusal)
                        .unwrap_or("That route is refused at the desk.");
                    Err(Refusal(refusal.to_string()))
                } else if collision_choice.is_none() {
                    Err(Refusal(
                        "Read the staged collision: two collars, one sigil, \
                         one beast — say what their collision specifies"
                            .to_string(),
                    ))
                } else {
                    Err(Refusal(
                        "That reading does not hold. Neither keeper erred, \
                         and the door keeps no first-forged rule: read the \
                         collision as the rule stating its own shape"
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

    /// Act on one of the Herald's sealed letters (Floor 2's
    /// secondary beat): the letter is filed, and the narration
    /// says what the acting found. A letter announcing a service
    /// no beast offers wastes the free miniature expedition —
    /// the floor's first wrong room. Filing is free and final.
    pub fn floor2_herald_act(&mut self, letter: usize) -> Result<String, Refusal> {
        let Some(sealed) = crate::content::HERALD_LETTERS.get(letter) else {
            return Err(Refusal("no such letter".to_string()));
        };
        if self.floor2.herald_filed.len() < crate::content::HERALD_LETTERS.len() {
            self.floor2
                .herald_filed
                .resize(crate::content::HERALD_LETTERS.len(), false);
        }
        if self.floor2.herald_filed[letter] {
            return Err(Refusal("that letter already stands filed".to_string()));
        }
        self.floor2.herald_filed[letter] = true;
        Ok(if sealed.offered {
            crate::content::HERALD_FILED
        } else {
            crate::content::HERALD_WRONG_ROOM
        }
        .to_string())
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

    // ------------------------------------------------------------------
    // Floor 2's FOE — the Menagerie at its desk.
    // ------------------------------------------------------------------

    /// View a hint rung for Floor 2's FOE: Floor 1's ladder law,
    /// kept on Floor 2's own record.
    pub fn floor2_foe_view_hint(&mut self) -> u8 {
        self.floor2.foe_hints_used = (self.floor2.foe_hints_used + 1).min(4);
        self.floor2.foe_hints_used
    }

    /// The deepest hint rung recorded for Floor 2's FOE.
    pub fn floor2_foe_hints_used(&self) -> u8 {
        self.floor2.foe_hints_used
    }

    /// The player's Admission Desk draft: the stored text, or the
    /// scaffold while the draft stands untouched.
    pub fn floor2_foe_draft(&self) -> &str {
        if self.floor2.foe.editor_code.is_empty() && !self.floor2.foe.editor_started {
            GATE2_SCAFFOLD
        } else {
            &self.floor2.foe.editor_code
        }
    }

    /// The player's Admission Desk draft, writable: an untouched
    /// draft first takes the scaffold's shape.
    pub fn floor2_foe_draft_mut(&mut self) -> &mut String {
        if !self.floor2.foe.editor_started {
            self.floor2.foe.editor_started = true;
            if self.floor2.foe.editor_code.is_empty() {
                self.floor2.foe.editor_code = GATE2_SCAFFOLD.to_string();
            }
        }
        &mut self.floor2.foe.editor_code
    }

    /// Take the audit: the desk's table stands open — seven slips
    /// to read against their beasts' own sigils.
    pub fn floor2_foe_take_audit(&mut self) {
        self.floor2.foe.audit_taken = true;
    }

    /// Exhibit one audit row: the beast's sigils, read against
    /// the bound of the post its slip claims. When every row of a
    /// service stands exhibited, the audit has read that service's
    /// bound, and the record says so.
    pub fn floor2_foe_exhibit(&mut self, row: usize) -> Result<String, Refusal> {
        if !self.floor2.foe.audit_taken {
            return Err(Refusal(
                "there is no audit yet — take the desk's admissions first".to_string(),
            ));
        }
        let Some(admission) = FOE_ADMISSIONS.get(row) else {
            return Err(Refusal("no such admission".to_string()));
        };
        if self.floor2.foe.audit_exhibited.len() < FOE_ADMISSIONS.len() {
            self.floor2
                .foe
                .audit_exhibited
                .resize(FOE_ADMISSIONS.len(), false);
        }
        self.floor2.foe.audit_exhibited[row] = true;
        let service = admission.service;
        let service_done = FOE_ADMISSIONS
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.service == service)
            .all(|(index, _)| self.floor2.foe.audit_exhibited[index]);
        if service_done {
            if self.floor2.foe.audited.len() < ADMISSION_SERVICES.len() {
                self.floor2
                    .foe
                    .audited
                    .resize(ADMISSION_SERVICES.len(), false);
            }
            self.floor2.foe.audited[service] = true;
        }
        self.floor2_foe_maybe_expose();
        Ok(format!(
            "Exhibited: {} — sigils {} against the bound of {}.",
            admission.name,
            admission.sigils.join(", "),
            ADMISSION_SERVICES[service].name,
        ))
    }

    /// Name the admission whose slip and sigils disagree. A sound
    /// admission is refused with its own evidence: the sigils that
    /// satisfy its post's bound.
    pub fn floor2_foe_name_disagreement(&mut self, row: usize) -> Result<String, Refusal> {
        if !self.floor2.foe.audit_taken {
            return Err(Refusal(
                "there is no audit yet — take the desk's admissions first".to_string(),
            ));
        }
        let Some(admission) = FOE_ADMISSIONS.get(row) else {
            return Err(Refusal("no such admission".to_string()));
        };
        if !foe2_row_disagrees(row) {
            let post = ADMISSION_SERVICES[admission.service].name;
            let bound = ADMISSION_SERVICES[admission.service].bound.join(", ");
            return Err(Refusal(format!(
                "{}'s slip agrees with its sigils: {} stand against the bound \
                 of {post} ({bound}). The seam is elsewhere on the table",
                admission.name,
                admission.sigils.join(", "),
            )));
        }
        self.floor2.foe.disagreement = Some(row);
        self.floor2_foe_maybe_expose();
        Ok(format!(
            "Named: {} — the slip reads '{}', while its sigils read {}. \
             The slip and the sigils disagree.",
            admission.name,
            admission.slip,
            admission.sigils.join(", "),
        ))
    }

    /// Calibrate the Lens: run the named suspect against the
    /// valley's one known landmark — the lit brazier — and keep
    /// its report with the audit. The suspect must be named
    /// first: calibration answers a suspicion; it does not go
    /// looking for one.
    pub fn floor2_foe_calibrate(&mut self) -> Result<String, Refusal> {
        let Some(row) = self.floor2.foe.disagreement else {
            return Err(Refusal(
                "calibrate whom? Name the admission whose slip and sigils \
                 disagree first — the audit points at the suspect"
                    .to_string(),
            ));
        };
        let Some(admission) = FOE_ADMISSIONS.get(row) else {
            return Err(Refusal(
                "the named admission is not on the table".to_string(),
            ));
        };
        self.floor2.foe.calibrated = true;
        self.floor2_foe_maybe_expose();
        Ok(format!(
            "Set before the lit brazier, {} lowers its nose, breathes deep, \
             and reports fog. The calibration record stands with the audit.",
            admission.name,
        ))
    }

    /// Whether the FOE's proof is complete: every admission on
    /// the desk's table exhibited, the one disagreement named,
    /// and the Lens calibrated against the brazier.
    pub fn floor2_foe_proof_complete(&self) -> bool {
        let foe = &self.floor2.foe;
        foe.disagreement == foe2_disagreement_row()
            && foe.calibrated
            && (0..FOE_ADMISSIONS.len())
                .all(|row| foe.audit_exhibited.get(row).copied().unwrap_or(false))
    }

    /// The moment the audit proof completes, the FOE stands exposed.
    fn floor2_foe_maybe_expose(&mut self) {
        if self.floor2.foe.state == FoeState::Standing && self.floor2_foe_proof_complete() {
            self.floor2.foe.state = FoeState::Exposed;
        }
    }

    /// Record a passed cargo gate on the Desk. The gate itself is
    /// run by the shell; this only commits its verdict.
    pub fn floor2_foe_mark_gate_passed(&mut self) {
        self.floor2.foe.gate_passed = true;
    }

    /// Reset the FOE's proof surface: the audit exhibits, the
    /// naming, and the calibration are given back; the player's
    /// gate verdict and Desk draft are untouched. Always available.
    pub fn floor2_foe_reset_proof(&mut self) {
        let foe = &mut self.floor2.foe;
        foe.audit_exhibited.fill(false);
        foe.audited.fill(false);
        foe.disagreement = None;
        foe.calibrated = false;
        foe.strike = StrikePhase::Idle;
        if foe.state == FoeState::Exposed {
            foe.state = FoeState::Standing;
        }
    }

    /// Arm the repair strike. Impossible without complete proof:
    /// the Menagerie exposed by its audit, and the gate passed.
    pub fn floor2_foe_arm_strike(&mut self) -> Result<(), Refusal> {
        match self.floor2.foe.state {
            FoeState::Fallen => Err(Refusal("the Menagerie has already fallen".to_string())),
            FoeState::Standing => Err(Refusal(
                "the seal sleeps — the audit proof is incomplete".to_string(),
            )),
            FoeState::Exposed => {
                if !self.floor2.foe.gate_passed {
                    return Err(Refusal(
                        "the Desk first — re-post the fire-watch bound as the \
                         Admission Desk and pass the vault's gate; then the \
                         strike may be armed"
                            .to_string(),
                    ));
                }
                self.floor2.foe.strike = StrikePhase::Armed;
                Ok(())
            }
        }
    }

    /// Stand the strike down.
    pub fn floor2_foe_disarm_strike(&mut self) {
        self.floor2.foe.strike = StrikePhase::Idle;
    }

    /// Confirm the strike: the fire-watch bound is re-posted at
    /// the desk in letters of iron, and the mist-hound is
    /// honourably reassigned to the mist-watch it is sigilled
    /// for. Awards the Lens Stone and the Plugin Menagerie, once.
    pub fn floor2_foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.floor2.foe.state == FoeState::Fallen {
            return Err(Refusal("the Menagerie has already fallen".to_string()));
        }
        if self.floor2.foe.strike != StrikePhase::Armed {
            return Err(Refusal(
                "the strike is not armed — press the seal first".to_string(),
            ));
        }
        self.floor2.foe.strike = StrikePhase::Idle;
        self.floor2.foe.state = FoeState::Fallen;
        self.floor2.foe.fallen_date = Some(date.to_string());
        if !self.codex.iter().any(|stone| stone == LENS_STONE) {
            self.codex.push(LENS_STONE.to_string());
        }
        if !self.pack.iter().any(|loot| loot == PLUGIN_MENAGERIE) {
            self.pack.push(PLUGIN_MENAGERIE.to_string());
        }
        Ok(
            "The strike lands — not a beast punished, but a bound re-posted: \
             the fire-watch's charter is nailed to the desk in letters of \
             iron, and the desk refuses domain-less admissions from this \
             day. The mist-hound goes honourably to the mist-watch it was \
             always sigilled for, and reports the fog it was made to read."
                .to_string(),
        )
    }

    /// Whether a Floor 3 chamber's diagnosis is complete.
    /// All five chambers judge their work here.
    pub fn floor3_diagnosed(&self, chamber: usize) -> bool {
        let Some(progress) = self.floor3.chambers.get(chamber) else {
            return false;
        };
        let work = &progress.work;
        match chamber {
            0 => {
                work.watched
                    && work
                        .pair_choices
                        .iter()
                        .enumerate()
                        .all(|(report, choice)| {
                            matches!(choice, Some(choice) if crate::puzzles::failure_pair_ok(report, *choice))
                        })
            }
            1 => {
                work.watched
                    && matches!(
                        (work.portion_choice, work.dedup_choice, work.halt_choice),
                        (Some(portion), Some(dedup), Some(halt))
                            if crate::puzzles::conveyor_diagnosed(portion, dedup, halt)
                    )
            }
            2 => {
                work.watched
                    && work
                        .shout_choices
                        .iter()
                        .enumerate()
                        .all(|(case, verdict)| {
                            crate::puzzles::shout_case_ok(
                                case,
                                *verdict,
                                work.invariant_choices[case],
                            )
                        })
            }
            3 => work.watched && crate::puzzles::rail_complete(&work.slip_stages),
            4 => {
                work.watched
                    && work
                        .checklist_choices
                        .iter()
                        .enumerate()
                        .all(|(item, verdict)| {
                            crate::puzzles::inspection_item_ok(item, *verdict)
                        })
                    && crate::puzzles::inspection_reason_ok(work.na_reason)
                    && crate::puzzles::msrv_ok(work.msrv_choice)
            }
            _ => false,
        }
    }
    /// Act on one slip of Chamber 4's rail: advance it one
    /// stage in its order, seal it (only from bodied — the
    /// press's interlock refuses earlier), or send it to the
    /// returns tray (only the body-less slip belongs there).
    /// Actions: 0 advance, 1 seal, 2 return.
    pub fn floor3_slip_act(&mut self, slip: usize, action: usize) -> Result<String, Refusal> {
        let Some(meta) = crate::content::REQUEST_SLIPS.get(slip) else {
            return Err(Refusal("no such slip".to_string()));
        };
        let work = &mut self.floor3.chambers[3].work;
        let stage = work.slip_stages[slip];
        match action {
            0 => {
                if stage >= 3 {
                    return Err(Refusal(
                        "the slip has left the rail — there is nothing to advance".to_string(),
                    ));
                }
                if stage == 1 && meta.body_missing {
                    return Err(Refusal(
                        "the body tray is empty: this slip's body page is \
                         missing, and the rail will not pretend one. It \
                         belongs in the returns tray"
                            .to_string(),
                    ));
                }
                work.slip_stages[slip] = stage + 1;
                Ok(format!(
                    "{} moves on: now {}.",
                    meta.name,
                    crate::content::stage_name(stage + 1)
                ))
            }
            1 => {
                if stage == 3 {
                    return Err(Refusal("the slip already stands sealed".to_string()));
                }
                if stage == 4 {
                    return Err(Refusal(
                        "the slip stands in the returns tray; the press does \
                         not reach there"
                            .to_string(),
                    ));
                }
                if stage < 2 {
                    return Err(Refusal(
                        "the press will not fall: the rail has not brought a \
                         bodied request beneath it. Iron, not policy"
                            .to_string(),
                    ));
                }
                work.slip_stages[slip] = 3;
                Ok(format!("{} is sealed under the press.", meta.name))
            }
            _ => {
                if stage == 4 {
                    return Err(Refusal(
                        "the slip already stands in the returns tray".to_string(),
                    ));
                }
                if !meta.body_missing {
                    return Err(Refusal(
                        "the returns tray is for slips that cannot be bodied \
                         honestly — this one can"
                            .to_string(),
                    ));
                }
                work.slip_stages[slip] = 4;
                Ok(format!(
                    "{} goes to the returns tray, its missing page noted.",
                    meta.name
                ))
            }
        }
    }

    /// Reset one Floor 3 chamber's work to its starting
    /// state: hint usage stays on the record. (Lands late —
    /// D-1189 fold-in: the button existed from Floor 3's
    /// first step but the press had nowhere to land.)
    pub fn floor3_reset_chamber(&mut self, chamber: usize) {
        if let Some(progress) = self.floor3.chambers.get_mut(chamber) {
            progress.state = ChamberState::Unworked;
            progress.facet = false;
            progress.work = GatehouseWork::default();
        }
    }

    /// File a Floor 3 chamber's proof: the diagnosis is checked
    /// facet by facet and the chamber moves to Diagnosed.
    pub fn floor3_file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        if !self.floor3_diagnosed(chamber) {
            return Err(Refusal(match chamber {
                0 => "the sort is not yet true: every report under its own \
                      pair, and the Broken bin untouched"
                    .to_string(),
                1 => "the belt does not yet deliver the slate: set the \
                      three stations and let the desk run the chain"
                    .to_string(),
                2 => "the audit is not yet true: every shout under its \
                      kind, and every standing shout carrying its invariant"
                    .to_string(),
                3 => "the rail is not yet true: both honest slips sealed, \
                      and the body-less slip in the returns tray"
                    .to_string(),
                4 => "the inspection is not yet true: every line under its \
                      honest verdict, the reason named, and the stamp the \
                      oldest road"
                    .to_string(),
                _ => "this desk is not open yet".to_string(),
            }));
        }
        let progress = self
            .floor3
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        if progress.state == ChamberState::Repaired {
            return Err(Refusal("this chamber already stands repaired".to_string()));
        }
        progress.state = ChamberState::Diagnosed;
        Ok(match chamber {
            0 => "Proof filed. Four reports hang under four named doors, \
                  each with its remedy written under the name — and the \
                  Broken bin stands empty for the first time in the \
                  Gatewright's keeping."
                .to_string(),
            1 => "Proof filed. The desk ran the chain itself: the whole \
                  belt taken, first sightings kept, the cracked gear \
                  lifted off — gear, spring, cam, bell, in order, and \
                  silence past the end."
                .to_string(),
            2 => "Proof filed. Two shouts are struck from the ledger and \
                  re-entered as named refusals, posted where travellers \
                  can read them; two stand, each with its invariant \
                  written beneath in the keeper's hand."
                .to_string(),
            3 => "Proof filed. Two seals stand in the wax, and the \
                  returns tray holds the third slip with its missing \
                  page noted — nothing sealed that was not bodied, \
                  nothing bodied that was not addressed."
                .to_string(),
            _ => "Proof filed. Five lines answered honestly — two \
                  conform, two stand repaired, one set aside with its \
                  reason named — and the stamp reads the oldest road: \
                  1.85."
                .to_string(),
        })
    }

    /// Perform the next act of Chamber 5's amendment rite:
    /// read the proposal, second it, enter it in the book —
    /// in that order, at the side desk.
    pub fn floor3_rite_act(&mut self) -> Result<String, Refusal> {
        let work = &mut self.floor3.chambers[4].work;
        let stage = work.rite_stage;
        if stage >= 3 {
            return Err(Refusal(
                "the rite stands complete — the amendment is in the book".to_string(),
            ));
        }
        work.rite_stage = stage + 1;
        Ok(match stage {
            0 => "You read the proposal aloud, and the office hears it: \
                  a lantern lit by schedule is no lantern."
                .to_string(),
            1 => "The Gatewright seconds it, in her own hand: run, or \
                  the release does not leave."
                .to_string(),
            _ => "Entered in the book, under the Guidelines' five lines — \
                  the wall has a sixth line now, and the Release will \
                  be judged by it."
                .to_string(),
        })
    }

    /// Take up the Release's log-slate: the FOE is Exposed
    /// from this moment.
    pub fn floor3_foe_take_slate(&mut self) {
        self.floor3.foe.slate_taken = true;
        if self.floor3.foe.state == FoeState::Standing {
            self.floor3.foe.state = FoeState::Exposed;
        }
    }

    /// Whether the slate stands reconciled: every entry's
    /// verdict true, the true slate written.
    pub fn floor3_foe_slate_done(&self) -> bool {
        self.floor3.foe.slate_done
    }

    /// Reconcile the slate: with every entry judged, write the
    /// true slate over the claimed one.
    pub fn floor3_foe_reconcile(&mut self) -> Result<String, Refusal> {
        let foe = &mut self.floor3.foe;
        if !foe.slate_taken {
            return Err(Refusal(
                "take the slate up first — a reconciliation needs the slate in hand".to_string(),
            ));
        }
        let judged = foe
            .slate_choices
            .iter()
            .enumerate()
            .all(|(entry, verdict)| crate::puzzles::slate_entry_ok(entry, *verdict));
        if !judged {
            return Err(Refusal(
                "the verdicts do not yet read true: judge every entry by \
                 its paper and its hand, then reconcile"
                    .to_string(),
            ));
        }
        foe.slate_done = true;
        Ok(
            "The slate is reconciled: two passes struck through as never \
            run, one standing as it was written. The lantern over the \
            door flickers — it is not used to being believed honestly."
                .to_string(),
        )
    }

    /// Perform the next act of the Release's amendment rite —
    /// the same rite as the Inspection's side desk, performed
    /// here on the release's own amendment.
    pub fn floor3_foe_rite_act(&mut self) -> Result<String, Refusal> {
        let stage = self.floor3.foe.rite_stage;
        if stage >= 3 {
            return Err(Refusal(
                "the rite stands complete — the amendment is in the book".to_string(),
            ));
        }
        self.floor3.foe.rite_stage = stage + 1;
        Ok(match stage {
            0 => "You read the proposal aloud at the great door.".to_string(),
            1 => "The Gatewright seconds it, in her own hand.".to_string(),
            _ => "Entered in the book. The Release stands under a line \
                  it did not have this morning."
                .to_string(),
        })
    }

    /// The Release's proof, whole: the slate reconciled, the
    /// stamp the oldest road, the rite complete. (The gate is
    /// the strike's other half, tracked apart.)
    pub fn floor3_foe_proof_complete(&self) -> bool {
        self.floor3.foe.slate_done
            && crate::puzzles::msrv_ok(self.floor3.foe.stamp_choice)
            && self.floor3.foe.rite_stage >= 3
    }

    /// Light a lantern by its doing (game-side, on the shell's
    /// reports): kind 0 takes the formatted draft back, kind 1
    /// is the lint run's word.
    pub fn floor3_foe_light_lantern(&mut self, kind: u8, formatted: Option<String>) {
        if kind == 0 {
            if let Some(text) = formatted {
                self.floor3.foe.draft = Some(text);
            }
            self.floor3.foe.lanterns[0] = true;
        } else if kind == 1 {
            self.floor3.foe.lanterns[1] = true;
        }
    }

    /// The gate's tests stood: the third lantern lights.
    pub fn floor3_foe_mark_tests_passed(&mut self) {
        self.floor3.foe.lanterns[2] = true;
    }

    /// The third gate passed whole: the strike may be armed.
    pub fn floor3_foe_mark_gate_passed(&mut self) {
        self.floor3.foe.gate_passed = true;
    }

    /// The Release's draft as the editor reads it: the
    /// scaffold until the editor first touches it.
    pub fn floor3_foe_draft(&self) -> &str {
        if !self.floor3.foe.editor_started {
            crate::content::GATE3_SCAFFOLD
        } else {
            self.floor3.foe.draft.as_deref().unwrap_or("")
        }
    }

    /// The Release's draft, materializing the scaffold on
    /// first touch (the editor's "empty = untouched" rule,
    /// with the Step 5 lesson: emptied is not untouched — see
    /// `editor_started`).
    pub fn floor3_foe_draft_mut(&mut self) -> &mut String {
        let foe = &mut self.floor3.foe;
        if !foe.editor_started {
            foe.editor_started = true;
            if foe.draft.is_none() {
                foe.draft = Some(crate::content::GATE3_SCAFFOLD.to_string());
            }
        }
        foe.draft.as_mut().expect("draft materialized")
    }

    /// Any touch of the draft douses all three lanterns and
    /// unmakes the gate's pass: a lantern lit for a draft that
    /// no longer stands is the FOE's own failure wearing the
    /// player's hand.
    pub fn floor3_foe_douse_lanterns(&mut self) {
        self.floor3.foe.lanterns = [false; 3];
        self.floor3.foe.gate_passed = false;
    }

    /// Arm the Release's strike: proof complete and the gate
    /// passed, as with every FOE.
    pub fn floor3_foe_arm_strike(&mut self) -> Result<(), Refusal> {
        if self.floor3.foe.state == FoeState::Fallen {
            return Err(Refusal("the Release has already left".to_string()));
        }
        if !self.floor3_foe_proof_complete() {
            return Err(Refusal(
                "the strike sleeps until the proof is complete: the slate \
                 reconciled, the stamp true, the rite entered"
                    .to_string(),
            ));
        }
        if !self.floor3.foe.gate_passed {
            return Err(Refusal(
                "the strike sleeps until the third gate passes: three \
                 lanterns, lit by doing, on the draft as it stands"
                    .to_string(),
            ));
        }
        self.floor3.foe.strike = StrikePhase::Armed;
        Ok(())
    }

    /// Confirm the strike: the Release falls, and the Lantern
    /// Stone and the Gatehouse Key change hands.
    pub fn floor3_foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.floor3.foe.state == FoeState::Fallen {
            return Err(Refusal("the Release has already left".to_string()));
        }
        if self.floor3.foe.strike != StrikePhase::Armed {
            return Err(Refusal(
                "the strike is not armed — press the seal first".to_string(),
            ));
        }
        self.floor3.foe.strike = StrikePhase::Idle;
        self.floor3.foe.state = FoeState::Fallen;
        self.floor3.foe.fallen_date = Some(date.to_string());
        if !self.codex.iter().any(|stone| stone == LANTERN_STONE) {
            self.codex.push(LANTERN_STONE.to_string());
        }
        if !self.pack.iter().any(|loot| loot == GATEHOUSE_KEY) {
            self.pack.push(GATEHOUSE_KEY.to_string());
        }
        Ok("The strike lands — not a release punished, but a lantern \
            rewired: its wire to the clock is cut in front of the whole \
            office, and the new lantern is fitted with no wire at all. \
            It will burn when the work is done, and not before. The \
            Release leaves by the great door under its own true light."
            .to_string())
    }

    /// Record that the player read a Release hint rung.
    pub fn floor3_foe_view_hint(&mut self) -> u8 {
        self.floor3.foe_hints_used = (self.floor3.foe_hints_used + 1).min(4);
        self.floor3.foe_hints_used
    }

    /// The deepest Release hint rung recorded.
    pub fn floor3_foe_hints_used(&self) -> u8 {
        self.floor3.foe_hints_used
    }

    /// Repair a Floor 3 chamber whose proof stands filed, and
    /// settle its facet when the hints stayed shallow — the
    /// two-step seal's second step.
    pub fn floor3_repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .floor3
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        if chamber == 4 && progress.state == ChamberState::Diagnosed && progress.work.rite_stage < 3
        {
            return Err(Refusal(
                "the amendment rite stands undone at the side desk — read it, \
                 second it, enter it, and the seal will answer"
                    .to_string(),
            ));
        }
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
                    Ok("Repaired at full standard. A facet of the Lantern \
                         Stone settles into the codex."
                        .to_string())
                } else {
                    Ok("Repaired — though the seam was named, and the record \
                         keeps it. No facet, but the floor still completes."
                        .to_string())
                }
            }
        }
    }

    /// Record that the player read a Floor 3 hint rung.
    pub fn floor3_view_hint(&mut self, chamber: usize) -> u8 {
        let Some(progress) = self.floor3.chambers.get_mut(chamber) else {
            return 0;
        };
        progress.hints_used = (progress.hints_used + 1).min(4);
        progress.hints_used
    }

    /// The deepest hint rung recorded for a Floor 3 chamber.
    pub fn floor3_hints_used(&self, chamber: usize) -> u8 {
        self.floor3
            .chambers
            .get(chamber)
            .map_or(0, |progress| progress.hints_used)
    }

    /// Whether this chamber's work currently proves (§5.4).
    pub fn floor4_diagnosed(&self, chamber: usize) -> bool {
        let Some(progress) = self.floor4.chambers.get(chamber) else {
            return false;
        };
        let work = &progress.work;
        match chamber {
            // Chamber 1: every stone measured, and every
            // surprise carrying its true annotation.
            0 => {
                work.watched
                    && work.measured.iter().all(|done| *done)
                    && (0..crate::content::BENCH_STONES.len()).all(|stone| {
                        !crate::puzzles::bench_missed(
                            stone,
                            work.pred_size[stone],
                            work.pred_align[stone],
                        ) || work.annotate[stone] == Some(crate::content::BENCH_TRUE_CAUSE[stone])
                    })
            }
            // Chamber 2: all five seated at the least span.
            1 => {
                let pairs: Vec<(u32, u32)> = crate::content::WALL_STONES
                    .iter()
                    .map(|(_, size, align)| (*size, *align))
                    .collect();
                work.watched && work.wall_order.len() == crate::content::WALL_STONES.len() && {
                    let seated: Vec<(u32, u32)> = work
                        .wall_order
                        .iter()
                        .map(|stone| pairs[*stone as usize])
                        .collect();
                    crate::puzzles::wall_layout(&seated).0 == crate::puzzles::wall_minimum(&pairs)
                }
            }
            2 => {
                progress.work.watched
                    && progress.work.crest_proven.iter().all(|&proven| proven)
                    && (0..3).all(|crest| {
                        crate::puzzles::crest_choice_ok(crest, progress.work.crest_choices[crest])
                    })
            }
            3 => {
                let dims: Vec<(u32, u32)> = crate::content::TREATY_STONES[..4]
                    .iter()
                    .map(|&(_, size, align)| (size, align))
                    .collect();
                let truth = crate::puzzles::c_offsets(&dims);
                progress.work.watched
                    && progress.work.treaty_order == vec![0, 1, 2, 3]
                    && progress.work.treaty_proven
                    && progress.work.final_promise == truth[3]
            }
            // Chamber 5: both stones filled and tagged as
            // filled, the blank full and used whole.
            4 => {
                work.watched
                    && (0..crate::content::HOLLOW_STONES.len()).all(|stone| {
                        work.fillings[stone] != 0 && work.tags[stone] == work.fillings[stone]
                    })
                    && work.blank_cells.iter().all(|cell| *cell)
                    && work.blank_used
            }
            _ => false,
        }
    }

    /// File this chamber's proof (the Prove verb): Unworked →
    /// Diagnosed, when the work proves.
    pub fn floor4_file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        if !self.floor4_diagnosed(chamber) {
            return Err(Refusal(match chamber {
                0 => "the bench is not yet true: every stone predicted, \
                      measured, and its surprises annotated in your own hand"
                    .to_string(),
                1 => "the wall is not yet the smallest: seat all five \
                      stones so the line reads the least span"
                    .to_string(),
                2 => "the crests are not yet cut to their least: choose \
                      each cut, let the line measure it, and pay for no \
                      tag a niche would carry free"
                    .to_string(),
                3 => "the treaty wall does not yet stand proven: seat \
                      the four treaty stones in order, drop the line, \
                      and amend the book's final promise to the offset \
                      the line reads"
                    .to_string(),
                4 => "the hollow stones do not yet read true: fill \
                      both, tag each as it is filled, and use the blank \
                      only when it stands whole"
                    .to_string(),
                _ => "this gallery is not open yet".to_string(),
            }));
        }
        let progress = self
            .floor4
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        if progress.state == ChamberState::Repaired {
            return Err(Refusal("this chamber already stands repaired".to_string()));
        }
        progress.state = ChamberState::Diagnosed;
        Ok(match chamber {
            0 => "Filed: six stones, predicted before they were                   measured, and every surprise owned in writing."
                .to_string(),
            1 => "Filed: the same five stones, and the voids counted \
                  out of the wall."
                .to_string(),
            2 => "Filed: three crests, each cut where its mark rides \
                  free in a value that can never occur."
                .to_string(),
            3 => "Filed: four offsets the line itself read, and a book \
                  amended where the book was careless."
                .to_string(),
            4 => "Filed: two stones read as they are, and a blank \
                  used whole — nothing trusted to a head."
                .to_string(),
            _ => "The proof is filed with the Plumbmason's office.".to_string(),
        })
    }

    /// Repair a diagnosed chamber (the seal): Diagnosed →
    /// Repaired, with the facet at full standard (§5.5).
    pub fn floor4_repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .floor4
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
                    Ok("Repaired at full standard. A facet of the Brass \
                         Eye Stone settles into the codex."
                        .to_string())
                } else {
                    Ok("Repaired — though the seam was named, and the record \
                         keeps it. No facet, but the floor still completes."
                        .to_string())
                }
            }
        }
    }

    /// Reset one Floor 4 chamber's work to its starting
    /// state: hint usage stays on the record.
    pub fn floor4_reset_chamber(&mut self, chamber: usize) {
        if let Some(progress) = self.floor4.chambers.get_mut(chamber) {
            progress.state = ChamberState::Unworked;
            progress.facet = false;
            progress.work = FoundationWork::default();
        }
    }

    /// Record that the player read a Floor 4 hint rung.
    pub fn floor4_view_hint(&mut self, chamber: usize) -> u8 {
        let Some(progress) = self.floor4.chambers.get_mut(chamber) else {
            return 0;
        };
        progress.hints_used = (progress.hints_used + 1).min(4);
        progress.hints_used
    }

    /// The deepest hint rung recorded for a Floor 4 chamber.
    pub fn floor4_hints_used(&self, chamber: usize) -> u8 {
        self.floor4
            .chambers
            .get(chamber)
            .map_or(0, |progress| progress.hints_used)
    }

    /// Floor 4 stands open when the Stratum I → II stairs
    /// stand lit (§5.6, over the §5.2 prerequisite of Floor 1):
    /// FOE 1 fallen, FOE 2 or FOE 3 fallen, and every Stratum I
    /// lantern keeper-lit.
    pub fn floor4_reachable(&self) -> bool {
        self.stairs_clauses().iter().all(|(lit, _)| *lit)
    }

    /// How many Floor 4 chambers stand repaired.
    pub fn floor4_chambers_repaired(&self) -> usize {
        self.floor4
            .chambers
            .iter()
            .filter(|chamber| chamber.state == ChamberState::Repaired)
            .count()
    }

    /// Whether Floor 4 is complete: five chambers + FOE.
    pub fn floor4_complete(&self) -> bool {
        self.floor4_chambers_repaired() == 5 && self.floor4.foe.state == FoeState::Fallen
    }

    /// Whether a Floor 5 chamber's work presently proves
    /// out (§5.4 of the brief).
    pub fn floor5_diagnosed(&self, chamber: usize) -> bool {
        let Some(progress) = self.floor5.chambers.get(chamber) else {
            return false;
        };
        let work = &progress.work;
        match chamber {
            // Chamber 1: all six stones resolved under their
            // cards, the cracked one refused by name.
            0 => work.watched && work.walked == 6 && work.refused,
            // Chamber 2: the calls stamped with at least
            // seven true, and every miss annotated with the
            // rule that decided it.
            1 => {
                work.watched
                    && work.judged
                    && crate::puzzles::tenancy_correct(&work.predictions) >= 7
                    && (0..crate::content::TENANCIES.len()).all(|index| {
                        !crate::puzzles::tenancy_missed(&work.predictions, index)
                            || work.annotations[index] == Some(crate::content::TENANCIES[index].2)
                    })
            }
            // Chamber 3: all twelve lines checkable.
            2 => work.watched && crate::puzzles::cards_all_checkable(&work.card_lines),
            // Chamber 4: seamed at a sound split, and the
            // referee's walk complete — the over-reach refused,
            // both halves worked, both dropped in order.
            3 => {
                work.watched
                    && work.seamed
                    && crate::puzzles::beam_sound_split(work.split)
                    && work.access_refused
                    && work.worked == [true, true]
                    && work.dropped == [true, true]
            }
            4 => hunt_complete(&work.lensed, &work.verdicts, &work.cards),
            _ => false,
        }
    }

    /// File one Floor 5 chamber's proof once its work proves:
    /// Unworked → Diagnosed, and the expedition log gains a line.
    pub fn floor5_file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        if !self.floor5_diagnosed(chamber) {
            return Err(Refusal(match chamber {
                0 => "the path is not yet walked true: read every card \
                      before its step, walk the sound steps, and refuse \
                      the one whose card lies"
                    .to_string(),
                1 => "the tenancies are not yet judged true: call all \
                      eight, let the referee stamp them, and annotate \
                      every miss with the rule that decided it"
                    .to_string(),
                2 => "the cards do not yet read checkable: cycle every \
                      line of all four tools to the phrasing a sceptic \
                      can test"
                    .to_string(),
                3 => "the beam does not yet stand seamed: choose the \
                      seam, pass the referee's three rulings, work both \
                      halves, and drop them in the order prescribed"
                    .to_string(),
                4 => "the hunt is not yet presented: lens all six \
                      stations, name the three faults truly, and rewrite \
                      the cards that allowed them"
                    .to_string(),
                _ => "this workshop is not open yet".to_string(),
            }));
        }
        let progress = self
            .floor5
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        if progress.state == ChamberState::Repaired {
            return Err(Refusal("this chamber already stands repaired".to_string()));
        }
        progress.state = ChamberState::Diagnosed;
        Ok(match chamber {
            0 => "Filed: six steps, every card read before its step, and \
                  the lying card refused by name."
                .to_string(),
            1 => "Filed: eight tenancies called, stamped, and every miss \
                  owned with its deciding rule."
                .to_string(),
            2 => "Filed: four tools, twelve lines, every one a sceptic \
                  can check."
                .to_string(),
            3 => "Filed: one beam, two halves, a seam the referee walked \
                  and found sound."
                .to_string(),
            4 => "Filed: three faults found underneath, and their cards \
                  rewritten at the root."
                .to_string(),
            _ => "The proof is filed with the Veilwalker's office.".to_string(),
        })
    }

    /// Repair a diagnosed chamber (the seal): Diagnosed →
    /// Repaired, with the facet at full standard (§5.5).
    pub fn floor5_repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .floor5
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
                    Ok("Repaired at full standard. A facet of the Ice Stone \
                        settles into the codex."
                        .to_string())
                } else {
                    Ok("Repaired — though the seam was named, and the record \
                        keeps it. No facet, but the floor still completes."
                        .to_string())
                }
            }
        }
    }

    /// Reset one Floor 5 chamber's work to its starting
    /// state: hint usage stays on the record.
    pub fn floor5_reset_chamber(&mut self, chamber: usize) {
        if let Some(progress) = self.floor5.chambers.get_mut(chamber) {
            progress.state = ChamberState::Unworked;
            progress.facet = false;
            progress.work = VeilWork::default();
        }
    }

    /// Record that the player read a Floor 5 hint rung.
    pub fn floor5_view_hint(&mut self, chamber: usize) -> u8 {
        let Some(progress) = self.floor5.chambers.get_mut(chamber) else {
            return 0;
        };
        progress.hints_used = (progress.hints_used + 1).min(4);
        progress.hints_used
    }

    /// Hints used on a Floor 5 chamber.
    pub fn floor5_hints_used(&self, chamber: usize) -> u8 {
        self.floor5
            .chambers
            .get(chamber)
            .map(|progress| progress.hints_used)
            .unwrap_or(0)
    }

    /// Floor 5 stands open when the Crooked Foundation has
    /// fallen and Floor 4 stands complete.
    pub fn floor5_reachable(&self) -> bool {
        self.floor4_complete()
    }

    /// How many Floor 5 chambers stand repaired.
    pub fn floor5_chambers_repaired(&self) -> usize {
        self.floor5
            .chambers
            .iter()
            .filter(|chamber| chamber.state == ChamberState::Repaired)
            .count()
    }

    /// Whether Floor 5 is complete: five chambers + FOE.
    pub fn floor5_complete(&self) -> bool {
        self.floor5_chambers_repaired() == 5 && self.floor5.foe.state == FoeState::Fallen
    }

    /// Exhibit one sight of the trace, in the referee's
    /// order: the live shared loan, the growth while it was
    /// out, the lens's verdict. A sight taken out of order is
    /// refused, naming the sight that must come first; the
    /// first sight taken stands the Thing Exposed.
    pub fn floor5_foe_exhibit(&mut self, exhibit: usize) -> Result<String, Refusal> {
        if exhibit >= crate::content::FOE5_EXHIBITS.len() {
            return Err(Refusal("no such sight in the trace".to_string()));
        }
        if self.floor5.foe.exhibits[exhibit] {
            return Ok("That sight already stands exhibited.".to_string());
        }
        for prior in 0..exhibit {
            if !self.floor5.foe.exhibits[prior] {
                return Err(Refusal(format!(
                    "the trace is read in order: first, {} — nothing \
                     stands behind the veil until each sight follows \
                     the last",
                    crate::content::FOE5_EXHIBITS[prior].0
                )));
            }
        }
        self.floor5.foe.exhibits[exhibit] = true;
        if self.floor5.foe.state == FoeState::Standing {
            self.floor5.foe.state = FoeState::Exposed;
        }
        Ok(crate::content::FOE5_EXHIBITS[exhibit].1.to_string())
    }

    /// The Thing's proof, whole: the trace exhibited in
    /// order, all three sights.
    pub fn floor5_foe_proof_complete(&self) -> bool {
        self.floor5.foe.exhibits.iter().all(|seen| *seen)
    }

    /// Walk one of the referee's five standing-gate cases
    /// against the beam: the case is run on the model in
    /// `puzzles`, and its outcome must be the discipline's.
    /// The walk waits on the diagnosis — cases are run
    /// against a diagnosed fault, not a rumour.
    pub fn floor5_foe_walk_case(&mut self, case: usize) -> Result<String, Refusal> {
        if !self.floor5_foe_proof_complete() {
            return Err(Refusal(
                "the referee walks cases against a diagnosed fault, \
                 not a rumour: exhibit the trace first"
                    .to_string(),
            ));
        }
        if case >= crate::content::FOE5_CASES.len() {
            return Err(Refusal("no such case on the referee's slate".to_string()));
        }
        if self.floor5.foe.walked[case] {
            return Ok("That case already stands walked.".to_string());
        }
        if !crate::puzzles::beam_case_holds(case) {
            return Err(Refusal(
                "the case does not hold — the beam's discipline is \
                 not what its card promises"
                    .to_string(),
            ));
        }
        self.floor5.foe.walked[case] = true;
        self.floor5_foe_refresh_gate();
        Ok(crate::content::FOE5_CASES[case].1.to_string())
    }

    /// The gate is the referee's walk: all five cases hold.
    fn floor5_foe_refresh_gate(&mut self) {
        let foe = &mut self.floor5.foe;
        foe.gate_passed = foe.walked.iter().all(|done| *done);
    }

    /// The repair stands when the beam's card shows its
    /// three checkable truths.
    pub fn floor5_foe_repair_ok(&self) -> bool {
        foe_card_repaired(&self.floor5.foe.card)
    }

    /// Arm the Thing's strike: the trace exhibited in order,
    /// the referee's walk complete, and the beam's card
    /// rewritten — as with every FOE.
    pub fn floor5_foe_arm_strike(&mut self) -> Result<(), Refusal> {
        if self.floor5.foe.state == FoeState::Fallen {
            return Err(Refusal(
                "the Thing has already stepped out from behind the veil".to_string(),
            ));
        }
        if !self.floor5_foe_proof_complete() {
            return Err(Refusal(
                "the strike sleeps until the trace stands exhibited: \
                 the loan, the growth, the lens's verdict, in order"
                    .to_string(),
            ));
        }
        if !self.floor5.foe.gate_passed {
            return Err(Refusal(
                "the strike sleeps until the referee's walk stands \
                 complete: five cases against the beam"
                    .to_string(),
            ));
        }
        if !self.floor5_foe_repair_ok() {
            return Err(Refusal(
                "the strike sleeps until the beam's card stands \
                 rewritten: three lines, three checkable truths"
                    .to_string(),
            ));
        }
        self.floor5.foe.strike = StrikePhase::Armed;
        Ok(())
    }

    /// Confirm the strike: the beam is re-hung unborrowed,
    /// the veil drawn aside for good, and the Ice Stone and
    /// the Veilwalker's Buffer change hands.
    pub fn floor5_foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.floor5.foe.state == FoeState::Fallen {
            return Err(Refusal(
                "the Thing has already stepped out from behind the veil".to_string(),
            ));
        }
        if self.floor5.foe.strike != StrikePhase::Armed {
            return Err(Refusal(
                "the strike is not armed — press the seal first".to_string(),
            ));
        }
        self.floor5.foe.strike = StrikePhase::Idle;
        self.floor5.foe.state = FoeState::Fallen;
        self.floor5.foe.fallen_date = Some(date.to_string());
        if !self.codex.iter().any(|stone| stone == ICE_STONE) {
            self.codex.push(ICE_STONE.to_string());
        }
        if !self.pack.iter().any(|loot| loot == VEILWALKERS_BUFFER) {
            self.pack.push(VEILWALKERS_BUFFER.to_string());
        }
        Ok("The strike lands — not a beam smashed, but a discipline \
            restored: the loans are called home, the chalk is struck \
            through, and the beam is re-hung under the rewritten card, \
            its addresses good because they cannot move. Behind the \
            gauze there is only gauze."
            .to_string())
    }

    /// Record that the player read a Thing hint rung.
    pub fn floor5_foe_view_hint(&mut self) -> u8 {
        self.floor5.foe_hints_used = (self.floor5.foe_hints_used + 1).min(4);
        self.floor5.foe_hints_used
    }

    /// The deepest Thing hint rung recorded.
    pub fn floor5_foe_hints_used(&self) -> u8 {
        self.floor5.foe_hints_used
    }

    /// Whether a Floor 6 chamber's work presently proves out.
    /// No chamber proves yet: the judges land with their
    /// chambers in Steps 2–4.
    pub fn floor6_diagnosed(&self, chamber: usize) -> bool {
        let Some(progress) = self.floor6.chambers.get(chamber) else {
            return false;
        };
        let work = &progress.work;
        match chamber {
            0 => work.watched && work.day_walked.iter().all(|walked| *walked) && work.bell_struck,
            1 => {
                work.watched
                    && work.calls.iter().all(|called| *called)
                    && crate::puzzles::brazier_replay(&work.calls).1 == 1
            }
            2 => {
                work.watched
                    && (1..=9).contains(&work.spin_bound)
                    && work.gave_up
                    && work.spin_annotations.iter().all(|choice| *choice != 0)
            }
            3 => work.watched && work.census_judged && work.census == crate::puzzles::day_census(),
            4 => work.watched && crate::puzzles::choosing_complete(&work.choices, &work.invariants),
            _ => false,
        }
    }

    /// File one Floor 6 chamber's proof once its work proves:
    /// Unworked → Diagnosed, and the expedition log gains a line.
    pub fn floor6_file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        if !self.floor6_diagnosed(chamber) {
            return Err(Refusal(match chamber {
                0 => "the day is not yet walked true: walk the six \
                      operations in their order, and let the bell strike \
                      at the barred writing flag"
                    .to_string(),
                1 => "the brazier's account is not yet true: walk the \
                      five calls between the porters, and let the \
                      flame's count stand at exactly one"
                    .to_string(),
                2 => "the watchman is not yet answered: set the bound, \
                      let the heart give up honestly at it, and record \
                      all four orderings — provisional, as they stand"
                    .to_string(),
                3 => "the census does not yet tally: count the states \
                      the day can truly reach — free, shared, \
                      exclusive — and submit them to the judge"
                    .to_string(),
                4 => "the three data do not yet stand housed true: \
                      match each to its cell, and write the true \
                      sentence on each door"
                    .to_string(),
                _ => "this chamber is not open yet".to_string(),
            }));
        }
        let progress = self
            .floor6
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        if progress.state == ChamberState::Repaired {
            return Err(Refusal("this chamber already stands repaired".to_string()));
        }
        progress.state = ChamberState::Diagnosed;
        Ok(match chamber {
            0 => "Filed: one day of six operations, the barred flag \
                  witnessed, the heart unharmed."
                .to_string(),
            1 => "Filed: five calls between two porters, and the \
                  brazier lit exactly once."
                .to_string(),
            2 => "Filed: a bound kept honestly, and four orderings \
                  recorded, provisional until the Loom Halls."
                .to_string(),
            3 => "Filed: the day's census, counted by enumeration \
                  and judged true."
                .to_string(),
            4 => "Filed: three data housed, each under the sentence \
                  that governs it."
                .to_string(),
            _ => "The proof is filed with Lockheart's office.".to_string(),
        })
    }

    /// Repair a diagnosed chamber (the seal): Diagnosed →
    /// Repaired, with the facet at full standard (§5.5).
    pub fn floor6_repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .floor6
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
                    Ok("Repaired at full standard. A facet of the Lockheart Stone \
                        settles into the codex."
                        .to_string())
                } else {
                    Ok("Repaired — though the seam was named, and the record \
                        keeps it. No facet, but the floor still completes."
                        .to_string())
                }
            }
        }
    }

    /// Reset one Floor 6 chamber's work to its starting
    /// state: hint usage stays on the record.
    pub fn floor6_reset_chamber(&mut self, chamber: usize) {
        if let Some(progress) = self.floor6.chambers.get_mut(chamber) {
            progress.state = ChamberState::Unworked;
            progress.facet = false;
            progress.work = LockWork::default();
        }
    }

    /// Record that the player read a Floor 6 hint rung.
    pub fn floor6_view_hint(&mut self, chamber: usize) -> u8 {
        let Some(progress) = self.floor6.chambers.get_mut(chamber) else {
            return 0;
        };
        progress.hints_used = (progress.hints_used + 1).min(4);
        progress.hints_used
    }

    /// Hints used on a Floor 6 chamber.
    pub fn floor6_hints_used(&self, chamber: usize) -> u8 {
        self.floor6
            .chambers
            .get(chamber)
            .map(|progress| progress.hints_used)
            .unwrap_or(0)
    }

    /// Floor 6 stands open when the Veilworks stands charted
    /// through its third chamber — the Pointer Walks, the
    /// Two Tenants, and the Contract Cards repaired (the
    /// prerequisite chart's chamber-depth refinement).
    pub fn floor6_reachable(&self) -> bool {
        self.floor5
            .chambers
            .iter()
            .take(3)
            .all(|chamber| chamber.state == ChamberState::Repaired)
    }

    /// How many Floor 6 chambers stand repaired.
    pub fn floor6_chambers_repaired(&self) -> usize {
        self.floor6
            .chambers
            .iter()
            .filter(|chamber| chamber.state == ChamberState::Repaired)
            .count()
    }

    /// Whether Floor 6 is complete: five chambers + FOE.
    pub fn floor6_complete(&self) -> bool {
        self.floor6_chambers_repaired() == 5 && self.floor6.foe.state == FoeState::Fallen
    }
    /// Exhibit one sight of the staff, in its order: the
    /// census Lockheart counted over the broken staff, the
    /// worked day that ends fused, the torn reading. A sight
    /// taken out of order is refused, naming the sight that
    /// must come first; the first sight taken stands the
    /// Heart Exposed. The accounts are spoken from the
    /// models — counted and replayed at the exhibiting.
    pub fn floor6_foe_exhibit(&mut self, exhibit: usize) -> Result<String, Refusal> {
        if exhibit >= crate::content::FOE6_EXHIBIT_NAMES.len() {
            return Err(Refusal("no such sight of the staff".to_string()));
        }
        if self.floor6.foe.exhibits[exhibit] {
            return Ok("That sight already stands exhibited.".to_string());
        }
        for prior in 0..exhibit {
            if !self.floor6.foe.exhibits[prior] {
                return Err(Refusal(format!(
                    "the staff is read in order: first, {} — nothing \
                     stands behind the curtain until each sight \
                     follows the last",
                    crate::content::FOE6_EXHIBIT_NAMES[prior]
                )));
            }
        }
        self.floor6.foe.exhibits[exhibit] = true;
        if self.floor6.foe.state == FoeState::Standing {
            self.floor6.foe.state = FoeState::Exposed;
        }
        Ok(match exhibit {
            0 => {
                let census = crate::puzzles::staff_census();
                format!(
                    "Exhibit, first sight: the staff's own census, \
                     counted over the broken staff — {} free, {} \
                     shared, {} exclusive, and {} FUSED: a state no \
                     lawful staff can stand in, counted three times \
                     over in the staff's own reach.",
                    census[0], census[1], census[2], census[3]
                )
            }
            1 => {
                let end = crate::puzzles::staff_worked_day();
                format!(
                    "Exhibit, second sight: a worked day on the \
                     staff — two porters open the heart shared, and \
                     then the writer comes, and the staff lets her \
                     in over them both. The day ends {}.",
                    crate::puzzles::staff_state_name(end)
                )
            }
            _ => {
                let (first, second) = crate::puzzles::staff_torn_read();
                format!(
                    "Exhibit, third sight: the heart read mid-write — \
                     its first word of day {first}, its second of \
                     day {second}, in one reading. Verdict: the \
                     heart was torn between two days, and the \
                     reader holds the tear."
                )
            }
        })
    }

    /// The Heart's proof, whole: the staff exhibited in
    /// order, all three sights.
    pub fn floor6_foe_proof_complete(&self) -> bool {
        self.floor6.foe.exhibits.iter().all(|seen| *seen)
    }

    /// Walk one of the model-loom's four cases: the case is
    /// run on the loom in `puzzles`, and its computed verdict
    /// must be the recorded one — a disagreement is refused,
    /// fail-closed. The walk waits on the diagnosis.
    pub fn floor6_foe_walk_case(&mut self, case: usize) -> Result<String, Refusal> {
        if !self.floor6_foe_proof_complete() {
            return Err(Refusal(
                "the loom is walked against a diagnosed fault, not \
                 a rumour: exhibit the staff first"
                    .to_string(),
            ));
        }
        let Some((name, expected, outcome)) = crate::content::FOE6_CASES.get(case) else {
            return Err(Refusal("no such case on the loom's table".to_string()));
        };
        if self.floor6.foe.loom_walked[case] {
            return Ok("That case already stands walked.".to_string());
        }
        if crate::puzzles::loom_case_torn(case) != Some(*expected) {
            return Err(Refusal(
                "the case does not hold — the loom's computation is \
                 not what its record promises"
                    .to_string(),
            ));
        }
        let _ = name;
        self.floor6.foe.loom_walked[case] = true;
        self.floor6_foe_refresh_gate();
        Ok(outcome.to_string())
    }

    /// The gate is the loom's walk: all four cases hold.
    fn floor6_foe_refresh_gate(&mut self) {
        let foe = &mut self.floor6.foe;
        foe.gate_passed = foe.loom_walked.iter().all(|done| *done);
    }

    /// The repair stands when the staff's card shows its
    /// three checkable truths.
    pub fn floor6_foe_repair_ok(&self) -> bool {
        crate::puzzles::staff_card_repaired(&self.floor6.foe.card_lines)
    }

    /// Arm the Heart's strike: the staff exhibited in order,
    /// the loom's walk complete, and the staff's card
    /// rewritten — as with every FOE.
    pub fn floor6_foe_arm_strike(&mut self) -> Result<(), Refusal> {
        if self.floor6.foe.state == FoeState::Fallen {
            return Err(Refusal(
                "the Heart has already been locked once, and truly".to_string(),
            ));
        }
        if !self.floor6_foe_proof_complete() {
            return Err(Refusal(
                "the strike sleeps until the staff stands exhibited: \
                 the census, the worked day, the torn reading, in \
                 order"
                    .to_string(),
            ));
        }
        if !self.floor6.foe.gate_passed {
            return Err(Refusal(
                "the strike sleeps until the loom's walk stands \
                 complete: four interleavings against the torn heart"
                    .to_string(),
            ));
        }
        if !self.floor6_foe_repair_ok() {
            return Err(Refusal(
                "the strike sleeps until the staff's card stands \
                 rewritten: three lines, three checkable truths"
                    .to_string(),
            ));
        }
        self.floor6.foe.strike = StrikePhase::Armed;
        Ok(())
    }

    /// Confirm the strike: the staff is re-forged as an enum
    /// with its count inside, and the Lockheart Stone and
    /// the Lockheart Pair change hands.
    pub fn floor6_foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.floor6.foe.state == FoeState::Fallen {
            return Err(Refusal(
                "the Heart has already been locked once, and truly".to_string(),
            ));
        }
        if self.floor6.foe.strike != StrikePhase::Armed {
            return Err(Refusal(
                "the strike is not armed — press the seal first".to_string(),
            ));
        }
        self.floor6.foe.strike = StrikePhase::Idle;
        self.floor6.foe.state = FoeState::Fallen;
        self.floor6.foe.fallen_date = Some(date.to_string());
        if !self.codex.iter().any(|stone| stone == LOCKHEART_STONE) {
            self.codex.push(LOCKHEART_STONE.to_string());
        }
        if !self.pack.iter().any(|loot| loot == LOCKHEART_PAIR) {
            self.pack.push(LOCKHEART_PAIR.to_string());
        }
        Ok("The strike lands — not a staff smashed, but a state \
            made honest: the staff is re-forged as an enum of \
            three, its count inside the shared state where it \
            lives, and exclusive issues from free alone. Behind \
            the curtain there is only a heart, locked once, and \
            truly."
            .to_string())
    }

    /// Record that the player read a Heart hint rung.
    pub fn floor6_foe_view_hint(&mut self) -> u8 {
        self.floor6.foe_hints_used = (self.floor6.foe_hints_used + 1).min(4);
        self.floor6.foe_hints_used
    }

    /// The deepest Heart hint rung recorded.
    pub fn floor6_foe_hints_used(&self) -> u8 {
        self.floor6.foe_hints_used
    }

    /// Whether a Floor 7 chamber's work presently proves out.
    /// No chamber proves yet: the judges land with their
    /// chambers in Steps 2–4.
    /// Floor 7's demon: has she demonstrated exhibit `n`
    /// (0-based)?
    pub fn floor7_foe_exhibit_seen(&self, exhibit: usize) -> bool {
        self.floor7
            .foe
            .exhibits
            .get(exhibit)
            .copied()
            .unwrap_or(false)
    }

    /// Record one of the Seamstress's demonstrations, in
    /// her order: the claim orders (0), the waits-for chart
    /// (1) — which may not be shown before the orders are
    /// spoken — and the deadlocking interleaving (2), run
    /// on the model loom itself, which requires the drawn
    /// chart complete and the loom's own deadlock found.
    pub fn floor7_foe_show_exhibit(&mut self, exhibit: usize) -> Result<(), Refusal> {
        if !(0..3).contains(&exhibit) {
            return Err(Refusal("she has three exhibits, no more".to_string()));
        }
        if exhibit > 0 && !self.floor7_foe_exhibit_seen(exhibit - 1) {
            return Err(Refusal(
                "her demonstrations come in her order: the one                  before this stands unshown"
                    .to_string(),
            ));
        }
        if exhibit == 2 {
            if !(0..4).all(|edge| self.floor7_foe_edge_connected(edge)) {
                return Err(Refusal(
                    "the run waits on the chart: draw all four edges                      first — the deadlock is run against the drawn                      cycle, not a spoken one"
                        .to_string(),
                ));
            }
            let report =
                crate::puzzles::loom::explore(&crate::puzzles::foe_servant_programs(), 4096);
            if report.deadlock.is_none() {
                return Err(Refusal(
                    "the loom found no deadlock — the chart is not                      the servants' tangle"
                        .to_string(),
                ));
            }
            self.floor7.foe.loom_run = true;
        }
        self.floor7.foe.exhibits[exhibit] = true;
        Ok(())
    }

    /// Is one waits-for edge drawn between the servants?
    pub fn floor7_foe_edge_connected(&self, edge: usize) -> bool {
        self.floor7.foe.edges.get(edge).copied().unwrap_or(false)
    }

    /// Draw one waits-for edge. The chart exhibit must
    /// have been shown: edges are drawn on the chart, not
    /// before it.
    pub fn floor7_foe_connect_edge(&mut self, edge: usize) -> Result<(), Refusal> {
        if !(0..4).contains(&edge) {
            return Err(Refusal("the chart has four edges".to_string()));
        }
        if !self.floor7_foe_exhibit_seen(1) {
            return Err(Refusal(
                "the chart is not yet unrolled — her demonstrations                  come in her order"
                    .to_string(),
            ));
        }
        self.floor7.foe.edges[edge] = true;
        Ok(())
    }

    /// Has one stated load been walked on the queue?
    pub fn floor7_foe_load_walked(&self, load: usize) -> bool {
        self.floor7.foe.loads.get(load).copied().unwrap_or(false)
    }

    /// Walk one stated load on the bounded queue and
    /// return its report: the producers, per-producer
    /// counts, and stated capacity come from the floor's
    /// tables. The load counts as walked only when the
    /// queue model reconciles the received set exactly.
    pub fn floor7_foe_walk_load(
        &mut self,
        load: usize,
    ) -> Result<crate::puzzles::QueueReport, Refusal> {
        let Some((_, producers, per_producer)) = crate::content::FOE7_LOADS.get(load) else {
            return Err(Refusal("the queue has three stated loads".to_string()));
        };
        let report = crate::puzzles::queue_run(
            *producers,
            *per_producer,
            crate::content::FOE7_QUEUE_CAPACITY,
        );
        if !report.reconciled {
            return Err(Refusal(
                "the load does not reconcile — the queue is not                  walked on a hope"
                    .to_string(),
            ));
        }
        self.floor7.foe.loads[load] = true;
        self.floor7_foe_refresh_gate();
        Ok(report)
    }

    /// Has the at-capacity exhibit been shown — the queue
    /// full, the next send waiting?
    pub fn floor7_foe_pressure_shown(&self) -> bool {
        self.floor7.foe.pressure
    }

    /// Show the at-capacity exhibit: fill the queue to its
    /// stated capacity and find the next send standing
    /// pending — waited, not dropped, not spun. Returns
    /// the standing count and whether the send pends.
    pub fn floor7_foe_show_pressure(&mut self) -> Result<(usize, bool), Refusal> {
        let (standing, waits) =
            crate::puzzles::queue_at_capacity(crate::content::FOE7_QUEUE_CAPACITY);
        if !waits {
            return Err(Refusal("the queue does not stand full".to_string()));
        }
        self.floor7.foe.pressure = true;
        self.floor7_foe_refresh_gate();
        Ok((standing, waits))
    }

    /// Her gate is the queue walked: all three stated
    /// loads reconciled and the pressure exhibit shown.
    /// Recomputed from the work each time it changes —
    /// never merely claimed.
    fn floor7_foe_refresh_gate(&mut self) {
        let loads_ok = (0..3).all(|load| self.floor7_foe_load_walked(load));
        self.floor7.foe.gate_passed = loads_ok && self.floor7.foe.pressure;
    }

    /// The rank posted for one claimable (1-based index
    /// into the door's list; 0 = unranked).
    pub fn floor7_foe_order_rank(&self, claimable: usize) -> u8 {
        self.floor7.foe.order.get(claimable).copied().unwrap_or(0)
    }

    /// Cycle one claimable's posted rank: unranked, then
    /// 1..=4, then unranked again.
    pub fn floor7_foe_cycle_order(&mut self, claimable: usize) -> Result<(), Refusal> {
        if !(0..4).contains(&claimable) {
            return Err(Refusal("the door lists four claimables".to_string()));
        }
        self.floor7.foe.order[claimable] = (self.floor7.foe.order[claimable] + 1) % 5;
        Ok(())
    }

    /// Does the posted order stand as the repair? A valid
    /// total order under which the loom — run on both
    /// servants re-cut to it — finds no deadlock.
    pub fn floor7_foe_order_holds(&self) -> bool {
        crate::puzzles::posted_order_holds(&self.floor7.foe.order)
    }

    /// The strike's proof: the orders spoken, the chart
    /// drawn, the deadlock run on the loom, and the posted
    /// order holding — every piece of her demonstration
    /// complete.
    pub fn floor7_foe_proof_complete(&self) -> bool {
        (0..3).all(|exhibit| self.floor7_foe_exhibit_seen(exhibit))
            && self.floor7.foe.loom_run
            && self.floor7_foe_order_holds()
    }

    /// How many of her ladder's rungs have been read.
    pub fn floor7_foe_hints_used(&self) -> u8 {
        self.floor7.foe_hints_used
    }

    /// Read the next rung of her ladder (the count is the
    /// ladder's memory, as on every floor).
    pub fn floor7_foe_view_hint(&mut self) {
        self.floor7.foe_hints_used = (self.floor7.foe_hints_used + 1).min(4);
    }

    /// Arm the Seamstress's sitting: the seal will strike
    /// only when her whole demonstration stands — exhibits
    /// run, the queue walked, the order posted and
    /// holding.
    pub fn floor7_foe_arm_strike(&mut self) -> Result<(), Refusal> {
        if self.floor7.foe.state == FoeState::Fallen {
            return Err(Refusal(
                "the Seamstress has already sat, and stays seated".to_string(),
            ));
        }
        if !self.floor7_foe_proof_complete() {
            return Err(Refusal(
                "the strike sleeps until the demonstration stands \
                 complete: the orders spoken, the chart drawn, the \
                 deadlock run on the loom, and a total order posted \
                 under which it cannot recur"
                    .to_string(),
            ));
        }
        if !self.floor7.foe.gate_passed {
            return Err(Refusal(
                "the strike sleeps until the queue stands walked: \
                 three stated loads reconciled, and the next send \
                 seen waiting at capacity"
                    .to_string(),
            ));
        }
        self.floor7.foe.strike = StrikePhase::Armed;
        Ok(())
    }

    /// The Seamstress sits. Requires the armed strike. The
    /// loomstone joins the codex; the Threadweaver's Arc &
    /// Queue goes in the pack.
    pub fn floor7_foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.floor7.foe.state == FoeState::Fallen {
            return Err(Refusal(
                "the Seamstress has already sat, and stays seated".to_string(),
            ));
        }
        if self.floor7.foe.strike != StrikePhase::Armed {
            return Err(Refusal(
                "the strike is not armed — press the seal first".to_string(),
            ));
        }
        self.floor7.foe.strike = StrikePhase::Idle;
        self.floor7.foe.state = FoeState::Fallen;
        self.floor7.foe.fallen_date = Some(date.to_string());
        if !self
            .codex
            .iter()
            .any(|stone| stone == crate::content::LOOMSTONE_TEXT)
        {
            self.codex.push(crate::content::LOOMSTONE_TEXT.to_string());
        }
        if !self
            .pack
            .iter()
            .any(|loot| loot == "The Threadweaver's Arc & Queue")
        {
            self.pack.push("The Threadweaver's Arc & Queue".to_string());
        }
        Ok(crate::content::FOE7_DEMONSTRATION.to_string())
    }

    /// The strike's receipt: she has sat, and the hall
    /// keeps her seated.
    pub fn floor7_receipt_standing(&self) -> bool {
        self.floor7.foe.state == FoeState::Fallen
    }

    pub fn floor7_diagnosed(&self, chamber: usize) -> bool {
        let Some(progress) = self.floor7.chambers.get(chamber) else {
            return false;
        };
        let work = &progress.work;
        match chamber {
            0 => {
                work.watched && work.scoped && crate::puzzles::scoped_order_valid(&work.join_order)
            }
            1 => work.watched && work.judged && crate::puzzles::seals_complete(&work.seals),
            2 => {
                work.watched
                    && work.graded
                    && crate::puzzles::wall_graded(&work.edges, work.weak_marked, work.stale_found)
            }
            3 => work.watched && work.judged && crate::puzzles::arc_foretold(&work.predictions),
            4 => {
                work.watched
                    && crate::puzzles::stack_verdicts_true(&work.verdicts)
                    && work.witness == crate::content::WITNESS_KEY + 1
            }
            _ => false,
        }
    }

    /// File one Floor 7 chamber's proof once its work proves:
    /// Unworked → Diagnosed, and the expedition log gains a line.
    pub fn floor7_file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        if !self.floor7_diagnosed(chamber) {
            return Err(Refusal(match chamber {
                0 => "the join order is not yet true: assign every \
                      thread its slot, with the two early threads \
                      joined before their lives run out"
                    .to_string(),
                1 => "the seals are not yet true: seal each of the \
                      six tools as it truly is — and read the fine \
                      print before you seal"
                    .to_string(),
                2 => "the wall is not yet graded: place the true \
                      happens-before edges, mark the weakened \
                      claim, and find the stale tide by its witness"
                    .to_string(),
                3 => "the foretelling is not yet true: foretell all \
                      eight steps of the book — the upgrades and \
                      the counts after — as the referee's book runs"
                    .to_string(),
                4 => "the histories are not yet judged: mark each \
                      of the six linearizable or not, and name the \
                      witness pair of the one that will not order"
                    .to_string(),
                _ => "this chamber is not open yet".to_string(),
            }));
        }
        let progress = self
            .floor7
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("no such chamber".to_string()))?;
        if progress.state == ChamberState::Repaired {
            return Err(Refusal("this chamber already stands repaired".to_string()));
        }
        progress.state = ChamberState::Diagnosed;
        Ok(match chamber {
            0 => "Filed: six threads joined inside the scope, the \
                  early threads home while they still lived."
                .to_string(),
            1 => "Filed: six tools sealed as they truly are, the \
                  phantom seal refused at the desk."
                .to_string(),
            2 => "Filed: the wall's true edges, its weakened claim \
                  marked, its stale tide found by the witness."
                .to_string(),
            3 => "Filed: eight steps foretold true — the echoes and \
                  the counts as the book itself ran them."
                .to_string(),
            4 => "Filed: six histories judged, and the one that \
                  will not order witnessed by its pair."
                .to_string(),
            _ => "The proof is filed with Threadweaver's office.".to_string(),
        })
    }

    /// Repair a diagnosed chamber (the seal): Diagnosed →
    /// Repaired, with the facet at full standard (§5.5).
    pub fn floor7_repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .floor7
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
                    Ok("Repaired at full standard. A facet of the Loomstone \
                        settles into the codex."
                        .to_string())
                } else {
                    Ok("Repaired — though the seam was named, and the record \
                        keeps it. No facet, but the floor still completes."
                        .to_string())
                }
            }
        }
    }

    /// Reset one Floor 7 chamber's work to its starting
    /// state: hint usage stays on the record.
    pub fn floor7_reset_chamber(&mut self, chamber: usize) {
        if let Some(progress) = self.floor7.chambers.get_mut(chamber) {
            progress.state = ChamberState::Unworked;
            progress.facet = false;
            progress.work = LoomWork::default();
        }
    }

    /// Record that the player read a Floor 7 hint rung.
    pub fn floor7_view_hint(&mut self, chamber: usize) -> u8 {
        let Some(progress) = self.floor7.chambers.get_mut(chamber) else {
            return 0;
        };
        progress.hints_used = (progress.hints_used + 1).min(4);
        progress.hints_used
    }

    /// Hints used on a Floor 7 chamber.
    pub fn floor7_hints_used(&self, chamber: usize) -> u8 {
        self.floor7
            .chambers
            .get(chamber)
            .map(|progress| progress.hints_used)
            .unwrap_or(0)
    }

    /// Floor 7 stands open when the Veilworks stands
    /// complete — five chambers repaired and the Thing
    /// Behind the Veil fallen (the prerequisite of record).
    /// The Lockheart is recommended — its hearts are this
    /// floor's furniture — but the door does not wait on it:
    /// the recommendation is advisory text, never a gate.
    pub fn floor7_reachable(&self) -> bool {
        self.floor5_complete()
    }

    /// How many Floor 7 chambers stand repaired.
    pub fn floor7_chambers_repaired(&self) -> usize {
        self.floor7
            .chambers
            .iter()
            .filter(|chamber| chamber.state == ChamberState::Repaired)
            .count()
    }

    /// Whether Floor 7 is complete: five chambers + FOE.
    pub fn floor7_complete(&self) -> bool {
        self.floor7_chambers_repaired() == 5 && self.floor7.foe.state == FoeState::Fallen
    }

    /// Judges land with each chamber's step.
    pub fn floor8_diagnosed(&self, chamber: usize) -> bool {
        let Some(progress) = self.floor8.chambers.get(chamber) else {
            return false;
        };
        let work = &progress.work;
        match chamber {
            0 => crate::puzzles::hour_bell_diagnosed(&work.polls, work.rung),
            1 => crate::puzzles::hour_tapestry_diagnosed(work.torn, work.pinned, work.moved),
            2 => crate::puzzles::hour_waker_diagnosed(&work.parts, work.trial_walked),
            3 => crate::puzzles::hour_attendant_diagnosed(&work.roster, work.judged),
            4 => crate::puzzles::hour::ledger_holds(&work.ledger),
            _ => false,
        }
    }

    pub fn floor8_ledger_cycle(&mut self, line: usize) -> Result<String, Refusal> {
        let progress = &mut self.floor8.chambers[4];
        let Some(slot) = progress.work.ledger.get_mut(line) else {
            return Err(Refusal("No such ledger line.".into()));
        };
        *slot = *slot % 3 + 1;
        progress.state = ChamberState::Unworked;
        progress.facet = false;
        Ok("Accounting changed; file the reconciled ledger again.".into())
    }

    pub fn floor8_foe_exhibit(&mut self, index: usize) -> Result<String, Refusal> {
        if index >= 3 {
            return Err(Refusal("No such exhibit.".into()));
        }
        if index > 0 && !self.floor8.foe.exhibits[index - 1] {
            return Err(Refusal(format!(
                "First exhibit the {}.",
                crate::content::HOUR_EXHIBITS[index - 1]
            )));
        }
        self.floor8.foe.exhibits[index] = true;
        self.floor8.foe.pressure = (self.floor8.foe.pressure + 1).min(3);
        let trace = crate::puzzles::hour::stale_trace();
        let evidence: Vec<_> = trace
            .into_iter()
            .filter(|e| match index {
                0 => matches!(e, crate::puzzles::executor::Event::Poll { .. }),
                _ => matches!(e, crate::puzzles::executor::Event::StaleRefused { .. }),
            })
            .take(if index == 0 { 2 } else { 1 })
            .collect();
        Ok(format!(
            "{}: {evidence:?}. Bounded misleading bell: {:?}; no completion published, proof preserved.",
            crate::content::HOUR_EXHIBITS[index],
            crate::puzzles::hour::stale_signals()[index]
        ))
    }

    pub fn floor8_foe_predict(&mut self, kind: u8) -> Result<String, Refusal> {
        let foe = &mut self.floor8.foe;
        let selected = usize::from(foe.selected);
        let Some(predictions) = foe.predictions.get_mut(selected) else {
            return Err(Refusal("No such roster.".into()));
        };
        let events = crate::puzzles::hour::presentation(selected);
        let Some(event) = events.get(predictions.len()) else {
            return Err(Refusal("This roster is already fully walked.".into()));
        };
        if crate::puzzles::hour_event_kind(event) != kind {
            return Err(Refusal(format!(
                "Event {} mismatch; read its provenance and retry. The walk does not advance.",
                predictions.len() + 1
            )));
        }
        predictions.push(kind);
        self.floor8_refresh_gate();
        Ok("Prediction matches the computed event; walk advanced.".into())
    }

    pub fn floor8_refresh_gate(&mut self) {
        let foe = &self.floor8.foe;
        let gate = foe.predictions.iter().enumerate().all(|(i, p)| {
            p.as_slice()
                == crate::puzzles::hour::presentation(i)
                    .iter()
                    .map(crate::puzzles::hour_event_kind)
                    .collect::<Vec<_>>()
        }) && foe.unsequenced == Some(crate::content::HOUR_UNSEQUENCED)
            && foe.policy_seen
            && crate::puzzles::hour::policy_holds();
        self.floor8.foe.gate_passed = gate;
    }

    pub fn floor8_foe_rewrite(&self) -> bool {
        let f = &self.floor8.foe;
        f.card_lines == [1; 3]
            && f.signal_seal == 1
            && crate::puzzles::hour::accepts_signal("living errand", u32::from(f.signal_generation))
    }

    pub fn floor8_foe_prove(&mut self) -> Result<String, Refusal> {
        if !self.floor8.foe.exhibits.iter().all(|v| *v) {
            return Err(Refusal("File all three exhibits in sequence first.".into()));
        }
        if self.floor8.foe.state != FoeState::Fallen {
            self.floor8.foe.state = FoeState::Exposed;
        }
        Ok("The wrong-room herald is exposed: a wake does not publish completion.".into())
    }

    pub fn floor8_foe_arm_strike(&mut self) -> Result<(), Refusal> {
        self.floor8_refresh_gate();
        if self.floor8.foe.state != FoeState::Exposed
            || !self.floor8.foe.exhibits.iter().all(|v| *v)
            || !self.floor8.foe.gate_passed
            || !self.floor8_foe_rewrite()
        {
            return Err(Refusal("The seal requires exposed proof, all four rosters and policy demonstrations, Unsequenced Bell identified, and the typed signal rewrite matching the living seal/generation.".into()));
        }
        self.floor8.foe.strike = StrikePhase::Armed;
        Ok(())
    }

    pub fn floor8_foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.floor8.foe.strike != StrikePhase::Armed {
            return Err(Refusal("Press the proven seal first.".into()));
        }
        self.floor8_foe_arm_strike()?;
        self.floor8.foe.strike = StrikePhase::Idle;
        self.floor8.foe.state = FoeState::Fallen;
        self.floor8.foe.fallen_date = Some(date.into());
        if !self
            .codex
            .iter()
            .any(|s| s == crate::content::PINSTONE_TEXT)
        {
            self.codex.push(crate::content::PINSTONE_TEXT.into());
        }
        if !self.pack.iter().any(|s| s == "The Pinwright's Executor") {
            self.pack.push("The Pinwright's Executor".into());
        }
        Ok("The Suspended Hour falls. The Pinstone enters the codex; The Pinwright's Executor enters the pack. Distinct message kinds prevent wrong-meaning handling; generation tokens refuse stale completions at the face.".into())
    }

    pub fn floor8_add_part(&mut self, part: u8) -> Result<String, Refusal> {
        let work = &mut self.floor8.chambers[2].work;
        let Some(index) = work.parts.iter().position(|&p| p == 0) else {
            return Err(Refusal("All four waker parts are already fitted.".into()));
        };
        if usize::from(part) != index + 1 {
            return Err(Refusal(format!(
                "Fit {} first; order unchanged.",
                crate::content::F8_WAKER_PARTS[index]
            )));
        }
        work.parts[index] = part;
        Ok(format!("Fitted {}.", crate::content::F8_WAKER_PARTS[index]))
    }

    pub fn floor8_sleep_trial(&mut self) -> Result<String, Refusal> {
        if !crate::puzzles::hour_waker_diagnosed(&self.floor8.chambers[2].work.parts, true) {
            return Err(Refusal("Fit counter, errand name, generation stamp, then wake face before the sleep trial.".into()));
        }
        self.floor8.chambers[2].work.trial_walked = true;
        let (sleeps, wakes, polls, spin) = crate::puzzles::hour_sleep_evidence();
        Ok(format!(
            "Hall dimmed for {sleeps} sleeps; {wakes} generation-stamped wakes, {polls} polls. Spinning twin: {spin} polls. Both are real authored executor drives; no host timing."
        ))
    }

    pub fn floor8_predict(&mut self, index: usize) -> Result<String, Refusal> {
        let work = &mut self.floor8.chambers[3].work;
        let Some(slot) = work.roster.get_mut(index) else {
            return Err(Refusal("No such event slot.".into()));
        };
        *slot = *slot % 9 + 1;
        work.judged = false;
        self.floor8.chambers[3].state = ChamberState::Unworked;
        self.floor8.chambers[3].facet = false;
        Ok("Prediction changed; rejudge before filing.".into())
    }

    pub fn floor8_judge_roster(&mut self) -> Result<String, Refusal> {
        let expected = crate::puzzles::hour_attendant_trace();
        let work = &mut self.floor8.chambers[3].work;
        if let Some((i, event)) = expected
            .iter()
            .enumerate()
            .find(|(i, event)| work.roster[*i] != crate::puzzles::hour_event_kind(event))
        {
            work.judged = false;
            return Err(Refusal(format!(
                "Event {} mismatch: computed {event:?}; prediction retained for correction.",
                i + 1
            )));
        }
        work.judged = true;
        Ok(format!(
            "All {} computed events match, using the Lockheart borrowing rules.",
            expected.len()
        ))
    }

    pub fn floor8_poll(&mut self, report: u8) -> Result<String, Refusal> {
        let work = &mut self.floor8.chambers[0].work;
        let Some(index) = work.polls.iter().position(|&p| p == 0) else {
            return Err(Refusal(
                "The future completed: do not poll after Ready; ring the bell.".into(),
            ));
        };
        if crate::puzzles::hour_bell_reports()[index] != report {
            return Err(Refusal(format!(
                "State {} ({}) refuses that report; no transition.",
                index + 1,
                crate::content::F8_BELL_STATES[index]
            )));
        }
        work.polls[index] = report;
        Ok(format!(
            "State {}: {}.",
            index + 1,
            if report == 1 {
                "Pending"
            } else {
                "Ready — ring now"
            }
        ))
    }

    pub fn floor8_ring(&mut self) -> Result<String, Refusal> {
        let work = &mut self.floor8.chambers[0].work;
        if work.polls.as_slice() != crate::puzzles::hour_bell_reports() || work.rung {
            return Err(Refusal(
                "Ring exactly once at the whole future's completion.".into(),
            ));
        }
        work.rung = true;
        Ok("The completed future rings once.".into())
    }

    pub fn floor8_tapestry(&mut self, pin: bool) -> Result<String, Refusal> {
        let work = &mut self.floor8.chambers[1].work;
        if pin {
            if !work.torn {
                return Err(Refusal("First move unpinned and exhibit the tear.".into()));
            }
            if work.pinned {
                return Err(Refusal("The referent is already pinned.".into()));
            }
            work.pinned = true;
            return Ok("Referent pinned at address A; self-reference points to A.".into());
        }
        if !work.torn {
            work.torn = true;
            return Ok("Unpinned referent moves A to B; its self-reference still points to A: TORN. Rehang at A before pinning.".into());
        }
        if !work.pinned {
            return Err(Refusal(
                "The tear is witnessed. Pin before the sound handle move.".into(),
            ));
        }
        if work.moved {
            return Err(Refusal("The sound handle move is already complete.".into()));
        }
        work.moved = true;
        Ok("Handle/frame token moved; pinned referent stays at A, self-reference A: SOUND.".into())
    }

    pub fn floor8_file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        if !self.floor8_diagnosed(chamber) {
            let mismatch = match chamber {
                0 => {
                    "POLL TRACE + COMPLETION mismatch: five correct reports and one completion ring are required."
                }
                1 => {
                    "MOVEMENT RECORD mismatch: exhibit the tear, pin the referent, then move the handle soundly."
                }
                2 => {
                    "WAKER mismatch: four ordered parts and the two-sleep/two-wake trial are required."
                }
                3 => "ROSTER mismatch: predict and explicitly judge every computed event.",
                4 => {
                    "TERMINAL ACCOUNTING mismatch: every one of the six started errands must match its computed completed, cancelled-with-cleanup or running-by-name accounting."
                }
                _ => "The chamber's work does not yet prove its claim.",
            };
            return Err(Refusal(mismatch.into()));
        }
        let progress = self
            .floor8
            .chambers
            .get_mut(chamber)
            .ok_or_else(|| Refusal("No such chamber.".into()))?;
        if progress.state == ChamberState::Repaired {
            return Err(Refusal("Already repaired.".into()));
        }
        progress.state = ChamberState::Diagnosed;
        Ok("Filed with Pinwright: the work proves its claim.".into())
    }

    /// Repair a diagnosed chamber (the seal): Diagnosed →
    /// Repaired, with the facet at full standard (§5.5).
    pub fn floor8_repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        let progress = self
            .floor8
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
                    Ok("Repaired at full standard. A facet of the Pinstone \
                        settles into the codex."
                        .to_string())
                } else {
                    Ok("Repaired — though the seam was named, and the record \
                        keeps it. No facet, but the floor still completes."
                        .to_string())
                }
            }
        }
    }

    /// Reset one Floor 8 chamber's work to its starting
    /// state: hint usage stays on the record.
    pub fn floor8_reset_chamber(&mut self, chamber: usize) {
        if let Some(progress) = self.floor8.chambers.get_mut(chamber) {
            progress.state = ChamberState::Unworked;
            progress.facet = false;
            progress.work = HourWork::default();
        }
    }

    /// Record that the player read a Floor 8 hint rung.
    pub fn floor8_view_hint(&mut self, chamber: usize) -> u8 {
        let Some(progress) = self.floor8.chambers.get_mut(chamber) else {
            return 0;
        };
        progress.hints_used = (progress.hints_used + 1).min(4);
        progress.hints_used
    }

    /// Hints used on a Floor 8 chamber.
    pub fn floor8_hints_used(&self, chamber: usize) -> u8 {
        self.floor8
            .chambers
            .get(chamber)
            .map(|progress| progress.hints_used)
            .unwrap_or(0)
    }

    pub fn floor9_diagnosed(&self, chamber: usize) -> bool {
        let Some(p) = self.floor9.chambers.get(chamber) else {
            return false;
        };
        match chamber {
            0 => p.work.judged && p.work.predictions == crate::puzzles::hoard_shelf_trace().0,
            1 => {
                let (slots, nexts) = crate::puzzles::hoard_chain_indices();
                p.work.slots == slots && p.work.nexts == nexts
            }
            2 => {
                let (probes, flood) = crate::puzzles::hoard_cellar_trace();
                p.work.judged && p.work.probes == probes && p.work.flood_cost == flood
            }
            3 => {
                let missed = p.work.steward_missed || p.work.misses_explained.iter().any(|&v| v);
                p.work.predicted == crate::puzzles::hoard_steward_run().1
                    && p.work.ran
                    && !p.work.steward_missed
                    && (!missed || p.work.misses_explained[crate::content::STEWARD_TRAP_OP])
            }
            4 => {
                p.work.suite_walked
                    && usize::from(p.work.last_mile) == crate::puzzles::hoard_last_mile()
                    && p.work.blocks == crate::puzzles::hoard_intern_transcript(true).0
            }
            _ => false,
        }
    }
    pub fn floor9_file_proof(&mut self, chamber: usize) -> Result<String, Refusal> {
        if chamber > 4
            || (chamber > 0 && self.floor9.chambers[chamber - 1].state != ChamberState::Repaired)
        {
            return Err(Refusal(
                "Repair the prior chamber first; this proof is unavailable.".into(),
            ));
        }
        if !self.floor9_diagnosed(chamber) {
            let mismatch = match chamber {
                0 => {
                    "LENGTH + CAPACITY TRACE mismatch: judge all eight exact pairs against the real shelf, including its capacity boundary."
                }
                1 => {
                    "SLOTS + NEXT INDICES mismatch: use first-visit arena slots, preserve the one shared tail and end chains with 255; cycles are not chains."
                }
                2 => {
                    "PROBE TRACE + FLOOD COST mismatch: place the five keys, flood a fresh cellar with the four flood keys, and judge exact counts including landing slots."
                }
                4 => {
                    "SUITE + BLOCK COUNT + LAST MILE mismatch: walk the unchanged suite; identify the absent string and intern it. A diagnosis with blocks[last] = 255 is not the full transcript."
                }
                _ => {
                    "ALLOCATION COUNT + EXPLANATION mismatch: run the filed prediction; explain a pending miss by naming the growth operation before re-predicting, then run the exact count."
                }
            };
            return Err(Refusal(mismatch.into()));
        }
        let p = &mut self.floor9.chambers[chamber];
        if p.state == ChamberState::Repaired {
            return Err(Refusal("Already repaired.".into()));
        }
        p.state = ChamberState::Diagnosed;
        Ok("Filed with Heapwarden: the record proves its claim.".into())
    }
    pub fn floor9_repair_chamber(&mut self, chamber: usize) -> Result<String, Refusal> {
        if !self.floor9_diagnosed(chamber)
            || (chamber > 0 && self.floor9.chambers[chamber - 1].state != ChamberState::Repaired)
        {
            return Err(Refusal(
                "The seal sleeps: complete the ordered proof first.".into(),
            ));
        }
        let p = &mut self.floor9.chambers[chamber];
        if p.state != ChamberState::Diagnosed {
            return Err(Refusal(
                "File the proof before repairing; a repaired seal cannot strike twice.".into(),
            ));
        }
        p.state = ChamberState::Repaired;
        p.facet = p.hints_used <= 3;
        Ok(if p.facet {
            "Repaired at full standard. A facet of The Hoardstone settles into the codex."
        } else {
            "Repaired; the fourth hint named the seam, so no facet is earned."
        }
        .into())
    }
    pub fn floor9_foe_proof(&self) -> bool {
        let f = &self.floor9.foe;
        let (classes, _, census) = crate::puzzles::hoard_census();
        self.floor9_chambers_repaired() == 5
            && (0..5).all(|c| self.floor9_diagnosed(c))
            && f.exhibits.iter().all(|v| *v)
            && f.classes == classes
            && f.census.as_ref() == Some(&census)
    }
    pub fn floor9_foe_gate(&self) -> bool {
        self.floor9.foe.trials.iter().all(|v| *v) && self.floor9.foe.adversaries.iter().all(|v| *v)
    }
    pub fn floor9_foe_rewrite(&self) -> bool {
        self.floor9
            .foe
            .reissued
            .iter()
            .enumerate()
            .all(|(i, v)| crate::puzzles::hoard_accept_tally(i, v.clone()))
    }
    pub fn floor9_foe_exhibit(&mut self, index: usize) -> Result<String, Refusal> {
        if index >= 3 || self.floor9.foe.state == FoeState::Fallen {
            return Err(Refusal("No live exhibit at that index.".into()));
        }
        if self.floor9_chambers_repaired() != 5 {
            return Err(Refusal("Repair all five chambers first.".into()));
        }
        if index > 0 && !self.floor9.foe.exhibits[index - 1] {
            return Err(Refusal(format!(
                "First exhibit the {}.",
                crate::content::F9_EXHIBITS[index - 1]
            )));
        }
        let (classes, counts, census) = crate::puzzles::hoard_census();
        if index > 0 && self.floor9.foe.classes != classes {
            return Err(Refusal("OVERLAP CLASSES mismatch: sort six physical identities by their actual A/B/C membership.".into()));
        }
        if index == 2 {
            self.floor9.foe.census = Some(census.clone());
        }
        self.floor9.foe.exhibits[index] = true;
        self.floor9.foe.pressure = (self.floor9.foe.pressure + 1).min(3);
        Ok(format!(
            "{}: honest tallies {counts:?}; identity census {:?}. Frontier desk {}/3 still quotes bare sum {} — misleading; proof preserved.",
            crate::content::F9_EXHIBITS[index],
            if index == 2 { Some(census) } else { None },
            self.floor9.foe.pressure,
            counts.iter().sum::<u32>()
        ))
    }
    pub fn floor9_foe_prove(&mut self) -> Result<String, Refusal> {
        if !self.floor9_foe_proof() {
            return Err(Refusal("File all three ordered exhibits, six exact classes and the stamped identity census after five repairs.".into()));
        }
        if self.floor9.foe.state == FoeState::Standing {
            self.floor9.foe.state = FoeState::Exposed;
        }
        Ok("The Hoard Beneath is EXPOSED: the honest tallies overlap; the sum lies.".into())
    }
    pub fn floor9_foe_reissue(&mut self, index: usize) -> Result<String, Refusal> {
        if index >= 3
            || !self.floor9_foe_proof()
            || !self.floor9_foe_gate()
            || self.floor9.foe.state == FoeState::Fallen
        {
            return Err(Refusal(
                "Complete proof and the trial/refusal ledger first.".into(),
            ));
        }
        let (_, counts, _) = crate::puzzles::hoard_census();
        let scroll = crate::puzzles::hoard_tally_scroll(index);
        let candidate = match self.floor9.foe.stamps[index] {
            1 => Some(crate::puzzles::Stamped::seal(counts[index], scroll)),
            2 => Some(crate::puzzles::Stamped::seal(
                counts[index],
                scroll ^ u32::MAX,
            )),
            _ => None,
        };
        if !crate::puzzles::hoard_accept_tally(index, candidate.clone()) {
            return Err(Refusal("BARE / FOREIGN CENSUS refused: the desk requires the count fused to its own census-scroll.".into()));
        }
        self.floor9.foe.reissued[index] = candidate;
        Ok("Stamped re-issue accepted; provenance is part of the value.".into())
    }
    pub fn floor9_foe_arm_strike(&mut self) -> Result<(), Refusal> {
        self.floor9.foe.gate_passed = self.floor9_foe_gate();
        if self.floor9.foe.state != FoeState::Exposed
            || !self.floor9_foe_proof()
            || !self.floor9_foe_gate()
            || !self.floor9_foe_rewrite()
        {
            return Err(Refusal("The seal requires exposed identity proof, all trial/refusal records and three accepted stamped re-issues.".into()));
        }
        self.floor9.foe.strike = StrikePhase::Armed;
        Ok(())
    }
    pub fn floor9_foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.floor9.foe.strike != StrikePhase::Armed {
            return Err(Refusal("Press the proven seal first.".into()));
        }
        self.floor9_foe_arm_strike()?;
        self.floor9.foe.strike = StrikePhase::Idle;
        self.floor9.foe.state = FoeState::Fallen;
        self.floor9.foe.fallen_date = Some(date.into());
        if !self
            .codex
            .iter()
            .any(|s| s == crate::content::HOARDSTONE_TEXT)
        {
            self.codex.push(crate::content::HOARDSTONE_TEXT.into());
        }
        if !self.pack.iter().any(|s| s == "The Heapwarden's Trio") {
            self.pack.push("The Heapwarden's Trio".into());
        }
        Ok("The Hoard Beneath falls. The Hoardstone enters the codex; The Heapwarden's Trio enters the pack. The desk itself refuses anything else.".into())
    }
    pub fn floor9_reachable(&self) -> bool {
        self.floor5_complete()
    }
    pub fn floor9_chambers_repaired(&self) -> usize {
        self.floor9
            .chambers
            .iter()
            .filter(|c| c.state == ChamberState::Repaired)
            .count()
    }
    pub fn floor9_complete(&self) -> bool {
        self.floor9_chambers_repaired() == 5 && self.floor9.foe.state == FoeState::Fallen
    }

    /// Floor 2 fallen and Floor 6 complete; Floor 7 is not a gate.
    pub fn floor8_reachable(&self) -> bool {
        self.floor2.foe.state == FoeState::Fallen && self.floor6_complete()
    }

    /// How many Floor 8 chambers stand repaired.
    pub fn floor8_chambers_repaired(&self) -> usize {
        self.floor8
            .chambers
            .iter()
            .filter(|chamber| chamber.state == ChamberState::Repaired)
            .count()
    }

    /// Whether Floor 8 is complete: five chambers + FOE.
    pub fn floor8_complete(&self) -> bool {
        self.floor8_chambers_repaired() == 5 && self.floor8.foe.state == FoeState::Fallen
    }

    /// Take the dimension book down: the re-survey may begin,
    /// and the Foundation stands Exposed.
    pub fn floor4_foe_take_book(&mut self) {
        self.floor4.foe.book_taken = true;
        if self.floor4.foe.state == FoeState::Standing {
            self.floor4.foe.state = FoeState::Exposed;
        }
    }

    /// Drop the line on one book entry's stone: the measured
    /// size against the book's figure, spoken plainly either
    /// way.
    pub fn floor4_foe_measure(&mut self, entry: usize) -> Result<String, Refusal> {
        if !self.floor4.foe.book_taken {
            return Err(Refusal(
                "take the dimension book down first — the re-survey \
                 begins with the book in hand"
                    .to_string(),
            ));
        }
        let Some(slot) = self.floor4.foe.measured.get_mut(entry) else {
            return Err(Refusal("no such entry in the book".to_string()));
        };
        *slot = true;
        let (name, figure) = crate::content::DIMENSION_BOOK[entry];
        let truth = crate::content::dimension_book_true(entry);
        if figure == truth {
            Ok(format!(
                "The line drops on {name}: the stone measures {truth}, \
                 and the book says {figure}. This entry is honest."
            ))
        } else {
            Ok(format!(
                "The line drops on {name}: the stone measures {truth}, \
                 and the book says {figure}. The entry is false to the \
                 ground — exhibit it beside its true value."
            ))
        }
    }

    /// Exhibit one measured entry beside its true value:
    /// only a false entry can be exhibited, and only once
    /// the line has actually dropped on its stone.
    pub fn floor4_foe_exhibit(&mut self, entry: usize) -> Result<String, Refusal> {
        if !self
            .floor4
            .foe
            .measured
            .get(entry)
            .copied()
            .unwrap_or(false)
        {
            return Err(Refusal(
                "drop the line on the stone first — an unmeasured \
                 entry cannot be exhibited, only believed"
                    .to_string(),
            ));
        }
        if !crate::puzzles::book_disagrees(entry) {
            return Err(Refusal(
                "this entry agrees with its stone: there is nothing \
                 to exhibit but an honest figure"
                    .to_string(),
            ));
        }
        self.floor4.foe.exhibited[entry] = true;
        let (name, figure) = crate::content::DIMENSION_BOOK[entry];
        let truth = crate::content::dimension_book_true(entry);
        Ok(format!(
            "Exhibited: {name} — the book's {figure} beside the \
             stone's {truth}, off by exactly the shadow's offset."
        ))
    }

    /// The Foundation's proof, whole: the book taken, every
    /// entry measured at its stone, and every false entry
    /// exhibited beside its true value. (The re-cut — the
    /// strike's other half — is the gate, tracked apart.)
    pub fn floor4_foe_proof_complete(&self) -> bool {
        let foe = &self.floor4.foe;
        foe.book_taken
            && foe.measured.iter().all(|done| *done)
            && (0..crate::content::DIMENSION_BOOK.len())
                .all(|entry| !crate::puzzles::book_disagrees(entry) || foe.exhibited[entry])
    }

    /// Prove the foreign re-cut: four stones seated in the
    /// spec's order, no shadow-cut stone among them — its
    /// stencil betrays it before its seat is even read — and
    /// the line reading the promised offsets from `c_offsets`.
    pub fn floor4_foe_prove_foreign(&mut self) -> Result<String, Refusal> {
        let order = self.floor4.foe.foreign_order.clone();
        if order.len() < 4 {
            return Err(Refusal(
                "the line drops on a finished set: four stones, no fewer".to_string(),
            ));
        }
        for &stone in &order {
            let claimed = crate::content::RECUT_FOREIGN_STENCIL[stone as usize];
            let measured = crate::content::RECUT_FOREIGN[stone as usize].1;
            if claimed != measured {
                return Err(Refusal(format!(
                    "the line stops at {}: its stencil claims {claimed}, \
                     and the stone measures {measured} — it was cut from \
                     the crooked book, and no spec can seat it",
                    crate::content::RECUT_FOREIGN[stone as usize].0
                )));
            }
        }
        if order != vec![0, 1, 2, 3] {
            return Err(Refusal(
                "the stones stand, but not in the spec's seats: the \
                 foreign order is sill, post, beam, cap"
                    .to_string(),
            ));
        }
        let dims: Vec<(u32, u32)> = order
            .iter()
            .map(|&stone| {
                let (_, size, align) = crate::content::RECUT_FOREIGN[stone as usize];
                (size, align)
            })
            .collect();
        let offsets = crate::puzzles::c_offsets(&dims);
        self.floor4.foe.foreign_proven = true;
        self.floor4_foe_refresh_gate();
        Ok(format!(
            "The line verifies the foreign set: seats at {}.",
            offsets
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    /// Prove the free re-cut: four stones ordered so the
    /// wall's total meets the budget — the budget being the
    /// least span the four allow.
    pub fn floor4_foe_prove_free(&mut self) -> Result<String, Refusal> {
        let order = self.floor4.foe.free_order.clone();
        if order.len() < 4 {
            return Err(Refusal(
                "the line drops on a finished set: four stones, no fewer".to_string(),
            ));
        }
        let pairs: Vec<(u32, u32)> = crate::content::RECUT_FREE
            .iter()
            .map(|(_, size, align)| (*size, *align))
            .collect();
        let seated: Vec<(u32, u32)> = order.iter().map(|&stone| pairs[stone as usize]).collect();
        let (total, _) = crate::puzzles::wall_layout(&seated);
        let budget = crate::puzzles::wall_minimum(&pairs);
        if total != budget {
            return Err(Refusal(format!(
                "the line reads the free set at {total}; the budget \
                 is {budget} — the least span these four allow"
            )));
        }
        self.floor4.foe.free_proven = true;
        self.floor4_foe_refresh_gate();
        Ok(format!(
            "The line verifies the free set: {total} against a \
             budget of {budget} — not a unit wasted."
        ))
    }

    /// The gate is the line itself: both re-cut sets laid
    /// true and verified, the mark stands.
    fn floor4_foe_refresh_gate(&mut self) {
        let foe = &mut self.floor4.foe;
        foe.gate_passed = foe.foreign_proven && foe.free_proven;
    }

    /// Arm the Foundation's strike: proof complete and the
    /// re-cut verified by the line, as with every FOE.
    pub fn floor4_foe_arm_strike(&mut self) -> Result<(), Refusal> {
        if self.floor4.foe.state == FoeState::Fallen {
            return Err(Refusal(
                "the Foundation has already been re-laid".to_string(),
            ));
        }
        if !self.floor4_foe_proof_complete() {
            return Err(Refusal(
                "the strike sleeps until the re-survey stands complete: \
                 every entry measured at its stone, every false one \
                 exhibited beside its true value"
                    .to_string(),
            ));
        }
        if !self.floor4.foe.gate_passed {
            return Err(Refusal(
                "the strike sleeps until the re-cut stands verified: \
                 the foreign set to its spec, the free set at its budget"
                    .to_string(),
            ));
        }
        self.floor4.foe.strike = StrikePhase::Armed;
        Ok(())
    }

    /// Confirm the strike: the Foundation is re-laid from
    /// the measured stones, and the Brass Eye Stone and the
    /// Plumb Line change hands.
    pub fn floor4_foe_confirm_strike(&mut self, date: &str) -> Result<String, Refusal> {
        if self.floor4.foe.state == FoeState::Fallen {
            return Err(Refusal(
                "the Foundation has already been re-laid".to_string(),
            ));
        }
        if self.floor4.foe.strike != StrikePhase::Armed {
            return Err(Refusal(
                "the strike is not armed — press the seal first".to_string(),
            ));
        }
        self.floor4.foe.strike = StrikePhase::Idle;
        self.floor4.foe.state = FoeState::Fallen;
        self.floor4.foe.fallen_date = Some(date.to_string());
        if !self.codex.iter().any(|stone| stone == BRASS_EYE_STONE) {
            self.codex.push(BRASS_EYE_STONE.to_string());
        }
        if !self.pack.iter().any(|loot| loot == PLUMB_LINE) {
            self.pack.push(PLUMB_LINE.to_string());
        }
        Ok("The strike lands — not a foundation condemned, but a \
            survey replaced: the crooked stones are lifted, the book \
            is burned in the Plumbmason's own brazier, and the master \
            foundation is re-laid from figures the line read off the \
            ground itself. It will settle no more."
            .to_string())
    }

    /// Record that the player read a Foundation hint rung.
    pub fn floor4_foe_view_hint(&mut self) -> u8 {
        self.floor4.foe_hints_used = (self.floor4.foe_hints_used + 1).min(4);
        self.floor4.foe_hints_used
    }

    /// The deepest Foundation hint rung recorded.
    pub fn floor4_foe_hints_used(&self) -> u8 {
        self.floor4.foe_hints_used
    }

    /// Floor 3 stands open when both valley FOEs have fallen
    /// (FULL-DESIGN §5.2: Floor 3 requires Floors 1 and 2).
    pub fn floor3_reachable(&self) -> bool {
        self.foe.state == FoeState::Fallen && self.floor2.foe.state == FoeState::Fallen
    }

    /// How many Floor 3 chambers stand repaired.
    pub fn floor3_chambers_repaired(&self) -> usize {
        self.floor3
            .chambers
            .iter()
            .filter(|c| c.state == ChamberState::Repaired)
            .count()
    }

    /// Floor 3 is complete when its five chambers stand repaired
    /// and its FOE has fallen.
    pub fn floor3_complete(&self) -> bool {
        self.floor3_chambers_repaired() == 5 && self.floor3.foe.state == FoeState::Fallen
    }

    /// The Stratum I stairs clauses, each with its standing. The
    /// stairs out of Stratum I need more than this floor can give;
    /// the unmet clauses say so plainly.
    pub fn stairs_clauses(&self) -> [(bool, String); 3] {
        let foe2_fallen = self.floor2.foe.state == FoeState::Fallen;
        let foe3_fallen = self.floor3.foe.state == FoeState::Fallen;
        // §5.6: a valley FOE's lantern is keeper-lit when its
        // crate has passed the standing gate — or, for the FOE
        // that has not fallen, when its five chambers stand
        // repaired; the stairs do not demand the fall itself.
        let lantern1 = self.foe.gate_passed || self.chambers_repaired() == 5;
        let lantern2 = self.floor2.foe.gate_passed || self.floor2_chambers_repaired() == 5;
        let lantern3 = self.floor3.foe.gate_passed || self.floor3_chambers_repaired() == 5;
        let lantern_text = |floor: u8, lit: bool| {
            if lit {
                format!("Floor {floor}'s lantern is lit")
            } else {
                format!("Floor {floor}'s lantern waits on its gate or its chambers")
            }
        };
        [
            (
                self.foe.state == FoeState::Fallen,
                "FOE 1, the Borrowed Hoard, has fallen".to_string(),
            ),
            (
                foe2_fallen || foe3_fallen,
                match (foe2_fallen, foe3_fallen) {
                    (true, true) => "FOE 2, the Menagerie, and FOE 3, the Gatehouse \
                         Release, have fallen"
                        .to_string(),
                    (true, false) => "FOE 2, the Menagerie, has fallen".to_string(),
                    (false, true) => "FOE 3, the Gatehouse Release, has fallen".to_string(),
                    (false, false) => "FOE 2 or FOE 3 has fallen".to_string(),
                },
            ),
            (
                lantern1 && lantern2 && lantern3,
                format!(
                    "every Stratum I boss crate clippy-clean — {}; {}; {}",
                    lantern_text(1, lantern1),
                    lantern_text(2, lantern2),
                    lantern_text(3, lantern3)
                ),
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
            floor3: self.floor3.clone(),
            floor4: self.floor4.clone(),
            floor5: self.floor5.clone(),
            floor6: self.floor6.clone(),
            floor7: self.floor7.clone(),
            floor8: self.floor8.clone(),
            floor9: self.floor9.clone(),
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
            floor3: file.floor3,
            floor4: file.floor4,
            floor5: file.floor5,
            floor6: file.floor6,
            floor7: file.floor7,
            floor8: file.floor8,
            floor9: file.floor9,
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
        if self.floor3.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five Floor 3 chambers"));
        }
        if self.floor4.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five Floor 4 chambers"));
        }
        if self.floor5.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five Floor 5 chambers"));
        }
        if self.floor6.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five Floor 6 chambers"));
        }
        if self.floor7.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five Floor 7 chambers"));
        }
        // Floor 9 — only this floor's proofs and facets are extended here.
        if self.floor9.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five Floor 9 chambers"));
        }
        for (chamber, progress) in self.floor9.chambers.iter().enumerate() {
            if progress.facet
                && (progress.state != ChamberState::Repaired || progress.hints_used > 3)
            {
                return Err(corrupt(
                    "a Floor 9 facet stands without a full-standard repair",
                ));
            }
            if progress.state != ChamberState::Unworked
                && chamber > 0
                && self.floor9.chambers[chamber - 1].state != ChamberState::Repaired
            {
                return Err(corrupt("a Floor 9 proof bypasses the prior repair"));
            }
            if progress.state != ChamberState::Unworked && !self.floor9_diagnosed(chamber) {
                return Err(corrupt(
                    "the save claims a Floor 9 proof its work does not hold",
                ));
            }
        }
        let f = &self.floor9.foe;
        if f.page > 2
            || f.pressure > 3
            || f.stamps.iter().any(|v| *v > 2)
            || f.classes.iter().any(|v| *v > 7)
            || (f.exhibits[1] && !f.exhibits[0])
            || (f.exhibits[2] && !f.exhibits[1])
            || (f.exhibits.iter().any(|v| *v) && self.floor9_chambers_repaired() != 5)
            || (f.exhibits[1] && f.classes != crate::puzzles::hoard_census().0)
            || (f.census.is_some()
                && (!f.exhibits[2] || f.census.as_ref() != Some(&crate::puzzles::hoard_census().2)))
            || (f.exhibits[2] && f.census.is_none())
            || (f.gate_passed != self.floor9_foe_gate())
            || ((f.trials.iter().any(|v| *v) || f.adversaries.iter().any(|v| *v))
                && !self.floor9_foe_proof())
            || (f.strike == StrikePhase::Armed && f.state != FoeState::Exposed)
            || (f.fallen_date.is_some() != (f.state == FoeState::Fallen))
            || (f.reissued.iter().any(Option::is_some)
                && (!self.floor9_foe_proof() || !self.floor9_foe_gate()))
            || f.reissued
                .iter()
                .enumerate()
                .any(|(i, v)| v.is_some() && !crate::puzzles::hoard_accept_tally(i, v.clone()))
            || (f.state != FoeState::Standing && !self.floor9_foe_proof())
            || ((f.state == FoeState::Fallen || f.strike == StrikePhase::Armed)
                && (!self.floor9_foe_gate() || !self.floor9_foe_rewrite()))
            || (f.state == FoeState::Fallen
                && (f.fallen_date.is_none()
                    || f.strike != StrikePhase::Idle
                    || !self
                        .codex
                        .iter()
                        .any(|s| s == crate::content::HOARDSTONE_TEXT)
                    || !self.pack.iter().any(|s| s == "The Heapwarden's Trio")))
        {
            return Err(corrupt(
                "the Floor 9 Hoard proof, gate, stamped re-issue or strike is forged",
            ));
        }
        if self.floor8.chambers.len() != 5 {
            return Err(corrupt("the save does not hold five Floor 8 chambers"));
        }
        for chamber in 0..5 {
            if self.floor3.chambers[chamber].state != ChamberState::Unworked
                && !self.floor3_diagnosed(chamber)
            {
                return Err(corrupt(
                    "the save claims a Floor 3 proof its work does not hold",
                ));
            }
        }
        for chamber in 0..5 {
            if self.floor4.chambers[chamber].state != ChamberState::Unworked
                && !self.floor4_diagnosed(chamber)
            {
                return Err(corrupt(
                    "the save claims a Floor 4 proof its work does not hold",
                ));
            }
        }
        for chamber in 0..5 {
            if self.floor5.chambers[chamber].state != ChamberState::Unworked
                && !self.floor5_diagnosed(chamber)
            {
                return Err(corrupt(
                    "the save claims a Floor 5 proof its work does not hold",
                ));
            }
        }
        for chamber in 0..5 {
            if self.floor6.chambers[chamber].state != ChamberState::Unworked
                && !self.floor6_diagnosed(chamber)
            {
                return Err(corrupt(
                    "the save claims a Floor 6 proof its work does not hold",
                ));
            }
        }
        for chamber in 0..5 {
            if self.floor7.chambers[chamber].state != ChamberState::Unworked
                && !self.floor7_diagnosed(chamber)
            {
                return Err(corrupt(
                    "the save claims a Floor 7 proof its work does not hold",
                ));
            }
        }
        for chamber in 0..5 {
            let progress = &self.floor8.chambers[chamber];
            if progress.facet
                && (progress.state != ChamberState::Repaired || progress.hints_used > 3)
            {
                return Err(corrupt(
                    "a Floor 8 facet stands without a full-standard repair",
                ));
            }
            if self.floor8.chambers[chamber].state != ChamberState::Unworked
                && !self.floor8_diagnosed(chamber)
            {
                return Err(corrupt(
                    "the save claims a Floor 8 proof its work does not hold",
                ));
            }
        }
        let f = &self.floor8.foe;
        if f.page > 2
            || f.selected > 3
            || f.pressure > 3
            || f.signal_seal > 1
            || f.signal_generation > 3
            || f.card_lines.iter().any(|&v| v > 2)
            || f.unsequenced.is_some_and(|i| i >= 4)
            || f.exhibits
                .iter()
                .enumerate()
                .any(|(i, &seen)| seen && f.exhibits[..i].iter().any(|v| !*v))
            || f.predictions.iter().enumerate().any(|(i, p)| {
                let events = crate::puzzles::hour::presentation(i);
                p.len() > events.len()
                    || p.iter()
                        .zip(events.iter())
                        .any(|(&kind, event)| kind != crate::puzzles::hour_event_kind(event))
            })
        {
            return Err(corrupt("invalid Floor 8 evidence or roster prefix"));
        }
        let real_gate = f
            .predictions
            .iter()
            .enumerate()
            .all(|(i, p)| p.len() == crate::puzzles::hour::presentation(i).len())
            && f.unsequenced == Some(crate::content::HOUR_UNSEQUENCED)
            && f.policy_seen
            && crate::puzzles::hour::policy_holds();
        if f.gate_passed && !real_gate {
            return Err(corrupt("Floor 8 gate lacks computed roster evidence"));
        }
        if f.state != FoeState::Standing && !f.exhibits.iter().all(|v| *v) {
            return Err(corrupt("Floor 8 exposure lacks sequenced exhibits"));
        }
        if (f.state == FoeState::Fallen || f.strike == StrikePhase::Armed)
            && (!real_gate || !self.floor8_foe_rewrite() || f.state == FoeState::Standing)
        {
            return Err(corrupt("Floor 8 strike lacks proven typed repair"));
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
        if !self.floor2.herald_filed.is_empty()
            && self.floor2.herald_filed.len() != HERALD_LETTERS.len()
        {
            return Err(corrupt("the herald record is the wrong size"));
        }

        if !self.floor2.foe.audit_exhibited.is_empty()
            && self.floor2.foe.audit_exhibited.len() != FOE_ADMISSIONS.len()
        {
            return Err(corrupt("the audit exhibit record is the wrong size"));
        }
        if self.floor2.foe.editor_code.len() > MAX_EDITOR_BYTES {
            return Err(corrupt("the Desk draft exceeds its bound"));
        }
        if let Some(row) = self.floor2.foe.disagreement
            && row >= FOE_ADMISSIONS.len()
        {
            return Err(corrupt(
                "the audit names an admission that is not on the table",
            ));
        }
        match self.floor2.foe.state {
            FoeState::Exposed | FoeState::Fallen => {
                if !self.floor2_foe_proof_complete() {
                    return Err(corrupt("Floor 2's FOE stands exposed without its audit"));
                }
            }
            FoeState::Standing => {}
        }
        if self.floor2.foe.state == FoeState::Fallen {
            if !self.floor2.foe.gate_passed {
                return Err(corrupt("Floor 2's FOE fell without the gate"));
            }
            if !self.codex.iter().any(|stone| stone == LENS_STONE)
                || !self.pack.iter().any(|loot| loot == PLUGIN_MENAGERIE)
            {
                return Err(corrupt("Floor 2's FOE fell without its awards"));
            }
            if self.floor2.foe.fallen_date.is_none() {
                return Err(corrupt("Floor 2's FOE fell without a date on the chart"));
            }
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
        3 => work.drove_round && registry_bars_ok(&work.bar_choices),
        4 => {
            matches!(work.route_choice, Some(choice) if collar_route_ok(choice))
                && matches!(work.collision_choice, Some(choice) if collision_reading_ok(choice))
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
        let future = text.replace("\"version\":9", "\"version\":99");
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
    /// with a fresh Floor 2, and re-saves as version 4.
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
        assert!(resaved.contains("\"version\":9"));
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

    /// Validation: a version-2 save (no floor3 field at all)
    /// loads with a fresh Floor 3, and re-saves as version 4.
    #[test]
    fn version_two_saves_load_with_default_floor3() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value.as_object_mut().expect("object").remove("floor3");
        value["version"] = serde_json::json!(2);
        let loaded = Expedition::load_json(&value.to_string()).expect("v2 loads");
        assert_eq!(loaded.floor3, Floor3Progress::new());
        let resaved = loaded.save_json().expect("re-save");
        assert!(resaved.contains("\"version\":9"));
    }

    /// Validation: a version-3 save (no floor4 field at all)
    /// loads with a fresh Floor 4, and re-saves as version 4.
    #[test]
    fn version_three_saves_load_with_default_floor4() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value.as_object_mut().expect("object").remove("floor4");
        value["version"] = serde_json::json!(3);
        let loaded = Expedition::load_json(&value.to_string()).expect("v3 loads");
        assert_eq!(loaded.floor4, Floor4Progress::new());
        let resaved = loaded.save_json().expect("re-save");
        assert!(resaved.contains("\"version\":9"));
    }

    /// Validation: a version-4 save (no floor5 field at all)
    /// loads with a fresh Floor 5, and re-saves as version 5.
    #[test]
    fn version_four_saves_load_with_default_floor5() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value.as_object_mut().expect("object").remove("floor5");
        value["version"] = serde_json::json!(4);
        let loaded = Expedition::load_json(&value.to_string()).expect("v4 loads");
        assert_eq!(loaded.floor5, Floor5Progress::new());
        let resaved = loaded.save_json().expect("re-save");
        assert!(resaved.contains("\"version\":9"));
    }

    /// Validation: Floor 5 opens only when Floor 4 stands
    /// complete — five chambers repaired and the Crooked
    /// Foundation fallen.
    #[test]
    fn floor5_reachable_follows_floor4_completion() {
        let mut expedition = Expedition::new();
        assert!(!expedition.floor5_reachable());
        for chamber in expedition.floor4.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        assert!(!expedition.floor5_reachable(), "the FOE still stands");
        expedition.floor4.foe.state = FoeState::Fallen;
        assert!(expedition.floor5_reachable());
        assert!(!expedition.floor5_complete());
    }

    /// Floor 6 Step 4 state: Choosing the Cell judges on
    /// the housed kinds and the true sentences; the Heart
    /// Locked Twice falls to exhibits in order, the loom's
    /// walk, the rewritten card, and the strike — and its
    /// stone and pair change hands.
    #[test]
    fn floor6_choosing_and_heart_fall() {
        let mut expedition = Expedition::new();
        // Choosing the Cell.
        {
            let work = &mut expedition.floor6.chambers[4].work;
            work.watched = true;
            work.choices = [2, 3, 1];
            work.invariants = [3, 2, 2];
        }
        assert!(expedition.floor6_diagnosed(4));
        assert!(expedition.floor6_file_proof(4).is_ok());
        assert!(expedition.floor6_repair_chamber(4).is_ok());
        // The FOE: sights out of order are refused.
        assert!(expedition.floor6_foe_exhibit(1).is_err());
        expedition.floor6_foe_exhibit(0).expect("the census");
        expedition.floor6_foe_exhibit(1).expect("the worked day");
        expedition.floor6_foe_exhibit(2).expect("the torn reading");
        assert!(expedition.floor6_foe_proof_complete());
        // The strike sleeps until the loom and the card.
        assert!(expedition.floor6_foe_arm_strike().is_err());
        for case in 0..4 {
            expedition.floor6_foe_walk_case(case).expect("loom case");
        }
        assert!(expedition.floor6.foe.gate_passed);
        assert!(expedition.floor6_foe_arm_strike().is_err());
        expedition.floor6.foe.card_lines = crate::content::FOE6_CARD_KEY;
        assert!(expedition.floor6_foe_repair_ok());
        expedition.floor6_foe_arm_strike().expect("strike arms");
        expedition
            .floor6_foe_confirm_strike("2026-11-10")
            .expect("the Heart falls");
        assert_eq!(expedition.floor6.foe.state, FoeState::Fallen);
        assert_eq!(
            expedition.floor6.foe.fallen_date.as_deref(),
            Some("2026-11-10")
        );
        assert!(
            expedition
                .codex
                .iter()
                .any(|stone| stone == crate::content::LOCKHEART_STONE)
        );
        assert!(
            expedition
                .pack
                .iter()
                .any(|loot| loot == crate::content::LOCKHEART_PAIR)
        );
    }

    /// Adversarial: the loom refuses to walk before the
    /// diagnosis, a fourth sight does not exist, and a
    /// fallen Heart cannot fall again.
    #[test]
    fn floor6_heart_adversarial() {
        let mut expedition = Expedition::new();
        assert!(expedition.floor6_foe_walk_case(0).is_err());
        assert!(expedition.floor6_foe_exhibit(3).is_err());
        assert!(expedition.floor6_foe_confirm_strike("2026-11-10").is_err());
        for exhibit in 0..3 {
            expedition.floor6_foe_exhibit(exhibit).expect("exhibit");
        }
        for case in 0..4 {
            expedition.floor6_foe_walk_case(case).expect("case");
        }
        assert!(expedition.floor6_foe_walk_case(4).is_err());
        expedition.floor6.foe.card_lines = crate::content::FOE6_CARD_KEY;
        expedition.floor6_foe_arm_strike().expect("arm");
        expedition
            .floor6_foe_confirm_strike("2026-11-10")
            .expect("fall");
        assert!(expedition.floor6_foe_confirm_strike("2026-11-10").is_err());
        assert!(expedition.floor6_foe_arm_strike().is_err());
    }
    /// Floor 6 Step 3 state: the Spin Heart judges on the
    /// bound, the honest give-up, and four recorded claims;
    /// the Census on a judged-true tally against the
    /// enumeration.
    #[test]
    fn floor6_second_chambers_judged() {
        let mut expedition = Expedition::new();
        // The spin: bound without a run proves nothing.
        {
            let work = &mut expedition.floor6.chambers[2].work;
            work.watched = true;
            work.spin_bound = 5;
            work.spin_annotations = [1, 1, 1, 1];
        }
        assert!(!expedition.floor6_diagnosed(2));
        expedition.floor6.chambers[2].work.gave_up = true;
        assert!(expedition.floor6_diagnosed(2));
        assert!(expedition.floor6_file_proof(2).is_ok());
        assert!(expedition.floor6_repair_chamber(2).is_ok());
        // The census: judged flags alone do not prove — the
        // tally itself must equal the enumeration.
        {
            let work = &mut expedition.floor6.chambers[3].work;
            work.watched = true;
            work.census = [1, 3, 1];
            work.census_judged = true;
        }
        assert!(expedition.floor6_diagnosed(3));
        expedition.floor6.chambers[3].work.census = [2, 3, 1];
        assert!(
            !expedition.floor6_diagnosed(3),
            "a flattering tally is not the day"
        );
        assert!(expedition.floor6_file_proof(3).is_err());
    }

    /// Adversarial: an unset bound (0) and a bound past the
    /// stepper's reach (10) both fail the spin judge, and
    /// one unrecorded claim fails it too.
    #[test]
    fn floor6_second_chambers_adversarial() {
        let mut expedition = Expedition::new();
        {
            let work = &mut expedition.floor6.chambers[2].work;
            work.watched = true;
            work.gave_up = true;
            work.spin_annotations = [2, 2, 2, 2];
        }
        assert!(!expedition.floor6_diagnosed(2), "bound 0 is unset");
        expedition.floor6.chambers[2].work.spin_bound = 10;
        assert!(!expedition.floor6_diagnosed(2), "the bound is 1..=9");
        expedition.floor6.chambers[2].work.spin_bound = 9;
        expedition.floor6.chambers[2].work.spin_annotations[3] = 0;
        assert!(!expedition.floor6_diagnosed(2), "one claim unrecorded");
        expedition.floor6.chambers[2].work.spin_annotations[3] = 3;
        assert!(expedition.floor6_diagnosed(2));
        // A reset clears the spin and the census alike.
        expedition.floor6_reset_chamber(2);
        let work = &expedition.floor6.chambers[2].work;
        assert_eq!(work.spin_bound, 0);
        assert!(!work.gave_up);
        assert_eq!(work.spin_annotations, [0; 4]);
    }

    /// Floor 7 Step 4 state: the Lock-Free Stack judges on
    /// the referee's searched verdicts plus the named
    /// witness.
    #[test]
    fn floor7_stack_chamber_judged() {
        let mut expedition = Expedition::new();
        {
            let work = &mut expedition.floor7.chambers[4].work;
            work.watched = true;
            work.verdicts = [1, 1, 2, 1, 1, 1];
        }
        // Verdicts true, witness unnamed: not yet.
        assert!(!expedition.floor7_diagnosed(4));
        // Either wrong pair named: still not yet, though
        // the verdicts stand.
        expedition.floor7.chambers[4].work.witness = 1;
        assert!(!expedition.floor7_diagnosed(4));
        expedition.floor7.chambers[4].work.witness = 3;
        assert!(!expedition.floor7_diagnosed(4));
        expedition.floor7.chambers[4].work.witness = 2;
        assert!(expedition.floor7_diagnosed(4));
        assert!(expedition.floor7_file_proof(4).is_ok());
        assert!(expedition.floor7_repair_chamber(4).is_ok());
        expedition.floor7_reset_chamber(4);
        assert_eq!(expedition.floor7.chambers[4].work.verdicts, [0; 6]);
    }

    /// Floor 7's demon at the state level: exhibits in her
    /// order (the chart before its edges, the run after the
    /// chart), the queue walked load by load, the posted
    /// order holding — then the strike's receipt.
    #[test]
    fn floor7_seamstress_demonstration_judged() {
        let mut expedition = Expedition::new();
        // The chart cannot precede the spoken orders, and
        // its edges cannot precede the chart.
        assert!(expedition.floor7_foe_show_exhibit(1).is_err());
        assert!(expedition.floor7_foe_connect_edge(0).is_err());
        expedition
            .floor7_foe_show_exhibit(0)
            .expect("orders spoken");
        expedition.floor7_foe_show_exhibit(1).expect("chart shown");
        // The run refuses an unfinished chart.
        assert!(expedition.floor7_foe_show_exhibit(2).is_err());
        for edge in 0..4 {
            expedition
                .floor7_foe_connect_edge(edge)
                .expect("edge drawn");
        }
        expedition.floor7_foe_show_exhibit(2).expect("deadlock run");
        assert!(expedition.floor7.foe.loom_run);
        // The gate: three loads and the pressure exhibit.
        assert!(!expedition.floor7.foe.gate_passed);
        for load in 0..3 {
            let report = expedition.floor7_foe_walk_load(load).expect("load walked");
            assert!(report.reconciled);
        }
        assert!(!expedition.floor7.foe.gate_passed);
        let (standing, waits) = expedition.floor7_foe_show_pressure().expect("pressure");
        assert_eq!(standing, crate::content::FOE7_QUEUE_CAPACITY);
        assert!(waits);
        assert!(expedition.floor7.foe.gate_passed);
        // The repair: no order posted, no proof, no arm.
        assert!(!expedition.floor7_foe_proof_complete());
        assert!(expedition.floor7_foe_arm_strike().is_err());
        assert!(expedition.floor7_foe_confirm_strike("2026-11-10").is_err());
        for (claimable, rank) in [(0usize, 1u8), (1, 2), (2, 3), (3, 4)] {
            for _ in 0..rank {
                expedition
                    .floor7_foe_cycle_order(claimable)
                    .expect("rank posted");
            }
            assert_eq!(expedition.floor7_foe_order_rank(claimable), rank);
        }
        assert!(expedition.floor7_foe_order_holds());
        assert!(expedition.floor7_foe_proof_complete());
        assert!(!expedition.floor7_complete());
        expedition.floor7_foe_arm_strike().expect("the strike arms");
        expedition
            .floor7_foe_confirm_strike("2026-11-10")
            .expect("she sits");
        assert_eq!(expedition.floor7.foe.state, FoeState::Fallen);
        assert!(expedition.floor7_receipt_standing());
        assert!(
            expedition
                .codex
                .iter()
                .any(|stone| stone == crate::content::LOOMSTONE_TEXT)
        );
        assert!(
            expedition
                .pack
                .iter()
                .any(|loot| loot == "The Threadweaver's Arc & Queue")
        );
        assert!(!expedition.floor7_complete(), "chambers still stand");
    }

    /// Adversarial: a posted order that ranks the stairs
    /// but doubles a rank elsewhere is no total order —
    /// the loom never gets to save it.
    #[test]
    fn floor7_posted_order_must_be_total_in_state() {
        let mut expedition = Expedition::new();
        expedition.floor7.foe.order = [1, 2, 2, 4];
        assert!(!expedition.floor7_foe_order_holds());
        expedition.floor7.foe.order = [3, 4, 1, 2];
        assert!(expedition.floor7_foe_order_holds());
    }
    /// Floor 7 Step 3 state: the Charter Wall judges on the
    /// graded wall, recomputed; Arc and Weak Echoes on the
    /// foretold script, traced by the model.
    #[test]
    fn floor7_second_chambers_judged() {
        let mut expedition = Expedition::new();
        // The wall graded, but not by its own pieces: the
        // stamp alone proves nothing (the judge recomputes).
        {
            let work = &mut expedition.floor7.chambers[2].work;
            work.watched = true;
            work.graded = true;
        }
        assert!(!expedition.floor7_diagnosed(2));
        {
            let work = &mut expedition.floor7.chambers[2].work;
            work.edges = crate::content::WALL_EDGE_KEY;
            work.weak_marked = 3;
            work.stale_found = true;
        }
        assert!(expedition.floor7_diagnosed(2));
        assert!(expedition.floor7_file_proof(2).is_ok());
        assert!(expedition.floor7_repair_chamber(2).is_ok());
        // The book: the model's own trace, judged.
        {
            let work = &mut expedition.floor7.chambers[3].work;
            work.watched = true;
            work.predictions = crate::puzzles::arc_trace();
            work.judged = true;
        }
        assert!(expedition.floor7_diagnosed(3));
        assert!(expedition.floor7_file_proof(3).is_ok());
        assert!(expedition.floor7_repair_chamber(3).is_ok());
    }

    /// Adversarial: Floor 6's provisional annotations are
    /// not this floor's to judge — a player who annotated
    /// "wrong" at the Spin Heart (by Floor 7's key) still
    /// completes the Charter Wall on the wall's own pieces,
    /// and the annotations ride the save untouched (DS-7).
    #[test]
    fn floor7_wall_never_rejudges_the_spin_annotations() {
        let mut expedition = Expedition::new();
        expedition.floor6.chambers[2].work.spin_annotations = [1, 1, 1, 1];
        {
            let work = &mut expedition.floor7.chambers[2].work;
            work.watched = true;
            work.edges = crate::content::WALL_EDGE_KEY;
            work.weak_marked = 3;
            work.stale_found = true;
            work.graded = true;
        }
        assert!(expedition.floor7_diagnosed(2));
        assert_eq!(
            expedition.floor6.chambers[2].work.spin_annotations,
            [1, 1, 1, 1],
            "the wall displays the annotations; it does not touch them"
        );
        expedition.floor7_reset_chamber(2);
        let work = &expedition.floor7.chambers[2].work;
        assert_eq!(work.edges, [false; 5]);
        assert!(!work.graded);
        assert!(!expedition.floor7_diagnosed(2));
    }
    /// Floor 7 Step 2 state: the Scoped Loom judges on the
    /// posted order and the judge's stamp; the Border Seals
    /// on the true seals, judged whole.
    #[test]
    fn floor7_first_chambers_judged() {
        let mut expedition = Expedition::new();
        // A true order, unstamped: not proven.
        {
            let work = &mut expedition.floor7.chambers[0].work;
            work.watched = true;
            work.join_order = [4, 2, 5, 6, 3, 1];
        }
        assert!(!expedition.floor7_diagnosed(0));
        assert!(expedition.floor7_file_proof(0).is_err());
        expedition.floor7.chambers[0].work.scoped = true;
        assert!(expedition.floor7_diagnosed(0));
        assert!(expedition.floor7_file_proof(0).is_ok());
        assert!(expedition.floor7_repair_chamber(0).is_ok());
        // The seals: five true of six is not a set.
        {
            let work = &mut expedition.floor7.chambers[1].work;
            work.watched = true;
            work.seals = [1, 2, 1, 2, 1, 1];
            work.judged = true;
        }
        assert!(
            !expedition.floor7_diagnosed(1),
            "the phantom under SEND fails"
        );
        expedition.floor7.chambers[1].work.seals = [1, 2, 1, 2, 3, 1];
        assert!(expedition.floor7_diagnosed(1));
        assert!(expedition.floor7_file_proof(1).is_ok());
        assert!(expedition.floor7_repair_chamber(1).is_ok());
    }

    /// Adversarial: a stranded early thread under a judge's
    /// stamp still proves nothing — the judge in state
    /// recomputes the order — and a reset clears both
    /// chambers' work.
    #[test]
    fn floor7_first_chambers_adversarial() {
        let mut expedition = Expedition::new();
        {
            let work = &mut expedition.floor7.chambers[0].work;
            work.watched = true;
            work.join_order = [1, 6, 2, 3, 4, 5];
            work.scoped = true;
        }
        assert!(
            !expedition.floor7_diagnosed(0),
            "Bryn stranded is not scoped"
        );
        assert!(expedition.floor7_repair_chamber(0).is_err());
        expedition.floor7_reset_chamber(0);
        let work = &expedition.floor7.chambers[0].work;
        assert_eq!(work.join_order, [0; 6]);
        assert!(!work.scoped);
        {
            let work = &mut expedition.floor7.chambers[1].work;
            work.watched = true;
            work.seals = [1, 2, 1, 2, 3, 1];
            work.judged = true;
        }
        assert!(expedition.floor7_diagnosed(1));
        expedition.floor7_reset_chamber(1);
        assert_eq!(expedition.floor7.chambers[1].work.seals, [0; 6]);
        assert!(!expedition.floor7_diagnosed(1));
    }
    #[test]
    fn version_seven_loads_fresh_hour_and_preserves_prior_records() {
        let mut e = Expedition::new();
        e.floor6.chambers[2].work.spin_annotations = [1, 2, 3, 1];
        let mut value: serde_json::Value = serde_json::from_str(&e.save_json().unwrap()).unwrap();
        value["version"] = serde_json::json!(7);
        value.as_object_mut().unwrap().remove("floor8");
        let loaded = Expedition::load_json(&value.to_string()).unwrap();
        assert_eq!(loaded.floor8, Floor8Progress::new());
        assert_eq!(loaded.floor6, e.floor6);
        assert!(loaded.save_json().unwrap().contains("\"version\":9"));
    }

    fn hour_test_walk_gate(e: &mut Expedition) {
        for i in 0..4 {
            e.floor8.foe.selected = i;
            for event in crate::puzzles::hour::presentation(usize::from(i)) {
                e.floor8_foe_predict(crate::puzzles::hour_event_kind(&event))
                    .unwrap();
            }
        }
        e.floor8.foe.unsequenced = Some(crate::content::HOUR_UNSEQUENCED);
        e.floor8.foe.policy_seen = crate::puzzles::hour::policy_holds();
        e.floor8_refresh_gate();
    }

    #[test]
    fn hour_ledger_repairs_and_loads_with_real_timer_cleanup() {
        let mut e = Expedition::new();
        for (i, (_, state)) in crate::puzzles::hour::ledger().iter().enumerate() {
            for _ in 0..*state {
                e.floor8_ledger_cycle(i).unwrap();
            }
        }
        assert!(e.floor8_diagnosed(4));
        e.floor8_file_proof(4).unwrap();
        e.floor8_repair_chamber(4).unwrap();
        assert!(
            Expedition::load_json(&e.save_json().unwrap())
                .unwrap()
                .floor8_diagnosed(4)
        );
    }
    #[test]
    fn hour_ledger_invalid_index_wrong_lines_and_forgery_fail() {
        let mut e = Expedition::new();
        assert!(e.floor8_ledger_cycle(6).is_err());
        for i in 0..6 {
            e.floor8_ledger_cycle(i).unwrap();
        }
        assert!(e.floor8_file_proof(4).is_err());
        e.floor8.chambers[4].state = ChamberState::Repaired;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
    }
    #[test]
    fn hour_foe_sequenced_pressure_gate_and_proven_strike_load() {
        let mut e = Expedition::new();
        for i in 0..3 {
            e.floor8_foe_exhibit(i).unwrap();
        }
        for _ in 0..5 {
            e.floor8_foe_exhibit(2).unwrap();
        }
        assert_eq!(e.floor8.foe.pressure, 3);
        assert!(e.floor8.foe.exhibits.iter().all(|v| *v));
        e.floor8_foe_prove().unwrap();
        hour_test_walk_gate(&mut e);
        assert!(e.floor8.foe.gate_passed);
        e.floor8.foe.card_lines = [1; 3];
        e.floor8.foe.signal_seal = 1;
        e.floor8.foe.signal_generation = crate::puzzles::hour::living_signal().1 as u8;
        e.floor8_foe_arm_strike().unwrap();
        e.floor8_foe_confirm_strike("2026-11-10").unwrap();
        assert_eq!(e.floor8.foe.state, FoeState::Fallen);
        assert!(e.codex.iter().any(|s| s == crate::content::PINSTONE_TEXT));
        assert!(e.pack.iter().any(|s| s == "The Pinwright's Executor"));
        assert_eq!(e, Expedition::load_json(&e.save_json().unwrap()).unwrap());
        assert!(e.floor8_foe_confirm_strike("2026-11-10").is_err());
    }
    #[test]
    fn hour_foe_refuses_skipped_exhibits_wrong_predictions_and_premature_strikes() {
        let mut e = Expedition::new();
        let before = e.floor8.foe.clone();
        assert!(e.floor8_foe_exhibit(2).unwrap_err().0.contains("wake log"));
        assert_eq!(e.floor8.foe, before);
        assert!(e.floor8_foe_exhibit(3).is_err());
        assert!(e.floor8_foe_prove().is_err());
        assert!(e.floor8_foe_arm_strike().is_err());
        assert!(e.floor8_foe_confirm_strike("today").is_err());
        assert!(e.floor8_foe_predict(9).is_err());
        assert!(e.floor8.foe.predictions[0].is_empty());
        e.floor8.foe.gate_passed = true;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
        e.floor8.foe.gate_passed = false;
        e.floor8.foe.predictions[0].push(9);
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
    }
    #[test]
    fn hour_foe_typed_rewrite_requires_both_identity_and_generation() {
        let mut e = Expedition::new();
        e.floor8.foe.card_lines = [1; 3];
        e.floor8.foe.signal_generation = crate::puzzles::hour::living_signal().1 as u8;
        assert!(!e.floor8_foe_rewrite());
        e.floor8.foe.signal_seal = 1;
        assert!(e.floor8_foe_rewrite());
        e.floor8.foe.signal_generation -= 1;
        assert!(!e.floor8_foe_rewrite());
    }
    #[test]
    fn hour_foe_forged_exposure_strike_and_secondary_are_rejected() {
        let mut e = Expedition::new();
        e.floor8.foe.state = FoeState::Exposed;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
        e.floor8.foe.state = FoeState::Standing;
        e.floor8.foe.strike = StrikePhase::Armed;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
        e.floor8.foe.strike = StrikePhase::Idle;
        hour_test_walk_gate(&mut e);
        e.floor8.foe.unsequenced = Some(0);
        e.floor8_refresh_gate();
        assert!(!e.floor8.foe.gate_passed);
    }

    #[test]
    fn hour_waker_trial_computes_sleep_not_spin_and_loads() {
        let mut e = Expedition::new();
        for part in 1..=4 {
            e.floor8_add_part(part).unwrap();
        }
        e.floor8_sleep_trial().unwrap();
        assert!(e.floor8_diagnosed(2));
        let (sleeps, wakes, polls, spin) = crate::puzzles::hour_sleep_evidence();
        assert_eq!((sleeps, wakes, polls), (2, 2, 3));
        assert!(spin > 100);
        e.floor8_file_proof(2).unwrap();
        e.floor8_repair_chamber(2).unwrap();
        assert!(
            Expedition::load_json(&e.save_json().unwrap())
                .unwrap()
                .floor8_diagnosed(2)
        );
    }

    #[test]
    fn hour_waker_refuses_wrong_part_early_trial_and_forged_trial() {
        let mut e = Expedition::new();
        assert!(e.floor8_add_part(4).is_err());
        assert!(e.floor8_sleep_trial().is_err());
        assert_eq!(e.floor8.chambers[2].work.parts, [0; 4]);
        e.floor8.chambers[2].work.trial_walked = true;
        e.floor8.chambers[2].state = ChamberState::Diagnosed;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
    }

    #[test]
    fn hour_four_errands_produce_twelve_real_heart_events() {
        let mut e = Expedition::new();
        let trace = crate::puzzles::hour_attendant_trace();
        assert_eq!(trace.len(), 12);
        for (i, event) in trace.iter().enumerate() {
            for _ in 0..crate::puzzles::hour_event_kind(event) {
                e.floor8_predict(i).unwrap();
            }
        }
        e.floor8_judge_roster().unwrap();
        assert!(e.floor8_diagnosed(3));
        e.floor8_file_proof(3).unwrap();
        e.floor8_repair_chamber(3).unwrap();
        assert!(
            Expedition::load_json(&e.save_json().unwrap())
                .unwrap()
                .floor8_diagnosed(3)
        );
    }

    #[test]
    fn hour_roster_refuses_wrong_predictions_and_invalidates_edited_proof() {
        let mut e = Expedition::new();
        assert!(e.floor8_judge_roster().is_err());
        assert!(e.floor8_predict(12).is_err());
        for (i, event) in crate::puzzles::hour_attendant_trace().iter().enumerate() {
            for _ in 0..crate::puzzles::hour_event_kind(event) {
                e.floor8_predict(i).unwrap();
            }
        }
        assert!(!e.floor8_diagnosed(3));
        e.floor8_judge_roster().unwrap();
        e.floor8_file_proof(3).unwrap();
        e.floor8_repair_chamber(3).unwrap();
        e.floor8_predict(0).unwrap();
        assert!(!e.floor8_diagnosed(3));
        assert_eq!(e.floor8.chambers[3].state, ChamberState::Unworked);
        assert!(!e.floor8.chambers[3].facet);
    }

    #[test]
    fn hour_countdown_completes_and_roundtrips() {
        let mut e = Expedition::new();
        for report in crate::puzzles::hour_bell_reports() {
            e.floor8_poll(report).unwrap();
        }
        e.floor8_ring().unwrap();
        assert!(e.floor8_diagnosed(0));
        e.floor8_file_proof(0).unwrap();
        e.floor8_repair_chamber(0).unwrap();
        assert!(
            Expedition::load_json(&e.save_json().unwrap())
                .unwrap()
                .floor8_diagnosed(0)
        );
    }

    #[test]
    fn hour_wrong_poll_and_early_duplicate_ring_leave_work_unchanged() {
        let mut e = Expedition::new();
        let before = e.floor8.chambers[0].work.clone();
        assert!(e.floor8_poll(2).is_err());
        assert!(e.floor8_ring().is_err());
        assert_eq!(before, e.floor8.chambers[0].work);
        for report in crate::puzzles::hour_bell_reports() {
            e.floor8_poll(report).unwrap();
        }
        e.floor8_ring().unwrap();
        assert!(e.floor8_ring().is_err());
        assert!(e.floor8_poll(2).is_err());
    }

    #[test]
    fn hour_tapestry_requires_tear_pin_handle_sequence() {
        let mut e = Expedition::new();
        e.floor8_tapestry(false).unwrap();
        e.floor8_tapestry(true).unwrap();
        e.floor8_tapestry(false).unwrap();
        assert!(e.floor8_diagnosed(1));
        e.floor8_file_proof(1).unwrap();
        e.floor8_repair_chamber(1).unwrap();
        assert!(
            Expedition::load_json(&e.save_json().unwrap())
                .unwrap()
                .floor8_diagnosed(1)
        );
    }

    #[test]
    fn hour_tapestry_refuses_pin_first_and_unpinned_second_move() {
        let mut e = Expedition::new();
        assert!(e.floor8_tapestry(true).is_err());
        assert!(!e.floor8_diagnosed(1));
        e.floor8_tapestry(false).unwrap();
        assert!(e.floor8_tapestry(false).is_err());
        e.floor8.chambers[1].work.moved = true;
        assert!(!e.floor8_diagnosed(1));
    }

    #[test]
    fn hour_refuses_forged_proof_and_wrong_chamber_count() {
        let mut e = Expedition::new();
        e.floor8.chambers[0].state = ChamberState::Diagnosed;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
        e.floor8 = Floor8Progress::new();
        e.floor8.chambers.pop();
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
    }

    #[test]
    fn hour_entry_uses_both_prerequisites_and_never_floor_seven() {
        let mut e = Expedition::new();
        assert!(!e.floor8_reachable());
        e.floor2.foe.state = FoeState::Fallen;
        assert!(!e.floor8_reachable());
        for chamber in &mut e.floor6.chambers {
            chamber.state = ChamberState::Repaired;
        }
        e.floor6.foe.state = FoeState::Fallen;
        assert!(e.floor8_reachable());
        assert!(!e.floor7_complete());
        e.floor2.foe.state = FoeState::Standing;
        assert!(!e.floor8_reachable());
    }

    /// Validation: a version-6 save (no floor7 field at all)
    /// loads with a fresh Floor 7, and re-saves as version 7.
    /// Floor 6's records — the spin annotations included —
    /// carry forward unjudged (DS-7).
    #[test]
    fn version_six_saves_load_with_default_floor7() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value.as_object_mut().expect("object").remove("floor7");
        value["version"] = serde_json::json!(6);
        let loaded = Expedition::load_json(&value.to_string()).expect("v6 loads");
        assert_eq!(loaded.floor7, Floor7Progress::new());
        let resaved = loaded.save_json().expect("re-save");
        assert!(resaved.contains("\"version\":9"));
    }

    /// Validation: Floor 7 opens when the Veilworks stands
    /// complete — and the Lockheart, recommended though it
    /// is, is never a gate: Floor 6 stands fresh here.
    #[test]
    fn floor7_reachable_follows_floor5_complete() {
        let mut expedition = Expedition::new();
        assert!(!expedition.floor7_reachable());
        for chamber in expedition.floor5.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        assert!(
            !expedition.floor7_reachable(),
            "the chambers alone do not complete the Veilworks"
        );
        expedition.floor5.foe.state = FoeState::Fallen;
        assert!(expedition.floor5_complete());
        assert!(expedition.floor7_reachable());
        assert_eq!(
            expedition.floor6,
            Floor6Progress::new(),
            "the door does not wait on the Lockheart"
        );
        assert!(!expedition.floor7_complete());
    }

    /// Floor 6 Step 2 state: the Cell Heart judges on the
    /// walked day and the struck bell; the Brazier on five
    /// calls and a single kindling.
    #[test]
    fn floor6_first_chambers_judged() {
        let mut expedition = Expedition::new();
        // The day walked without the bell: not proven.
        {
            let work = &mut expedition.floor6.chambers[0].work;
            work.watched = true;
            work.day_walked = [true; 6];
        }
        assert!(!expedition.floor6_diagnosed(0));
        assert!(expedition.floor6_file_proof(0).is_err());
        // The bell struck: proven, filed, repaired.
        expedition.floor6.chambers[0].work.bell_struck = true;
        assert!(expedition.floor6_diagnosed(0));
        assert!(expedition.floor6_file_proof(0).is_ok());
        assert!(expedition.floor6_repair_chamber(0).is_ok());
        assert_eq!(expedition.floor6.chambers[0].state, ChamberState::Repaired);
        // The brazier: four calls are not five.
        {
            let work = &mut expedition.floor6.chambers[1].work;
            work.watched = true;
            work.calls = [true, true, true, true, false];
        }
        assert!(!expedition.floor6_diagnosed(1));
        expedition.floor6.chambers[1].work.calls = [true; 5];
        assert!(expedition.floor6_diagnosed(1));
        assert!(expedition.floor6_file_proof(1).is_ok());
        assert!(expedition.floor6_repair_chamber(1).is_ok());
    }

    /// Adversarial: the unwatched day proves nothing, and a
    /// reset clears the walked day and the calls alike.
    #[test]
    fn floor6_first_chambers_adversarial() {
        let mut expedition = Expedition::new();
        {
            let work = &mut expedition.floor6.chambers[0].work;
            work.day_walked = [true; 6];
            work.bell_struck = true;
        }
        assert!(!expedition.floor6_diagnosed(0), "the watch is required");
        assert!(expedition.floor6_repair_chamber(0).is_err());
        expedition.floor6_reset_chamber(0);
        let work = &expedition.floor6.chambers[0].work;
        assert_eq!(work.day_walked, [false; 6]);
        assert!(!work.bell_struck);
        {
            let work = &mut expedition.floor6.chambers[1].work;
            work.watched = true;
            work.calls = [true; 5];
        }
        assert!(expedition.floor6_diagnosed(1));
        expedition.floor6_reset_chamber(1);
        assert_eq!(expedition.floor6.chambers[1].work.calls, [false; 5]);
        assert!(!expedition.floor6_diagnosed(1));
    }
    /// Validation: a version-5 save (no floor6 field at all)
    /// loads with a fresh Floor 6, and re-saves as version 6.
    #[test]
    fn version_five_saves_load_with_default_floor6() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value.as_object_mut().expect("object").remove("floor6");
        value["version"] = serde_json::json!(5);
        let loaded = Expedition::load_json(&value.to_string()).expect("v5 loads");
        assert_eq!(loaded.floor6, Floor6Progress::new());
        let resaved = loaded.save_json().expect("re-save");
        assert!(resaved.contains("\"version\":9"));
    }

    /// Validation: Floor 6 opens when the Veilworks stands
    /// charted through its third chamber only — the first
    /// three chambers repaired, whatever the rest of the
    /// floor's state.
    #[test]
    fn floor6_reachable_follows_floor5_third_chamber() {
        let mut expedition = Expedition::new();
        assert!(!expedition.floor6_reachable());
        expedition.floor5.chambers[0].state = ChamberState::Repaired;
        expedition.floor5.chambers[1].state = ChamberState::Repaired;
        assert!(
            !expedition.floor6_reachable(),
            "two of three is not through"
        );
        expedition.floor5.chambers[3].state = ChamberState::Repaired;
        assert!(
            !expedition.floor6_reachable(),
            "the fourth chamber is not the third"
        );
        expedition.floor5.chambers[2].state = ChamberState::Repaired;
        assert!(expedition.floor6_reachable());
        assert!(
            expedition.floor5.foe.state == FoeState::Standing,
            "the door does not wait on the Thing"
        );
        assert!(!expedition.floor6_complete());
    }
    /// Floor 5 Step 2 state: the walk judges on stones
    /// resolved under their cards, and the tenancies on
    /// stamped calls with every miss annotated by its rule.
    #[test]
    fn floor5_walk_and_tenancies_judged() {
        let mut expedition = Expedition::new();
        // Chamber 1: a completed walk, unwatched, does not prove.
        {
            let work = &mut expedition.floor5.chambers[0].work;
            work.walked = 6;
            work.refused = true;
        }
        assert!(!expedition.floor5_diagnosed(0));
        expedition.floor5.chambers[0].work.watched = true;
        assert!(expedition.floor5_diagnosed(0));
        expedition.floor5.chambers[0].work.refused = false;
        assert!(!expedition.floor5_diagnosed(0), "the refusal is the step");
        expedition.floor5.chambers[0].work.refused = true;
        expedition.floor5_file_proof(0).expect("walk proof");
        expedition.floor5_repair_chamber(0).expect("walk repair");
        // Chamber 2: seven true calls, the miss annotated.
        {
            let work = &mut expedition.floor5.chambers[1].work;
            work.watched = true;
            for (index, (_, sound, _)) in crate::content::TENANCIES.iter().enumerate() {
                work.predictions[index] = if *sound { 1 } else { 2 };
            }
            work.predictions[1] = 1; // the repainting writer, called sound
            work.judged = true;
        }
        assert!(!expedition.floor5_diagnosed(1), "the miss is unannotated");
        expedition.floor5.chambers[1].work.annotations[1] = Some(2);
        assert!(expedition.floor5_diagnosed(1));
        expedition.floor5.chambers[1].work.annotations[1] = Some(0);
        assert!(
            !expedition.floor5_diagnosed(1),
            "the wrong rule does not annotate a miss"
        );
    }

    /// Floor 5's UB Hunt, judged at the state level: the
    /// full assignment repairs Chamber 5; any omission
    /// leaves it diagnosed-false.
    #[test]
    fn floor5_hunt_judged() {
        let mut expedition = Expedition::new();
        assert!(!expedition.floor5_diagnosed(4));
        let work = &mut expedition.floor5.chambers[4].work;
        work.lensed = [true; 6];
        work.verdicts = [1, 2, 2, 1, 2, 1];
        work.cards = [0, 2, 0, 0, 1, 0];
        assert!(expedition.floor5_diagnosed(4));
        expedition.floor5.chambers[4].work.cards[2] = 1;
        assert!(!expedition.floor5_diagnosed(4));
    }

    /// The Thing's arc at the state level: exhibits refuse
    /// out of order naming the missing sight; the walk waits
    /// on the diagnosis; the strike waits on proof, gate,
    /// and card in turn; the fall records stone and loot.
    #[test]
    fn floor5_foe_arc() {
        let mut expedition = Expedition::new();
        assert!(expedition.floor5_foe_arm_strike().is_err());
        let refusal = expedition.floor5_foe_exhibit(1).unwrap_err();
        assert!(refusal.0.contains("a live shared loan"), "{}", refusal.0);
        assert!(expedition.floor5_foe_walk_case(0).is_err());
        expedition.floor5_foe_exhibit(0).expect("loan");
        assert_eq!(expedition.floor5.foe.state, FoeState::Exposed);
        expedition.floor5_foe_exhibit(1).expect("growth");
        expedition.floor5_foe_exhibit(2).expect("verdict");
        assert!(expedition.floor5_foe_proof_complete());
        assert!(expedition.floor5_foe_arm_strike().is_err());
        for case in 0..5 {
            expedition.floor5_foe_walk_case(case).expect("case");
        }
        assert!(expedition.floor5.foe.gate_passed);
        assert!(expedition.floor5_foe_arm_strike().is_err());
        expedition.floor5.foe.card = crate::content::FOE5_CARD_KEY;
        assert!(expedition.floor5_foe_repair_ok());
        expedition.floor5_foe_arm_strike().expect("arm");
        expedition
            .floor5_foe_confirm_strike("2026-11-10")
            .expect("confirm");
        assert_eq!(expedition.floor5.foe.state, FoeState::Fallen);
        assert!(
            expedition
                .codex
                .iter()
                .any(|stone| stone == crate::content::ICE_STONE)
        );
        assert!(
            expedition
                .pack
                .iter()
                .any(|loot| loot == crate::content::VEILWALKERS_BUFFER)
        );
        assert!(expedition.floor5_foe_confirm_strike("2026-11-10").is_err());
    }
    /// Floor 5 Step 3 state: the cards judge on all twelve
    /// lines checkable, and the beam on a sound seam with the
    /// referee's walk complete — reach refused, halves
    /// worked, halves dropped in the card's order.
    #[test]
    fn floor5_cards_and_beam_judged() {
        let mut expedition = Expedition::new();
        // Chamber 3: the key set, unwatched, does not prove.
        {
            let work = &mut expedition.floor5.chambers[2].work;
            for (tool, (_, _, key)) in crate::content::CARD_TOOLS.iter().enumerate() {
                work.card_lines[tool] = *key;
            }
        }
        assert!(!expedition.floor5_diagnosed(2));
        expedition.floor5.chambers[2].work.watched = true;
        assert!(expedition.floor5_diagnosed(2));
        expedition.floor5.chambers[2].work.card_lines[1][1] = 0;
        assert!(
            !expedition.floor5_diagnosed(2),
            "one vapour line left standing unmakes the rack"
        );
        expedition.floor5.chambers[2].work.card_lines[1][1] = 2;
        assert!(expedition.floor5_diagnosed(2));
        expedition.floor5_file_proof(2).expect("cards proof");
        expedition.floor5_repair_chamber(2).expect("cards repair");
        // Chamber 4: a sound seam alone is not the walk.
        {
            let work = &mut expedition.floor5.chambers[3].work;
            work.watched = true;
            work.split = 3;
            work.seamed = true;
        }
        assert!(!expedition.floor5_diagnosed(3));
        {
            let work = &mut expedition.floor5.chambers[3].work;
            work.access_refused = true;
            work.worked = [true, true];
            work.dropped = [true, false];
        }
        assert!(
            !expedition.floor5_diagnosed(3),
            "the right half still stands on its trestle"
        );
        expedition.floor5.chambers[3].work.dropped = [true, true];
        assert!(expedition.floor5_diagnosed(3));
        // An end seam unmakes it even with the walk walked:
        // the seam's soundness is part of the proof.
        expedition.floor5.chambers[3].work.split = 0;
        assert!(
            !expedition.floor5_diagnosed(3),
            "no seam stands at the beam's very end"
        );
    }
    /// Floor 4 Step 3 state: the crests judge on measured
    /// sizes, and the treaty wall judges the seating, the
    /// line, and the amended promise together.
    #[test]
    fn floor4_crests_and_treaty_judged() {
        let mut expedition = Expedition::new();
        // Chamber 3: proven losers do not diagnose.
        {
            let work = &mut expedition.floor4.chambers[2].work;
            work.watched = true;
            work.crest_choices = [Some(1), Some(1), Some(2)];
            work.crest_proven = [true; 3];
        }
        assert!(!expedition.floor4_diagnosed(2));
        expedition.floor4.chambers[2].work.crest_choices[0] = Some(2);
        assert!(expedition.floor4_diagnosed(2));
        // An unproven crest unmakes it.
        expedition.floor4.chambers[2].work.crest_proven[1] = false;
        assert!(!expedition.floor4_diagnosed(2));
        expedition.floor4.chambers[2].work.crest_proven[1] = true;
        expedition.floor4_file_proof(2).expect("crest proof");
        expedition.floor4_repair_chamber(2).expect("crest repair");
        // Chamber 4: the promise starts at the book's figure.
        assert_eq!(expedition.floor4.chambers[3].work.final_promise, 10);
        {
            let work = &mut expedition.floor4.chambers[3].work;
            work.watched = true;
            work.treaty_order = vec![0, 1, 2, 4];
            work.treaty_proven = true;
            work.final_promise = 16;
        }
        assert!(
            !expedition.floor4_diagnosed(3),
            "the free twin cannot hold the treaty's last seat"
        );
        expedition.floor4.chambers[3].work.treaty_order = vec![0, 1, 2, 3];
        expedition.floor4.chambers[3].work.final_promise = 10;
        assert!(
            !expedition.floor4_diagnosed(3),
            "the book's naive promise left standing refuses"
        );
        expedition.floor4.chambers[3].work.final_promise = 16;
        assert!(expedition.floor4_diagnosed(3));
        expedition.floor4.chambers[3].work.treaty_proven = false;
        assert!(!expedition.floor4_diagnosed(3));
        expedition.floor4.chambers[3].work.treaty_proven = true;
        expedition.floor4_file_proof(3).expect("treaty proof");
        expedition.floor4_repair_chamber(3).expect("treaty repair");
        assert_eq!(expedition.floor4_chambers_repaired(), 2);
    }

    /// Floor 4 Step 4 state: the hollow stones judge tags
    /// against fillings and the blank against wholeness; the
    /// Foundation's whole arc — re-survey, exhibition, the
    /// two re-cuts, the gate, the strike — holds together,
    /// and each refusal fires where the design puts it.
    #[test]
    fn floor4_hollow_stones_and_foundation_judged() {
        let mut expedition = Expedition::new();
        // Chamber 5: a filling without its tag is not a proof.
        {
            let work = &mut expedition.floor4.chambers[4].work;
            work.watched = true;
            work.fillings = [1, 2];
            work.tags = [1, 0];
            work.blank_cells = [true; 4];
            work.blank_used = true;
        }
        assert!(!expedition.floor4_diagnosed(4), "an untagged ratio refuses");
        expedition.floor4.chambers[4].work.tags = [2, 2];
        assert!(!expedition.floor4_diagnosed(4), "a wrong tag refuses");
        expedition.floor4.chambers[4].work.tags = [1, 2];
        assert!(expedition.floor4_diagnosed(4));
        expedition.floor4.chambers[4].work.blank_used = false;
        assert!(!expedition.floor4_diagnosed(4), "an unused blank refuses");
        expedition.floor4.chambers[4].work.blank_used = true;
        expedition.floor4_file_proof(4).expect("hollow proof");
        expedition.floor4_repair_chamber(4).expect("hollow repair");
        // The FOE refuses its strike before any work, and a
        // measurement before the book is taken.
        assert!(expedition.floor4_foe_arm_strike().is_err());
        assert!(
            expedition.floor4_foe_measure(0).is_err(),
            "the book still hangs in its frame"
        );
        expedition.floor4_foe_take_book();
        assert_eq!(expedition.floor4.foe.state, FoeState::Exposed);
        // An honest entry cannot be exhibited; an unmeasured
        // one cannot either.
        expedition.floor4_foe_measure(0).expect("measure keystone");
        assert!(expedition.floor4_foe_exhibit(0).is_err());
        assert!(expedition.floor4_foe_exhibit(1).is_err());
        for entry in 1..5 {
            expedition.floor4_foe_measure(entry).expect("measure entry");
        }
        assert!(!expedition.floor4_foe_proof_complete());
        for entry in [1, 3, 4] {
            expedition
                .floor4_foe_exhibit(entry)
                .expect("exhibit false entry");
        }
        assert!(expedition.floor4_foe_proof_complete());
        // The gate still sleeps: the re-cut is not laid.
        assert!(expedition.floor4_foe_arm_strike().is_err());
        // The foreign set refuses the shadow-cut stone by its
        // stencil, then stands to its spec.
        expedition.floor4.foe.foreign_order = vec![0, 1, 4, 3];
        assert!(expedition.floor4_foe_prove_foreign().is_err());
        expedition.floor4.foe.foreign_order = vec![0, 2, 1, 3];
        assert!(
            expedition.floor4_foe_prove_foreign().is_err(),
            "the spec's order is part of the spec"
        );
        expedition.floor4.foe.foreign_order = vec![0, 1, 2, 3];
        expedition
            .floor4_foe_prove_foreign()
            .expect("the foreign set stands to its spec");
        assert!(!expedition.floor4.foe.gate_passed);
        // The free set refuses the tray order, then meets its
        // budget in the falling-alignment seating.
        expedition.floor4.foe.free_order = vec![0, 1, 2, 3];
        assert!(expedition.floor4_foe_prove_free().is_err());
        expedition.floor4.foe.free_order = vec![1, 2, 3, 0];
        expedition
            .floor4_foe_prove_free()
            .expect("the free set at its budget");
        assert!(expedition.floor4.foe.gate_passed);
        // The strike lands, the awards change hands, and a
        // second strike is refused.
        expedition.floor4_foe_arm_strike().expect("arm");
        expedition
            .floor4_foe_confirm_strike("2026-11-10")
            .expect("strike lands");
        assert_eq!(expedition.floor4.foe.state, FoeState::Fallen);
        assert!(
            expedition
                .codex
                .iter()
                .any(|stone| stone == crate::content::BRASS_EYE_STONE)
        );
        assert!(
            expedition
                .pack
                .iter()
                .any(|loot| loot == crate::content::PLUMB_LINE)
        );
        assert!(expedition.floor4_foe_confirm_strike("2026-11-10").is_err());
        // Save-compat: a save written before the Foundation's
        // re-survey fields existed loads with them defaulted.
        // (Run on a fresh expedition, so no repaired chamber's
        // stripped work can trip the invariant check.)
        let fresh = Expedition::new();
        let mut value: serde_json::Value =
            serde_json::from_str(&fresh.save_json().expect("save")).expect("save parses");
        let foe = value
            .pointer_mut("/floor4/foe")
            .and_then(serde_json::Value::as_object_mut)
            .expect("the foe record");
        for key in [
            "book_taken",
            "measured",
            "exhibited",
            "foreign_order",
            "foreign_proven",
            "free_order",
            "free_proven",
        ] {
            foe.remove(key);
        }
        let work = value
            .pointer_mut("/floor4/chambers/4/work")
            .and_then(serde_json::Value::as_object_mut)
            .expect("the chamber work record");
        for key in [
            "fillings",
            "tags",
            "blank_cells",
            "blank_used",
            "referee_catches",
        ] {
            work.remove(key);
        }
        let text = serde_json::to_string(&value).expect("re-serialize");
        let loaded = Expedition::load_json(&text).expect("the stripped save loads");
        assert!(!loaded.floor4.foe.book_taken);
        assert!(loaded.floor4.foe.measured.iter().all(|done| !done));
        assert_eq!(loaded.floor4.chambers[4].work.referee_catches, 0);
    }

    /// Floor 4 Chambers 1–2: the bench judge (true slates,
    /// honest surprises annotated truly, a wrong annotation
    /// refused) and the wall judge (the bench order refused,
    /// the least span accepted), on the record.
    #[test]
    fn floor4_bench_and_wall_judged() {
        let mut expedition = Expedition::new();
        // Chamber 1: true slates everywhere, except two
        // honest surprises — the naive sum and the paid tag.
        {
            let work = &mut expedition.floor4.chambers[0].work;
            work.watched = true;
            for stone in 0..crate::content::BENCH_STONES.len() {
                let (size, align) = crate::content::bench_stone_facts(stone);
                work.pred_size[stone] = Some(size);
                work.pred_align[stone] = Some(align);
                work.measured[stone] = true;
            }
            work.pred_size[1] = Some(5);
            work.pred_size[4] = Some(16);
        }
        assert!(!expedition.floor4_diagnosed(0), "surprises unannotated");
        expedition.floor4.chambers[0].work.annotate[1] = Some(crate::content::BENCH_TRUE_CAUSE[1]);
        assert!(!expedition.floor4_diagnosed(0), "one surprise still bare");
        expedition.floor4.chambers[0].work.annotate[4] = Some(crate::content::BENCH_TRUE_CAUSE[4]);
        assert!(expedition.floor4_diagnosed(0));
        // A wrong annotation unmakes it.
        expedition.floor4.chambers[0].work.annotate[4] = Some(0);
        assert!(
            !expedition.floor4_diagnosed(0),
            "the wrong cause is no annotation"
        );
        expedition.floor4.chambers[0].work.annotate[4] = Some(crate::content::BENCH_TRUE_CAUSE[4]);
        expedition.floor4_file_proof(0).expect("proof c1");
        expedition.floor4_repair_chamber(0).expect("repair c1");
        // Chamber 2: the bench order is not the smallest wall.
        {
            let work = &mut expedition.floor4.chambers[1].work;
            work.watched = true;
            work.wall_order = vec![0, 1, 2, 3, 4];
        }
        assert!(
            !expedition.floor4_diagnosed(1),
            "bench order carries its voids"
        );
        expedition.floor4.chambers[1].work.wall_order = vec![1, 2, 4, 3, 0];
        assert!(expedition.floor4_diagnosed(1), "the least span, found");
        expedition.floor4_file_proof(1).expect("proof c2");
        expedition.floor4_repair_chamber(1).expect("repair c2");
        assert_eq!(expedition.floor4_chambers_repaired(), 2);
        // The reset stone restores the bench.
        expedition.floor4_reset_chamber(0);
        assert_eq!(expedition.floor4.chambers[0].state, ChamberState::Unworked);
        assert!(!expedition.floor4.chambers[0].work.measured[0]);
    }

    /// Floor 4 opens only when the Stratum I stairs stand
    /// fully lit (§5.6): FOE 1 fallen, FOE 2 or FOE 3 fallen,
    /// and every Stratum I lantern keeper-lit.
    #[test]
    fn floor4_reachable_needs_the_whole_stairs() {
        let mut expedition = Expedition::new();
        assert!(!expedition.floor4_reachable());
        expedition.foe.state = FoeState::Fallen;
        assert!(
            !expedition.floor4_reachable(),
            "FOE 1 alone is not the stairs"
        );
        expedition.floor2.foe.state = FoeState::Fallen;
        assert!(!expedition.floor4_reachable(), "the lanterns still wait");
        // Lanterns: Floor 1 by its gate, Floor 2 by its
        // chambers, Floor 3 by its gate.
        expedition.foe.gate_passed = true;
        expedition.floor3.foe.gate_passed = true;
        for chamber in expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        assert!(expedition.floor4_reachable());
        assert!(!expedition.floor4_complete());
        assert_eq!(Expedition::new().floor4, Floor4Progress::new());
    }

    /// Validation: Floor 3 opens only when both valley FOEs
    /// have fallen (§5.2).
    #[test]
    fn floor3_reachable_needs_both_valley_foes() {
        let mut expedition = Expedition::new();
        assert!(!expedition.floor3_reachable());
        expedition.foe.state = FoeState::Fallen;
        assert!(!expedition.floor3_reachable(), "Floor 2 still stands");
        expedition.floor2.foe.state = FoeState::Fallen;
        assert!(expedition.floor3_reachable());
    }

    /// Validation: the stairs read Floor 3 — clause 2 lights on
    /// FOE 3 alone, and clause 3's third lantern can be lit by
    /// chambers without the fall (§5.6).
    #[test]
    fn stairs_clauses_read_floor3() {
        let mut expedition = Expedition::new();
        expedition.floor3.foe.state = FoeState::Fallen;
        let clauses = expedition.stairs_clauses();
        assert!(clauses[1].0, "clause 2 lit by FOE 3 alone");
        assert!(clauses[1].1.contains("Gatehouse Release"));
        assert!(!clauses[2].0);
        // Light every lantern: gates for 1 and 3, chambers for 2.
        expedition.foe.gate_passed = true;
        expedition.floor3.foe.gate_passed = true;
        for chamber in &mut expedition.floor2.chambers {
            chamber.state = ChamberState::Repaired;
        }
        let clauses = expedition.stairs_clauses();
        assert!(clauses[2].0, "all three lanterns lit: {}", clauses[2].1);
        assert!(clauses[2].1.contains("Floor 3's lantern is lit"));
    }

    /// Adversarial: a forged Floor 3 record of the wrong size is
    /// refused as corrupt.
    #[test]
    fn load_refuses_forged_floor3_lengths() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value["floor3"]["chambers"]
            .as_array_mut()
            .expect("chambers array")
            .pop();
        match Expedition::load_json(&value.to_string()) {
            Err(SaveError::Corrupt { .. }) => {}
            other => panic!("expected corrupt refusal, got {other:?}"),
        }
    }

    /// Validation: Chamber 1's sort is judged pair by pair —
    /// the Broken bin files nothing, the proof files only when
    /// all four reports stand under their own pairs, and the
    /// repair follows the seal's order.
    #[test]
    fn floor3_chamber1_sort_is_judged() {
        let mut expedition = Expedition::new();
        expedition.floor3.chambers[0].work.watched = true;
        for (report, entry) in crate::content::FAILURE_REPORTS.iter().enumerate() {
            expedition.floor3.chambers[0].work.pair_choices[report] = Some(entry.pair);
        }
        assert!(expedition.floor3_diagnosed(0));
        // One report under the Broken pair unmakes the sort.
        expedition.floor3.chambers[0].work.pair_choices[2] = Some(4);
        assert!(!expedition.floor3_diagnosed(0));
        assert!(expedition.floor3_file_proof(0).is_err());
        expedition.floor3.chambers[0].work.pair_choices[2] =
            Some(crate::content::FAILURE_REPORTS[2].pair);
        expedition.floor3_file_proof(0).expect("proof files");
        assert_eq!(expedition.floor3.chambers[0].state, ChamberState::Diagnosed);
        expedition.floor3_repair_chamber(0).expect("repair lands");
        assert_eq!(expedition.floor3.chambers[0].state, ChamberState::Repaired);
        assert!(expedition.floor3.chambers[0].facet);
    }

    /// Adversarial: the sort unwatched files nothing, however
    /// true the pairs — the charter is a facet, not a garnish.
    #[test]
    fn floor3_chamber1_sort_needs_the_charter() {
        let mut expedition = Expedition::new();
        for (report, entry) in crate::content::FAILURE_REPORTS.iter().enumerate() {
            expedition.floor3.chambers[0].work.pair_choices[report] = Some(entry.pair);
        }
        assert!(!expedition.floor3_diagnosed(0));
        assert!(expedition.floor3_file_proof(0).is_err());
    }

    /// Validation: Chamber 2's belt files only at the one true
    /// setting, and the repair completes the pair of chambers.
    #[test]
    fn floor3_chamber2_belt_is_judged() {
        let mut expedition = Expedition::new();
        {
            let work = &mut expedition.floor3.chambers[1].work;
            work.watched = true;
            work.portion_choice = Some(2);
            work.dedup_choice = Some(0);
            work.halt_choice = Some(1);
        }
        assert!(!expedition.floor3_diagnosed(1), "run-joining is not enough");
        expedition.floor3.chambers[1].work.dedup_choice = Some(1);
        assert!(expedition.floor3_diagnosed(1));
        expedition.floor3_file_proof(1).expect("proof files");
        expedition.floor3_repair_chamber(1).expect("repair lands");
        assert_eq!(expedition.floor3_chambers_repaired(), 1);
    }

    /// Adversarial: a save claiming a Floor 3 proof its work
    /// does not hold is refused — the record is recomputed,
    /// never trusted.
    #[test]
    fn forged_floor3_proof_is_refused() {
        let mut expedition = Expedition::new();
        expedition.floor3.chambers[0].state = ChamberState::Repaired;
        let text = expedition.save_json().expect("save");
        assert!(Expedition::load_json(&text).is_err());
    }

    /// Validation: Chamber 3's audit is judged verdict by
    /// verdict — a standing shout without its invariant files
    /// nothing, and the proof completes only at the true sort.
    #[test]
    fn floor3_chamber3_audit_is_judged() {
        let mut expedition = Expedition::new();
        {
            let work = &mut expedition.floor3.chambers[2].work;
            work.watched = true;
            work.shout_choices = [Some(0), Some(1), Some(0), Some(1)];
            work.invariant_choices = [None, Some(0), None, None];
        }
        assert!(
            !expedition.floor3_diagnosed(2),
            "one shout lacks its invariant"
        );
        expedition.floor3.chambers[2].work.invariant_choices[3] = Some(1);
        assert!(expedition.floor3_diagnosed(2));
        expedition.floor3_file_proof(2).expect("proof files");
        expedition.floor3_repair_chamber(2).expect("repair lands");
        assert_eq!(expedition.floor3.chambers[2].state, ChamberState::Repaired);
    }

    /// Validation and adversarial: Chamber 4's rail enforces
    /// its order in iron — the press refuses an unbodied slip,
    /// the body-less slip cannot advance, the honest slips
    /// cannot be returned, and the true end state files.
    #[test]
    fn floor3_chamber4_rail_enforces_its_order() {
        let mut expedition = Expedition::new();
        expedition.floor3.chambers[3].work.watched = true;
        // The press will not fall on an unaddressed slip.
        assert!(expedition.floor3_slip_act(0, 1).is_err());
        // Nor on an addressed one.
        expedition.floor3_slip_act(0, 0).expect("address it");
        assert!(expedition.floor3_slip_act(0, 1).is_err());
        // Body it, then the seal falls.
        expedition.floor3_slip_act(0, 0).expect("body it");
        expedition.floor3_slip_act(0, 1).expect("seal it");
        // The winter tally arrives addressed.
        expedition.floor3_slip_act(1, 0).expect("body the tally");
        expedition.floor3_slip_act(1, 1).expect("seal the tally");
        // The stranger's slip cannot be bodied, and cannot be
        // sealed from the rail; it goes to the returns tray.
        assert!(expedition.floor3_slip_act(2, 0).is_err());
        assert!(expedition.floor3_slip_act(2, 1).is_err());
        expedition.floor3_slip_act(2, 2).expect("return it");
        // An honest slip cannot be returned.
        assert!(expedition.floor3_slip_act(0, 2).is_err());
        assert!(expedition.floor3_diagnosed(3));
        expedition.floor3_file_proof(3).expect("proof files");
        expedition.floor3_repair_chamber(3).expect("repair lands");
        assert_eq!(expedition.floor3_chambers_repaired(), 1);
    }

    /// Adversarial: a save claiming a Chamber 4 proof with the
    /// rail unrun is refused — the recompute covers it now.
    #[test]
    fn forged_floor3_rail_proof_is_refused() {
        let mut expedition = Expedition::new();
        expedition.floor3.chambers[3].state = ChamberState::Diagnosed;
        let text = expedition.save_json().expect("save");
        assert!(Expedition::load_json(&text).is_err());
    }

    /// Validation: Chamber 5's inspection is judged line by
    /// line — and its repair waits on the amendment rite even
    /// after the proof stands filed.
    #[test]
    fn floor3_chamber5_inspection_and_rite() {
        let mut expedition = Expedition::new();
        {
            let work = &mut expedition.floor3.chambers[4].work;
            work.watched = true;
            for (item, entry) in crate::content::INSPECTION_ITEMS.iter().enumerate() {
                work.checklist_choices[item] = Some(entry.verdict);
            }
            work.na_reason = Some(0);
            work.msrv_choice = Some(1);
        }
        assert!(
            !expedition.floor3_diagnosed(4),
            "the pin is not the oldest road"
        );
        expedition.floor3.chambers[4].work.msrv_choice = Some(0);
        assert!(expedition.floor3_diagnosed(4));
        expedition.floor3_file_proof(4).expect("proof files");
        // The seal refuses while the rite stands undone.
        assert!(expedition.floor3_repair_chamber(4).is_err());
        expedition.floor3_rite_act().expect("read");
        expedition.floor3_rite_act().expect("second");
        expedition.floor3_rite_act().expect("enter");
        assert!(
            expedition.floor3_rite_act().is_err(),
            "the rite does not repeat"
        );
        expedition.floor3_repair_chamber(4).expect("repair lands");
        assert_eq!(expedition.floor3_chambers_repaired(), 1);
    }

    /// Validation: the Release's whole arc at state level —
    /// slate judged and reconciled, stamp, rite, lanterns,
    /// gate, strike — and the awards land.
    #[test]
    fn floor3_foe_arc_completes() {
        let mut expedition = Expedition::new();
        expedition.floor3_foe_take_slate();
        assert_eq!(expedition.floor3.foe.state, FoeState::Exposed);
        expedition.floor3.foe.slate_choices = [Some(1), Some(1), Some(0)];
        expedition.floor3_foe_reconcile().expect("reconcile");
        expedition.floor3.foe.stamp_choice = Some(0);
        expedition.floor3_foe_rite_act().expect("read");
        expedition.floor3_foe_rite_act().expect("second");
        expedition.floor3_foe_rite_act().expect("enter");
        assert!(expedition.floor3_foe_proof_complete());
        // The strike sleeps until the gate passes.
        assert!(expedition.floor3_foe_arm_strike().is_err());
        expedition.floor3_foe_light_lantern(0, Some("formatted".to_string()));
        expedition.floor3_foe_light_lantern(1, None);
        expedition.floor3_foe_mark_tests_passed();
        expedition.floor3_foe_mark_gate_passed();
        assert_eq!(expedition.floor3.foe.lanterns, [true, true, true]);
        // A touch of the draft douses all three.
        expedition.floor3_foe_douse_lanterns();
        assert_eq!(expedition.floor3.foe.lanterns, [false, false, false]);
        assert!(!expedition.floor3.foe.gate_passed);
        expedition.floor3_foe_light_lantern(0, None);
        expedition.floor3_foe_light_lantern(1, None);
        expedition.floor3_foe_mark_tests_passed();
        expedition.floor3_foe_mark_gate_passed();
        expedition.floor3_foe_arm_strike().expect("arm");
        expedition
            .floor3_foe_confirm_strike("2026-11-10")
            .expect("strike lands");
        assert_eq!(expedition.floor3.foe.state, FoeState::Fallen);
        assert!(
            expedition
                .codex
                .iter()
                .any(|s| s == crate::content::LANTERN_STONE)
        );
        assert!(
            expedition
                .pack
                .iter()
                .any(|l| l == crate::content::GATEHOUSE_KEY)
        );
        assert!(!expedition.floor3_complete(), "the chambers still stand");
    }

    /// Adversarial: the slate refuses reconciliation while an
    /// entry is misjudged — the old paper is the tell.
    #[test]
    fn floor3_foe_slate_refuses_a_faithful_reading() {
        let mut expedition = Expedition::new();
        expedition.floor3_foe_take_slate();
        expedition.floor3.foe.slate_choices = [Some(0), Some(0), Some(0)];
        assert!(expedition.floor3_foe_reconcile().is_err());
        assert!(!expedition.floor3_foe_slate_done());
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

    /// Floor 2 Chamber 4 solved as designed: the round driven,
    /// the three bars named in book order.
    fn solve_f2c4(expedition: &mut Expedition) {
        let work = &mut expedition.floor2.chambers[3].work;
        work.drove_round = true;
        work.bar_choices = vec![Some(0), Some(1), Some(2)];
    }

    /// Validation: Chamber 4's proof diagnoses, and the repair
    /// lands at full standard with its facet.
    #[test]
    fn floor2_chamber4_proof_diagnoses_and_repairs() {
        let mut expedition = Expedition::new();
        solve_f2c4(&mut expedition);
        expedition.floor2_file_proof(3).expect("proof");
        assert_eq!(expedition.floor2.chambers[3].state, ChamberState::Diagnosed);
        assert!(expedition.floor2.chambers[3].work.bars_ok);
        expedition.floor2_repair_chamber(3).expect("repair");
        assert_eq!(expedition.floor2.chambers[3].state, ChamberState::Repaired);
        assert!(expedition.floor2.chambers[3].facet);
    }

    /// Adversarial: filing before the round is driven is refused;
    /// an unnamed barred beast is named in the refusal; swapped
    /// bars are refused with the placard's own wording; a forged
    /// out-of-range label is refused without panic.
    #[test]
    fn floor2_chamber4_undriven_and_misnamed_are_refused() {
        let mut expedition = Expedition::new();
        solve_f2c4(&mut expedition);
        expedition.floor2.chambers[3].work.drove_round = false;
        let refusal = expedition.floor2_file_proof(3).expect_err("refused");
        assert!(refusal.message().contains("nothing undriven"));
        assert!(!expedition.floor2.chambers[3].work.bars_ok);
        expedition.floor2.chambers[3].work.drove_round = true;
        expedition.floor2.chambers[3].work.bar_choices = vec![Some(0), None, Some(2)];
        let refusal = expedition.floor2_file_proof(3).expect_err("refused");
        assert!(
            refusal
                .message()
                .contains("Tessell stands barred and unnamed")
        );
        expedition.floor2.chambers[3].work.bar_choices = vec![Some(1), Some(0), Some(2)];
        let refusal = expedition.floor2_file_proof(3).expect_err("refused");
        assert!(refusal.message().contains("does not say Generic method"));
        expedition.floor2.chambers[3].work.bar_choices = vec![Some(usize::MAX), Some(1), Some(2)];
        assert!(expedition.floor2_file_proof(3).is_err());
        assert_eq!(expedition.floor2.chambers[3].state, ChamberState::Unworked);
    }

    /// Floor 2 Chamber 5 solved as designed: the guild collar
    /// route taken, the one-claim reading held to.
    fn solve_f2c5(expedition: &mut Expedition) {
        let work = &mut expedition.floor2.chambers[4].work;
        work.route_choice = Some(1);
        work.collision_choice = Some(2);
    }

    /// Validation: Chamber 5's proof diagnoses, and the repair
    /// lands at full standard with its facet.
    #[test]
    fn floor2_chamber5_proof_diagnoses_and_repairs() {
        let mut expedition = Expedition::new();
        solve_f2c5(&mut expedition);
        expedition.floor2_file_proof(4).expect("proof");
        assert_eq!(expedition.floor2.chambers[4].state, ChamberState::Diagnosed);
        assert!(expedition.floor2.chambers[4].work.route_ok);
        assert!(expedition.floor2.chambers[4].work.collision_ok);
        expedition.floor2_repair_chamber(4).expect("repair");
        assert_eq!(expedition.floor2.chambers[4].state, ChamberState::Repaired);
        assert!(expedition.floor2.chambers[4].facet);
    }

    /// Adversarial: the direct stamp and the blanket joining are
    /// refused with the Sigilwright's authored reasons; a
    /// first-forged reading of the collision is refused; a forged
    /// out-of-range route is refused without panic.
    #[test]
    fn floor2_chamber5_unlawful_routes_and_readings_are_refused() {
        let mut expedition = Expedition::new();
        solve_f2c5(&mut expedition);
        expedition.floor2.chambers[4].work.route_choice = Some(0);
        let refusal = expedition.floor2_file_proof(4).expect_err("refused");
        assert!(refusal.message().contains("Foreign marks on foreign hides"));
        assert!(!expedition.floor2.chambers[4].work.route_ok);
        expedition.floor2.chambers[4].work.route_choice = Some(2);
        let refusal = expedition.floor2_file_proof(4).expect_err("refused");
        assert!(refusal.message().contains("Coherence is not waived"));
        expedition.floor2.chambers[4].work.route_choice = Some(1);
        expedition.floor2.chambers[4].work.collision_choice = Some(0);
        let refusal = expedition.floor2_file_proof(4).expect_err("refused");
        assert!(refusal.message().contains("That reading does not hold"));
        expedition.floor2.chambers[4].work.route_choice = Some(usize::MAX);
        assert!(expedition.floor2_file_proof(4).is_err());
        assert_eq!(expedition.floor2.chambers[4].state, ChamberState::Unworked);
    }

    /// Validation: the Herald's true letters file plainly, the
    /// false letter wastes the miniature expedition, and the
    /// filing stands on the record through a save.
    #[test]
    fn floor2_herald_letters_file_and_the_false_one_wastes_the_ride() {
        let mut expedition = Expedition::new();
        let filed = expedition.floor2_herald_act(0).expect("hearth letter");
        assert_eq!(filed, crate::content::HERALD_FILED);
        let wasted = expedition.floor2_herald_act(1).expect("frost letter");
        assert_eq!(wasted, crate::content::HERALD_WRONG_ROOM);
        assert!(expedition.floor2.herald_filed[1]);
        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert_eq!(loaded.floor2.herald_filed, expedition.floor2.herald_filed);
    }

    /// Adversarial: a filed letter cannot be filed twice, a
    /// letter that does not exist is refused, and a save written
    /// before the beat landed (no herald record) loads with every
    /// letter standing unfiled.
    #[test]
    fn floor2_herald_refuses_repeats_and_loads_old_saves() {
        let mut expedition = Expedition::new();
        expedition.floor2_herald_act(2).expect("valley letter");
        let refusal = expedition.floor2_herald_act(2).expect_err("refused");
        assert!(refusal.message().contains("already stands filed"));
        assert!(expedition.floor2_herald_act(usize::MAX).is_err());
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        value["floor2"]
            .as_object_mut()
            .expect("floor2 object")
            .remove("herald_filed");
        let loaded = Expedition::load_json(&value.to_string()).expect("load old save");
        assert!(loaded.floor2.herald_filed.is_empty());
    }

    /// Validation: the Step 3 work fields round-trip, and a save
    /// written before they existed loads with them defaulted.
    #[test]
    fn floor2_step3_work_round_trips_and_defaults() {
        let mut expedition = Expedition::new();
        solve_f2c4(&mut expedition);
        solve_f2c5(&mut expedition);
        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert_eq!(
            loaded.floor2.chambers[3].work,
            expedition.floor2.chambers[3].work
        );
        assert_eq!(
            loaded.floor2.chambers[4].work,
            expedition.floor2.chambers[4].work
        );
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("json");
        for chamber in [3, 4] {
            let work = value["floor2"]["chambers"][chamber]
                .as_object_mut()
                .expect("chamber object")
                .get_mut("work")
                .and_then(|w| w.as_object_mut())
                .expect("work object");
            for key in [
                "drove_round",
                "bar_choices",
                "bars_ok",
                "route_choice",
                "route_ok",
                "collision_choice",
                "collision_ok",
            ] {
                work.remove(key);
            }
        }
        let loaded = Expedition::load_json(&value.to_string()).expect("load old save");
        assert_eq!(loaded.floor2.chambers[3].work, Floor2Work::default());
        assert_eq!(loaded.floor2.chambers[4].work, Floor2Work::default());
    }

    /// Validation: with all five chambers solved and repaired,
    /// the floor's chamber count stands complete and survives a
    /// save/load round-trip — and two identical playthroughs of
    /// the floor produce byte-identical saves.
    #[test]
    fn floor2_all_chambers_repair_and_playthrough_is_deterministic() {
        fn solve_floor2(expedition: &mut Expedition) {
            solve_f2c1(expedition);
            expedition.floor2_file_proof(0).expect("proof c1");
            expedition.floor2_repair_chamber(0).expect("repair c1");
            solve_f2c2(expedition);
            expedition.floor2_file_proof(1).expect("proof c2");
            expedition.floor2_repair_chamber(1).expect("repair c2");
            solve_f2c3(expedition);
            expedition.floor2_file_proof(2).expect("proof c3");
            expedition.floor2_repair_chamber(2).expect("repair c3");
            solve_f2c4(expedition);
            expedition.floor2_file_proof(3).expect("proof c4");
            expedition.floor2_repair_chamber(3).expect("repair c4");
            solve_f2c5(expedition);
            expedition.floor2_file_proof(4).expect("proof c5");
            expedition.floor2_repair_chamber(4).expect("repair c5");
            for letter in 0..crate::content::HERALD_LETTERS.len() {
                expedition.floor2_herald_act(letter).expect("herald");
            }
        }
        let mut first = Expedition::new();
        solve_floor2(&mut first);
        assert_eq!(first.floor2_chambers_repaired(), 5);
        let first_text = first.save_json().expect("save");
        let loaded = Expedition::load_json(&first_text).expect("load");
        assert_eq!(loaded.floor2_chambers_repaired(), 5);
        let mut second = Expedition::new();
        solve_floor2(&mut second);
        assert_eq!(second.save_json().expect("save"), first_text);
    }

    /// Adversarial: a forged save that stands Floor 2's Step 3
    /// chambers repaired or diagnosed on empty work is refused
    /// as corrupt — the load invariants recompute the new proofs.
    #[test]
    fn load_refuses_forged_floor2_step3_repairs() {
        let expedition = Expedition::new();
        let original_text = expedition.save_json().expect("save");
        for (chamber, state_str) in [(3, "Repaired"), (4, "Diagnosed")] {
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
    /// Solve Floor 2's FOE through the state API: the audit
    /// taken, every admission exhibited, the one disagreement
    /// named, and the Lens calibrated against the brazier.
    fn solve_floor2_foe(expedition: &mut Expedition) {
        expedition.floor2_foe_take_audit();
        for row in 0..crate::content::FOE_ADMISSIONS.len() {
            expedition.floor2_foe_exhibit(row).expect("exhibit");
        }
        let seam = crate::puzzles::foe2_disagreement_row().expect("seam row");
        expedition
            .floor2_foe_name_disagreement(seam)
            .expect("name the seam");
        expedition.floor2_foe_calibrate().expect("calibrate");
    }

    /// Validation: Floor 2's FOE arc — audit, naming, calibration,
    /// gate, strike, awards — in design order, and the fallen
    /// FOE's save loads whole under the recomputed invariants.
    #[test]
    fn floor2_foe_full_arc() {
        let mut expedition = Expedition::new();
        solve_floor2_foe(&mut expedition);
        assert_eq!(expedition.floor2.foe.state, FoeState::Exposed);
        assert!(expedition.floor2.foe.audited.iter().all(|read| *read));
        assert!(expedition.floor2_foe_arm_strike().is_err(), "gate first");
        expedition.floor2_foe_mark_gate_passed();
        expedition.floor2_foe_arm_strike().expect("arm");
        expedition
            .floor2_foe_confirm_strike("2026-11-10")
            .expect("strike lands");
        assert_eq!(expedition.floor2.foe.state, FoeState::Fallen);
        assert!(expedition.codex.iter().any(|s| s == LENS_STONE));
        assert!(expedition.pack.iter().any(|s| s == PLUGIN_MENAGERIE));
        assert_eq!(
            expedition.floor2.foe.fallen_date.as_deref(),
            Some("2026-11-10")
        );
        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert_eq!(loaded, expedition);
    }

    /// Adversarial: the audit refuses out of order — exhibits and
    /// namings before the audit is taken, a sound row named as the
    /// seam, calibration before any naming — and the refused acts
    /// leave the record untouched.
    #[test]
    fn floor2_foe_refuses_out_of_order() {
        let mut expedition = Expedition::new();
        assert!(expedition.floor2_foe_exhibit(0).is_err());
        assert!(expedition.floor2_foe_name_disagreement(3).is_err());
        expedition.floor2_foe_take_audit();
        let refusal = expedition
            .floor2_foe_name_disagreement(0)
            .expect_err("a sound row refuses");
        assert!(refusal.message().contains("agrees with its sigils"));
        assert!(expedition.floor2.foe.disagreement.is_none());
        assert!(expedition.floor2_foe_calibrate().is_err());
        assert!(!expedition.floor2.foe.calibrated);
        assert_eq!(expedition.floor2.foe.state, FoeState::Standing);
    }

    /// Adversarial: a forged save that stands Floor 2's FOE fallen
    /// on an empty audit is refused as corrupt — the load
    /// invariants recompute the FOE's proof, as they do Floor 1's.
    #[test]
    fn load_refuses_forged_floor2_foe() {
        let expedition = Expedition::new();
        let text = expedition.save_json().expect("save");
        let mut value: serde_json::Value = serde_json::from_str(&text).expect("parse json");
        value["floor2"]["foe"]["state"] = "Fallen".into();
        match Expedition::load_json(&value.to_string()) {
            Err(SaveError::Corrupt { .. }) => {}
            other => panic!("expected corrupt error, got: {other:?}"),
        }
    }

    /// Validation: the Desk draft starts as the scaffold, the
    /// player's edits are their own, and the draft survives the
    /// save round trip.
    #[test]
    fn floor2_foe_draft_round_trips() {
        let mut expedition = Expedition::new();
        assert_eq!(
            expedition.floor2_foe_draft(),
            crate::content::GATE2_SCAFFOLD
        );
        expedition
            .floor2_foe_draft_mut()
            .push_str("\n// the bound, re-posted\n");
        let text = expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&text).expect("load");
        assert!(
            loaded
                .floor2_foe_draft()
                .ends_with("the bound, re-posted\n")
        );
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

#[cfg(test)]
mod floor9_scaffolding_tests {
    use super::*;

    #[test]
    fn version_eight_loads_fresh_hoard_and_preserves_earned_records() {
        let earned = include_str!("../tests/fixtures/floor7-earned.json");
        let expedition = Expedition::load_json(earned).unwrap();
        let mut value: serde_json::Value =
            serde_json::from_str(&expedition.save_json().unwrap()).unwrap();
        value["version"] = serde_json::json!(8);
        value.as_object_mut().unwrap().remove("floor9");
        let loaded = Expedition::load_json(&value.to_string()).unwrap();
        assert_eq!(loaded.floor9, Floor9Progress::new());
        assert_eq!(loaded.floor5, expedition.floor5);
        assert_eq!(loaded.floor7, expedition.floor7);
        let resaved: serde_json::Value =
            serde_json::from_str(&loaded.save_json().unwrap()).unwrap();
        assert_eq!(resaved["version"], 9);
        for (key, old) in value.as_object().unwrap() {
            if key != "version" {
                assert_eq!(&resaved[key], old, "old record {key}");
            }
        }
        // Every field newly introduced in v9 accepts omission.
        let mut partial = resaved;
        partial["floor9"] = serde_json::json!({"chambers": [{}, {}, {}, {}, {}], "foe": {}});
        assert_eq!(
            Expedition::load_json(&partial.to_string()).unwrap().floor9,
            Floor9Progress::new()
        );
        partial["floor9"] = serde_json::json!({});
        assert_eq!(
            Expedition::load_json(&partial.to_string()).unwrap().floor9,
            Floor9Progress::new()
        );
    }

    #[test]
    fn floor9_reachability_uses_floor5_without_floor8_gate() {
        let mut expedition =
            Expedition::load_json(include_str!("../tests/fixtures/floor7-earned.json")).unwrap();
        assert!(expedition.floor5_complete());
        assert!(!expedition.floor8_complete());
        assert!(expedition.floor9_reachable());
        expedition.floor9.chambers[0].work.watched = true;
        let loaded = Expedition::load_json(&expedition.save_json().unwrap()).unwrap();
        assert!(loaded.floor9.chambers[0].work.watched);
        assert!(!loaded.floor9_diagnosed(0));
        assert!(!loaded.floor9_complete());
    }

    #[test]
    fn floor9_missing_prerequisite_and_forged_proofs_refused() {
        let mut expedition = Expedition::new();
        expedition.floor8.foe.state = FoeState::Fallen;
        assert!(!expedition.floor9_reachable());
        expedition.floor8 = Floor8Progress::new();
        for state in [ChamberState::Diagnosed, ChamberState::Repaired] {
            expedition.floor9.chambers[0].state = state;
            assert!(expedition.validate_invariants().is_err());
            assert!(Expedition::load_json(&expedition.save_json().unwrap()).is_err());
        }
        expedition.floor9 = Floor9Progress::new();
        expedition.floor9.chambers.pop();
        assert!(expedition.validate_invariants().is_err());
    }

    #[test]
    fn floor9_facets_need_supported_repair() {
        let mut expedition = Expedition::new();
        expedition.floor9.chambers[0].facet = true;
        assert!(expedition.validate_invariants().is_err());
        expedition.floor9.chambers[0].state = ChamberState::Repaired;
        expedition.floor9.chambers[0].hints_used = 4;
        assert!(expedition.validate_invariants().is_err());
        expedition.floor9.chambers[0].hints_used = 3;
        assert!(
            expedition.validate_invariants().is_err(),
            "no Step 1 proof supports even a low-hint repair"
        );
    }
}

#[cfg(test)]
mod floor9_step2_tests {
    use super::*;
    fn shelf(e: &mut Expedition) {
        e.floor9.chambers[0].work.predictions = crate::puzzles::hoard_shelf_trace().0;
        e.floor9.chambers[0].work.judged = true;
    }
    fn chains(e: &mut Expedition) {
        let (slots, nexts) = crate::puzzles::hoard_chain_indices();
        e.floor9.chambers[1].work.slots = slots;
        e.floor9.chambers[1].work.nexts = nexts;
    }
    #[test]
    fn shelf_proof_repair_and_facet() {
        let mut e = Expedition::new();
        shelf(&mut e);
        assert!(e.floor9_diagnosed(0));
        e.floor9_file_proof(0).unwrap();
        assert!(
            e.floor9_repair_chamber(0)
                .unwrap()
                .contains("The Hoardstone")
        );
        assert!(e.floor9.chambers[0].facet);
    }
    #[test]
    fn chains_repair_and_roundtrip() {
        let mut e = Expedition::new();
        shelf(&mut e);
        e.floor9_file_proof(0).unwrap();
        e.floor9_repair_chamber(0).unwrap();
        chains(&mut e);
        e.floor9_file_proof(1).unwrap();
        e.floor9_repair_chamber(1).unwrap();
        assert_eq!(Expedition::load_json(&e.save_json().unwrap()).unwrap(), e);
    }
    #[test]
    fn fourth_hint_repairs_without_facet() {
        let mut e = Expedition::new();
        shelf(&mut e);
        e.floor9.chambers[0].hints_used = 4;
        e.floor9_file_proof(0).unwrap();
        e.floor9_repair_chamber(0).unwrap();
        assert!(!e.floor9.chambers[0].facet);
    }
    #[test]
    fn old_work_defaults_and_hints_name_seams() {
        let w: HoardWork = serde_json::from_str(r#"{"watched":true}"#).unwrap();
        assert!(w.watched);
        assert_eq!(w.predictions, [[0; 2]; 8]);
        assert!(!w.judged);
        for c in 0..2 {
            assert!(crate::content::floor9_chamber_hints(c)[3].contains("seam"));
        }
    }
    #[test]
    fn every_off_by_one_and_unjudged_trace_refused() {
        let mut e = Expedition::new();
        shelf(&mut e);
        e.floor9.chambers[0].work.judged = false;
        assert!(e.floor9_file_proof(0).is_err());
        for row in 0..8 {
            for field in 0..2 {
                shelf(&mut e);
                e.floor9.chambers[0].work.predictions[row][field] += 1;
                assert!(!e.floor9_diagnosed(0));
                assert!(e.floor9_file_proof(0).unwrap_err().0.contains("CAPACITY"));
            }
        }
    }
    #[test]
    fn wrong_shared_tail_and_cycles_refused() {
        let mut e = Expedition::new();
        shelf(&mut e);
        e.floor9_file_proof(0).unwrap();
        e.floor9_repair_chamber(0).unwrap();
        chains(&mut e);
        e.floor9.chambers[1].work.slots[3] = 3;
        assert!(
            e.floor9_file_proof(1)
                .unwrap_err()
                .0
                .contains("shared tail")
        );
        for s in 0..6 {
            chains(&mut e);
            e.floor9.chambers[1].work.nexts[s] = s as u8;
            assert!(!e.floor9_diagnosed(1));
        }
    }
    #[test]
    fn premature_repairs_and_order_bypasses_refused() {
        let mut e = Expedition::new();
        assert!(e.floor9_repair_chamber(0).is_err());
        shelf(&mut e);
        assert!(e.floor9_repair_chamber(0).is_err());
        chains(&mut e);
        assert!(e.floor9_file_proof(1).is_err());
        assert!(e.floor9_repair_chamber(1).is_err());
        e.floor9.chambers[1].state = ChamberState::Diagnosed;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
    }
    #[test]
    fn forged_boundary_proof_and_repeat_repair_refused() {
        let mut e = Expedition::new();
        shelf(&mut e);
        e.floor9_file_proof(0).unwrap();
        e.floor9_repair_chamber(0).unwrap();
        assert!(e.floor9_repair_chamber(0).is_err());
        e.floor9.chambers[0].work.predictions[4][1] -= 1;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
    }
}

#[cfg(test)]
mod floor9_step3_tests {
    use super::*;
    use crate::{content, puzzles};
    fn through_two() -> Expedition {
        let mut e = Expedition::new();
        e.floor9.chambers[0].work.predictions = puzzles::hoard_shelf_trace().0;
        e.floor9.chambers[0].work.judged = true;
        e.floor9_file_proof(0).unwrap();
        e.floor9_repair_chamber(0).unwrap();
        let (slots, nexts) = puzzles::hoard_chain_indices();
        e.floor9.chambers[1].work.slots = slots;
        e.floor9.chambers[1].work.nexts = nexts;
        e.floor9_file_proof(1).unwrap();
        e.floor9_repair_chamber(1).unwrap();
        e
    }
    fn cellar(e: &mut Expedition) {
        let w = &mut e.floor9.chambers[2].work;
        (w.probes, w.flood_cost) = puzzles::hoard_cellar_trace();
        w.judged = true;
    }
    fn through_three() -> Expedition {
        let mut e = through_two();
        cellar(&mut e);
        e.floor9_file_proof(2).unwrap();
        e.floor9_repair_chamber(2).unwrap();
        e
    }
    fn exact(e: &mut Expedition) {
        let w = &mut e.floor9.chambers[3].work;
        w.predicted = puzzles::hoard_steward_run().1;
        w.ran = true;
    }
    #[test]
    fn cellar_exact_repair_and_facet_roundtrip() {
        let mut e = through_two();
        cellar(&mut e);
        e.floor9.chambers[2].hints_used = 3;
        e.floor9_file_proof(2).unwrap();
        assert!(
            e.floor9_repair_chamber(2)
                .unwrap()
                .contains("The Hoardstone")
        );
        assert!(e.floor9.chambers[2].facet);
        assert_eq!(Expedition::load_json(&e.save_json().unwrap()).unwrap(), e);
    }
    #[test]
    fn steward_first_exact_needs_no_miss_flags() {
        let mut e = through_three();
        exact(&mut e);
        assert!(e.floor9_diagnosed(3));
        e.floor9_file_proof(3).unwrap();
        e.floor9_repair_chamber(3).unwrap();
        assert!(
            e.floor9.chambers[3]
                .work
                .misses_explained
                .iter()
                .all(|&v| !v)
        );
        assert_eq!(Expedition::load_json(&e.save_json().unwrap()).unwrap(), e);
    }
    #[test]
    fn steward_accepted_history_is_sticky_and_fourth_hint_loses_facet() {
        let mut e = through_three();
        exact(&mut e);
        e.floor9.chambers[3].work.misses_explained[content::STEWARD_TRAP_OP] = true;
        e.floor9.chambers[3].hints_used = 4;
        e.floor9_file_proof(3).unwrap();
        e.floor9_repair_chamber(3).unwrap();
        assert!(!e.floor9.chambers[3].facet);
        let loaded = Expedition::load_json(&e.save_json().unwrap()).unwrap();
        assert!(loaded.floor9.chambers[3].work.misses_explained[content::STEWARD_TRAP_OP]);
    }
    #[test]
    fn step3_fields_default_and_ladders_name_seam() {
        let w: HoardWork = serde_json::from_str(r#"{"watched":true}"#).unwrap();
        assert_eq!(w.probes, [0; 5]);
        assert_eq!(w.flood_cost, 0);
        assert_eq!(w.predicted, 0);
        assert!(!w.ran);
        assert!(!w.steward_missed);
        assert_eq!(w.misses_explained, [false; 10]);
        for c in 2..4 {
            assert!(content::floor9_chamber_hints(c)[3].contains("seam"));
        }
    }
    #[test]
    fn cellar_off_by_one_flood_undercount_and_premature_proofs_refuse() {
        let mut e = through_two();
        cellar(&mut e);
        e.floor9.chambers[2].work.judged = false;
        assert!(e.floor9_file_proof(2).is_err());
        e.floor9.chambers[2].work.judged = true;
        for i in 0..5 {
            let original = e.floor9.chambers[2].work.probes[i];
            for wrong in [original - 1, original + 1] {
                e.floor9.chambers[2].work.probes[i] = wrong;
                assert!(
                    e.floor9_file_proof(2)
                        .unwrap_err()
                        .0
                        .contains("PROBE TRACE")
                );
            }
            e.floor9.chambers[2].work.probes[i] = original;
        }
        e.floor9.chambers[2].work.flood_cost -= 1;
        assert!(!e.floor9_diagnosed(2));
    }
    #[test]
    fn steward_pending_miss_survives_correction_and_load() {
        let mut e = through_three();
        exact(&mut e);
        e.floor9.chambers[3].work.steward_missed = true;
        assert!(!e.floor9_diagnosed(3));
        assert!(e.floor9_file_proof(3).is_err());
        let loaded = Expedition::load_json(&e.save_json().unwrap()).unwrap();
        assert!(loaded.floor9.chambers[3].work.steward_missed);
        assert!(!loaded.floor9_diagnosed(3));
        e.floor9.chambers[3].work.misses_explained[content::STEWARD_TRAP_OP] = true;
        assert!(!e.floor9_diagnosed(3));
    }
    #[test]
    fn steward_wrong_flag_unrun_and_wrong_total_refuse() {
        let mut e = through_three();
        exact(&mut e);
        e.floor9.chambers[3].work.ran = false;
        assert!(!e.floor9_diagnosed(3));
        e.floor9.chambers[3].work.ran = true;
        e.floor9.chambers[3].work.misses_explained[(content::STEWARD_TRAP_OP + 1) % 10] = true;
        assert!(!e.floor9_diagnosed(3));
        e.floor9.chambers[3].work.misses_explained[content::STEWARD_TRAP_OP] = true;
        let total = puzzles::hoard_steward_run().1;
        for wrong in [total - 1, total + 1] {
            e.floor9.chambers[3].work.predicted = wrong;
            assert!(!e.floor9_diagnosed(3));
        }
    }
    #[test]
    fn step3_order_bypass_and_forged_saved_proofs_refuse() {
        let mut e = through_two();
        cellar(&mut e);
        exact(&mut e);
        assert!(e.floor9_file_proof(3).is_err());
        assert!(e.floor9_repair_chamber(3).is_err());
        e.floor9.chambers[3].state = ChamberState::Diagnosed;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
        let mut e = through_three();
        exact(&mut e);
        e.floor9_file_proof(3).unwrap();
        e.floor9.chambers[3].work.steward_missed = true;
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
        assert!(!e.floor9_diagnosed(4));
    }
}

#[cfg(test)]
pub(crate) mod floor9_step4_tests {
    use super::*;
    use crate::{content, puzzles};
    pub(crate) fn ready() -> Expedition {
        let mut e = Expedition::new();
        e.floor9.chambers[0].work.predictions = puzzles::hoard_shelf_trace().0;
        e.floor9.chambers[0].work.judged = true;
        let (s, n) = puzzles::hoard_chain_indices();
        e.floor9.chambers[1].work.slots = s;
        e.floor9.chambers[1].work.nexts = n;
        let (p, f) = puzzles::hoard_cellar_trace();
        e.floor9.chambers[2].work.probes = p;
        e.floor9.chambers[2].work.flood_cost = f;
        e.floor9.chambers[2].work.judged = true;
        e.floor9.chambers[3].work.predicted = puzzles::hoard_steward_run().1;
        e.floor9.chambers[3].work.ran = true;
        for c in 0..4 {
            e.floor9_file_proof(c).unwrap();
            e.floor9_repair_chamber(c).unwrap();
        }
        e
    }
    fn fifth(e: &mut Expedition) {
        let w = &mut e.floor9.chambers[4].work;
        w.blocks = puzzles::hoard_intern_transcript(true).0;
        w.last_mile = puzzles::hoard_last_mile() as u8;
        w.suite_walked = true;
        e.floor9_file_proof(4).unwrap();
        e.floor9_repair_chamber(4).unwrap();
    }
    fn proof(e: &mut Expedition) {
        fifth(e);
        e.floor9_foe_exhibit(0).unwrap();
        e.floor9.foe.classes = puzzles::hoard_census().0;
        e.floor9_foe_exhibit(1).unwrap();
        e.floor9_foe_exhibit(2).unwrap();
        e.floor9_foe_prove().unwrap();
    }
    fn full(e: &mut Expedition) {
        proof(e);
        e.floor9.foe.trials = [true; 3];
        e.floor9.foe.adversaries = [true; 3];
        e.floor9.foe.gate_passed = e.floor9_foe_gate();
        e.floor9.foe.stamps = [1; 3];
        for i in 0..3 {
            e.floor9_foe_reissue(i).unwrap();
        }
    }
    #[test]
    fn fifth_proof_facet_and_loadback() {
        let mut e = ready();
        e.floor9.chambers[4].hints_used = 3;
        fifth(&mut e);
        assert!(e.floor9.chambers[4].facet);
        assert_eq!(Expedition::load_json(&e.save_json().unwrap()).unwrap(), e);
        assert!(content::floor9_chamber_hints(4)[3].contains("seam"));
    }
    #[test]
    fn ordered_census_gate_and_stamped_repair_fall_once() {
        let mut e = ready();
        full(&mut e);
        e.floor9_foe_arm_strike().unwrap();
        e.floor9_foe_confirm_strike("2026-11-11").unwrap();
        assert!(e.floor9_complete());
        assert_eq!(
            e.codex
                .iter()
                .filter(|s| *s == content::HOARDSTONE_TEXT)
                .count(),
            1
        );
        assert_eq!(
            e.pack
                .iter()
                .filter(|s| s.as_str() == "The Heapwarden's Trio")
                .count(),
            1
        );
        assert_eq!(Expedition::load_json(&e.save_json().unwrap()).unwrap(), e);
        assert!(e.floor9_foe_confirm_strike("again").is_err());
        assert!(e.floor9_foe_arm_strike().is_err());
    }
    #[test]
    fn pressure_is_bounded_and_preserves_identity_proof() {
        let mut e = ready();
        proof(&mut e);
        let census = e.floor9.foe.census.clone();
        for _ in 0..20 {
            e.floor9_foe_exhibit(2).unwrap();
        }
        assert_eq!(e.floor9.foe.pressure, 3);
        assert_eq!(e.floor9.foe.census, census);
        assert!(e.floor9_foe_proof());
        assert_eq!(Expedition::load_json(&e.save_json().unwrap()).unwrap(), e);
    }
    #[test]
    fn older_work_records_default_new_fields() {
        let w: HoardWork = serde_json::from_str("{}").unwrap();
        assert_eq!(w.blocks, [0; 8]);
        assert!(!w.suite_walked);
        let f: Floor9FoeProgress = serde_json::from_str("{}").unwrap();
        assert_eq!(f, Floor9FoeProgress::default());
        let mut e = ready();
        e.floor9.chambers[4].hints_used = 4;
        fifth(&mut e);
        assert!(!e.floor9.chambers[4].facet);
    }
    #[test]
    fn diagnosis_phase_and_each_wrong_block_refuse_final_proof() {
        let mut e = ready();
        fifth(&mut e);
        let valid = e.floor9.chambers[4].work.clone();
        for i in 0..8 {
            e.floor9.chambers[4].work = valid.clone();
            e.floor9.chambers[4].work.blocks[i] = valid.blocks[i].wrapping_add(1);
            assert!(!e.floor9_diagnosed(4));
        }
        e.floor9.chambers[4].work = valid.clone();
        e.floor9.chambers[4].work.blocks = puzzles::hoard_intern_transcript(false).0;
        assert!(e.floor9_file_proof(4).unwrap_err().0.contains("255"));
        e.floor9.chambers[4].work = valid.clone();
        e.floor9.chambers[4].work.suite_walked = false;
        assert!(!e.floor9_diagnosed(4));
        e.floor9.chambers[4].work = valid;
        e.floor9.chambers[4].work.last_mile = 255;
        assert!(!e.floor9_diagnosed(4));
        assert!(Expedition::load_json(&e.save_json().unwrap()).is_err());
    }
    #[test]
    fn out_of_order_wrong_membership_and_premature_strikes_refuse() {
        let mut e = ready();
        assert!(e.floor9_foe_exhibit(0).is_err());
        fifth(&mut e);
        assert!(
            e.floor9_foe_exhibit(2)
                .unwrap_err()
                .0
                .contains("overlap classes")
        );
        assert!(
            e.floor9_foe_exhibit(1)
                .unwrap_err()
                .0
                .contains("three honest tallies")
        );
        assert!(e.floor9_foe_prove().is_err());
        assert!(e.floor9_foe_confirm_strike("x").is_err());
        e.floor9_foe_exhibit(0).unwrap();
        assert!(e.floor9_foe_exhibit(1).is_err());
        e.floor9.foe.classes = puzzles::hoard_census().0;
        for i in 0..6 {
            e.floor9.foe.classes[i] ^= 1;
            assert!(e.floor9_foe_exhibit(1).is_err());
            e.floor9.foe.classes[i] ^= 1;
        }
        assert!(e.floor9_foe_arm_strike().is_err());
        assert!(e.floor9_foe_exhibit(3).is_err());
    }
    #[test]
    fn forged_gate_stamp_census_and_missing_chamber_saves_refuse() {
        let mut e = ready();
        full(&mut e);
        for kind in 0..7 {
            let mut bad = e.clone();
            match kind {
                0 => bad.floor9.foe.gate_passed = false,
                1 => bad.floor9.foe.trials[1] = false,
                2 => bad.floor9.foe.adversaries[2] = false,
                3 => bad.floor9.foe.census = Some(puzzles::Stamped::seal(999, 0)),
                4 => bad.floor9.foe.reissued[0] = Some(puzzles::Stamped::seal(999, 0)),
                5 => bad.floor9.foe.classes[0] ^= 1,
                _ => bad.floor9.chambers[4].state = ChamberState::Unworked,
            }
            assert!(
                Expedition::load_json(&bad.save_json().unwrap()).is_err(),
                "forge {kind}"
            );
        }
        e.floor9.foe.stamps = [0; 3];
        e.floor9.foe.reissued = std::array::from_fn(|_| None);
        for i in 0..3 {
            assert!(e.floor9_foe_reissue(i).unwrap_err().0.contains("BARE"));
            e.floor9.foe.stamps[i] = 2;
            assert!(e.floor9_foe_reissue(i).is_err());
        }
        assert!(e.floor9_foe_reissue(3).is_err());
    }
    #[test]
    fn armed_strike_rechecks_provenance_and_chamber_order() {
        let mut e = ready();
        full(&mut e);
        e.floor9_foe_arm_strike().unwrap();
        e.floor9.foe.reissued[1] = None;
        assert!(e.floor9_foe_confirm_strike("x").is_err());
        assert_eq!(e.floor9.foe.state, FoeState::Exposed);
        let mut e = ready();
        e.floor9.chambers[3].state = ChamberState::Unworked;
        e.floor9.chambers[4].work.suite_walked = true;
        e.floor9.chambers[4].work.last_mile = puzzles::hoard_last_mile() as u8;
        e.floor9.chambers[4].work.blocks = puzzles::hoard_intern_transcript(true).0;
        assert!(e.floor9_file_proof(4).is_err());
        assert!(e.floor9_repair_chamber(4).is_err());
    }
}
