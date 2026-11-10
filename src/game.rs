//! The engine-independent game: screens, overlays, input, layout,
//! and every interaction rule that is not the state machine itself.
//!
//! The shell translates native input into this module's calls
//! (pointer, abstract keys, editor keys), draws from [`layout`],
//! and services [`Game::io`] — the game's only requests to the
//! outside world (save reads/writes, gate runs). Because the rules
//! live here and not in the shell, the same game can be driven by
//! any shell, and its playthrough can be scripted end to end.

use crate::content::{self, CHAMBERS, FOE_CHALLENGE};
use crate::core::SaveError;
use crate::gate::GateReport;
use crate::puzzles;
use crate::state::{ChamberState, Expedition, FoeState, StrikePhase};

/// The design stage's width, in design units.
pub const DESIGN_W: f32 = 1280.0;
/// The design stage's height, in design units.
pub const DESIGN_H: f32 = 800.0;
/// Room grid columns.
pub const ROOM_COLS: i32 = 12;
/// Room grid rows.
pub const ROOM_ROWS: i32 = 9;
/// Room cell size, in design units.
pub const CELL: f32 = 62.0;
/// Room grid origin x.
pub const GRID_X: f32 = 24.0;
/// Room grid origin y.
pub const GRID_Y: f32 = 84.0;
/// Token speed, design units per second.
const MOVE_SPEED: f32 = 460.0;
/// Seconds per cell step under reduced motion.
const STEP_SECONDS: f32 = 0.14;
/// Editor lines shown at once.
pub const EDITOR_VISIBLE_LINES: usize = 26;

/// A rectangle in design coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Rect {
    /// A rectangle from parts.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Horizontal centre.
    pub fn cx(self) -> f32 {
        self.x + self.w / 2.0
    }

    /// Vertical centre.
    pub fn cy(self) -> f32 {
        self.y + self.h / 2.0
    }

    /// Whether the point is inside (edges included).
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
}

/// The room cell's rectangle.
pub fn cell_rect(cx: i32, cy: i32) -> Rect {
    Rect::new(
        GRID_X + cx as f32 * CELL,
        GRID_Y + cy as f32 * CELL,
        CELL,
        CELL,
    )
}

/// Fixed layout rectangles.
pub mod rects {
    use super::Rect;

    /// The header band.
    pub const HEADER: Rect = Rect::new(0.0, 0.0, 1280.0, 64.0);
    /// Header tab: back to the floor hub.
    pub const TAB_FLOOR: Rect = Rect::new(748.0, 10.0, 84.0, 44.0);
    /// Header tab: the map.
    pub const TAB_MAP: Rect = Rect::new(840.0, 10.0, 84.0, 44.0);
    /// Header tab: the log.
    pub const TAB_LOG: Rect = Rect::new(932.0, 10.0, 84.0, 44.0);
    /// Header tab: the codex.
    pub const TAB_CODEX: Rect = Rect::new(1024.0, 10.0, 84.0, 44.0);
    /// Header button: pause & settings.
    pub const PAUSE_BTN: Rect = Rect::new(1116.0, 10.0, 92.0, 44.0);
    /// The room.
    pub const ROOM: Rect = Rect::new(16.0, 76.0, 760.0, 588.0);
    /// The side panel.
    pub const PANEL: Rect = Rect::new(792.0, 76.0, 472.0, 588.0);
    /// The proof slot (kept slot, for Chamber 4).
    pub const SLOT: Rect = Rect::new(808.0, 128.0, 440.0, 56.0);
    /// Chamber 4's second slot (restored).
    pub const SLOT2: Rect = Rect::new(808.0, 188.0, 440.0, 40.0);
    /// The strike seal's bounding square.
    pub const SEAL: Rect = Rect::new(990.0, 514.0, 76.0, 76.0);
    /// Strike confirmation.
    pub const SEAL_CONFIRM: Rect = Rect::new(808.0, 604.0, 200.0, 40.0);
    /// Strike stand-down.
    pub const SEAL_CANCEL: Rect = Rect::new(1016.0, 604.0, 200.0, 40.0);
    /// The bottom bar.
    pub const BOTTOM: Rect = Rect::new(16.0, 672.0, 1248.0, 112.0);
    /// Title panel.
    pub const TITLE_PANEL: Rect = Rect::new(340.0, 110.0, 600.0, 580.0);
    /// Title: begin.
    pub const BEGIN: Rect = Rect::new(490.0, 396.0, 300.0, 48.0);
    /// Title: continue.
    pub const CONTINUE: Rect = Rect::new(490.0, 460.0, 300.0, 48.0);
    /// Reading overlay panel.
    pub const READING: Rect = Rect::new(230.0, 84.0, 820.0, 632.0);
    /// Reading overlay: text scale cycle.
    pub const READING_SCALE: Rect = Rect::new(690.0, 100.0, 200.0, 44.0);
    /// Overlay close button (reading-shaped overlays).
    pub const OVERLAY_CLOSE: Rect = Rect::new(906.0, 100.0, 120.0, 44.0);
    /// Map overlay panel.
    pub const MAP_PANEL: Rect = Rect::new(24.0, 72.0, 1232.0, 656.0);
    /// Map overlay close.
    pub const MAP_CLOSE: Rect = Rect::new(1176.0, 84.0, 56.0, 44.0);
    /// Pause overlay panel.
    pub const PAUSE_PANEL: Rect = Rect::new(390.0, 110.0, 500.0, 560.0);
    /// Editor overlay panel.
    pub const EDITOR_PANEL: Rect = Rect::new(40.0, 48.0, 1200.0, 704.0);
    /// Editor text area.
    pub const EDITOR_AREA: Rect = Rect::new(64.0, 118.0, 1152.0, 520.0);
    /// Editor: run the gate.
    pub const RUN_GATE: Rect = Rect::new(64.0, 660.0, 230.0, 44.0);
    /// Editor: reset the proof (restore the scaffold).
    pub const RESET_PROOF: Rect = Rect::new(310.0, 660.0, 230.0, 44.0);
    /// Editor: open the gate record.
    pub const GATE_RECORD_BTN: Rect = Rect::new(556.0, 660.0, 230.0, 44.0);
    /// Editor: close.
    pub const EDITOR_CLOSE: Rect = Rect::new(1002.0, 660.0, 214.0, 44.0);
    /// Hint reading: climb one rung deeper.
    pub const HINT_DEEPER: Rect = Rect::new(262.0, 648.0, 280.0, 44.0);
    /// Chamber/FOE panel: reset this challenge's proof.
    pub const RESET_BTN: Rect = Rect::new(1096.0, 80.0, 152.0, 30.0);

    /// Verb chip `index` of `count`, along the bottom bar.
    pub fn chip(index: usize, count: usize) -> Rect {
        let count = count.max(1) as f32;
        let width = ((660.0 - (count - 1.0) * 6.0) / count).min(124.0);
        Rect::new(28.0 + index as f32 * (width + 6.0), 688.0, width, 44.0)
    }

    /// Pause/settings row `index`.
    pub fn setting(index: usize) -> Rect {
        Rect::new(415.0, 196.0 + index as f32 * 58.0, 450.0, 46.0)
    }

    /// Title settings-strip button `index` (0 scale, 1 contrast, 2 motion).
    pub fn title_setting(index: usize) -> Rect {
        Rect::new(382.0 + index as f32 * 172.0, 560.0, 164.0, 42.0)
    }
}

/// The top-level screens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    /// The title.
    Title,
    /// Floor 1's hub: camp, doors, stairs.
    Hub,
    /// A chamber, by index (0–4).
    Chamber(usize),
    /// The FOE's chamber.
    Foe,
    /// Floor 2's FOE chamber: the admission desk.
    Floor2Foe,
    /// Floor 2's hub: grounds, doors, roster.
    Floor2Hub,
    /// A Floor 2 chamber, by index (0–4).
    Floor2Chamber(usize),
    /// Floor 3's hub: the release office.
    Floor3Hub,
    /// Floor 4's hub: the Foundation Galleries.
    Floor4Hub,
    /// A Floor 4 chamber.
    Floor4Chamber(usize),
    /// A Floor 3 chamber, by index (0–4).
    Floor3Chamber(usize),
    /// Floor 3's FOE: the Gatehouse Release.
    Floor3Foe,
    /// Floor 4's FOE: the Crooked Foundation.
    Floor4Foe,
    /// Floor 5's hub: the Veilworks.
    Floor5Hub,
    /// A Floor 5 chamber, by index (0-4).
    Floor5Chamber(usize),
    /// Floor 5's FOE: the Thing Behind the Veil.
    Floor5Foe,
    /// Floor 6's hub: the Lockheart.
    Floor6Hub,
    /// A Floor 6 chamber, by index (0-4).
    Floor6Chamber(usize),
    /// Floor 6's FOE: the Heart Locked Twice.
    Floor6Foe,
}

/// What a reading overlay is showing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadingKind {
    /// The chamber's (or FOE's) carved rule.
    Inscription,
    /// The mechanism demonstrated, replayable.
    Demonstration,
    /// One rung of the hint ladder.
    Hint,
    /// The stairs gate's clauses.
    Stairs,
    /// The floor inscription at the camp stone.
    FloorIntro,
    /// The FOE's master plan, raw.
    MasterPlan,
}

/// The overlays, above any screen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Overlay {
    /// No overlay.
    None,
    /// A reading view.
    Reading(ReadingKind),
    /// The Ferronia chart.
    Map,
    /// The expedition log.
    Log,
    /// Pause & settings.
    Pause,
    /// The Principle Codex and the pack.
    Codex,
    /// The editor: the Interner's Hoard draft.
    Editor,
    /// The gate record: the last verdict in full.
    GateRecord,
}

/// The bottom-bar verbs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verb {
    /// Look closely at the room's subject.
    Examine,
    /// Read the carved text.
    Read,
    /// File the chamber's proof.
    Prove,
    /// Enumerate a plan into a census (FOE).
    Enumerate,
    /// Exhibit the proof's key item (FOE).
    Exhibit,
    /// Climb the hint ladder.
    Hint,
    /// Open the map.
    Map,
}

impl Verb {
    /// The chip's label.
    pub fn label(self) -> &'static str {
        match self {
            Verb::Examine => "Examine · E",
            Verb::Read => "Read · R",
            Verb::Prove => "Prove · X",
            Verb::Enumerate => "Enumerate · N",
            Verb::Exhibit => "Exhibit · X",
            Verb::Hint => "Hint · H",
            Verb::Map => "Map · M",
        }
    }
}

/// An interactive element. Layout produces these with rectangles;
/// pointer and keyboard both activate them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum El {
    /// Title: begin a fresh expedition.
    Begin,
    /// Title: continue from the save.
    Continue,
    /// A settings row (pause) or strip button (title), by index.
    Setting(usize),
    /// Header: to the floor hub.
    TabFloor,
    /// Header: the map.
    TabMap,
    /// Header: the log.
    TabLog,
    /// Header: the codex.
    TabCodex,
    /// Header: pause.
    PauseBtn,
    /// Hub: a chamber door, by index.
    DoorChamber(u8),
    /// Hub: the FOE's door.
    DoorFoe,
    /// Floor 2 hub: the FOE's door.
    DoorFoe2,
    /// Hub: the stairs door.
    DoorStairs,
    /// Hub: the Floor 2 door.
    DoorFloor2,
    /// Floor 2 hub: the Floor 3 door.
    DoorFloor3,
    /// Floor 3 hub: a chamber door, by index.
    Floor3Door(u8),
    /// The door to Floor 4, on Floor 1's hub.
    DoorFloor4,
    /// A Floor 4 chamber door.
    Floor4Door(u8),
    /// The door to Floor 5, on Floor 4's hub.
    DoorFloor5,
    /// A Floor 5 chamber door.
    Floor5Door(u8),
    /// The door to Floor 6, on Floor 5's hub.
    DoorFloor6,
    /// A Floor 6 chamber door.
    Floor6Door(u8),
    /// Floor 2 hub: a chamber door, by index.
    Floor2Door(u8),
    /// Floor 2 hub: collapse or expand a roster group, by index.
    RosterToggle(u8),
    /// Floor 2 Chamber 1: watch both beasts perform the call.
    F2Watch,
    /// Floor 3 chambers: read the desk's charter.
    F3Watch,
    /// Floor 4: read the chamber's charter (the watch).
    F4Watch,
    /// Floor 5: watch the demonstration.
    F5Watch,
    /// Floor 6: watch the demonstration.
    F6Watch,
    /// Floor 6 Chamber 1: walk one operation of the day.
    F6DayOp(u8),
    /// Floor 6 Chamber 2: make one call on the brazier.
    F6BrazierCall(u8),
    /// Floor 6 Chamber 3: raise the spin bound.
    F6BoundUp,
    /// Floor 6 Chamber 3: lower the spin bound.
    F6BoundDown,
    /// Floor 6 Chamber 3: run the heart at its bound.
    F6SpinRun,
    /// Floor 6 Chamber 3: cycle one claim's annotation.
    F6Annotate(u8),
    /// Floor 6 Chamber 4: count one more state of a class.
    F6CensusCycle(u8),
    /// Floor 6 Chamber 4: submit the census to the slate.
    F6CensusSubmit,
    /// Floor 6 Chamber 5: house one datum in the next kind.
    F6Choose(u8),
    /// Floor 6 Chamber 5: write one datum's door sentence.
    F6Invariant(u8),
    /// Floor 6's hub: the FOE's door.
    DoorFoe6,
    /// The FOE: exhibit one sight of the staff.
    F6Exhibit(u8),
    /// The FOE: walk one of the loom's four cases.
    F6Loom(u8),
    /// The FOE: cycle one line of the staff's card.
    F6FoeCard(u8),
    /// Floor 5 Chamber 1: read one step's card.
    F5Card(u8),
    /// Floor 5 Chamber 1: walk the next step.
    F5Walk,
    /// Floor 5 Chamber 1: refuse the next step by name.
    F5RefuseStep,
    /// Floor 5 Chamber 2: cycle one placard's call.
    F5Placard(u8),
    /// Floor 5 Chamber 2: call the referee.
    F5Judge,
    /// Floor 5 Chamber 2: cycle one miss's annotation.
    F5Annotate(u8),
    /// Floor 5 Chamber 3: cycle one line of one tool's card.
    F5CardLine(u8, u8),
    /// Floor 5 Chamber 4: move the seam one word left.
    F5SplitDown,
    /// Floor 5 Chamber 4: move the seam one word right.
    F5SplitUp,
    /// Floor 5 Chamber 4: saw the seam at the chosen split.
    F5Seam,
    /// Floor 5 Chamber 4: reach past a half's end.
    F5Reach,
    /// Floor 5 Chamber 4: work both halves at once.
    F5WorkHalves,
    /// Floor 5 Chamber 4: drop the left half.
    F5DropLeft,
    /// Floor 5 Chamber 4: drop the right half.
    F5DropRight,
    /// Chamber 5: pass the lens over one station.
    F5Lens(u8),
    /// Chamber 5: cycle one station's verdict.
    F5Verdict(u8),
    /// Chamber 5: cycle one faulted station's card.
    F5FixCard(u8),
    /// The FOE: exhibit one sight of the trace.
    F5Exhibit(u8),
    /// The FOE: walk one of the referee's five cases.
    F5Case(u8),
    /// The FOE: cycle one line of the beam's rewritten card.
    F5FoeCard(u8),
    /// Floor 5's hub: the FOE's door.
    DoorFoe5,
    /// Floor 4 Chamber 1: put this stone on the slate.
    F4Stone(u8),
    /// Floor 4 Chamber 1: step the slate's size down.
    F4SizeDown,
    /// Floor 4 Chamber 1: step the slate's size up.
    F4SizeUp,
    /// Floor 4 Chamber 1: step the slate's alignment down.
    F4AlignDown,
    /// Floor 4 Chamber 1: step the slate's alignment up.
    F4AlignUp,
    /// Floor 4 Chamber 1: drop the plumb line.
    F4DropLine,
    /// Floor 4 Chamber 1: cycle the surprise's annotation.
    F4Annotate,
    /// Floor 4 Chamber 2: seat this stone next in the wall.
    F4Seat(u8),
    /// Floor 4 Chamber 2: clear the wall back to the bench.
    F4WallReset,
    /// Floor 4 Chamber 3: cycle one crest's cut.
    F4Crest(u8),
    /// Floor 4 Chamber 3: let the line measure one crest's cut.
    F4CrestProve(u8),
    /// Floor 4 Chamber 4: seat one stone next in the treaty wall.
    F4TreatySeat(u8),
    /// Floor 4 Chamber 4: take the treaty wall down.
    F4TreatyReset,
    /// Floor 4 Chamber 4: drop the line on the treaty wall.
    F4TreatyProve,
    /// Floor 4 Chamber 4: raise the book's final promise.
    F4PromiseUp,
    /// Floor 4 Chamber 4: lower the book's final promise.
    F4PromiseDown,
    /// Floor 4 Chamber 5: cycle one hollow stone's filling.
    F4Fill(u8),
    /// Floor 4 Chamber 5: cycle one hollow stone's tag.
    F4Tag(u8),
    /// Floor 4 Chamber 5: read one hollow stone as tagged.
    F4Read(u8),
    /// Floor 4 Chamber 5: fill one cell of the blank stone.
    F4Cell(u8),
    /// Floor 4 Chamber 5: use the blank stone.
    F4UseBlank,
    /// Floor 4's FOE: take the dimension book down.
    F4BookTake,
    /// Floor 4's FOE: drop the line on one book entry.
    F4Measure(u8),
    /// Floor 4's FOE: exhibit one entry beside its truth.
    F4Exhibit(u8),
    /// Floor 4's FOE: seat one foreign stone in the re-cut.
    F4ForeignSeat(u8),
    /// Floor 4's FOE: take the foreign set down.
    F4ForeignReset,
    /// Floor 4's FOE: drop the line on the foreign set.
    F4ForeignProve,
    /// Floor 4's FOE: seat one free stone in the re-cut.
    F4FreeSeat(u8),
    /// Floor 4's FOE: take the free set down.
    F4FreeReset,
    /// Floor 4's FOE: drop the line on the free set.
    F4FreeProve,
    /// Floor 3 Chamber 1: cycle one report's pair, by report.
    F3Pair(u8),
    /// Floor 3 Chamber 2: cycle one station's setting, by slot
    /// (0 portioner, 1 joiner, 2 gate).
    F3Station(u8),
    /// Floor 3 Chamber 3: cycle one ledger entry's verdict.
    F3Shout(u8),
    /// Floor 3 Chamber 3: cycle one entry's invariant.
    F3Invariant(u8),
    /// Floor 3 Chamber 4: work one slip (slip, action:
    /// 0 advance, 1 seal, 2 return).
    F3Slip(u8, u8),
    /// Floor 3 hub: the FOE's door.
    DoorFoe3,
    /// Floor 4 hub: the FOE's door.
    DoorFoe4,
    /// Floor 3 Chamber 5: cycle one line's verdict, by item.
    F3Check(u8),
    /// Floor 3 Chamber 5: cycle the set-aside line's reason.
    F3NaReason,
    /// Floor 3 Chamber 5: cycle the oldest-road stamp.
    F3Msrv,
    /// Floor 3 Chamber 5 and the Release: perform the rite's
    /// next act.
    F3Rite,
    /// The Release: take up the log-slate.
    F3SlateTake,
    /// The Release: cycle one slate entry's verdict.
    F3SlateEntry(u8),
    /// The Release: reconcile the slate.
    F3Reconcile,
    /// The Release: cycle the stamp.
    F3Stamp,
    /// The Release: light a lantern (0 format, 1 lint).
    F3Lantern(u8),
    /// Floor 2 Chamber 1: strike a charter candidate, by index.
    F2Charter(u8),
    /// Floor 2 Chamber 1: name the blind run's body, by performer index.
    F2RunChoice(u8),
    /// Floor 2 Chamber 2: toggle one sigil in one service's bound
    /// (service index, sigil bit index).
    F2BoundToggle(u8, u8),
    /// Floor 2 Chamber 3: run both kennels' deliveries.
    F2RunDeliveries,
    /// Floor 2 Chamber 3: name the contorted errand, by index.
    F2Errand(u8),
    /// Floor 2 Chamber 3: name the kennel that runs it naturally.
    F2Kennel(u8),
    /// Floor 2 Chamber 4: drive the Dawn Round through the Wide Door.
    F2DriveRound,
    /// Floor 2 Chamber 4: name one barred beast's bar (the barred
    /// beast's position in book order, the bar's label index).
    F2BarLabel(u8, u8),
    /// Floor 2 Chamber 5: take one joining route, by index.
    F2Route(u8),
    /// Floor 2 Chamber 5: hold to one reading of the collision, by index.
    F2Reading(u8),
    /// Floor 2 Chamber 5: the panel's sub-view (0 joining, 1 collision).
    F2PanelTab(u8),
    /// Floor 2 hub: act on one of the Herald's letters, by index.
    F2HeraldAct(u8),
    /// Floor 2 FOE: exhibit one admission's sigils, by audit row.
    F2FoeExhibit(u8),
    /// Floor 2 FOE: name one admission as the disagreement, by
    /// audit row.
    F2FoeName(u8),
    /// Floor 2 FOE: calibrate the Lens against the lit brazier.
    F2FoeCalibrate,
    /// Hub: the camp stone.
    CampStone,
    /// The room's subject (desk / engine / chart).
    Desk,
    /// The inscription stone.
    Stone,
    /// A bottom-bar verb chip.
    Chip(Verb),
    /// The strike seal.
    Seal,
    /// Confirm an armed strike.
    SealConfirm,
    /// Stand an armed strike down.
    SealCancel,
    /// C1: toggle a porter for an item (item, porter).
    HolderToggle(u8, u8),
    /// C1: the panel's sub-view (0 packing, 1 toll).
    PanelTab(u8),
    /// C1: file a toll candidate.
    TollChoice(u8),
    /// C2: cycle a person's block.
    PickToggle(u8),
    /// C3: step a lease field (lease, field 0=start/1=end, dir).
    LeaseStep(u8, u8, i8),
    /// C4: run the delivery.
    RunDelivery,
    /// C4: toggle committing the second room.
    CommitToggle,
    /// C4: select a record card.
    RecordCard(u8),
    /// C4: exhibit the selected record into a slot (0 kept, 1 restored).
    ProofSlot(u8),
    /// C5: cycle a placard's judgement.
    JudgeToggle(u8),
    /// C5: previous placard page.
    PagePrev,
    /// C5: next placard page.
    PageNext,
    /// FOE: verify a census row.
    VerifyRow(u8),
    /// FOE: click a census row (exhibit attempt).
    CensusCard(u8),
    /// FOE: open the editor.
    EditorOpen,
    /// Chamber/FOE: reset this challenge's proof to its authored
    /// start. Always exists, so no state is unsolvable.
    ResetChamber,
    /// Editor: the text area (focus).
    EditorArea,
    /// Editor: run the gate.
    RunGate,
    /// Editor: restore the scaffold.
    ResetProof,
    /// Editor: open the gate record.
    GateRecordBtn,
    /// Hint overlay: climb one rung deeper.
    HintDeeper,
    /// Reading overlay: text scale.
    ReadingScale,
    /// Overlay: close.
    OverlayClose,
    /// Pause: export the save (write it, announced).
    Export,
    /// Pause: import the save (read it).
    Import,
    /// Pause: resume.
    Resume,
}

/// Abstract keys, translated by the shell from native key events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    /// Move up.
    Up,
    /// Move down.
    Down,
    /// Move left.
    Left,
    /// Move right.
    Right,
    /// Examine verb.
    Examine,
    /// Read verb.
    Read,
    /// Enumerate verb.
    Enumerate,
    /// Prove / exhibit verb.
    Exhibit,
    /// Hint verb.
    Hint,
    /// Map overlay.
    Map,
    /// Log overlay.
    Log,
    /// Codex overlay.
    Codex,
    /// Cycle keyboard focus.
    Tab,
    /// Activate the focused element.
    Confirm,
    /// Close / stand down.
    Cancel,
}

/// The status line's tone. Every tone also carries a shape in the
/// shell's rendering — never colour alone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusKind {
    /// Plain information.
    Info,
    /// A refusal, with its reason.
    Refusal,
    /// Something proven or earned.
    Good,
}

/// A request from the game to the outside world, serviced by the
/// shell. One at a time; the game polls its own field each frame.
#[derive(Clone, Debug)]
pub enum IoRequest {
    /// Write the save file.
    WriteSave {
        /// Narrate the write on completion.
        announce: bool,
    },
    /// Read the save file and hand back the result.
    ReadSave,
    /// Run a FOE gate on this code.
    RunGate {
        /// The player's submitted draft.
        code: String,
        /// Which floor's gate judges it: Floor 2's Desk when
        /// true, Floor 1's Hoard when false.
        floor2: bool,
        /// Floor 3's Release gate when true (it outranks the
        /// Desk flag; a draft belongs to one FOE at a time).
        floor3: bool,
    },
    /// Light one of the Release's lanterns on this code
    /// (kind 0 format, 1 lint).
    RunLantern {
        /// Which lantern.
        kind: u8,
        /// The draft as it stands.
        code: String,
    },
}

/// What an armed seal will strike.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArmedFor {
    /// A chamber's repair.
    Chamber(usize),
    /// A Floor 2 chamber's repair.
    Floor2Chamber(usize),
    /// A Floor 3 chamber's repair.
    Floor3Chamber(usize),
    /// A Floor 4 chamber's repair.
    Floor4Chamber(usize),
    /// A Floor 5 chamber's repair.
    Floor5Chamber(usize),
    /// A Floor 6 chamber's repair.
    Floor6Chamber(usize),
    /// Floor 3 FOE's strike.
    Foe3,
    /// Floor 4 FOE's strike.
    Foe4,
    /// Floor 5 FOE's strike.
    Foe5,
    /// Floor 6 FOE's strike.
    Foe6,
    /// The FOE's strike.
    Foe,
    /// Floor 2 FOE's strike.
    Foe2,
}

/// The game. Owns the expedition and every transient interaction
/// state the save does not record.
pub struct Game {
    /// The campaign state (the save's contents).
    pub expedition: Expedition,
    /// The current screen.
    pub screen: Screen,
    /// The current overlay.
    pub overlay: Overlay,
    /// The status line's text.
    pub status: String,
    /// The status line's tone.
    pub status_kind: StatusKind,
    /// A save exists on this platform.
    pub save_present: bool,
    /// The pending IO request, if any.
    pub io: Option<IoRequest>,
    /// Today's date (`YYYY-MM-DD`), supplied by the shell; used
    /// only to inscribe the chart when the FOE falls.
    pub today: String,
    /// A gate run is in flight.
    pub gate_running: bool,
    /// The gate run judges Floor 2's Desk (set when the run is
    /// filed; read when its verdict returns).
    pub gate_floor2: bool,
    /// The gate in flight judges Floor 3's Release.
    pub gate_floor3: bool,
    /// Floor 4 Chamber 1: the stone on the slate (ephemeral).
    pub f4_stone: u8,
    /// The last gate report's detail (the record view's body).
    pub gate_detail: String,
    /// The hint rung currently shown in the hint reading.
    pub hint_showing: u8,
    /// The editor's cursor, a byte index into the draft.
    pub editor_cursor: usize,
    /// The editor's selection anchor, when a selection stands.
    pub editor_anchor: Option<usize>,
    /// The editor's first visible line.
    pub editor_scroll: usize,
    /// The delver token's cell.
    player_cell: (i32, i32),
    /// The delver token's pixel position (top-left of its disc).
    player_px: (f32, f32),
    /// The cell the token is walking to, if any.
    walk_dest: Option<(i32, i32)>,
    /// Reduced-motion step timer.
    step_timer: f32,
    /// The element the pointer went down on.
    pointer_hit: Option<El>,
    /// Whether the pointer is currently down.
    pub pointer_down: bool,
    /// The keyboard-focused element.
    focus: Option<El>,
    /// The chamber arm of the seal, when armed for a repair.
    armed: Option<ArmedFor>,
    /// C1's panel sub-view (0 packing, 1 toll).
    c1_view: u8,
    /// Floor 2 C5's panel sub-view (0 joining, 1 collision).
    f2c5_view: u8,
    /// C5's placard page (0 or 1).
    c5_page: u8,
    /// C4's selected record card.
    selected_record: Option<usize>,
    /// Visible-state revision; the shell rebuilds when it moves.
    revision: u64,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// A new game at the title screen.
    pub fn new() -> Self {
        Self {
            expedition: Expedition::new(),
            screen: Screen::Title,
            overlay: Overlay::None,
            status: "The descent waits.".to_string(),
            status_kind: StatusKind::Info,
            save_present: false,
            io: None,
            today: String::new(),
            gate_running: false,
            gate_floor2: false,
            gate_floor3: false,
            f4_stone: 0,
            gate_detail: String::new(),
            hint_showing: 0,
            editor_cursor: 0,
            editor_anchor: None,
            editor_scroll: 0,
            player_cell: (5, 7),
            player_px: cell_center(5, 7),
            walk_dest: None,
            step_timer: 0.0,
            pointer_hit: None,
            pointer_down: false,
            focus: None,
            armed: None,
            c1_view: 0,
            f2c5_view: 0,
            c5_page: 0,
            selected_record: None,
            revision: 1,
        }
    }

    /// The visible-state revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Mark a visual change (no save implied).
    fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    /// Mark a state change: visual change + autosave, per the
    /// design's save discipline (on every state change).
    fn changed(&mut self) {
        self.bump();
        if self.io.is_none() {
            self.io = Some(IoRequest::WriteSave { announce: false });
        }
    }

    /// Set the status line.
    fn say(&mut self, kind: StatusKind, text: impl Into<String>) {
        self.status = text.into();
        self.status_kind = kind;
        self.bump();
    }

    /// The token's pixel position (its disc's centre).
    pub fn player_px(&self) -> (f32, f32) {
        self.player_px
    }

    /// Whether the token is visible on this screen.
    pub fn token_visible(&self) -> bool {
        matches!(
            self.screen,
            Screen::Hub
                | Screen::Floor2Hub
                | Screen::Floor3Hub
                | Screen::Floor4Hub
                | Screen::Floor4Chamber(_)
                | Screen::Floor5Hub
                | Screen::Floor5Chamber(_)
                | Screen::Floor6Hub
                | Screen::Floor6Chamber(_)
                | Screen::Chamber(_)
                | Screen::Floor2Chamber(_)
                | Screen::Floor3Chamber(_)
                | Screen::Floor3Foe
                | Screen::Floor4Foe
                | Screen::Floor5Foe
                | Screen::Floor6Foe
                | Screen::Foe
                | Screen::Floor2Foe
        )
    }

    /// The keyboard-focused element, if any.
    pub fn focused_element(&self) -> Option<El> {
        self.focus
    }

    /// Flip a Floor 2 roster group's collapsed state; the shell
    /// draws the board from `expedition.floor2.roster_collapsed`.
    pub fn toggle_roster_group(&mut self, group: usize) {
        if let Some(flag) = self.expedition.floor2.roster_collapsed.get_mut(group) {
            *flag = !*flag;
        }
        self.changed();
    }
    /// Flip a Floor 2 roster group's collapsed state; the shell
    /// draws the board from `expedition.floor2.roster_collapsed`.
    pub fn c1_view(&self) -> u8 {
        self.c1_view
    }

    /// Floor 2 C5's panel sub-view, for the shell.
    pub fn f2c5_view(&self) -> u8 {
        self.f2c5_view
    }

    /// C5's placard page, for the shell.
    pub fn c5_page(&self) -> u8 {
        self.c5_page
    }

    /// C4's selected record card, for the shell.
    pub fn selected_record(&self) -> Option<usize> {
        self.selected_record
    }

    /// Whether the seal stands armed on the current screen.
    pub fn seal_armed(&self) -> bool {
        match self.screen {
            Screen::Chamber(index) => self.armed == Some(ArmedFor::Chamber(index)),
            Screen::Floor2Chamber(index) => self.armed == Some(ArmedFor::Floor2Chamber(index)),
            Screen::Floor3Chamber(index) => self.armed == Some(ArmedFor::Floor3Chamber(index)),
            Screen::Floor4Chamber(index) => self.armed == Some(ArmedFor::Floor4Chamber(index)),
            Screen::Floor5Chamber(index) => self.armed == Some(ArmedFor::Floor5Chamber(index)),
            Screen::Floor6Chamber(index) => self.armed == Some(ArmedFor::Floor6Chamber(index)),
            Screen::Foe => self.expedition.foe.strike == StrikePhase::Armed,
            Screen::Floor2Foe => self.expedition.floor2.foe.strike == StrikePhase::Armed,
            Screen::Floor3Foe => self.expedition.floor3.foe.strike == StrikePhase::Armed,
            Screen::Floor4Foe => self.expedition.floor4.foe.strike == StrikePhase::Armed,
            Screen::Floor5Foe => self.expedition.floor5.foe.strike == StrikePhase::Armed,
            Screen::Floor6Foe => self.expedition.floor6.foe.strike == StrikePhase::Armed,
            _ => false,
        }
    }

    /// Begin a fresh expedition.
    pub fn begin(&mut self) {
        self.expedition = Expedition::new();
        self.expedition
            .log_push("The descent began. Floor 1 — The Lending Vaults.");
        self.screen = Screen::Hub;
        self.overlay = Overlay::None;
        self.armed = None;
        self.place_at_spawn();
        self.say(
            StatusKind::Info,
            "Floor 1 — The Lending Vaults. Five chambers, one rule, and the Hoard at \
             the heart of it.",
        );
        self.changed();
    }

    /// Ask the shell to load the save.
    pub fn continue_expedition(&mut self) {
        self.io = Some(IoRequest::ReadSave);
    }

    /// The shell's answer to a save read.
    pub fn on_load_result(&mut self, result: Result<Expedition, SaveError>) {
        match result {
            Ok(expedition) => {
                self.expedition = expedition;
                self.screen = Screen::Hub;
                self.overlay = Overlay::None;
                self.armed = None;
                self.editor_cursor = 0;
                self.editor_anchor = None;
                self.place_at_spawn();
                let repaired = self.expedition.chambers_repaired();
                self.say(
                    StatusKind::Good,
                    format!(
                        "The expedition resumes: {repaired} of 5 chambers repaired, \
                         the Hoard {}.",
                        match self.expedition.foe.state {
                            FoeState::Standing => "standing",
                            FoeState::Exposed => "exposed",
                            FoeState::Fallen => "fallen",
                        }
                    ),
                );
                self.bump();
            }
            Err(error) => {
                self.say(StatusKind::Refusal, error.message());
            }
        }
    }

    /// The shell reports a completed save write.
    pub fn on_save_written(&mut self, bytes: usize) {
        self.save_present = true;
        self.say(
            StatusKind::Good,
            format!("The save is written ({bytes} bytes)."),
        );
    }

    /// The shell reports a failed save write.
    pub fn on_save_failed(&mut self, reason: String) {
        self.say(
            StatusKind::Refusal,
            format!("The save could not be written: {reason}"),
        );
    }

    /// The shell's answer to a gate run.
    /// The shell's answer to a lantern run: the lantern
    /// lights by its doing, and the format lantern's doing
    /// rewrites the draft it returns.
    pub fn on_lantern_result(&mut self, report: crate::gate::LanternReport) {
        self.gate_running = false;
        if report.passed {
            if report.kind == 0 {
                self.expedition
                    .floor3_foe_light_lantern(0, Some(report.text));
                self.say(
                    StatusKind::Good,
                    "The format lantern lights: the draft stands as \
                     rustfmt leaves it.",
                );
            } else {
                self.expedition.floor3_foe_light_lantern(1, None);
                self.say(
                    StatusKind::Good,
                    "The lint lantern lights: clippy burns green over \
                     the draft as it stands.",
                );
            }
        } else {
            self.say(
                StatusKind::Refusal,
                if report.kind == 0 {
                    "rustfmt could not run over the draft — check its \
                     shape in the editor."
                } else {
                    "clippy burns red over the draft as it stands."
                },
            );
        }
        self.changed();
    }

    pub fn on_gate_result(&mut self, report: GateReport) {
        self.gate_running = false;
        self.gate_detail = report.detail.clone();
        if self.gate_floor3 {
            self.expedition.floor3.foe.gate_record = Some(report.summary.clone());
            if report.tests_passed {
                self.expedition.floor3_foe_mark_tests_passed();
            }
            if report.passed() {
                self.expedition.floor3_foe_mark_gate_passed();
                self.expedition
                    .log_push("The gate stood open: the Gatehouse Release passed.");
                self.say(StatusKind::Good, report.summary.clone());
            } else {
                self.expedition
                    .log_push("The gate refused the Release; the draft is kept.");
                self.say(StatusKind::Refusal, report.summary.clone());
            }
            self.changed();
            return;
        }
        if self.gate_floor2 {
            self.expedition.floor2.foe.gate_record = Some(report.summary.clone());
            if report.passed() {
                self.expedition.floor2_foe_mark_gate_passed();
                self.expedition
                    .log_push("The gate stood open: the Admission Desk passed.");
                self.say(StatusKind::Good, report.summary.clone());
            } else {
                self.expedition
                    .log_push("The gate refused the Desk; the draft is kept.");
                self.say(StatusKind::Refusal, report.summary.clone());
            }
            self.changed();
            return;
        }
        self.expedition.gate_record = Some(report.summary.clone());
        if report.passed() {
            self.expedition.foe_mark_gate_passed();
            self.expedition
                .log_push("The gate stood open: the Interner's Hoard passed.");
            self.say(StatusKind::Good, report.summary.clone());
        } else {
            self.expedition
                .log_push("The gate refused the Hoard; the draft is kept.");
            self.say(StatusKind::Refusal, report.summary.clone());
        }
        self.changed();
    }

    /// Place the token at the current room's spawn.
    fn place_at_spawn(&mut self) {
        self.player_cell = (5, 7);
        self.player_px = cell_center(5, 7);
        self.walk_dest = None;
    }

    /// Enter a room screen, resetting per-room session state.
    fn enter_screen(&mut self, screen: Screen) {
        self.screen = screen;
        self.overlay = Overlay::None;
        self.armed = None;
        self.c1_view = 0;
        self.f2c5_view = 0;
        self.c5_page = 0;
        self.selected_record = None;
        self.place_at_spawn();
        self.bump();
    }

    /// The current challenge index for hints (0–4 chambers, 5 FOE),
    /// if the screen has one.
    fn current_challenge(&self) -> Option<usize> {
        match self.screen {
            Screen::Chamber(index) => Some(index),
            Screen::Foe => Some(FOE_CHALLENGE),
            _ => None,
        }
    }

    /// The verbs on this screen's bottom bar.
    pub fn verbs(&self) -> Vec<Verb> {
        match self.screen {
            Screen::Title => Vec::new(),
            Screen::Hub => vec![Verb::Read, Verb::Map],
            Screen::Floor2Hub
            | Screen::Floor3Hub
            | Screen::Floor4Hub
            | Screen::Floor5Hub
            | Screen::Floor6Hub => {
                vec![Verb::Read, Verb::Map]
            }
            Screen::Chamber(_)
            | Screen::Floor2Chamber(_)
            | Screen::Floor3Chamber(_)
            | Screen::Floor4Chamber(_)
            | Screen::Floor5Chamber(_)
            | Screen::Floor6Chamber(_) => vec![
                Verb::Examine,
                Verb::Read,
                Verb::Prove,
                Verb::Hint,
                Verb::Map,
            ],
            Screen::Foe => vec![
                Verb::Examine,
                Verb::Read,
                Verb::Enumerate,
                Verb::Exhibit,
                Verb::Hint,
                Verb::Map,
            ],
            Screen::Floor2Foe
            | Screen::Floor3Foe
            | Screen::Floor4Foe
            | Screen::Floor5Foe
            | Screen::Floor6Foe => vec![
                Verb::Examine,
                Verb::Read,
                Verb::Enumerate,
                Verb::Hint,
                Verb::Map,
            ],
        }
    }
}

/// A ratio as the gallery speaks it: plainly when it is
/// ordinary, in the surveyor's shorthand when it is not — a
/// wrong tag's nonsense should read as nonsense, not as a
/// page of zeros.
pub fn ratio_text(value: f32) -> String {
    if value != 0.0 && (value.abs() < 0.001 || value.abs() >= 1.0e7) {
        format!("{value:e}")
    } else {
        format!("{value}")
    }
}
/// A cell's centre in design coordinates (the token's home there).
fn cell_center(cx: i32, cy: i32) -> (f32, f32) {
    let rect = cell_rect(cx, cy);
    (rect.cx(), rect.cy())
}

// ---------------------------------------------------------------------------
// Geometry shared by layout and the shell's drawing: one source for
// where every element sits, so what is drawn is what is hit.
// ---------------------------------------------------------------------------

/// Element rectangles that depend on indices.
pub mod geom {
    use super::{Rect, cell_rect};

    /// A chamber door along the hub's top wall.
    pub fn door_chamber(index: usize) -> Rect {
        Rect::new(30.0 + index as f32 * 124.0, 78.0, 112.0, 58.0)
    }

    /// The FOE's door, hub bottom-right.
    pub fn door_foe() -> Rect {
        Rect::new(582.0, 518.0, 186.0, 124.0)
    }

    /// The stairs door, hub bottom-left.
    pub fn door_stairs() -> Rect {
        Rect::new(86.0, 518.0, 124.0, 124.0)
    }
    /// The Floor 2 door, hub bottom row between stairs and FOE.
    pub fn door_floor2() -> Rect {
        Rect::new(252.0, 518.0, 168.0, 124.0)
    }

    /// The Menagerie's door on Floor 2's hub: the open ground to
    /// the right, clear of the camp stone at the field's heart
    /// and of the Herald's strip along the bottom (the bottom
    /// stone, tried first, stood over the letters' lines).
    pub fn door_foe2() -> Rect {
        Rect::new(600.0, 292.0, 168.0, 124.0)
    }

    /// Floor 3's door on Floor 2's hub: the upper-right field,
    /// stairs-stone size, clear of the roster and the strip.
    pub fn door_floor3() -> Rect {
        Rect::new(640.0, 150.0, 124.0, 124.0)
    }

    /// Floor 4's door on Floor 1's hub: the upper-right field,
    /// stairs-stone size, below the chamber row's end and clear
    /// of the panel — the spot Floor 3's door proved on Floor
    /// 2's hub.
    pub fn door_floor4() -> Rect {
        Rect::new(640.0, 150.0, 124.0, 124.0)
    }

    /// Floor 5's door, on Floor 4's hub: the upper field
    /// below the chamber row's end — the spot Floor 4's door
    /// proved on Floor 1's hub, clear of the panel and the
    /// FOE's door below.
    pub fn door_floor5() -> Rect {
        Rect::new(640.0, 150.0, 124.0, 124.0)
    }

    /// Floor 6's door, on Floor 5's hub: the upper field
    /// below the chamber row's end — the spot Floor 5's door
    /// proved on Floor 4's hub, clear of the panel and the
    /// FOE's door below.
    pub fn door_floor6() -> Rect {
        Rect::new(640.0, 150.0, 124.0, 124.0)
    }

    /// Floor 6 chambers 1-4: one operation/call row.
    pub fn f6_op_row(index: usize) -> Rect {
        Rect::new(380.0, 100.0 + index as f32 * 56.0, 300.0, 30.0)
    }

    /// Floor 6's FOE door, on Floor 6's hub: the same ground
    /// the earlier floors proved for theirs.
    pub fn door_foe6() -> Rect {
        Rect::new(600.0, 292.0, 168.0, 124.0)
    }

    /// Floor 6 Chamber 5: one datum's housing row.
    pub fn f6_datum_row(index: usize) -> Rect {
        Rect::new(380.0, 96.0 + index as f32 * 150.0, 300.0, 30.0)
    }

    /// Floor 6 Chamber 5: one datum's door-sentence button.
    pub fn f6_sentence_btn(index: usize) -> Rect {
        Rect::new(380.0, 184.0 + index as f32 * 150.0, 300.0, 26.0)
    }

    /// The FOE: exhibit one sight of the staff.
    pub fn f6foe_exhibit_btn(exhibit: usize) -> Rect {
        Rect::new(56.0, 128.0 + exhibit as f32 * 52.0, 300.0, 30.0)
    }

    /// The FOE: walk one of the loom's cases.
    pub fn f6foe_case_btn(case: usize) -> Rect {
        Rect::new(56.0, 348.0 + case as f32 * 44.0, 300.0, 30.0)
    }

    /// The FOE: cycle one line of the staff's card (the
    /// line's current phrasing stands as text above it).
    pub fn f6foe_card_btn(line: usize) -> Rect {
        Rect::new(808.0, 222.0 + line as f32 * 92.0, 440.0, 26.0)
    }

    /// Floor 5 Chamber 1: one step's card row.
    pub fn f5_card_row(step: usize) -> Rect {
        Rect::new(380.0, 100.0 + step as f32 * 56.0, 300.0, 30.0)
    }

    /// Floor 5 Chamber 1: walk the next step.
    pub fn f5_walk_btn() -> Rect {
        Rect::new(380.0, 452.0, 180.0, 30.0)
    }

    /// Floor 5 Chamber 1: refuse the next step.
    pub fn f5_refuse_btn() -> Rect {
        Rect::new(580.0, 452.0, 188.0, 30.0)
    }

    /// Floor 5 Chamber 2: one placard's row.
    pub fn f5_placard_row(index: usize) -> Rect {
        Rect::new(380.0, 100.0 + index as f32 * 48.0, 300.0, 30.0)
    }

    /// Floor 5 Chamber 2: one miss's annotation cycle.
    pub fn f5_annotate_row(index: usize) -> Rect {
        Rect::new(688.0, 100.0 + index as f32 * 48.0, 88.0, 30.0)
    }

    /// Floor 5 Chamber 2: call the referee.
    pub fn f5_judge_btn() -> Rect {
        Rect::new(380.0, 508.0, 220.0, 30.0)
    }

    /// Floor 5 Chamber 3: one card line's row (tool-major).
    pub fn f5_cardline_row(tool: usize, line: usize) -> Rect {
        Rect::new(380.0, 100.0 + (tool * 3 + line) as f32 * 44.0, 300.0, 30.0)
    }

    /// Floor 5 Chamber 4: move the seam one word left.
    pub fn f5_split_down() -> Rect {
        Rect::new(380.0, 100.0, 120.0, 30.0)
    }

    /// Floor 5 Chamber 4: move the seam one word right.
    pub fn f5_split_up() -> Rect {
        Rect::new(510.0, 100.0, 120.0, 30.0)
    }

    /// Floor 5 Chamber 4: saw the seam at the chosen split.
    pub fn f5_seam_btn() -> Rect {
        Rect::new(640.0, 100.0, 136.0, 30.0)
    }

    /// Floor 5 Chamber 4: the referee's over-reach.
    pub fn f5_reach_btn() -> Rect {
        Rect::new(380.0, 150.0, 180.0, 30.0)
    }

    /// Floor 5 Chamber 4: work both halves at once.
    pub fn f5_work_btn() -> Rect {
        Rect::new(570.0, 150.0, 206.0, 30.0)
    }

    /// Floor 5 Chamber 4: drop the left half.
    pub fn f5_drop_left_btn() -> Rect {
        Rect::new(380.0, 200.0, 180.0, 30.0)
    }

    /// Floor 5 Chamber 4: drop the right half.
    pub fn f5_drop_right_btn() -> Rect {
        Rect::new(570.0, 200.0, 206.0, 30.0)
    }

    /// Floor 4 Chamber 1: one bench stone's row.
    pub fn f4_stone_row(stone: usize) -> Rect {
        Rect::new(380.0, 100.0 + stone as f32 * 56.0, 300.0, 30.0)
    }

    /// Floor 4 Chamber 1: the slate's size steppers.
    pub fn f4_size_down() -> Rect {
        Rect::new(380.0, 452.0, 44.0, 30.0)
    }

    /// Floor 4 Chamber 1: the slate's size steppers.
    pub fn f4_size_up() -> Rect {
        Rect::new(508.0, 452.0, 44.0, 30.0)
    }

    /// Floor 4 Chamber 1: the slate's alignment steppers.
    pub fn f4_align_down() -> Rect {
        Rect::new(596.0, 452.0, 44.0, 30.0)
    }

    /// Floor 4 Chamber 1: the slate's alignment steppers.
    pub fn f4_align_up() -> Rect {
        Rect::new(724.0, 452.0, 44.0, 30.0)
    }

    /// Floor 4 Chamber 1: the line drop.
    pub fn f4_drop_line() -> Rect {
        Rect::new(380.0, 500.0, 180.0, 30.0)
    }

    /// Floor 4 Chamber 1: the annotation cycle.
    pub fn f4_annotate() -> Rect {
        Rect::new(580.0, 500.0, 188.0, 30.0)
    }

    /// Floor 4 Chamber 2: one bench stone's seat row.
    pub fn f4_seat_row(stone: usize) -> Rect {
        Rect::new(380.0, 100.0 + stone as f32 * 56.0, 300.0, 30.0)
    }

    /// Floor 4 Chamber 2: clear the wall.
    pub fn f4_wall_reset() -> Rect {
        Rect::new(380.0, 500.0, 180.0, 30.0)
    }

    /// Floor 4 Chamber 3: one crest's row.
    pub fn f4_crest_row(crest: usize) -> Rect {
        Rect::new(380.0, 100.0 + crest as f32 * 76.0, 300.0, 30.0)
    }

    /// Floor 4 Chamber 3: one crest's measure button.
    pub fn f4_crest_prove(crest: usize) -> Rect {
        Rect::new(692.0, 100.0 + crest as f32 * 76.0, 76.0, 30.0)
    }

    /// Floor 4 Chamber 4: one stone's tray row.
    pub fn f4_treaty_row(stone: usize) -> Rect {
        Rect::new(380.0, 100.0 + stone as f32 * 56.0, 300.0, 30.0)
    }

    /// Floor 4 Chamber 4: drop the line.
    pub fn f4_treaty_prove() -> Rect {
        Rect::new(380.0, 500.0, 180.0, 30.0)
    }

    /// Floor 4 Chamber 4: take the wall down.
    pub fn f4_treaty_reset() -> Rect {
        Rect::new(580.0, 500.0, 188.0, 30.0)
    }

    /// Floor 4 Chamber 5: one hollow stone's filling cycle.
    pub fn f4_fill_btn(stone: usize) -> Rect {
        Rect::new(380.0, 120.0 + stone as f32 * 130.0, 170.0, 30.0)
    }

    /// Floor 4 Chamber 5: one hollow stone's tag cycle.
    pub fn f4_tag_btn(stone: usize) -> Rect {
        Rect::new(566.0, 120.0 + stone as f32 * 130.0, 174.0, 30.0)
    }

    /// Floor 4 Chamber 5: read one hollow stone as tagged.
    pub fn f4_read_btn(stone: usize) -> Rect {
        Rect::new(380.0, 162.0 + stone as f32 * 130.0, 170.0, 30.0)
    }

    /// Floor 4 Chamber 5: one cell of the blank stone.
    pub fn f4_cell(cell: usize) -> Rect {
        Rect::new(380.0 + cell as f32 * 92.0, 420.0, 84.0, 30.0)
    }

    /// Floor 4 Chamber 5: use the blank stone.
    pub fn f4_use_blank() -> Rect {
        Rect::new(380.0, 470.0, 220.0, 30.0)
    }

    /// Floor 4's FOE door on Floor 4's hub: the same proven
    /// ground the Release's door holds on Floor 3's hub.
    pub fn door_foe4() -> Rect {
        Rect::new(600.0, 292.0, 168.0, 124.0)
    }

    /// The Foundation: the take-the-book button.
    pub fn f4_book_take() -> Rect {
        Rect::new(380.0, 120.0, 300.0, 34.0)
    }

    /// The Foundation: drop the line on one book entry.
    pub fn f4_measure_btn(entry: usize) -> Rect {
        Rect::new(380.0, 96.0 + entry as f32 * 60.0, 170.0, 30.0)
    }

    /// The Foundation: exhibit one book entry.
    pub fn f4_exhibit_btn(entry: usize) -> Rect {
        Rect::new(566.0, 96.0 + entry as f32 * 60.0, 174.0, 30.0)
    }

    /// The Foundation: one foreign stone's tray row.
    pub fn f4_foreign_row(stone: usize) -> Rect {
        Rect::new(56.0, 96.0 + stone as f32 * 52.0, 300.0, 30.0)
    }

    /// The Foundation: drop the line on the foreign set.
    pub fn f4_foreign_prove() -> Rect {
        Rect::new(56.0, 372.0, 144.0, 30.0)
    }

    /// The Foundation: take the foreign set down.
    pub fn f4_foreign_reset() -> Rect {
        Rect::new(208.0, 372.0, 144.0, 30.0)
    }

    /// The Foundation: one free stone's tray row.
    pub fn f4_free_row(stone: usize) -> Rect {
        Rect::new(380.0, 416.0 + stone as f32 * 48.0, 300.0, 30.0)
    }

    /// The Foundation: drop the line on the free set.
    pub fn f4_free_prove() -> Rect {
        Rect::new(380.0, 620.0, 144.0, 30.0)
    }

    /// The Foundation: take the free set down.
    pub fn f4_free_reset() -> Rect {
        Rect::new(532.0, 620.0, 144.0, 30.0)
    }

    /// Floor 5's FOE door on Floor 5's hub: the ground the
    /// Foundation's door proved on the floor below.
    pub fn door_foe5() -> Rect {
        Rect::new(600.0, 292.0, 168.0, 124.0)
    }

    /// Chamber 5: one station's verdict button.
    pub fn f5_station_row(station: usize) -> Rect {
        Rect::new(808.0, 176.0 + station as f32 * 56.0, 300.0, 30.0)
    }

    /// Chamber 5: one station's lens button.
    pub fn f5_lens_btn(station: usize) -> Rect {
        Rect::new(1116.0, 176.0 + station as f32 * 56.0, 124.0, 30.0)
    }

    /// Chamber 5: one faulted station's card button.
    pub fn f5_fix_btn(station: usize) -> Rect {
        Rect::new(808.0, 210.0 + station as f32 * 56.0, 432.0, 22.0)
    }

    /// The FOE: exhibit one sight of the trace.
    pub fn f5foe_exhibit_btn(exhibit: usize) -> Rect {
        Rect::new(56.0, 128.0 + exhibit as f32 * 52.0, 300.0, 30.0)
    }

    /// The FOE: walk one of the referee's cases.
    pub fn f5foe_case_btn(case: usize) -> Rect {
        Rect::new(56.0, 348.0 + case as f32 * 44.0, 300.0, 30.0)
    }

    /// The FOE: cycle one line of the beam's card (the
    /// line's current phrasing stands as text above it).
    pub fn f5foe_card_btn(line: usize) -> Rect {
        Rect::new(808.0, 222.0 + line as f32 * 92.0, 440.0, 26.0)
    }
    /// A roster group header row on Floor 2's panel.
    pub fn roster_group(index: usize) -> Rect {
        Rect::new(808.0, 176.0 + index as f32 * 104.0, 440.0, 26.0)
    }

    /// The camp stone, hub centre.
    pub fn camp_stone() -> Rect {
        cell_rect(5, 4)
    }

    /// The room's subject block (desk / engine).
    pub fn desk() -> Rect {
        Rect::new(520.0, 208.0, 124.0, 124.0)
    }

    /// The inscription stone.
    pub fn stone() -> Rect {
        cell_rect(2, 7)
    }

    /// C1: an item row.
    pub fn c1_row(index: usize) -> Rect {
        Rect::new(808.0, 234.0 + index as f32 * 33.0, 440.0, 31.0)
    }

    /// C1: a porter toggle button within a row.
    pub fn holder_btn(item: usize, porter: usize) -> Rect {
        let row = c1_row(item);
        Rect::new(1096.0 + porter as f32 * 52.0, row.y + 2.0, 46.0, 27.0)
    }

    /// C1: a panel sub-view tab (0 packing, 1 toll).
    /// Floor 2 C1: a charter candidate button.
    pub fn f2_charter(choice: usize) -> Rect {
        Rect::new(808.0 + choice as f32 * 148.0, 388.0, 140.0, 30.0)
    }

    /// Floor 2 C1: a blind-run identification button.
    pub fn f2_run(choice: usize) -> Rect {
        Rect::new(808.0 + choice as f32 * 148.0, 452.0, 140.0, 30.0)
    }

    /// Floor 2 C1: the watch-performances button.
    pub fn f2_watch() -> Rect {
        Rect::new(808.0, 500.0, 220.0, 30.0)
    }

    /// Floor 3 Chamber 1: one report's pair button, right of
    /// the desk; the report's text stands above it, its chosen
    /// remedy below.
    pub fn f3_pair(report: usize) -> Rect {
        Rect::new(380.0, 104.0 + report as f32 * 120.0, 360.0, 30.0)
    }

    /// Floor 3 Chamber 2: one station's setting button.
    pub fn f3_station(slot: usize) -> Rect {
        Rect::new(380.0, 150.0 + slot as f32 * 90.0, 360.0, 30.0)
    }

    /// Floor 3 Chamber 3: one entry's verdict button.
    pub fn f3_shout(case: usize) -> Rect {
        Rect::new(380.0, 104.0 + case as f32 * 120.0, 170.0, 30.0)
    }

    /// Floor 3 Chamber 3: one entry's invariant button.
    pub fn f3_invariant(case: usize) -> Rect {
        Rect::new(566.0, 104.0 + case as f32 * 120.0, 174.0, 30.0)
    }

    /// Floor 3 Chamber 4: one slip's iron, by action
    /// (0 advance, 1 seal, 2 return).
    pub fn f3_slip(slip: usize, action: usize) -> Rect {
        Rect::new(
            380.0 + action as f32 * 122.0,
            130.0 + slip as f32 * 120.0,
            114.0,
            30.0,
        )
    }

    /// The Release's door on Floor 3's hub.
    pub fn door_foe3() -> Rect {
        Rect::new(600.0, 292.0, 168.0, 124.0)
    }

    /// Floor 3 Chamber 5: one Guideline line's verdict button.
    pub fn f3_check(item: usize) -> Rect {
        Rect::new(380.0, 104.0 + item as f32 * 76.0, 200.0, 30.0)
    }

    /// Floor 3 Chamber 5: the set-aside line's reason button.
    pub fn f3_na_reason() -> Rect {
        Rect::new(380.0, 500.0, 360.0, 30.0)
    }

    /// Floor 3 Chamber 5: the stamp button.
    pub fn f3_msrv() -> Rect {
        Rect::new(380.0, 556.0, 360.0, 30.0)
    }

    /// The rite's button in a room's field (Chamber 5's side
    /// desk and the Release's door share the shape).
    pub fn f3_rite_field() -> Rect {
        Rect::new(56.0, 556.0, 300.0, 30.0)
    }

    /// The Release: the take-the-slate button.
    pub fn f3_slate_take() -> Rect {
        Rect::new(380.0, 120.0, 300.0, 34.0)
    }

    /// The Release: one slate entry's verdict button.
    pub fn f3_slate_entry(entry: usize) -> Rect {
        Rect::new(380.0, 104.0 + entry as f32 * 120.0, 170.0, 30.0)
    }

    /// The Release: the reconcile button.
    pub fn f3_reconcile() -> Rect {
        Rect::new(380.0, 500.0, 360.0, 30.0)
    }

    /// The Release: the stamp button.
    pub fn f3_stamp() -> Rect {
        Rect::new(56.0, 262.0, 300.0, 30.0)
    }

    /// The Release: a lantern button (0 format, 1 lint).
    pub fn f3_lantern(kind: usize) -> Rect {
        Rect::new(56.0 + kind as f32 * 156.0, 422.0, 144.0, 30.0)
    }

    /// Floor 2 C2: a sigil toggle in a service's bound row.
    pub fn f2_bound(service: usize, sigil: usize) -> Rect {
        Rect::new(
            808.0 + sigil as f32 * 110.0,
            244.0 + service as f32 * 84.0,
            102.0,
            28.0,
        )
    }

    /// Floor 2 C3: the run-deliveries button.
    pub fn f2_run_deliveries() -> Rect {
        Rect::new(808.0, 500.0, 240.0, 30.0)
    }

    /// Floor 2 C3: an errand chip in the finding row.
    pub fn f2_errand(errand: usize) -> Rect {
        Rect::new(808.0 + errand as f32 * 148.0, 478.0, 140.0, 28.0)
    }

    /// Floor 2 C3: a kennel chip in the finding column.
    pub fn f2_kennel(kennel: usize) -> Rect {
        Rect::new(808.0, 528.0 + kennel as f32 * 30.0, 200.0, 26.0)
    }

    /// Floor 2 C4: the drive-the-round button.
    pub fn f2_drive() -> Rect {
        Rect::new(808.0, 560.0, 240.0, 30.0)
    }

    /// Floor 2 C4: a bar-label chip in one barred beast's naming
    /// row (barred-beast position, bar label).
    pub fn f2_bar(slot: usize, label: usize) -> Rect {
        Rect::new(
            808.0 + label as f32 * 148.0,
            292.0 + slot as f32 * 84.0,
            140.0,
            28.0,
        )
    }

    /// Floor 2 C5: a joining-route button.
    pub fn f2_route(route: usize) -> Rect {
        Rect::new(808.0, 352.0 + route as f32 * 76.0, 424.0, 30.0)
    }

    /// Floor 2 C5: a collision-reading chip.
    pub fn f2_reading(reading: usize) -> Rect {
        Rect::new(808.0 + reading as f32 * 148.0, 452.0, 140.0, 28.0)
    }

    /// Floor 2 hub: a Herald letter's act button, in the room.
    pub fn f2_herald_act(letter: usize) -> Rect {
        Rect::new(646.0, 468.0 + letter as f32 * 56.0, 122.0, 30.0)
    }

    pub fn panel_tab(view: usize) -> Rect {
        Rect::new(808.0 + view as f32 * 148.0, 196.0, 140.0, 30.0)
    }

    /// C1: a toll candidate button.
    pub fn toll_btn(index: usize) -> Rect {
        let (x, y) = match index {
            0 => (808.0, 420.0),
            1 => (1030.0, 420.0),
            2 => (808.0, 464.0),
            _ => (1030.0, 464.0),
        };
        Rect::new(x, y, 210.0, 36.0)
    }

    /// C2: a person's row.
    pub fn c2_row(index: usize) -> Rect {
        Rect::new(808.0, 234.0 + index as f32 * 52.0, 440.0, 46.0)
    }

    /// C2: a person's block-cycle button.
    pub fn pick_btn(person: usize) -> Rect {
        let row = c2_row(person);
        Rect::new(996.0, row.y + 6.0, 244.0, 34.0)
    }

    /// C3: a lease stepper button (lease, field 0=start/1=end, dir).
    pub fn lease_step(lease: usize, field: usize, dir: i8) -> Rect {
        let y = 356.0 + lease as f32 * 44.0;
        let x = match (field, dir) {
            (0, -1) => 936.0,
            (0, _) => 1038.0,
            (1, -1) => 1096.0,
            _ => 1198.0,
        };
        Rect::new(x, y, 30.0, 32.0)
    }

    /// C4: the run button.
    pub fn run_btn() -> Rect {
        Rect::new(808.0, 304.0, 210.0, 36.0)
    }

    /// C4: the commit toggle.
    pub fn commit_btn() -> Rect {
        Rect::new(1030.0, 304.0, 210.0, 36.0)
    }

    /// C4: a record card.
    pub fn record_card(index: usize) -> Rect {
        Rect::new(808.0, 352.0 + index as f32 * 66.0, 440.0, 58.0)
    }

    /// C5: a placard row on the current page.
    pub fn c5_row(local: usize) -> Rect {
        Rect::new(808.0, 234.0 + local as f32 * 62.0, 440.0, 58.0)
    }

    /// C5: a placard's judgement button.
    pub fn judge_btn(local: usize) -> Rect {
        let row = c5_row(local);
        Rect::new(1104.0, row.y + 2.0, 136.0, 26.0)
    }

    /// C5: previous page.
    pub fn page_prev() -> Rect {
        Rect::new(1090.0, 196.0, 70.0, 30.0)
    }

    /// C5: next page.
    pub fn page_next() -> Rect {
        Rect::new(1170.0, 196.0, 70.0, 30.0)
    }

    /// FOE: a census row for uncopiable position `pos`.
    pub fn foe_row(pos: usize) -> Rect {
        Rect::new(808.0, 240.0 + pos as f32 * 30.0, 440.0, 28.0)
    }

    /// FOE: a census row's verify button.
    pub fn verify_btn(pos: usize) -> Rect {
        let row = foe_row(pos);
        Rect::new(1152.0, row.y + 1.0, 88.0, 26.0)
    }

    /// FOE: the editor-open button.
    pub fn editor_open() -> Rect {
        Rect::new(1030.0, 196.0, 210.0, 34.0)
    }

    /// Floor 2 FOE: an audit row on the desk's table.
    pub fn f2foe_row(pos: usize) -> Rect {
        Rect::new(808.0, 240.0 + pos as f32 * 34.0, 440.0, 32.0)
    }

    /// Floor 2 FOE: an audit row's exhibit button.
    pub fn f2foe_exhibit_btn(pos: usize) -> Rect {
        let row = f2foe_row(pos);
        Rect::new(1152.0, row.y + 3.0, 88.0, 26.0)
    }

    /// Floor 2 FOE: the calibrate button, under the audit table.
    pub fn f2foe_calibrate() -> Rect {
        Rect::new(808.0, 484.0, 176.0, 32.0)
    }
}

impl Game {
    // ------------------------------------------------------------------
    // Movement.
    // ------------------------------------------------------------------

    /// Whether a cell can be stood on in the current room.
    fn cell_blocked(&self, cell: (i32, i32)) -> bool {
        let (cx, cy) = cell;
        if cx < 0 || cy < 0 || cx >= ROOM_COLS || cy >= ROOM_ROWS {
            return true;
        }
        match self.screen {
            Screen::Hub
            | Screen::Floor2Hub
            | Screen::Floor3Hub
            | Screen::Floor4Hub
            | Screen::Floor5Hub
            | Screen::Floor6Hub => cy == 0 || cell == (5, 4),
            Screen::Chamber(_)
            | Screen::Floor2Chamber(_)
            | Screen::Floor3Chamber(_)
            | Screen::Floor4Chamber(_)
            | Screen::Floor5Chamber(_)
            | Screen::Floor6Chamber(_)
            | Screen::Foe
            | Screen::Floor2Foe
            | Screen::Floor3Foe
            | Screen::Floor4Foe
            | Screen::Floor5Foe
            | Screen::Floor6Foe => (8..10).contains(&cx) && (2..4).contains(&cy) || cell == (2, 7),
            Screen::Title => true,
        }
    }

    /// Send the token walking toward a cell, if it can be stood on.
    fn walk_to(&mut self, cell: (i32, i32)) {
        if !self.cell_blocked(cell) {
            self.walk_dest = Some(cell);
        }
    }

    /// Advance the token.
    pub fn update(&mut self, dt: f32) {
        if self.overlay != Overlay::None {
            return;
        }
        let Some(dest) = self.walk_dest else { return };
        if dest == self.player_cell {
            self.walk_dest = None;
            return;
        }
        if self.expedition.settings.reduced_motion {
            self.step_timer += dt;
            if self.step_timer >= STEP_SECONDS {
                self.step_timer = 0.0;
                let (cx, cy) = self.player_cell;
                let next = if cx != dest.0 {
                    (cx + (dest.0 - cx).signum(), cy)
                } else {
                    (cx, cy + (dest.1 - cy).signum())
                };
                if self.cell_blocked(next) {
                    self.walk_dest = None;
                } else {
                    self.player_cell = next;
                    self.player_px = cell_center(next.0, next.1);
                    if next == dest {
                        self.walk_dest = None;
                    }
                }
            }
            return;
        }
        let target = cell_center(dest.0, dest.1);
        let (px, py) = self.player_px;
        let (dx, dy) = (target.0 - px, target.1 - py);
        let dist = (dx * dx + dy * dy).sqrt();
        let step = MOVE_SPEED * dt;
        if dist <= step || dist < 0.5 {
            // Arrive — unless the destination sits inside furniture
            // the straight line clipped; the cell check at walk_to
            // already refused those.
            self.player_px = target;
            self.player_cell = dest;
            self.walk_dest = None;
        } else {
            let nx = px + dx / dist * step;
            let ny = py + dy / dist * step;
            let next_cell = (
                ((nx - GRID_X) / CELL).floor() as i32,
                ((ny - GRID_Y) / CELL).floor() as i32,
            );
            if self.cell_blocked(next_cell) && next_cell != self.player_cell {
                self.walk_dest = None;
            } else {
                self.player_px = (nx, ny);
            }
        }
    }

    // ------------------------------------------------------------------
    // Pointer.
    // ------------------------------------------------------------------

    /// Pointer pressed at design coordinates.
    pub fn pointer_down(&mut self, x: f32, y: f32) {
        self.pointer_down = true;
        let hit = hit_test(self, x, y);
        if self.overlay == Overlay::Editor && hit == Some(El::EditorArea) {
            self.editor_click(x, y);
        }
        self.pointer_hit = hit;
        if hit.is_none() && self.overlay == Overlay::None && self.token_visible() {
            let cell = (
                ((x - GRID_X) / CELL).floor() as i32,
                ((y - GRID_Y) / CELL).floor() as i32,
            );
            if cell.0 >= 0 && cell.1 >= 0 && cell.0 < ROOM_COLS && cell.1 < ROOM_ROWS {
                self.walk_to(cell);
            }
        }
    }

    /// Pointer moved. (This build has no drag interactions; exhibit
    /// is tap-tap, per the design's touch mapping.)
    pub fn pointer_move(&mut self, _x: f32, _y: f32) {}

    /// Pointer released: a click is down-and-up on the same element.
    pub fn pointer_up(&mut self, x: f32, y: f32) {
        self.pointer_down = false;
        let hit = hit_test(self, x, y);
        if let Some(el) = hit
            && hit == self.pointer_hit
        {
            self.activate(el);
        }
        self.pointer_hit = None;
    }

    // ------------------------------------------------------------------
    // Keyboard.
    // ------------------------------------------------------------------

    /// An abstract key press.
    pub fn key(&mut self, key: Key) {
        match key {
            Key::Tab => {
                let els: Vec<El> = layout(self).into_iter().map(|(el, _)| el).collect();
                if els.is_empty() {
                    return;
                }
                let next = match self.focus {
                    Some(current) => els
                        .iter()
                        .position(|el| *el == current)
                        .map_or(0, |index| (index + 1) % els.len()),
                    None => 0,
                };
                self.focus = Some(els[next]);
                self.bump();
            }
            Key::Confirm => {
                if let Some(el) = self.focus {
                    // A stale focus from another screen never fires.
                    if layout(self).iter().any(|(candidate, _)| *candidate == el) {
                        self.activate(el);
                    }
                }
            }
            Key::Cancel => {
                if self.overlay != Overlay::None {
                    self.close_overlay();
                } else if self.armed.is_some()
                    || self.expedition.foe.strike == StrikePhase::Armed
                    || self.expedition.floor2.foe.strike == StrikePhase::Armed
                {
                    self.disarm();
                    self.say(StatusKind::Info, "The seal is stood down.");
                } else if self.screen != Screen::Title {
                    self.overlay = Overlay::Pause;
                    self.bump();
                }
            }
            Key::Up | Key::Down | Key::Left | Key::Right => {
                if self.overlay == Overlay::None && self.token_visible() {
                    let (cx, cy) = self.player_cell;
                    let dest = match key {
                        Key::Up => (cx, cy - 1),
                        Key::Down => (cx, cy + 1),
                        Key::Left => (cx - 1, cy),
                        _ => (cx + 1, cy),
                    };
                    self.walk_to(dest);
                }
            }
            Key::Examine => self.do_verb(Verb::Examine),
            Key::Read => self.do_verb(Verb::Read),
            Key::Enumerate => self.do_verb(Verb::Enumerate),
            Key::Exhibit => {
                if self.screen == Screen::Foe {
                    self.do_verb(Verb::Exhibit);
                } else {
                    self.do_verb(Verb::Prove);
                }
            }
            Key::Hint => self.do_verb(Verb::Hint),
            Key::Map => {
                self.overlay = Overlay::Map;
                self.bump();
            }
            Key::Log => {
                self.overlay = Overlay::Log;
                self.bump();
            }
            Key::Codex => {
                self.overlay = Overlay::Codex;
                self.bump();
            }
        }
    }

    /// Close the current overlay, persisting what it held.
    fn close_overlay(&mut self) {
        if self.overlay == Overlay::Editor {
            self.changed();
        }
        self.overlay = Overlay::None;
        self.bump();
    }

    /// Stand down any armed strike, silently.
    fn disarm(&mut self) {
        self.armed = None;
        if self.expedition.foe.strike == StrikePhase::Armed {
            self.expedition.foe_disarm_strike();
        }
        if self.expedition.floor2.foe.strike == StrikePhase::Armed {
            self.expedition.floor2_foe_disarm_strike();
        }
    }

    // ------------------------------------------------------------------
    // Verbs.
    // ------------------------------------------------------------------

    /// A bottom-bar verb (or its key).
    pub fn do_verb(&mut self, verb: Verb) {
        if self.overlay != Overlay::None && verb != Verb::Map {
            return;
        }
        self.disarm_unless_seal();
        match verb {
            Verb::Examine => {
                let text = match self.screen {
                    Screen::Chamber(index) => EXAMINE_TEXTS[index],
                    Screen::Floor2Chamber(index) => F2_EXAMINE_TEXTS[index],
                    Screen::Floor3Chamber(index) => F3_EXAMINE_TEXTS[index],
                    Screen::Floor4Chamber(index) => content::F4_EXAMINE_TEXTS[index],
                    Screen::Floor5Chamber(index) => content::F5_EXAMINE_TEXTS[index],
                    Screen::Floor6Chamber(index) => content::F6_EXAMINE_TEXTS[index],
                    Screen::Foe => {
                        "The hoard-engine: a ledger the size of a door, turning its \
                         pages by itself, smiling with every leaf."
                    }
                    Screen::Floor2Foe => {
                        "The admission desk: seven slips under a brass weight, a \
                         brazier burning at the door for the calibration, and the \
                         Menagerie watching its own table be read."
                    }
                    Screen::Floor4Foe => {
                        "The master foundation: five load-bearing stones \
                         under a settling arch, a dimension book in a \
                         frame, and a plumb line hanging still beside \
                         stones nobody has measured in a generation."
                    }
                    Screen::Floor5Foe => {
                        "The shared beam: six cells in a gauze-hung \
                         frame, borrowed at both ends, chalked with \
                         addresses — and a fullness in the gauze \
                         behind it, shaped almost like a seventh cell."
                    }
                    Screen::Floor6Foe => {
                        "The broken staff: three lawful states enamelled \
                         on its shaft, a fourth mark budding between \
                         shared and exclusive, and a counter beside it \
                         reading a number the state does not hold."
                    }
                    Screen::Floor3Foe => {
                        "The Gatehouse Release: a great door, a lantern \
                         burning before its keeper came, and a slate that \
                         claims three passes."
                    }
                    Screen::Hub => {
                        "The vaults: five chamber doors, the Hoard's door, and the \
                         stairs — shut, for now."
                    }
                    Screen::Floor2Hub => {
                        "The Menagerie Grounds: five chamber doors, the roster \
                         board, and the Menagerie watching from its wide door."
                    }
                    Screen::Floor3Hub => {
                        "The Gatehouse: five desks, a checklist on the wall, \
                         and a green lantern over a release nobody inspected."
                    }
                    Screen::Floor4Hub => {
                        "The Foundation Galleries: five galleries of stones, \
                         each stencilled with its true dimensions, and a \
                         plumb line hanging in every room."
                    }
                    Screen::Floor5Hub => {
                        "The Veilworks: five workshops behind gauze, a sharp \
                         tool on every bench, and the Veilwalker watching \
                         hands, not faces."
                    }
                    Screen::Floor6Hub => {
                        "The Lockheart: strongboxes in a quiet warren, \
                         reading hearts behind glass, and every chest \
                         posting the rules of its own opening."
                    }
                    Screen::Title => return,
                };
                self.say(StatusKind::Info, text);
            }
            Verb::Read => {
                self.overlay = Overlay::Reading(ReadingKind::Inscription);
                self.bump();
            }
            Verb::Enumerate => {
                if self.screen == Screen::Floor6Foe {
                    let first = !self.expedition.floor6.foe.exhibits[0];
                    match self.expedition.floor6_foe_exhibit(0) {
                        Ok(text) => {
                            if first {
                                self.expedition.log_push(
                                    "The staff's first sight was exhibited: the staff's own census.",
                                );
                            }
                            self.say(StatusKind::Good, text);
                        }
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
                if self.screen == Screen::Floor5Foe {
                    let first = !self.expedition.floor5.foe.exhibits[0];
                    match self.expedition.floor5_foe_exhibit(0) {
                        Ok(text) => {
                            if first {
                                self.expedition.log_push(
                                    "The trace's first sight was exhibited: a live shared loan.",
                                );
                            }
                            self.say(StatusKind::Good, text);
                        }
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
                if self.screen == Screen::Floor4Foe {
                    let first = !self.expedition.floor4.foe.book_taken;
                    self.expedition.floor4_foe_take_book();
                    if first {
                        self.expedition
                            .log_push("The dimension book was taken down for the re-survey.");
                    }
                    self.say(
                        StatusKind::Info,
                        "The book is down: five load-bearing entries in a \
                         beautiful hand. Drop the line on every stone it \
                         names — the book cannot acquit itself.",
                    );
                    self.changed();
                }
                if self.screen == Screen::Floor2Foe {
                    let first = !self.expedition.floor2.foe.audit_taken;
                    self.expedition.floor2_foe_take_audit();
                    if first {
                        self.expedition
                            .log_push("The desk's admissions were taken into audit.");
                    }
                    self.say(
                        StatusKind::Info,
                        format!(
                            "The audit is taken: {} admissions on the desk's table. \
                             Exhibit each beast's sigils against its post's bound; \
                             name the one admission whose slip and sigils disagree.",
                            content::FOE_ADMISSIONS.len()
                        ),
                    );
                    self.changed();
                }
                if self.screen == Screen::Foe {
                    let first = !self.expedition.foe.enumerated;
                    self.expedition.foe_enumerate();
                    if first {
                        self.expedition
                            .log_push("The master plan was enumerated into a census.");
                    }
                    let rows = puzzles::foe_census().rows.len();
                    self.say(
                        StatusKind::Info,
                        format!(
                            "The census is taken: {rows} items on the master plan. \
                             Verify each uncopiable row; exhibit the one that lies."
                        ),
                    );
                    self.changed();
                }
            }
            Verb::Exhibit => {
                if self.screen == Screen::Foe {
                    match puzzles::foe_double_row() {
                        Some(row) => self.exhibit_foe_row(row),
                        None => self.say(StatusKind::Refusal, "the census shows no seam"),
                    }
                }
            }
            Verb::Prove => {
                if let Screen::Chamber(index) = self.screen {
                    match self.expedition.file_proof(index) {
                        Ok(text) => {
                            self.expedition
                                .log_push(format!("Proof filed in {}.", CHAMBERS[index].name));
                            self.say(StatusKind::Good, text);
                        }
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
                if let Screen::Floor2Chamber(index) = self.screen {
                    match self.expedition.floor2_file_proof(index) {
                        Ok(text) => {
                            self.expedition.log_push(format!(
                                "Proof filed in {}.",
                                content::MENAGERIE_CHAMBERS[index].name
                            ));
                            self.say(StatusKind::Good, text);
                        }
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
                if let Screen::Floor3Chamber(index) = self.screen {
                    match self.expedition.floor3_file_proof(index) {
                        Ok(text) => {
                            self.expedition.log_push(format!(
                                "Proof filed in {}.",
                                content::GATEHOUSE_CHAMBERS[index].name
                            ));
                            self.say(StatusKind::Good, text);
                        }
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
                if let Screen::Floor6Chamber(index) = self.screen {
                    match self.expedition.floor6_file_proof(index) {
                        Ok(text) => {
                            self.expedition.log_push(format!(
                                "Proof filed in {}.",
                                content::LOCK_CHAMBERS[index].name
                            ));
                            self.say(StatusKind::Good, text);
                        }
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
                if let Screen::Floor5Chamber(index) = self.screen {
                    match self.expedition.floor5_file_proof(index) {
                        Ok(text) => {
                            self.expedition.log_push(format!(
                                "Proof filed in {}.",
                                content::VEIL_CHAMBERS[index].name
                            ));
                            self.say(StatusKind::Good, text);
                        }
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
                if let Screen::Floor4Chamber(index) = self.screen {
                    match self.expedition.floor4_file_proof(index) {
                        Ok(text) => {
                            self.expedition.log_push(format!(
                                "Proof filed in {}.",
                                content::FOUNDATION_CHAMBERS[index].name
                            ));
                            self.say(StatusKind::Good, text);
                        }
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            Verb::Hint => {
                if let Screen::Floor6Chamber(index) = self.screen {
                    if self.expedition.floor6_hints_used(index) == 0 {
                        self.expedition.floor6_view_hint(index);
                    }
                    self.hint_showing = self.expedition.floor6_hints_used(index).max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if let Screen::Floor5Chamber(index) = self.screen {
                    if self.expedition.floor5_hints_used(index) == 0 {
                        self.expedition.floor5_view_hint(index);
                    }
                    self.hint_showing = self.expedition.floor5_hints_used(index).max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if let Screen::Floor4Chamber(index) = self.screen {
                    if self.expedition.floor4_hints_used(index) == 0 {
                        self.expedition.floor4_view_hint(index);
                    }
                    self.hint_showing = self.expedition.floor4_hints_used(index).max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if let Screen::Floor3Chamber(index) = self.screen {
                    if self.expedition.floor3_hints_used(index) == 0 {
                        self.expedition.floor3_view_hint(index);
                    }
                    self.hint_showing = self.expedition.floor3_hints_used(index).max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if let Screen::Floor2Chamber(index) = self.screen {
                    if self.expedition.floor2_hints_used(index) == 0 {
                        self.expedition.floor2_view_hint(index);
                    }
                    self.hint_showing = self.expedition.floor2_hints_used(index).max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if self.screen == Screen::Floor2Foe {
                    if self.expedition.floor2_foe_hints_used() == 0 {
                        self.expedition.floor2_foe_view_hint();
                    }
                    self.hint_showing = self.expedition.floor2_foe_hints_used().max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if self.screen == Screen::Floor6Foe {
                    if self.expedition.floor6_foe_hints_used() == 0 {
                        self.expedition.floor6_foe_view_hint();
                    }
                    self.hint_showing = self.expedition.floor6_foe_hints_used().max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if self.screen == Screen::Floor5Foe {
                    if self.expedition.floor5_foe_hints_used() == 0 {
                        self.expedition.floor5_foe_view_hint();
                    }
                    self.hint_showing = self.expedition.floor5_foe_hints_used().max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if self.screen == Screen::Floor4Foe {
                    if self.expedition.floor4_foe_hints_used() == 0 {
                        self.expedition.floor4_foe_view_hint();
                    }
                    self.hint_showing = self.expedition.floor4_foe_hints_used().max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if self.screen == Screen::Floor3Foe {
                    if self.expedition.floor3_foe_hints_used() == 0 {
                        self.expedition.floor3_foe_view_hint();
                    }
                    self.hint_showing = self.expedition.floor3_foe_hints_used().max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                } else if let Some(challenge) = self.current_challenge() {
                    if self.expedition.hints_used(challenge) == 0 {
                        self.expedition.view_hint(challenge);
                    }
                    self.hint_showing = self.expedition.hints_used(challenge).max(1);
                    self.overlay = Overlay::Reading(ReadingKind::Hint);
                    self.changed();
                }
            }
            Verb::Map => {
                self.overlay = Overlay::Map;
                self.bump();
            }
        }
    }

    /// The disarm rule: any act other than working the seal stands
    /// an armed strike down.
    fn disarm_unless_seal(&mut self) {
        self.disarm();
    }

    /// Exhibit a FOE census row, narrating exposure when it lands.
    fn exhibit_foe_row(&mut self, row: usize) {
        let was = self.expedition.foe.state;
        match self.expedition.foe_exhibit_row(row) {
            Ok(text) => {
                if self.expedition.foe.state == FoeState::Exposed && was != FoeState::Exposed {
                    self.expedition
                        .log_push("The Borrowed Hoard stands exposed.");
                    self.say(
                        StatusKind::Good,
                        "The census is complete and the seam is exhibited: THE \
                         BORROWED HOARD STANDS EXPOSED. What remains is the Hoard \
                         itself — write it, and pass the gate.",
                    );
                } else {
                    self.say(StatusKind::Good, text);
                }
            }
            Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
        }
        self.changed();
    }

    /// Verify a FOE census row, narrating exposure when it lands.
    fn verify_foe_row(&mut self, row: usize) {
        let was = self.expedition.foe.state;
        match self.expedition.foe_verify_row(row) {
            Ok(()) => {
                if self.expedition.foe.state == FoeState::Exposed && was != FoeState::Exposed {
                    self.expedition
                        .log_push("The Borrowed Hoard stands exposed.");
                    self.say(
                        StatusKind::Good,
                        "The last verification lands and the census is complete: \
                         THE BORROWED HOARD STANDS EXPOSED.",
                    );
                } else {
                    self.say(StatusKind::Info, "Verified — the row stands as counted.");
                }
            }
            Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
        }
        self.changed();
    }

    /// Exhibit a Floor 2 audit row, narrating exposure when it lands.
    fn exhibit_foe2_row(&mut self, row: usize) {
        let was = self.expedition.floor2.foe.state;
        match self.expedition.floor2_foe_exhibit(row) {
            Ok(text) => {
                if self.expedition.floor2.foe.state == FoeState::Exposed && was != FoeState::Exposed
                {
                    self.expedition.log_push("The Menagerie stands exposed.");
                    self.say(
                        StatusKind::Good,
                        "The audit is complete and the Lens calibrated: THE \
                         MENAGERIE STANDS EXPOSED. What remains is the Desk \
                         itself — re-post the bound, and pass the gate.",
                    );
                } else {
                    self.say(StatusKind::Good, text);
                }
            }
            Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
        }
        self.changed();
    }

    /// Name a Floor 2 disagreement, narrating exposure when it lands.
    fn name_foe2_disagreement(&mut self, row: usize) {
        let was = self.expedition.floor2.foe.state;
        match self.expedition.floor2_foe_name_disagreement(row) {
            Ok(text) => {
                if self.expedition.floor2.foe.state == FoeState::Exposed && was != FoeState::Exposed
                {
                    self.expedition.log_push("The Menagerie stands exposed.");
                    self.say(
                        StatusKind::Good,
                        "The seam is named and the proof complete: THE MENAGERIE \
                         STANDS EXPOSED. What remains is the Desk itself — \
                         re-post the bound, and pass the gate.",
                    );
                } else {
                    self.say(StatusKind::Good, text);
                }
            }
            Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
        }
        self.changed();
    }

    /// Calibrate the Lens against the brazier, narrating
    /// exposure when the record completes the proof.
    fn calibrate_foe2(&mut self) {
        let was = self.expedition.floor2.foe.state;
        match self.expedition.floor2_foe_calibrate() {
            Ok(text) => {
                if self.expedition.floor2.foe.state == FoeState::Exposed && was != FoeState::Exposed
                {
                    self.expedition.log_push("The Menagerie stands exposed.");
                    self.say(
                        StatusKind::Good,
                        "The calibration lands and the audit is whole: THE \
                         MENAGERIE STANDS EXPOSED. What remains is the Desk \
                         itself — re-post the bound, and pass the gate.",
                    );
                } else {
                    self.say(StatusKind::Good, text);
                }
            }
            Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
        }
        self.changed();
    }
    // ------------------------------------------------------------------
    // Activation: every element's one behaviour.
    // ------------------------------------------------------------------

    /// Activate an element (click, tap, or keyboard confirm).
    pub fn activate(&mut self, el: El) {
        if !matches!(
            el,
            El::Seal | El::SealConfirm | El::SealCancel | El::Chip(Verb::Prove)
        ) {
            self.disarm_unless_seal();
        }
        match el {
            El::Begin => self.begin(),
            El::Continue => self.continue_expedition(),
            El::Setting(index) => self.apply_setting(index),
            El::TabFloor => self.enter_screen(Screen::Hub),
            El::TabMap => {
                self.overlay = Overlay::Map;
                self.bump();
            }
            El::TabLog => {
                self.overlay = Overlay::Log;
                self.bump();
            }
            El::TabCodex => {
                self.overlay = Overlay::Codex;
                self.bump();
            }
            El::PauseBtn => {
                self.overlay = Overlay::Pause;
                self.bump();
            }
            El::DoorChamber(index) => {
                let index = index as usize;
                if index > 0 && self.expedition.chambers[index - 1].state != ChamberState::Repaired
                {
                    self.say(
                        StatusKind::Refusal,
                        format!(
                            "The Oathwarden bars this door until {} stands repaired — \
                             the chambers teach in order.",
                            CHAMBERS[index - 1].name
                        ),
                    );
                } else {
                    self.enter_screen(Screen::Chamber(index));
                    self.say(
                        StatusKind::Info,
                        format!(
                            "Chamber {} — {}. {}",
                            index + 1,
                            CHAMBERS[index].name,
                            CHAMBERS[index].room
                        ),
                    );
                }
            }
            El::DoorFoe => {
                self.enter_screen(Screen::Foe);
                self.say(
                    StatusKind::Info,
                    "The Borrowed Hoard. Audit it, expose it, replace it, strike it.",
                );
            }
            El::DoorFoe2 => {
                self.enter_screen(Screen::Floor2Foe);
                self.say(
                    StatusKind::Info,
                    "The Menagerie at its desk. Audit the admissions, name the \
                     seam, calibrate the Lens, re-post the bound.",
                );
            }
            El::DoorStairs => {
                self.overlay = Overlay::Reading(ReadingKind::Stairs);
                self.bump();
            }
            El::DoorFloor4 => {
                let unmet = if self.expedition.floor4_reachable() {
                    None
                } else {
                    self.expedition
                        .stairs_clauses()
                        .iter()
                        .find(|(lit, _)| !lit)
                        .map(|(_, clause)| clause.clone())
                };
                match unmet {
                    None => {
                        self.enter_screen(Screen::Floor4Hub);
                        self.say(StatusKind::Info, content::FLOOR4_INTRO);
                    }
                    Some(clause) => {
                        self.say(
                            StatusKind::Refusal,
                            format!(
                                "The stairs answer by their inscription: \
                                 {clause} — not yet."
                            ),
                        );
                    }
                }
                self.bump();
            }

            El::DoorFloor3 => {
                if self.expedition.floor3_reachable() {
                    self.enter_screen(Screen::Floor3Hub);
                    self.say(
                        StatusKind::Info,
                        "Floor 3 — The Gatehouse. The Gatewright keeps the \
                         release office; every surface the guild sends out is \
                         inspected here.",
                    );
                } else {
                    self.say(
                        StatusKind::Refusal,
                        "The Gatehouse opens when both valley FOEs have fallen",
                    );
                }
                self.bump();
            }

            El::DoorFloor2 => {
                if self.expedition.floor2_reachable() {
                    self.enter_screen(Screen::Floor2Hub);
                    self.say(
                        StatusKind::Info,
                        "Floor 2 — The Menagerie Grounds. The Sigilwright keeps \
                         the roster; every beast is described by what it can do.",
                    );
                } else {
                    self.say(
                        StatusKind::Refusal,
                        "The way down is barred until the Borrowed Hoard has fallen",
                    );
                }
            }
            El::DoorFloor5 => {
                if self.expedition.floor5_reachable() {
                    self.enter_screen(Screen::Floor5Hub);
                    self.say(StatusKind::Info, content::FLOOR5_INTRO);
                } else {
                    self.say(
                        StatusKind::Refusal,
                        "The gauze door is barred until the Foundation \
                         Galleries stand complete — five chambers repaired \
                         and the Crooked Foundation fallen.",
                    );
                }
            }

            El::DoorFloor6 => {
                if self.expedition.floor6_reachable() {
                    self.enter_screen(Screen::Floor6Hub);
                    self.say(StatusKind::Info, content::FLOOR6_INTRO);
                } else {
                    self.say(
                        StatusKind::Refusal,
                        "The heart door is barred until the Veilworks \
                         stands charted through its third chamber — the \
                         Pointer Walks, the Two Tenants, and the Contract \
                         Cards repaired.",
                    );
                }
            }

            El::Floor4Door(index) => {
                let index = index as usize;
                self.enter_screen(Screen::Floor4Chamber(index));
                self.say(StatusKind::Info, content::FLOOR4_CHAMBER_INTROS[index]);
                self.bump();
            }

            El::Floor5Door(index) => {
                let index = index as usize;
                self.enter_screen(Screen::Floor5Chamber(index));
                self.say(StatusKind::Info, content::FLOOR5_CHAMBER_INTROS[index]);
                self.bump();
            }

            El::Floor6Door(index) => {
                let index = index as usize;
                self.enter_screen(Screen::Floor6Chamber(index));
                self.say(StatusKind::Info, content::FLOOR6_CHAMBER_INTROS[index]);
                self.bump();
            }

            El::Floor3Door(index) => {
                let index = index as usize;
                self.enter_screen(Screen::Floor3Chamber(index));
                self.say(StatusKind::Info, content::FLOOR3_CHAMBER_INTROS[index]);
                self.bump();
            }

            El::Floor2Door(index) => {
                let index = index as usize;
                if index <= 4
                    && (index == 0
                        || self.expedition.floor2.chambers[index - 1].state
                            == ChamberState::Repaired)
                {
                    self.enter_screen(Screen::Floor2Chamber(index));
                    self.say(StatusKind::Info, F2_DOOR_TEXTS[index]);
                } else if index <= 4 {
                    self.say(
                        StatusKind::Refusal,
                        format!(
                            "Chamber {} — {} stands barred until Chamber {} \
                             — {} is repaired: the grounds teach in order.",
                            index + 1,
                            content::MENAGERIE_CHAMBERS[index].name,
                            index,
                            content::MENAGERIE_CHAMBERS[index - 1].name
                        ),
                    );
                } else {
                    self.say(
                        StatusKind::Info,
                        format!(
                            "Chamber {} — {}: its build lands in a later step; \
                             the scaffolding shows its door.",
                            index + 1,
                            content::MENAGERIE_CHAMBERS[index].name
                        ),
                    );
                }
            }
            El::RosterToggle(group) => {
                self.toggle_roster_group(group as usize);
            }

            El::F4Crest(crest) => {
                if matches!(self.screen, Screen::Floor4Chamber(2)) {
                    let crest = crest as usize;
                    let work = &mut self.expedition.floor4.chambers[2].work;
                    work.crest_choices[crest] = match work.crest_choices[crest] {
                        None => Some(0),
                        Some(2) => Some(0),
                        Some(choice) => Some(choice + 1),
                    };
                    work.crest_proven[crest] = false;
                    let choice = work.crest_choices[crest].unwrap_or(0);
                    self.say(
                        StatusKind::Info,
                        format!(
                            "{}: cut as {}. Its price is not known until the line \
                             measures it.",
                            content::CRESTS[crest],
                            content::CREST_CANDIDATES[crest][choice]
                        ),
                    );
                    self.changed();
                }
            }
            El::F4CrestProve(crest) => {
                if matches!(self.screen, Screen::Floor4Chamber(2)) {
                    let crest = crest as usize;
                    let work = &self.expedition.floor4.chambers[2].work;
                    match work.crest_choices[crest] {
                        None => {
                            self.say(
                                StatusKind::Refusal,
                                "Choose the crest's cut before you ask the line to \
                                 measure it.",
                            );
                        }
                        Some(choice) => {
                            let (size, align) = content::crest_candidate_facts(crest, choice);
                            self.expedition.floor4.chambers[2].work.crest_proven[crest] = true;
                            self.say(
                                StatusKind::Info,
                                format!(
                                    "The line measures {}: size {size}, alignment \
                                     {align}. The stone is spent either way — the \
                                     seat keeps what the line finds.",
                                    content::CREST_CANDIDATES[crest][choice]
                                ),
                            );
                        }
                    }
                    self.changed();
                }
            }
            El::F4TreatySeat(stone) => {
                if matches!(self.screen, Screen::Floor4Chamber(3)) {
                    let work = &mut self.expedition.floor4.chambers[3].work;
                    if work.treaty_order.contains(&stone) {
                        self.say(
                            StatusKind::Info,
                            format!(
                                "{} already stands in the wall.",
                                content::TREATY_STONES[stone as usize].0
                            ),
                        );
                    } else if work.treaty_order.len() >= 4 {
                        self.say(
                            StatusKind::Refusal,
                            "The wall has four seats and all are filled.",
                        );
                    } else {
                        let seat = work.treaty_order.len();
                        work.treaty_order.push(stone);
                        work.treaty_proven = false;
                        self.say(
                            StatusKind::Info,
                            format!(
                                "{} takes seat {}, where the book promises offset {}.",
                                content::TREATY_STONES[stone as usize].0,
                                seat + 1,
                                content::TREATY_PROMISES[seat]
                            ),
                        );
                    }
                    self.changed();
                }
            }
            El::F4TreatyReset => {
                if matches!(self.screen, Screen::Floor4Chamber(3)) {
                    let work = &mut self.expedition.floor4.chambers[3].work;
                    work.treaty_order.clear();
                    work.treaty_proven = false;
                    self.say(
                        StatusKind::Info,
                        "The wall comes down; the stones return to the tray.",
                    );
                    self.changed();
                }
            }
            El::F4TreatyProve => {
                if matches!(self.screen, Screen::Floor4Chamber(3)) {
                    let order = self.expedition.floor4.chambers[3].work.treaty_order.clone();
                    if order.len() < 4 {
                        self.say(
                            StatusKind::Refusal,
                            "The line drops on a finished wall: four seats, no fewer.",
                        );
                    } else if order.contains(&4) {
                        self.say(
                            StatusKind::Refusal,
                            "The line stops at the free-cut arch: it is under no \
                             treaty and promises no offset. A wall that must be \
                             proven cannot stand on a stone that owes nothing.",
                        );
                    } else {
                        let dims: Vec<(u32, u32)> = order
                            .iter()
                            .map(|&stone| {
                                let (_, size, align) = content::TREATY_STONES[stone as usize];
                                (size, align)
                            })
                            .collect();
                        let offsets = puzzles::c_offsets(&dims);
                        self.expedition.floor4.chambers[3].work.treaty_proven = true;
                        let seats: Vec<String> = (0..4)
                            .map(|seat| {
                                let promised = content::TREATY_PROMISES[seat];
                                if offsets[seat] == promised {
                                    format!("seat {}: {}", seat + 1, offsets[seat])
                                } else {
                                    format!(
                                        "seat {}: the book says {promised}, the line \
                                         reads {}",
                                        seat + 1,
                                        offsets[seat]
                                    )
                                }
                            })
                            .collect();
                        self.say(
                            StatusKind::Info,
                            format!("The line reads the wall: {}.", seats.join(" · ")),
                        );
                    }
                    self.changed();
                }
            }
            El::F4PromiseUp => {
                if matches!(self.screen, Screen::Floor4Chamber(3)) {
                    let work = &mut self.expedition.floor4.chambers[3].work;
                    work.final_promise = (work.final_promise + 1).min(32);
                    let promise = work.final_promise;
                    self.say(
                        StatusKind::Info,
                        format!("The book's final promise now reads {promise}."),
                    );
                    self.changed();
                }
            }
            El::F4Fill(stone) => {
                if matches!(self.screen, Screen::Floor4Chamber(4)) {
                    let stone = stone as usize;
                    let work = &mut self.expedition.floor4.chambers[4].work;
                    work.fillings[stone] = (work.fillings[stone] + 1) % 3;
                    let (name, number, ratio) = content::HOLLOW_STONES[stone];
                    let text = match work.fillings[stone] {
                        1 => format!("{name} is filled with the whole number: {number}."),
                        2 => format!("{name} is filled with the ratio: {}.", ratio_text(ratio)),
                        _ => format!("{name} stands empty again."),
                    };
                    self.say(StatusKind::Info, text);
                    self.changed();
                }
            }
            El::F4Tag(stone) => {
                if matches!(self.screen, Screen::Floor4Chamber(4)) {
                    let stone = stone as usize;
                    let work = &mut self.expedition.floor4.chambers[4].work;
                    work.tags[stone] = (work.tags[stone] + 1) % 3;
                    let text = match work.tags[stone] {
                        1 => "The mason tags it in his head: a whole number.".to_string(),
                        2 => "The mason tags it in his head: a ratio.".to_string(),
                        _ => "The mason clears the tag from his head.".to_string(),
                    };
                    self.say(StatusKind::Info, text);
                    self.changed();
                }
            }
            El::F4Read(stone) => {
                if matches!(self.screen, Screen::Floor4Chamber(4)) {
                    let stone = stone as usize;
                    let work = &self.expedition.floor4.chambers[4].work;
                    let (name, number, ratio) = content::HOLLOW_STONES[stone];
                    let text = match puzzles::hollow_bits(work.fillings[stone], number, ratio) {
                        None => format!(
                            "{name} is empty: the tag claims over a cavity \
                             with nothing in it."
                        ),
                        Some(bits) => match work.tags[stone] {
                            0 => format!(
                                "{name} holds its filling untagged: the \
                                 mason has made no claim, and the stone \
                                 keeps its counsel."
                            ),
                            1 => format!(
                                "Read as the tag claims — a whole number: \
                                 {name} answers {bits}."
                            ),
                            _ => format!(
                                "Read as the tag claims — a ratio: {name} \
                                 answers {}.",
                                ratio_text(puzzles::hollow_as_ratio(bits))
                            ),
                        },
                    };
                    self.say(StatusKind::Info, text);
                    self.changed();
                }
            }
            El::F4Cell(cell) => {
                if matches!(self.screen, Screen::Floor4Chamber(4)) {
                    let cell = cell as usize;
                    let work = &mut self.expedition.floor4.chambers[4].work;
                    if work.blank_cells[cell] {
                        self.say(StatusKind::Info, "That cell already stands filled.");
                    } else {
                        work.blank_cells[cell] = true;
                        let filled = work.blank_cells.iter().filter(|c| **c).count();
                        self.say(
                            StatusKind::Info,
                            format!(
                                "The blank stone takes its filling: {filled} \
                                 of four cells stand full."
                            ),
                        );
                    }
                    self.changed();
                }
            }
            El::F4UseBlank => {
                if matches!(self.screen, Screen::Floor4Chamber(4)) {
                    let work = &mut self.expedition.floor4.chambers[4].work;
                    if work.blank_cells.iter().all(|cell| *cell) {
                        work.blank_used = true;
                        self.say(
                            StatusKind::Good,
                            "The blank is used whole: four cells, every one \
                             filled, and the gallery's referee nods it \
                             through.",
                        );
                    } else {
                        work.referee_catches += 1;
                        let filled = work.blank_cells.iter().filter(|c| **c).count();
                        let catches = work.referee_catches;
                        self.say(
                            StatusKind::Refusal,
                            format!(
                                "A whistle: Miri, the gallery's referee, \
                                 catches the use — {filled} of four cells \
                                 filled. The catch stands on the record \
                                 ({catches} so far). Fill the blank whole."
                            ),
                        );
                    }
                    self.changed();
                }
            }
            El::F5Lens(station) => {
                if matches!(self.screen, Screen::Floor5Chamber(4)) {
                    let station = station as usize;
                    let work = &mut self.expedition.floor5.chambers[4].work;
                    if let Some(slot) = work.lensed.get_mut(station) {
                        *slot = true;
                    }
                    match content::STATIONS.get(station) {
                        Some(meta) => match meta.fault {
                            Some(fault) => self.say(
                                StatusKind::Info,
                                format!(
                                    "Through the lens at {}: the fault shows \
                                     plain — {}.",
                                    meta.name,
                                    content::FAULT_KINDS[fault]
                                ),
                            ),
                            None => self.say(
                                StatusKind::Info,
                                format!(
                                    "Through the lens at {}: hands and \
                                     promise agree — nothing hunts here.",
                                    meta.name
                                ),
                            ),
                        },
                        None => self.say(StatusKind::Refusal, "No such station."),
                    }
                    self.changed();
                }
            }
            El::F5Verdict(station) => {
                if matches!(self.screen, Screen::Floor5Chamber(4)) {
                    let station = station as usize;
                    let work = &mut self.expedition.floor5.chambers[4].work;
                    let mut entered = None;
                    if let Some(slot) = work.verdicts.get_mut(station) {
                        *slot = match *slot {
                            0 => 1,
                            1 => 2,
                            _ => 1,
                        };
                        entered = Some(*slot);
                    }
                    match (content::STATIONS.get(station), entered) {
                        (Some(meta), Some(1)) => self.say(
                            StatusKind::Info,
                            format!("{} — verdict entered: sound.", meta.name),
                        ),
                        (Some(meta), Some(_)) => self.say(
                            StatusKind::Info,
                            format!(
                                "{} — verdict entered: faulted. If the \
                                 lens agrees, set its card to the one \
                                 true contract.",
                                meta.name
                            ),
                        ),
                        _ => self.say(StatusKind::Refusal, "No such station."),
                    }
                    self.changed();
                }
            }
            El::F5FixCard(station) => {
                if matches!(self.screen, Screen::Floor5Chamber(4)) {
                    let station = station as usize;
                    match content::STATIONS.get(station) {
                        Some(meta) if meta.fault.is_some() => {
                            let work = &mut self.expedition.floor5.chambers[4].work;
                            let mut showing = 0usize;
                            if let Some(slot) = work.cards.get_mut(station) {
                                *slot = (*slot + 1) % 3;
                                showing = *slot as usize;
                            }
                            self.say(
                                StatusKind::Info,
                                format!(
                                    "{}'s card now reads: {}",
                                    meta.name, meta.contracts[showing]
                                ),
                            );
                        }
                        Some(meta) => self.say(
                            StatusKind::Refusal,
                            format!("{} stands sound: it carries no card to rewrite.", meta.name),
                        ),
                        None => self.say(StatusKind::Refusal, "No such station."),
                    }
                    self.changed();
                }
            }
            El::F5Exhibit(exhibit) => {
                if self.screen == Screen::Floor5Foe {
                    match self.expedition.floor5_foe_exhibit(exhibit as usize) {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            El::F5Case(case) => {
                if self.screen == Screen::Floor5Foe {
                    match self.expedition.floor5_foe_walk_case(case as usize) {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            El::F5FoeCard(line) => {
                if self.screen == Screen::Floor5Foe {
                    let line = line as usize;
                    let foe = &mut self.expedition.floor5.foe;
                    let mut showing = None;
                    if let Some(slot) = foe.card.get_mut(line) {
                        *slot = (*slot + 1) % 3;
                        showing = Some(*slot as usize);
                    }
                    match (content::FOE5_CARD.get(line), showing) {
                        (Some(phrasings), Some(index)) => self.say(
                            StatusKind::Info,
                            format!("The beam's card, line {}: {}", line + 1, phrasings[index]),
                        ),
                        _ => self.say(StatusKind::Refusal, "The card has three lines."),
                    }
                    self.changed();
                }
            }
            El::F4BookTake => {
                if self.screen == Screen::Floor4Foe {
                    self.expedition.floor4_foe_take_book();
                    self.say(
                        StatusKind::Info,
                        "The book is in hand: five load-bearing entries. \
                         Drop the line on every stone it names.",
                    );
                    self.changed();
                }
            }
            El::F4Measure(entry) => {
                if self.screen == Screen::Floor4Foe {
                    match self.expedition.floor4_foe_measure(entry as usize) {
                        Ok(text) => self.say(StatusKind::Info, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            El::F4Exhibit(entry) => {
                if self.screen == Screen::Floor4Foe {
                    match self.expedition.floor4_foe_exhibit(entry as usize) {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            El::F4ForeignSeat(stone) => {
                if self.screen == Screen::Floor4Foe {
                    let foe = &mut self.expedition.floor4.foe;
                    if foe.foreign_order.contains(&stone) {
                        self.say(
                            StatusKind::Info,
                            format!(
                                "{} already stands in the set.",
                                content::RECUT_FOREIGN[stone as usize].0
                            ),
                        );
                    } else if foe.foreign_order.len() >= 4 {
                        self.say(
                            StatusKind::Refusal,
                            "The set has four seats and all are filled.",
                        );
                    } else {
                        foe.foreign_order.push(stone);
                        foe.foreign_proven = false;
                        foe.gate_passed = false;
                        let seat = foe.foreign_order.len();
                        self.say(
                            StatusKind::Info,
                            format!(
                                "{} takes seat {seat} of the foreign set.",
                                content::RECUT_FOREIGN[stone as usize].0
                            ),
                        );
                    }
                    self.changed();
                }
            }
            El::F4ForeignReset => {
                if self.screen == Screen::Floor4Foe {
                    let foe = &mut self.expedition.floor4.foe;
                    foe.foreign_order.clear();
                    foe.foreign_proven = false;
                    foe.gate_passed = false;
                    self.say(
                        StatusKind::Info,
                        "The foreign set is taken down; the tray is as it was.",
                    );
                    self.changed();
                }
            }
            El::F4ForeignProve => {
                if self.screen == Screen::Floor4Foe {
                    match self.expedition.floor4_foe_prove_foreign() {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            El::F4FreeSeat(stone) => {
                if self.screen == Screen::Floor4Foe {
                    let foe = &mut self.expedition.floor4.foe;
                    if foe.free_order.contains(&stone) {
                        self.say(
                            StatusKind::Info,
                            format!(
                                "{} already stands in the set.",
                                content::RECUT_FREE[stone as usize].0
                            ),
                        );
                    } else if foe.free_order.len() >= 4 {
                        self.say(
                            StatusKind::Refusal,
                            "The set has four seats and all are filled.",
                        );
                    } else {
                        foe.free_order.push(stone);
                        foe.free_proven = false;
                        foe.gate_passed = false;
                        let seat = foe.free_order.len();
                        self.say(
                            StatusKind::Info,
                            format!(
                                "{} takes seat {seat} of the free set.",
                                content::RECUT_FREE[stone as usize].0
                            ),
                        );
                    }
                    self.changed();
                }
            }
            El::F4FreeReset => {
                if self.screen == Screen::Floor4Foe {
                    let foe = &mut self.expedition.floor4.foe;
                    foe.free_order.clear();
                    foe.free_proven = false;
                    foe.gate_passed = false;
                    self.say(
                        StatusKind::Info,
                        "The free set is taken down; the tray is as it was.",
                    );
                    self.changed();
                }
            }
            El::F4FreeProve => {
                if self.screen == Screen::Floor4Foe {
                    match self.expedition.floor4_foe_prove_free() {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            El::F4PromiseDown => {
                if matches!(self.screen, Screen::Floor4Chamber(3)) {
                    let work = &mut self.expedition.floor4.chambers[3].work;
                    work.final_promise = work.final_promise.saturating_sub(1);
                    let promise = work.final_promise;
                    self.say(
                        StatusKind::Info,
                        format!("The book's final promise now reads {promise}."),
                    );
                    self.changed();
                }
            }

            El::F5Watch => {
                if let Screen::Floor5Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor5.chambers.get_mut(index) {
                        progress.work.watched = true;
                    }
                    self.say(
                        StatusKind::Info,
                        match index {
                            0 => content::WALK_CHARTER,
                            1 => content::TENANCY_CHARTER,
                            2 => content::CARD_CHARTER,
                            3 => content::BEAM_CHARTER,
                            4 => content::HUNT_CHARTER,
                            _ => "The Veilwalker nods at the gauze.",
                        },
                    );
                    self.changed();
                }
            }

            El::F5CardLine(tool, line) => {
                if let Screen::Floor5Chamber(2) = self.screen {
                    let tool = tool as usize;
                    let line = line as usize;
                    if let Some(slot) = self
                        .expedition
                        .floor5
                        .chambers
                        .get_mut(2)
                        .and_then(|progress| progress.work.card_lines.get_mut(tool))
                        .and_then(|lines| lines.get_mut(line))
                    {
                        *slot = (*slot + 1) % 3;
                        let phrasing = *slot;
                        let (name, lines, _) = &content::CARD_TOOLS[tool];
                        self.say(
                            StatusKind::Info,
                            format!(
                                "{} — line {} now reads: {}",
                                name,
                                line + 1,
                                lines[line][phrasing as usize]
                            ),
                        );
                    }
                    self.changed();
                }
            }

            El::F5SplitDown => {
                if let Screen::Floor5Chamber(3) = self.screen {
                    let message = {
                        let work = &mut self.expedition.floor5.chambers[3].work;
                        if work.split == 0 {
                            None
                        } else {
                            work.split -= 1;
                            if work.seamed {
                                work.seamed = false;
                                work.access_refused = false;
                                work.worked = [false, false];
                                work.dropped = [false, false];
                                Some(
                                    "A new seam chosen: the beam stands \
                                     unseamed again, and the referee's walk \
                                     is spent — it begins over at the saw."
                                        .to_string(),
                                )
                            } else {
                                Some(format!(
                                    "The chalked seam moves: {} word{} from \
                                     the left end.",
                                    work.split,
                                    if work.split == 1 { "" } else { "s" }
                                ))
                            }
                        }
                    };
                    match message {
                        None => self.say(
                            StatusKind::Info,
                            "The seam stands at the beam's left end already.",
                        ),
                        Some(text) => self.say(StatusKind::Info, text),
                    }
                    self.changed();
                }
            }

            El::F5SplitUp => {
                if let Screen::Floor5Chamber(3) = self.screen {
                    let message = {
                        let work = &mut self.expedition.floor5.chambers[3].work;
                        if work.split >= 6 {
                            None
                        } else {
                            work.split += 1;
                            if work.seamed {
                                work.seamed = false;
                                work.access_refused = false;
                                work.worked = [false, false];
                                work.dropped = [false, false];
                                Some(
                                    "A new seam chosen: the beam stands \
                                     unseamed again, and the referee's walk \
                                     is spent — it begins over at the saw."
                                        .to_string(),
                                )
                            } else {
                                Some(format!(
                                    "The chalked seam moves: {} word{} from \
                                     the left end.",
                                    work.split,
                                    if work.split == 1 { "" } else { "s" }
                                ))
                            }
                        }
                    };
                    match message {
                        None => self.say(
                            StatusKind::Info,
                            "The seam stands at the beam's right end already.",
                        ),
                        Some(text) => self.say(StatusKind::Info, text),
                    }
                    self.changed();
                }
            }

            El::F5Seam => {
                if let Screen::Floor5Chamber(3) = self.screen {
                    let work = &mut self.expedition.floor5.chambers[3].work;
                    if work.seamed {
                        self.say(
                            StatusKind::Info,
                            "The beam already stands seamed at this split.",
                        );
                    } else if work.split == 0 {
                        self.say(
                            StatusKind::Refusal,
                            "A seam at the very start leaves no left half \
                             at all: the face will not seam air.",
                        );
                    } else if work.split >= 6 {
                        self.say(
                            StatusKind::Refusal,
                            "A seam at the very end leaves no right half \
                             at all: the face will not seam air.",
                        );
                    } else {
                        work.seamed = true;
                        let (left, right) = puzzles::beam_halves(work.split);
                        self.say(
                            StatusKind::Good,
                            format!(
                                "Seamed: {} word{} to the left of the seam, \
                                 {} to the right. The seam's contract now \
                                 governs both halves.",
                                left.len(),
                                if left.len() == 1 { "" } else { "s" },
                                right.len()
                            ),
                        );
                    }
                    self.changed();
                }
            }

            El::F5Reach => {
                if let Screen::Floor5Chamber(3) = self.screen {
                    let work = &mut self.expedition.floor5.chambers[3].work;
                    if !work.seamed {
                        self.say(
                            StatusKind::Refusal,
                            "Seam the beam first: unseamed, there are no \
                             halves, and nothing for a hand to reach past.",
                        );
                    } else if work.access_refused {
                        self.say(
                            StatusKind::Info,
                            "The face has already refused the over-reach; \
                             the ruling stands on the record.",
                        );
                    } else {
                        work.access_refused = true;
                        self.say(
                            StatusKind::Refusal,
                            "A hand reaches one word past the left half's \
                             end — and the face refuses it, by the seam's \
                             card: a half ends where its contract says it \
                             ends. The ruling stands on the record.",
                        );
                    }
                    self.changed();
                }
            }

            El::F5WorkHalves => {
                if let Screen::Floor5Chamber(3) = self.screen {
                    let work = &mut self.expedition.floor5.chambers[3].work;
                    if !work.seamed {
                        self.say(
                            StatusKind::Refusal,
                            "Seam the beam first: the crews work halves, \
                             and unseamed there are none.",
                        );
                    } else if work.worked == [true, true] {
                        self.say(StatusKind::Info, "Both halves already stand worked.");
                    } else {
                        work.worked = [true, true];
                        self.say(
                            StatusKind::Good,
                            "Both crews work at once, one to each half; \
                             neither hand crosses the seam, and the beam \
                             holds under both.",
                        );
                    }
                    self.changed();
                }
            }

            El::F5DropLeft => {
                if let Screen::Floor5Chamber(3) = self.screen {
                    let work = &mut self.expedition.floor5.chambers[3].work;
                    if !work.seamed {
                        self.say(
                            StatusKind::Refusal,
                            "Seam the beam first: there is nothing to drop \
                             in order until the halves exist.",
                        );
                    } else if work.worked != [true, true] {
                        self.say(
                            StatusKind::Refusal,
                            "Work both halves first: the drop is the \
                             walk's last ruling, not its first.",
                        );
                    } else if work.dropped[0] {
                        self.say(StatusKind::Info, "The left half already lies on the floor.");
                    } else {
                        work.dropped[0] = true;
                        self.say(
                            StatusKind::Good,
                            "The left half comes down first, in the order \
                             the seam's card prescribes.",
                        );
                    }
                    self.changed();
                }
            }

            El::F5DropRight => {
                if let Screen::Floor5Chamber(3) = self.screen {
                    let work = &mut self.expedition.floor5.chambers[3].work;
                    if !work.seamed {
                        self.say(
                            StatusKind::Refusal,
                            "Seam the beam first: there is nothing to drop \
                             in order until the halves exist.",
                        );
                    } else if !work.dropped[0] {
                        self.say(
                            StatusKind::Refusal,
                            "The card's order is left first: a right-first \
                             drop is refused, with the reason on the card \
                             — the left half still stands on its trestle.",
                        );
                    } else if work.dropped[1] {
                        self.say(
                            StatusKind::Info,
                            "The right half already lies on the floor.",
                        );
                    } else {
                        work.dropped[1] = true;
                        self.say(
                            StatusKind::Good,
                            "The right half comes down after the left. The \
                             drop order stands kept, and the beam's walk \
                             is complete.",
                        );
                    }
                    self.changed();
                }
            }

            El::F5Card(step) => {
                if let Screen::Floor5Chamber(0) = self.screen {
                    let step = step as usize;
                    self.expedition.floor5.chambers[0].work.cards_read[step] = true;
                    let (card, seen, _) = content::STEP_CARDS[step];
                    self.say(
                        StatusKind::Info,
                        format!("Card {} — {card} The yard shows: {seen}.", step + 1),
                    );
                    self.changed();
                }
            }

            El::F5Walk => {
                if let Screen::Floor5Chamber(0) = self.screen {
                    let (walked, read) = {
                        let work = &self.expedition.floor5.chambers[0].work;
                        (
                            work.walked,
                            work.cards_read.get(work.walked).copied().unwrap_or(false),
                        )
                    };
                    if walked >= 6 {
                        self.say(StatusKind::Info, "The path stands walked.");
                    } else if !read {
                        self.say(
                            StatusKind::Refusal,
                            "Read the card first: the walker steps on read \
                             cards only — an unread step is a guess with \
                             your weight on it.",
                        );
                    } else if puzzles::step_cracked(walked) {
                        self.say(
                            StatusKind::Refusal,
                            "The pole rings the stone and it gives a dull, \
                             split note — the footing fails its card. This \
                             step is not walked; it is refused, by name.",
                        );
                    } else {
                        self.expedition.floor5.chambers[0].work.walked += 1;
                        let now = self.expedition.floor5.chambers[0].work.walked;
                        self.say(
                            StatusKind::Info,
                            if now == 6 {
                                "The last stone takes her weight true. The \
                                 path stands walked."
                                    .to_string()
                            } else {
                                format!("Step {} stands. The yard holds.", now)
                            },
                        );
                    }
                    self.changed();
                }
            }

            El::F5RefuseStep => {
                if let Screen::Floor5Chamber(0) = self.screen {
                    let (walked, read) = {
                        let work = &self.expedition.floor5.chambers[0].work;
                        (
                            work.walked,
                            work.cards_read.get(work.walked).copied().unwrap_or(false),
                        )
                    };
                    if walked >= 6 {
                        self.say(StatusKind::Info, "The path stands walked.");
                    } else if !read {
                        self.say(
                            StatusKind::Refusal,
                            "Read the card first: a refusal, like a step, \
                             is made on the card — not on a hunch.",
                        );
                    } else if puzzles::step_cracked(walked) {
                        let work = &mut self.expedition.floor5.chambers[0].work;
                        work.refused = true;
                        work.walked += 1;
                        self.say(
                            StatusKind::Good,
                            "Refused by name: the cracked stone is stepped \
                             around, its card honoured in the only way it \
                             can be. The refusal stands on the record.",
                        );
                    } else {
                        self.say(
                            StatusKind::Refusal,
                            "This footing holds: the yard rings true and \
                             the card allows the step. Refusing a sound \
                             step is not caution — it is a misread card.",
                        );
                    }
                    self.changed();
                }
            }

            El::F5Placard(index) => {
                if let Screen::Floor5Chamber(1) = self.screen {
                    let index = index as usize;
                    if self.expedition.floor5.chambers[1].work.judged {
                        self.say(
                            StatusKind::Info,
                            "The referee has stamped the calls; what \
                             remains is the annotations.",
                        );
                    } else if let Some(slot) = self.expedition.floor5.chambers[1]
                        .work
                        .predictions
                        .get_mut(index)
                    {
                        *slot = (*slot + 1) % 3;
                        let word = match *slot {
                            1 => "SOUND",
                            2 => "UNSOUND",
                            _ => "uncalled",
                        };
                        self.say(
                            StatusKind::Info,
                            format!("Tenancy {} called {word}.", index + 1),
                        );
                    }
                    self.changed();
                }
            }

            El::F5Judge => {
                if let Screen::Floor5Chamber(1) = self.screen {
                    let work = &mut self.expedition.floor5.chambers[1].work;
                    if work.judged {
                        self.say(
                            StatusKind::Info,
                            "The referee has already stamped this set.",
                        );
                    } else if work.predictions.contains(&0) {
                        self.say(
                            StatusKind::Refusal,
                            "Call every placard first: the referee stamps \
                             a full set, or none.",
                        );
                    } else {
                        work.judged = true;
                        let correct = puzzles::tenancy_correct(&work.predictions);
                        self.say(
                            StatusKind::Info,
                            format!(
                                "The referee stamps the set: {correct} of 8 \
                                 true. {}",
                                if correct >= 7 {
                                    "The calls stand — annotate every miss \
                                     with its deciding rule."
                                } else {
                                    "Below the bar of seven: reset the \
                                     chamber and call them again."
                                }
                            ),
                        );
                    }
                    self.changed();
                }
            }

            El::F5Annotate(index) => {
                if let Screen::Floor5Chamber(1) = self.screen {
                    let index = index as usize;
                    let missed = {
                        let work = &self.expedition.floor5.chambers[1].work;
                        work.judged && puzzles::tenancy_missed(&work.predictions, index)
                    };
                    if !missed {
                        self.say(
                            StatusKind::Refusal,
                            "Only a stamped miss is annotated: the true \
                             calls need no rule named under them.",
                        );
                    } else {
                        let work = &mut self.expedition.floor5.chambers[1].work;
                        let next = match work.annotations[index] {
                            None => Some(0),
                            Some(rule) if rule + 1 < content::TENANCY_RULES.len() => Some(rule + 1),
                            Some(_) => None,
                        };
                        work.annotations[index] = next;
                        self.say(
                            StatusKind::Info,
                            match next {
                                Some(rule) => format!(
                                    "Tenancy {} annotated: {}",
                                    index + 1,
                                    content::TENANCY_RULES[rule]
                                ),
                                None => format!("Tenancy {} annotation cleared.", index + 1),
                            },
                        );
                    }
                    self.changed();
                }
            }

            El::F6DayOp(op) => {
                if let Screen::Floor6Chamber(0) = self.screen {
                    let op = op as usize;
                    let walked = self.expedition.floor6.chambers[0].work.day_walked;
                    if walked.get(op).copied().unwrap_or(false) {
                        self.say(
                            StatusKind::Info,
                            "That operation already stands on the day.",
                        );
                    } else if walked.iter().take(op).any(|done| !*done) {
                        self.say(
                            StatusKind::Refusal,
                            "The day is walked in order: an operation \
                             stands only on the ones before it.",
                        );
                    } else {
                        self.expedition.floor6.chambers[0].work.day_walked[op] = true;
                        let walked = self.expedition.floor6.chambers[0].work.day_walked;
                        let (state, refused) = puzzles::cell_day_replay(&walked);
                        let (porter, action, _) = content::CELL_DAY[op];
                        if refused == Some(op) {
                            self.expedition.floor6.chambers[0].work.bell_struck = true;
                            self.say(
                                StatusKind::Good,
                                format!(
                                    "{porter} {action} — and the heart refuses: \
                                     a share still stands held. The bell strikes, \
                                     and the refusal stands on the record."
                                ),
                            );
                        } else {
                            self.say(
                                StatusKind::Info,
                                format!(
                                    "{porter} {action} — the heart stands {}.",
                                    puzzles::flag_state_name(state)
                                ),
                            );
                        }
                    }
                    self.changed();
                }
            }

            El::F6BrazierCall(call) => {
                if let Screen::Floor6Chamber(1) = self.screen {
                    let call = call as usize;
                    let calls = self.expedition.floor6.chambers[1].work.calls;
                    if calls.get(call).copied().unwrap_or(false) {
                        let (_, count) = puzzles::brazier_replay(&calls);
                        self.say(
                            StatusKind::Info,
                            format!(
                                "That call already stands. The brazier has \
                                 been kindled {count} time in all."
                            ),
                        );
                    } else if calls.iter().take(call).any(|done| !*done) {
                        self.say(
                            StatusKind::Refusal,
                            "The calls are made in order: a call stands \
                             only on the ones before it.",
                        );
                    } else {
                        self.expedition.floor6.chambers[1].work.calls[call] = true;
                        let calls = self.expedition.floor6.chambers[1].work.calls;
                        let (lit, count) = puzzles::brazier_replay(&calls);
                        self.say(
                            StatusKind::Info,
                            format!(
                                "{} — {}.",
                                content::BRAZIER_CALLS[call],
                                if lit {
                                    format!(
                                        "the flame stands lit; kindled {count} \
                                         time in all"
                                    )
                                } else {
                                    "the brazier stands dark".to_string()
                                }
                            ),
                        );
                    }
                    self.changed();
                }
            }

            El::F6BoundUp => {
                if let Screen::Floor6Chamber(2) = self.screen {
                    let bound = {
                        let work = &mut self.expedition.floor6.chambers[2].work;
                        if work.spin_bound < 9 {
                            work.spin_bound += 1;
                            work.gave_up = false;
                        }
                        work.spin_bound
                    };
                    self.say(StatusKind::Info, format!("The bound stands at {bound}."));
                    self.changed();
                }
            }

            El::F6BoundDown => {
                if let Screen::Floor6Chamber(2) = self.screen {
                    let bound = {
                        let work = &mut self.expedition.floor6.chambers[2].work;
                        if work.spin_bound > 0 {
                            work.spin_bound -= 1;
                            work.gave_up = false;
                        }
                        work.spin_bound
                    };
                    self.say(
                        StatusKind::Info,
                        if bound == 0 {
                            "The bound is lifted: the heart is unbounded \
                             again — and unproven."
                                .to_string()
                        } else {
                            format!("The bound stands at {bound}.")
                        },
                    );
                    self.changed();
                }
            }

            El::F6SpinRun => {
                if let Screen::Floor6Chamber(2) = self.screen {
                    let bound = self.expedition.floor6.chambers[2].work.spin_bound;
                    if bound == 0 {
                        self.say(
                            StatusKind::Refusal,
                            "Set the bound first: an unbounded spin is \
                             not a heart, it is a hazard with a handle.",
                        );
                    } else {
                        let (attempts, acquired) = puzzles::spin_heart_run(bound, false);
                        self.expedition.floor6.chambers[2].work.gave_up = !acquired;
                        self.say(
                            StatusKind::Good,
                            format!(
                                "The heart spins: {attempts} attempts, and \
                                 the flag never rises — the setter is \
                                 another porter's work, and she never \
                                 comes. At its bound the heart gives up, \
                                 honestly and on the record."
                            ),
                        );
                    }
                    self.changed();
                }
            }

            El::F6Annotate(claim) => {
                if let Screen::Floor6Chamber(2) = self.screen {
                    let claim = claim as usize;
                    if let Some(slot) = self.expedition.floor6.chambers[2]
                        .work
                        .spin_annotations
                        .get_mut(claim)
                    {
                        *slot = (*slot + 1) % 4;
                        let choice = *slot;
                        self.say(
                            StatusKind::Info,
                            if choice == 0 {
                                format!("Claim {} annotation cleared.", claim + 1)
                            } else {
                                let ordering = content::ORDERING_NAMES
                                    [content::SPIN_CLAIMS[claim].1[choice as usize - 1]];
                                format!(
                                    "Claim {} annotated: {ordering} — recorded, \
                                     provisional until the Loom Halls.",
                                    claim + 1
                                )
                            },
                        );
                    }
                    self.changed();
                }
            }

            El::F6CensusCycle(class) => {
                if let Screen::Floor6Chamber(3) = self.screen {
                    let class = class as usize;
                    let work = &mut self.expedition.floor6.chambers[3].work;
                    if let Some(slot) = work.census.get_mut(class) {
                        *slot = (*slot + 1) % 6;
                        work.census_judged = false;
                        let count = *slot;
                        self.say(
                            StatusKind::Info,
                            format!("{} counted at {count}.", content::CENSUS_CLASSES[class]),
                        );
                    }
                    self.changed();
                }
            }

            El::F6CensusSubmit => {
                if let Screen::Floor6Chamber(3) = self.screen {
                    let census = self.expedition.floor6.chambers[3].work.census;
                    if census == puzzles::day_census() {
                        self.expedition.floor6.chambers[3].work.census_judged = true;
                        self.say(
                            StatusKind::Good,
                            "The slate tallies with the day: every state \
                             the heart can truly reach, counted once, no \
                             more conjured. The census stands judged true.",
                        );
                    } else {
                        self.say(
                            StatusKind::Info,
                            "The slate disagrees with the day. Count \
                             again — every ordering of every turn, every \
                             refusal standing as itself — and submit \
                             again; a wrong census costs nothing but \
                             the recount.",
                        );
                    }
                    self.changed();
                }
            }

            El::F6Choose(datum) => {
                if let Screen::Floor6Chamber(4) = self.screen {
                    let datum = datum as usize;
                    if let Some(slot) = self.expedition.floor6.chambers[4]
                        .work
                        .choices
                        .get_mut(datum)
                    {
                        *slot = (*slot + 1) % 4;
                        let choice = *slot;
                        self.say(
                            StatusKind::Info,
                            if choice == 0 {
                                format!("{} stands unhoused.", content::CELL_DATA[datum].name)
                            } else {
                                format!(
                                    "{} is housed in {}.",
                                    content::CELL_DATA[datum].name,
                                    content::CELL_KINDS[choice as usize - 1]
                                )
                            },
                        );
                    }
                    self.changed();
                }
            }

            El::F6Invariant(datum) => {
                if let Screen::Floor6Chamber(4) = self.screen {
                    let datum = datum as usize;
                    if let Some(slot) = self.expedition.floor6.chambers[4]
                        .work
                        .invariants
                        .get_mut(datum)
                    {
                        *slot = (*slot + 1) % 4;
                        let choice = *slot;
                        self.say(
                            StatusKind::Info,
                            if choice == 0 {
                                format!(
                                    "{}'s door stands unwritten.",
                                    content::CELL_DATA[datum].name
                                )
                            } else {
                                format!(
                                    "{}'s door now reads: {}",
                                    content::CELL_DATA[datum].name,
                                    content::CELL_DATA[datum].sentences[choice as usize - 1]
                                )
                            },
                        );
                    }
                    self.changed();
                }
            }

            El::F6Exhibit(exhibit) => {
                if self.screen == Screen::Floor6Foe {
                    match self.expedition.floor6_foe_exhibit(exhibit as usize) {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }

            El::F6Loom(case) => {
                if self.screen == Screen::Floor6Foe {
                    match self.expedition.floor6_foe_walk_case(case as usize) {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }

            El::F6FoeCard(line) => {
                if self.screen == Screen::Floor6Foe {
                    let line = line as usize;
                    let foe = &mut self.expedition.floor6.foe;
                    let mut showing = None;
                    if let Some(slot) = foe.card_lines.get_mut(line) {
                        *slot = (*slot + 1) % 3;
                        showing = Some(*slot as usize);
                    }
                    match (content::FOE6_CARD.get(line), showing) {
                        (Some(phrasings), Some(index)) => self.say(
                            StatusKind::Info,
                            format!("The staff's card, line {}: {}", line + 1, phrasings[index]),
                        ),
                        _ => self.say(StatusKind::Refusal, "The card has three lines."),
                    }
                    self.changed();
                }
            }

            El::F6Watch => {
                if let Screen::Floor6Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor6.chambers.get_mut(index) {
                        progress.work.watched = true;
                    }
                    self.say(
                        StatusKind::Info,
                        match index {
                            0 => content::CELL_CHARTER,
                            1 => content::BRAZIER_CHARTER,
                            2 => content::SPIN_CHARTER,
                            3 => content::CENSUS_CHARTER,
                            4 => content::CHOOSING_CHARTER,
                            _ => "Lockheart nods at the strongboxes.",
                        },
                    );
                    self.changed();
                }
            }

            El::F4Watch => {
                if let Screen::Floor4Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor4.chambers.get_mut(index) {
                        progress.work.watched = true;
                    }
                    self.say(
                        StatusKind::Info,
                        match index {
                            0 => content::BENCH_CHARTER,
                            1 => content::WALL_CHARTER,
                            2 => content::CREST_CHARTER,
                            3 => content::TREATY_CHARTER,
                            4 => content::HOLLOW_CHARTER,
                            _ => "The Plumbmason nods at the gallery.",
                        },
                    );
                    self.changed();
                }
            }
            El::F4Stone(stone) => {
                if self.screen == Screen::Floor4Chamber(0) {
                    self.f4_stone = stone;
                    self.say(StatusKind::Info, content::BENCH_STONES[stone as usize]);
                    self.bump();
                }
            }
            El::F4SizeUp => {
                if self.screen == Screen::Floor4Chamber(0) {
                    let stone = self.f4_stone as usize;
                    let work = &mut self.expedition.floor4.chambers[0].work;
                    work.pred_size[stone] = Some(match work.pred_size[stone] {
                        None => 1,
                        Some(64) => 1,
                        Some(size) => size + 1,
                    });
                    self.changed();
                }
            }
            El::F4SizeDown => {
                if self.screen == Screen::Floor4Chamber(0) {
                    let stone = self.f4_stone as usize;
                    let work = &mut self.expedition.floor4.chambers[0].work;
                    work.pred_size[stone] = Some(match work.pred_size[stone] {
                        None => 64,
                        Some(1) => 64,
                        Some(size) => size - 1,
                    });
                    self.changed();
                }
            }
            El::F4AlignUp => {
                if self.screen == Screen::Floor4Chamber(0) {
                    let stone = self.f4_stone as usize;
                    let work = &mut self.expedition.floor4.chambers[0].work;
                    work.pred_align[stone] = Some(match work.pred_align[stone] {
                        None => 1,
                        Some(1) => 2,
                        Some(2) => 4,
                        Some(4) => 8,
                        Some(8) => 16,
                        Some(_) => 1,
                    });
                    self.changed();
                }
            }
            El::F4AlignDown => {
                if self.screen == Screen::Floor4Chamber(0) {
                    let stone = self.f4_stone as usize;
                    let work = &mut self.expedition.floor4.chambers[0].work;
                    work.pred_align[stone] = Some(match work.pred_align[stone] {
                        None => 16,
                        Some(16) => 8,
                        Some(8) => 4,
                        Some(4) => 2,
                        Some(2) => 1,
                        Some(_) => 16,
                    });
                    self.changed();
                }
            }
            El::F4DropLine => {
                if self.screen == Screen::Floor4Chamber(0) {
                    let stone = self.f4_stone as usize;
                    let said = {
                        let work = &mut self.expedition.floor4.chambers[0].work;
                        match (work.pred_size[stone], work.pred_align[stone]) {
                            (Some(size), Some(align)) => {
                                work.measured[stone] = true;
                                let (true_size, true_align) = content::bench_stone_facts(stone);
                                if (size, align) == (true_size, true_align) {
                                    (
                                        StatusKind::Good,
                                        format!(
                                            "The line falls: size {true_size}, \
                                             alignment {true_align} — the slate \
                                             is true."
                                        ),
                                    )
                                } else {
                                    (
                                        StatusKind::Info,
                                        format!(
                                            "The line falls: size {true_size}, \
                                             alignment {true_align} — the slate \
                                             said {size} and {align}. A \
                                             surprise: annotate it."
                                        ),
                                    )
                                }
                            }
                            _ => (
                                StatusKind::Refusal,
                                "The slate is blank for this stone: commit \
                                 a size and an alignment before the line \
                                 may drop."
                                    .to_string(),
                            ),
                        }
                    };
                    let (kind, text) = said;
                    self.say(kind, text);
                    self.changed();
                }
            }
            El::F4Annotate => {
                if self.screen == Screen::Floor4Chamber(0) {
                    let stone = self.f4_stone as usize;
                    let said = {
                        let work = &mut self.expedition.floor4.chambers[0].work;
                        if !work.measured[stone] {
                            None
                        } else {
                            work.annotate[stone] = Some(match work.annotate[stone] {
                                None => 0,
                                Some(2) => 0,
                                Some(choice) => choice + 1,
                            });
                            work.annotate[stone].map(|choice| content::BENCH_CAUSES[stone][choice])
                        }
                    };
                    match said {
                        Some(cause) => self.say(StatusKind::Info, cause),
                        None => self.say(
                            StatusKind::Refusal,
                            "There is nothing to annotate until the line \
                             has dropped on this stone.",
                        ),
                    }
                    self.changed();
                }
            }
            El::F4Seat(stone) => {
                if self.screen == Screen::Floor4Chamber(1) {
                    let said = {
                        let work = &mut self.expedition.floor4.chambers[1].work;
                        if work.wall_order.contains(&stone) {
                            (
                                StatusKind::Info,
                                "That stone already stands in the wall.".to_string(),
                            )
                        } else if work.wall_order.len() >= content::WALL_STONES.len() {
                            (
                                StatusKind::Info,
                                "The wall is full: five stones stand in it.".to_string(),
                            )
                        } else {
                            work.wall_order.push(stone);
                            let seated: Vec<(u32, u32)> = work
                                .wall_order
                                .iter()
                                .map(|seated| {
                                    let (_, size, align) = content::WALL_STONES[*seated as usize];
                                    (size, align)
                                })
                                .collect();
                            let (total, voids) = crate::puzzles::wall_layout(&seated);
                            (
                                StatusKind::Info,
                                format!(
                                    "Seated. The wall as it stands: span \
                                     {total}, voids {voids}."
                                ),
                            )
                        }
                    };
                    let (kind, text) = said;
                    self.say(kind, text);
                    self.changed();
                }
            }
            El::F4WallReset => {
                if self.screen == Screen::Floor4Chamber(1) {
                    self.expedition.floor4.chambers[1].work.wall_order.clear();
                    self.say(
                        StatusKind::Info,
                        "The wall is cleared: the five stones stand on \
                         the bench again.",
                    );
                    self.changed();
                }
            }

            El::F3Watch => {
                if let Screen::Floor3Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor3.chambers.get_mut(index) {
                        progress.work.watched = true;
                    }
                    self.say(
                        StatusKind::Info,
                        match index {
                            0 => {
                                "The charter of the Named Failures desk: every \
                                  failure leaves by a named door, with its \
                                  remedy written under the name."
                            }
                            1 => {
                                "The belt's charter: portion, join, gate — \
                                  in that order — and the desk runs the chain \
                                  itself."
                            }
                            2 => {
                                "The booth's charter: what a traveller causes \
                                  leaves by a named door; what only the house \
                                  can cause may stand — with its invariant \
                                  named."
                            }
                            3 => {
                                "The rail's charter: unaddressed, addressed, \
                                  bodied, sealed — and the press knows the \
                                  order in iron."
                            }
                            _ => {
                                "The wall's charter: answer every line \
                                  honestly, name the set-aside line's reason, \
                                  and stamp the oldest road."
                            }
                        },
                    );
                    self.changed();
                }
            }
            El::F3Pair(report) => {
                if self.screen == Screen::Floor3Chamber(0) {
                    let report = report as usize;
                    let next = match self.expedition.floor3.chambers[0]
                        .work
                        .pair_choices
                        .get(report)
                    {
                        Some(Some(4)) | None | Some(None) => Some(0),
                        Some(Some(choice)) => Some(choice + 1),
                    };
                    if let Some(slot) = self.expedition.floor3.chambers[0]
                        .work
                        .pair_choices
                        .get_mut(report)
                    {
                        *slot = next;
                    }
                    if let Some(pair) = next {
                        let (kind, remedy) = content::FAILURE_PAIRS[pair];
                        self.say(
                            StatusKind::Info,
                            if pair == 4 {
                                "The Gatewright's voice, mildly: 'That bin is \
                                 emptied into the river at night.'"
                                    .to_string()
                            } else {
                                format!("Filed under {kind} — {remedy}")
                            },
                        );
                    }
                    self.changed();
                }
            }
            El::F3Station(slot) => {
                if self.screen == Screen::Floor3Chamber(1) {
                    let slot = slot as usize;
                    let work = &mut self.expedition.floor3.chambers[1].work;
                    let choice = match slot {
                        0 => &mut work.portion_choice,
                        1 => &mut work.dedup_choice,
                        _ => &mut work.halt_choice,
                    };
                    *choice = Some(match *choice {
                        None => 0,
                        Some(2) => 0,
                        Some(choice) => choice + 1,
                    });
                    let picked = *choice;
                    let work = &self.expedition.floor3.chambers[1].work;
                    let (options, picked): (&[&str; 3], usize) = match slot {
                        0 => (&content::PORTION_OPTIONS, picked.unwrap_or(0)),
                        1 => (&content::DEDUP_OPTIONS, picked.unwrap_or(0)),
                        _ => (&content::HALT_OPTIONS, picked.unwrap_or(0)),
                    };
                    let mut line = options[picked].to_string();
                    if let (Some(portion), Some(dedup), Some(halt)) =
                        (work.portion_choice, work.dedup_choice, work.halt_choice)
                    {
                        let (out, _) = puzzles::conveyor_run(portion, dedup, halt);
                        line = format!("{line}. The belt would deliver: {}", out.join(" · "));
                    }
                    self.say(StatusKind::Info, line);
                    self.changed();
                }
            }

            El::F3Shout(case) => {
                if self.screen == Screen::Floor3Chamber(2) {
                    let case = case as usize;
                    let verdict = {
                        let work = &mut self.expedition.floor3.chambers[2].work;
                        match work.shout_choices.get_mut(case) {
                            Some(slot) => {
                                *slot = Some(match *slot {
                                    None => 0,
                                    Some(0) => 1,
                                    Some(_) => 0,
                                });
                                *slot
                            }
                            None => None,
                        }
                    };
                    if verdict.is_some() {
                        self.say(
                            StatusKind::Info,
                            match verdict {
                                Some(0) => {
                                    "A named refusal: the traveller is told \
                                            what was wrong, and may correct it."
                                }
                                _ => "The shout stands — if its invariant can be named.",
                            },
                        );
                    }
                    self.changed();
                }
            }
            El::F3Invariant(case) => {
                if self.screen == Screen::Floor3Chamber(2) {
                    let case = case as usize;
                    let picked = {
                        let work = &mut self.expedition.floor3.chambers[2].work;
                        match work.invariant_choices.get_mut(case) {
                            Some(slot) => {
                                *slot = Some(match *slot {
                                    None => 0,
                                    Some(3) => 0,
                                    Some(choice) => choice + 1,
                                });
                                *slot
                            }
                            None => None,
                        }
                    };
                    if let Some(choice) = picked {
                        self.say(StatusKind::Info, content::INVARIANT_CANDIDATES[choice]);
                    }
                    self.changed();
                }
            }
            El::F3Slip(slip, action) => {
                if self.screen == Screen::Floor3Chamber(3) {
                    match self
                        .expedition
                        .floor3_slip_act(slip as usize, action as usize)
                    {
                        Ok(text) => self.say(StatusKind::Info, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }

            El::DoorFoe3 => {
                if self.expedition.floor3.foe.state == FoeState::Fallen {
                    self.say(
                        StatusKind::Info,
                        "The great door stands open and empty: the Release \
                         has left, and its lantern burns honest without it.",
                    );
                } else {
                    self.enter_screen(Screen::Floor3Foe);
                    self.say(StatusKind::Info, content::FOE3_INTRO);
                }
                self.bump();
            }
            El::DoorFoe6 => {
                if self.expedition.floor6.foe.state == FoeState::Fallen {
                    self.say(
                        StatusKind::Info,
                        "The curtain hangs still: the staff stands \
                         re-forged in its cradle, and the heart is \
                         locked once, and truly.",
                    );
                } else {
                    self.enter_screen(Screen::Floor6Foe);
                    self.say(StatusKind::Info, content::FOE6_INTRO);
                }
                self.bump();
            }

            El::DoorFoe5 => {
                if self.expedition.floor5.foe.state == FoeState::Fallen {
                    self.say(
                        StatusKind::Info,
                        "The gauze hangs empty: the beam stands \
                         unborrowed in its frame, and nothing moves \
                         behind the veil.",
                    );
                } else {
                    self.enter_screen(Screen::Floor5Foe);
                    self.say(StatusKind::Info, content::FOE5_INTRO);
                }
                self.bump();
            }
            El::DoorFoe4 => {
                if self.expedition.floor4.foe.state == FoeState::Fallen {
                    self.say(
                        StatusKind::Info,
                        "The master foundation stands re-laid and level: \
                         the galleries settle no more, and the line hangs \
                         quiet beside honest stones.",
                    );
                } else {
                    self.enter_screen(Screen::Floor4Foe);
                    self.say(StatusKind::Info, content::FOE4_INTRO);
                }
                self.bump();
            }
            El::F3Check(item) => {
                if self.screen == Screen::Floor3Chamber(4) {
                    let item = item as usize;
                    let work = &mut self.expedition.floor3.chambers[4].work;
                    if let Some(slot) = work.checklist_choices.get_mut(item) {
                        *slot = Some(match *slot {
                            None => 0,
                            Some(2) => 0,
                            Some(choice) => choice + 1,
                        });
                        if let Some(choice) = *slot {
                            self.say(StatusKind::Info, content::INSPECTION_VERDICTS[choice]);
                        }
                    }
                    self.changed();
                }
            }
            El::F3NaReason => {
                if self.screen == Screen::Floor3Chamber(4) {
                    let work = &mut self.expedition.floor3.chambers[4].work;
                    work.na_reason = Some(match work.na_reason {
                        None => 0,
                        Some(2) => 0,
                        Some(choice) => choice + 1,
                    });
                    if let Some(choice) = work.na_reason {
                        self.say(StatusKind::Info, content::NA_REASONS[choice]);
                    }
                    self.changed();
                }
            }
            El::F3Msrv => {
                if self.screen == Screen::Floor3Chamber(4) {
                    let work = &mut self.expedition.floor3.chambers[4].work;
                    work.msrv_choice = Some(match work.msrv_choice {
                        None => 0,
                        Some(2) => 0,
                        Some(choice) => choice + 1,
                    });
                    if let Some(choice) = work.msrv_choice {
                        self.say(StatusKind::Info, content::MSRV_CANDIDATES[choice]);
                    }
                    self.changed();
                }
            }
            El::F3Rite => {
                if self.screen == Screen::Floor3Chamber(4) {
                    match self.expedition.floor3_rite_act() {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                } else if self.screen == Screen::Floor3Foe {
                    match self.expedition.floor3_foe_rite_act() {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            El::F3SlateTake => {
                if self.screen == Screen::Floor3Foe {
                    self.expedition.floor3_foe_take_slate();
                    self.say(
                        StatusKind::Info,
                        "The slate is in hand: three passes claimed. Judge \
                         each by its paper and its hand.",
                    );
                    self.changed();
                }
            }
            El::F3SlateEntry(entry) => {
                if self.screen == Screen::Floor3Foe {
                    let entry = entry as usize;
                    let verdict = {
                        let foe = &mut self.expedition.floor3.foe;
                        foe.slate_choices.get_mut(entry).map(|slot| {
                            *slot = Some(match *slot {
                                None => 0,
                                Some(0) => 1,
                                Some(_) => 0,
                            });
                            *slot
                        })
                    };
                    if let Some(choice) = verdict {
                        self.say(
                            StatusKind::Info,
                            if choice == Some(0) {
                                "It ran."
                            } else {
                                "It never ran."
                            },
                        );
                    }
                    self.changed();
                }
            }
            El::F3Reconcile => {
                if self.screen == Screen::Floor3Foe {
                    match self.expedition.floor3_foe_reconcile() {
                        Ok(text) => self.say(StatusKind::Good, text),
                        Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                    }
                    self.changed();
                }
            }
            El::F3Stamp => {
                if self.screen == Screen::Floor3Foe {
                    let foe = &mut self.expedition.floor3.foe;
                    foe.stamp_choice = Some(match foe.stamp_choice {
                        None => 0,
                        Some(2) => 0,
                        Some(choice) => choice + 1,
                    });
                    if let Some(choice) = foe.stamp_choice {
                        self.say(StatusKind::Info, content::MSRV_CANDIDATES[choice]);
                    }
                    self.changed();
                }
            }
            El::F3Lantern(kind) => {
                if self.screen == Screen::Floor3Foe {
                    if self.gate_running {
                        self.say(StatusKind::Info, "The engines are already awake on a run.");
                    } else if self.editor_text().trim().is_empty() {
                        self.say(
                            StatusKind::Refusal,
                            "The editor is empty — there is no draft to light a lantern over.",
                        );
                    } else {
                        self.io = Some(IoRequest::RunLantern {
                            kind,
                            code: self.editor_text().to_string(),
                        });
                        self.gate_running = true;
                        self.say(
                            StatusKind::Info,
                            if kind == 0 {
                                "rustfmt wakes: the format lantern is lit by the doing."
                            } else {
                                "clippy wakes: the lint lantern is lit by the doing."
                            },
                        );
                    }
                    self.bump();
                }
            }

            El::F2Watch => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.watched = true;
                    }
                    self.say(
                        StatusKind::Info,
                        "Both beasts perform the Anvil Call, each in its own \
                         body. Their performances stand in the panel — study \
                         the endings.",
                    );
                    self.changed();
                }
            }
            El::F2Charter(choice) => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.charter_choice = Some(choice as usize);
                    }
                    self.floor2_unfile(index);
                    self.changed();
                }
            }
            El::F2RunChoice(choice) => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.run_choice = Some(choice as usize);
                    }
                    self.floor2_unfile(index);
                    self.changed();
                }
            }
            El::F2BoundToggle(service, sigil) => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        let work = &mut progress.work;
                        if work.bound_masks.len() < content::STABLE_SERVICES.len() {
                            work.bound_masks
                                .resize(content::STABLE_SERVICES.len(), content::POSTED_BOUND);
                        }
                        if let Some(mask) = work.bound_masks.get_mut(service as usize) {
                            *mask ^= 1 << sigil;
                        }
                    }
                    self.floor2_unfile(index);
                    self.changed();
                }
            }
            El::F2RunDeliveries => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.ran_deliveries = true;
                    }
                    self.say(
                        StatusKind::Info,
                        "Both kennels run their errands. Their records stand \
                         in the panel — read how each errand actually ran \
                         from each house.",
                    );
                    self.changed();
                }
            }
            El::F2Errand(choice) => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.errand_choice = Some(choice as usize);
                    }
                    self.floor2_unfile(index);
                    self.changed();
                }
            }
            El::F2Kennel(choice) => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.kennel_choice = Some(choice as usize);
                    }
                    self.floor2_unfile(index);
                    self.changed();
                }
            }
            El::F2DriveRound => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.drove_round = true;
                    }
                    self.say(
                        StatusKind::Info,
                        "The Dawn Round is driven through the Wide Door: \
                         three beasts answer in their own bodies and pass; \
                         three the door refuses. Their placards stand in \
                         the panel — name each bar.",
                    );
                    self.changed();
                }
            }
            El::F2BarLabel(slot, label) => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        let work = &mut progress.work;
                        let barred = content::REGISTRY_BEASTS
                            .iter()
                            .filter(|entry| !entry.passes)
                            .count();
                        if work.bar_choices.len() < barred {
                            work.bar_choices.resize(barred, None);
                        }
                        if let Some(choice) = work.bar_choices.get_mut(slot as usize) {
                            *choice = Some(label as usize);
                        }
                    }
                    self.floor2_unfile(index);
                    self.changed();
                }
            }
            El::F2Route(route) => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.route_choice = Some(route as usize);
                    }
                    self.floor2_unfile(index);
                    self.changed();
                }
            }
            El::F2Reading(reading) => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if let Some(progress) = self.expedition.floor2.chambers.get_mut(index) {
                        progress.work.collision_choice = Some(reading as usize);
                    }
                    self.floor2_unfile(index);
                    self.changed();
                }
            }
            El::F2PanelTab(view) => {
                self.f2c5_view = view;
                self.bump();
            }
            El::F2HeraldAct(letter) => match self.expedition.floor2_herald_act(letter as usize) {
                Ok(text) => {
                    self.say(StatusKind::Info, text);
                    self.changed();
                }
                Err(refusal) => {
                    self.say(StatusKind::Refusal, refusal.message());
                }
            },

            El::F2FoeExhibit(row) => self.exhibit_foe2_row(row as usize),
            El::F2FoeName(row) => self.name_foe2_disagreement(row as usize),
            El::F2FoeCalibrate => self.calibrate_foe2(),
            El::CampStone => {
                self.overlay = Overlay::Reading(ReadingKind::FloorIntro);
                self.bump();
            }
            El::Desk => {
                self.overlay = Overlay::Reading(ReadingKind::Demonstration);
                self.bump();
            }
            El::Stone => {
                self.overlay = Overlay::Reading(ReadingKind::Inscription);
                self.bump();
            }
            El::Chip(verb) => self.do_verb(verb),
            El::Seal => self.press_seal(),
            El::SealConfirm => self.confirm_seal(),
            El::SealCancel => {
                self.disarm();
                self.say(StatusKind::Info, "The seal is stood down.");
            }
            El::HolderToggle(item, porter) => {
                let work = &mut self.expedition.chambers[0].work;
                if let Some(mask) = work.assignments.get_mut(item as usize) {
                    *mask ^= 1 << porter;
                }
                if work.packing_ok {
                    work.packing_ok = false;
                    self.unfile(0);
                }
                self.changed();
            }
            El::PanelTab(view) => {
                self.c1_view = view;
                self.bump();
            }
            El::TollChoice(choice) => {
                self.expedition.chambers[0].work.toll_choice = Some(choice as usize);
                if self.expedition.chambers[0].work.toll_ok {
                    self.expedition.chambers[0].work.toll_ok = false;
                    self.unfile(0);
                }
                let count = content::TOLL_CANDIDATES[choice as usize];
                self.say(
                    StatusKind::Info,
                    format!("The toll is chalked at {count} crates — file the plan to prove it."),
                );
                self.changed();
            }
            El::PickToggle(person) => {
                let work = &mut self.expedition.chambers[1].work;
                if let Some(pick) = work.picks.get_mut(person as usize) {
                    *pick = 1 - (*pick).min(1);
                }
                self.unfile(1);
                self.changed();
            }
            El::LeaseStep(lease, field, dir) => {
                let work = &mut self.expedition.chambers[2].work;
                if let Some(pair) = work.leases.get_mut(lease as usize) {
                    let slot = if field == 0 {
                        &mut pair[0]
                    } else {
                        &mut pair[1]
                    };
                    *slot = (*slot + i32::from(dir) * content::LEASE_STEP).clamp(0, 36);
                }
                self.unfile(2);
                self.changed();
            }
            El::RunDelivery => {
                match self.expedition.run_delivery(3) {
                    Ok(()) => {
                        self.selected_record = None;
                        self.unfile(3);
                        self.say(
                            StatusKind::Info,
                            "The delivery crosses both rooms. Two records come back — \
                             exhibit each where it belongs.",
                        );
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }
            El::CommitToggle => {
                let work = &mut self.expedition.chambers[3].work;
                work.commit_second = !work.commit_second;
                work.ran = false;
                work.records.clear();
                work.slot_kept = None;
                work.slot_restored = None;
                self.selected_record = None;
                self.unfile(3);
                self.say(
                    StatusKind::Info,
                    if self.expedition.chambers[3].work.commit_second {
                        "The second room will be committed in the book too. Run the \
                         delivery to see what that changes."
                    } else {
                        "The second room stays uncommitted. Run the delivery."
                    },
                );
                self.changed();
            }
            El::RecordCard(record) => {
                self.selected_record = Some(record as usize);
                self.bump();
            }
            El::ProofSlot(slot) => {
                let Some(record) = self.selected_record else {
                    self.say(
                        StatusKind::Info,
                        "Choose a record card first, then its slot.",
                    );
                    return;
                };
                match self.expedition.exhibit_record(3, record, slot as usize) {
                    Ok(text) => {
                        self.selected_record = None;
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }
            El::JudgeToggle(placard) => {
                let placard = placard as usize;
                let work = &mut self.expedition.chambers[4].work;
                if let Some(slot) = work.judgements.get_mut(placard) {
                    *slot = match *slot {
                        None => Some(true),
                        Some(true) => Some(false),
                        Some(false) => None,
                    };
                }
                work.wrong_marks.retain(|mark| *mark != placard);
                self.unfile(4);
                self.changed();
            }
            El::PagePrev => {
                self.c5_page = self.c5_page.saturating_sub(1);
                self.bump();
            }
            El::PageNext => {
                self.c5_page = (self.c5_page + 1).min(2);
                self.bump();
            }
            El::VerifyRow(row) => self.verify_foe_row(row as usize),
            El::CensusCard(row) => self.exhibit_foe_row(row as usize),
            El::ResetChamber => match self.screen {
                Screen::Chamber(index) => {
                    self.expedition.reset_chamber(index);
                    self.say(
                        StatusKind::Info,
                        "The proof is reset: the chamber stands as it was first \
                         entered. Hint rungs already climbed stay on the record.",
                    );
                    self.changed();
                }
                Screen::Floor2Chamber(index) => {
                    self.expedition.floor2_reset_chamber(index);
                    self.say(
                        StatusKind::Info,
                        "The proof is reset: the chamber stands as it was first \
                         entered. Hint rungs already climbed stay on the record.",
                    );
                    self.changed();
                }
                Screen::Foe => {
                    self.expedition.foe_reset_proof();
                    self.say(
                        StatusKind::Info,
                        "The census proof is reset: verifications and the exhibit \
                         are given back. The gate's verdict and your draft stand.",
                    );
                    self.changed();
                }
                Screen::Floor2Foe => {
                    self.expedition.floor2_foe_reset_proof();
                    self.say(
                        StatusKind::Info,
                        "The audit proof is reset: exhibits, the naming, and the \
                         calibration are given back. The gate's verdict and your \
                         Desk draft stand.",
                    );
                    self.changed();
                }
                Screen::Floor3Chamber(index) => {
                    self.expedition.floor3_reset_chamber(index);
                    self.say(
                        StatusKind::Info,
                        "The proof is reset: the chamber stands as it was first \
                         entered. Hint rungs already climbed stay on the record.",
                    );
                    self.changed();
                }

                Screen::Floor6Chamber(index) => {
                    self.expedition.floor6_reset_chamber(index);
                    self.say(
                        StatusKind::Info,
                        "The chamber is reset: flags home, slate wiped, \
                         the work begun again.",
                    );
                    self.changed();
                }
                Screen::Floor5Chamber(index) => {
                    self.expedition.floor5_reset_chamber(index);
                    self.say(
                        StatusKind::Info,
                        "The workshop is reset: tools racked, cards squared, \
                         the work begun again.",
                    );
                    self.changed();
                }
                Screen::Floor4Chamber(index) => {
                    self.expedition.floor4_reset_chamber(index);
                    self.say(
                        StatusKind::Info,
                        "The proof is reset: the chamber stands as it was first \
                         entered. Hint rungs already climbed stay on the record.",
                    );
                    self.changed();
                }

                _ => {}
            },
            El::EditorOpen => {
                self.overlay = Overlay::Editor;
                self.editor_cursor = self.editor_cursor.min(self.editor_text().len());
                self.say(
                    StatusKind::Info,
                    if self.screen == Screen::Floor2Foe {
                        "The Admission Desk, in your own hand. Run the gate when \
                         the bound is re-posted; the draft is kept whatever the \
                         verdict."
                    } else {
                        "The Interner's Hoard, in your own hand. Run the gate when \
                         it is written; the draft is kept whatever the verdict."
                    },
                );
            }
            El::EditorArea => {
                self.focus = Some(El::EditorArea);
                self.bump();
            }
            El::RunGate => {
                if self.gate_running {
                    self.say(StatusKind::Info, "The gate is already running.");
                } else if self.editor_text().trim().is_empty() {
                    self.say(
                        StatusKind::Refusal,
                        "The editor is empty — there is nothing to judge.",
                    );
                } else {
                    self.gate_floor2 = self.screen == Screen::Floor2Foe;
                    self.gate_floor3 = self.screen == Screen::Floor3Foe;
                    self.io = Some(IoRequest::RunGate {
                        code: self.editor_text().to_string(),
                        floor2: self.gate_floor2,
                        floor3: self.gate_floor3,
                    });
                    self.gate_running = true;
                    self.say(
                        StatusKind::Info,
                        "The vault's engines wake: cargo test, then the clippy \
                         lanterns. This takes a little while.",
                    );
                }
            }
            El::ResetProof => {
                if self.screen == Screen::Floor2Foe {
                    *self.expedition.floor2_foe_draft_mut() = content::GATE2_SCAFFOLD.to_string();
                } else if self.screen == Screen::Floor3Foe {
                    *self.expedition.floor3_foe_draft_mut() = content::GATE3_SCAFFOLD.to_string();
                    self.expedition.floor3_foe_douse_lanterns();
                } else {
                    self.expedition.editor_code = content::GATE_SCAFFOLD.to_string();
                }
                self.editor_cursor = 0;
                self.editor_anchor = None;
                self.editor_scroll = 0;
                self.say(
                    StatusKind::Info,
                    "The proof is reset: the scaffold stands in the editor again.",
                );
                self.changed();
            }
            El::GateRecordBtn => {
                self.overlay = Overlay::GateRecord;
                self.bump();
            }
            El::HintDeeper => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    self.expedition.floor2_view_hint(index);
                    self.hint_showing = self.expedition.floor2_hints_used(index);
                    self.changed();
                } else if self.screen == Screen::Floor2Foe {
                    self.expedition.floor2_foe_view_hint();
                    self.hint_showing = self.expedition.floor2_foe_hints_used();
                    self.changed();
                } else if let Some(challenge) = self.current_challenge() {
                    self.expedition.view_hint(challenge);
                    self.hint_showing = self.expedition.hints_used(challenge);
                    self.changed();
                }
            }
            El::ReadingScale => {
                self.expedition.settings.text_scale = self.expedition.settings.text_scale.next();
                self.changed();
            }
            El::OverlayClose => self.close_overlay(),
            El::Export => {
                self.io = Some(IoRequest::WriteSave { announce: true });
            }
            El::Import => {
                self.io = Some(IoRequest::ReadSave);
            }
            El::Resume => {
                self.overlay = Overlay::None;
                self.bump();
            }
        }
    }

    /// A settings row/strip activation: 0 scale, 1 contrast, 2 motion.
    fn apply_setting(&mut self, index: usize) {
        match index {
            0 => {
                self.expedition.settings.text_scale = self.expedition.settings.text_scale.next();
            }
            1 => {
                self.expedition.settings.high_contrast = !self.expedition.settings.high_contrast;
            }
            2 => {
                self.expedition.settings.reduced_motion = !self.expedition.settings.reduced_motion;
            }
            _ => return,
        }
        self.changed();
    }

    /// Return a diagnosed chamber to UNWORKED when its filed proof
    /// is edited — the filing no longer describes the work.
    fn unfile(&mut self, chamber: usize) {
        if self.expedition.chambers[chamber].state == ChamberState::Diagnosed {
            self.expedition.chambers[chamber].state = ChamberState::Unworked;
        }
    }

    /// The Floor 2 form of `unfile`.
    fn floor2_unfile(&mut self, chamber: usize) {
        if let Some(progress) = self.expedition.floor2.chambers.get_mut(chamber)
            && progress.state == ChamberState::Diagnosed
        {
            progress.state = ChamberState::Unworked;
        }
    }

    /// Press the seal: arm the current screen's strike.
    fn press_seal(&mut self) {
        match self.screen {
            Screen::Floor2Chamber(index) => match self.expedition.floor2.chambers[index].state {
                ChamberState::Diagnosed => {
                    self.armed = Some(ArmedFor::Floor2Chamber(index));
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to strike the repair home, or \
                         stand down.",
                    );
                }
                ChamberState::Repaired => {
                    self.say(StatusKind::Info, "This chamber already stands repaired.");
                }
                ChamberState::Unworked => {
                    self.say(
                        StatusKind::Refusal,
                        "The seal sleeps until the proof is complete.",
                    );
                }
            },
            Screen::Floor4Chamber(index) => match self.expedition.floor4.chambers[index].state {
                ChamberState::Diagnosed => {
                    self.armed = Some(ArmedFor::Floor4Chamber(index));
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to strike the repair home, or \
                         stand down.",
                    );
                }
                ChamberState::Repaired => {
                    self.say(StatusKind::Info, "This chamber already stands repaired.");
                }
                ChamberState::Unworked => {
                    self.say(
                        StatusKind::Refusal,
                        "The seal sleeps until the proof is complete.",
                    );
                }
            },

            Screen::Floor5Chamber(index) => match self.expedition.floor5.chambers[index].state {
                ChamberState::Diagnosed => {
                    self.armed = Some(ArmedFor::Floor5Chamber(index));
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to strike the repair home, or \
                         stand down.",
                    );
                }
                ChamberState::Repaired => {
                    self.say(StatusKind::Info, "This chamber already stands repaired.");
                }
                ChamberState::Unworked => {
                    self.say(
                        StatusKind::Refusal,
                        "The seal sleeps until the proof is complete.",
                    );
                }
            },

            Screen::Floor6Chamber(index) => match self.expedition.floor6.chambers[index].state {
                ChamberState::Diagnosed => {
                    self.armed = Some(ArmedFor::Floor6Chamber(index));
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to strike the repair home, or \
                         stand down.",
                    );
                }
                ChamberState::Repaired => {
                    self.say(StatusKind::Info, "This chamber already stands repaired.");
                }
                ChamberState::Unworked => {
                    self.say(
                        StatusKind::Refusal,
                        "The seal sleeps until the proof is complete.",
                    );
                }
            },

            Screen::Floor3Chamber(index) => match self.expedition.floor3.chambers[index].state {
                ChamberState::Diagnosed => {
                    self.armed = Some(ArmedFor::Floor3Chamber(index));
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to strike the repair home, or \
                         stand down.",
                    );
                }
                ChamberState::Repaired => {
                    self.say(StatusKind::Info, "This chamber already stands repaired.");
                }
                ChamberState::Unworked => {
                    self.say(
                        StatusKind::Refusal,
                        "The seal sleeps until the proof is complete.",
                    );
                }
            },

            Screen::Chamber(index) => match self.expedition.chambers[index].state {
                ChamberState::Diagnosed => {
                    self.armed = Some(ArmedFor::Chamber(index));
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to strike the repair home, or \
                         stand down.",
                    );
                }
                ChamberState::Repaired => {
                    self.say(StatusKind::Info, "This chamber already stands repaired.");
                }
                ChamberState::Unworked => {
                    self.say(
                        StatusKind::Refusal,
                        "The seal sleeps until the proof is complete.",
                    );
                }
            },
            Screen::Foe => match self.expedition.foe_arm_strike() {
                Ok(()) => {
                    self.armed = Some(ArmedFor::Foe);
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to strike the second claim \
                         and re-route it — or stand down.",
                    );
                }
                Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
            },

            Screen::Floor6Foe => match self.expedition.floor6_foe_arm_strike() {
                Ok(()) => {
                    self.armed = Some(ArmedFor::Foe6);
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to re-forge the \
                         staff and lock the Heart once, and truly — \
                         or stand down.",
                    );
                }
                Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
            },

            Screen::Floor5Foe => match self.expedition.floor5_foe_arm_strike() {
                Ok(()) => {
                    self.armed = Some(ArmedFor::Foe5);
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to draw the veil \
                         aside and strike the Thing from the beam's \
                         frame — or stand down.",
                    );
                }
                Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
            },
            Screen::Floor4Foe => match self.expedition.floor4_foe_arm_strike() {
                Ok(()) => {
                    self.armed = Some(ArmedFor::Foe4);
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to re-lay the master \
                         foundation from the measured stones — or stand down.",
                    );
                }
                Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
            },
            Screen::Floor3Foe => match self.expedition.floor3_foe_arm_strike() {
                Ok(()) => {
                    self.armed = Some(ArmedFor::Foe3);
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to strike the Release's \
                         lantern wire, or stand down.",
                    );
                }
                Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
            },

            Screen::Floor2Foe => match self.expedition.floor2_foe_arm_strike() {
                Ok(()) => {
                    self.armed = Some(ArmedFor::Foe2);
                    self.say(
                        StatusKind::Info,
                        "The seal is pressed. Confirm to re-post the bound and \
                         reassign the hound — or stand down.",
                    );
                }
                Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
            },
            _ => {}
        }
    }

    /// Confirm an armed strike.
    fn confirm_seal(&mut self) {
        match self.armed {
            Some(ArmedFor::Floor2Chamber(index)) => {
                self.armed = None;
                match self.expedition.floor2_repair_chamber(index) {
                    Ok(text) => {
                        self.expedition.log_push(format!(
                            "{} repaired.",
                            content::MENAGERIE_CHAMBERS[index].name
                        ));
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }
            Some(ArmedFor::Floor4Chamber(index)) => {
                self.armed = None;
                match self.expedition.floor4_repair_chamber(index) {
                    Ok(text) => {
                        self.expedition.log_push(format!(
                            "{} repaired.",
                            content::FOUNDATION_CHAMBERS[index].name
                        ));
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }

            Some(ArmedFor::Floor5Chamber(index)) => {
                self.armed = None;
                match self.expedition.floor5_repair_chamber(index) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("{} repaired.", content::VEIL_CHAMBERS[index].name));
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }

            Some(ArmedFor::Floor6Chamber(index)) => {
                self.armed = None;
                match self.expedition.floor6_repair_chamber(index) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("{} repaired.", content::LOCK_CHAMBERS[index].name));
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }

            Some(ArmedFor::Floor3Chamber(index)) => {
                self.armed = None;
                match self.expedition.floor3_repair_chamber(index) {
                    Ok(text) => {
                        self.expedition.log_push(format!(
                            "{} repaired.",
                            content::GATEHOUSE_CHAMBERS[index].name
                        ));
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }

            Some(ArmedFor::Chamber(index)) => {
                self.armed = None;
                match self.expedition.repair_chamber(index) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("{} repaired.", CHAMBERS[index].name));
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }
            Some(ArmedFor::Foe) => {
                self.armed = None;
                let date = if self.today.is_empty() {
                    "an unrecorded day".to_string()
                } else {
                    self.today.clone()
                };
                match self.expedition.foe_confirm_strike(&date) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("The Borrowed Hoard fell on {date}."));
                        self.expedition.log_push(
                            "The Plank Stone entered the codex; the Interner's \
                             Hoard, the pack.",
                        );
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }

            Some(ArmedFor::Foe3) => {
                self.armed = None;
                let date = if self.today.is_empty() {
                    "an unrecorded day".to_string()
                } else {
                    self.today.clone()
                };
                match self.expedition.floor3_foe_confirm_strike(&date) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("The Gatehouse Release fell on {date}."));
                        self.expedition.log_push(
                            "The Lantern Stone entered the codex; the Gatehouse \
                             Key, the pack.",
                        );
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }

            Some(ArmedFor::Foe6) => {
                self.armed = None;
                let date = if self.today.is_empty() {
                    "an unrecorded day".to_string()
                } else {
                    self.today.clone()
                };
                match self.expedition.floor6_foe_confirm_strike(&date) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("The Heart Locked Twice fell on {date}."));
                        self.expedition.log_push(
                            "The Lockheart Stone entered the codex; the \
                             Lockheart Pair, the pack.",
                        );
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }

            Some(ArmedFor::Foe5) => {
                self.armed = None;
                let date = if self.today.is_empty() {
                    "an unrecorded day".to_string()
                } else {
                    self.today.clone()
                };
                match self.expedition.floor5_foe_confirm_strike(&date) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("The Thing Behind the Veil fell on {date}."));
                        self.expedition.log_push(
                            "The Ice Stone entered the codex; the Veilwalker's \
                             Buffer, the pack.",
                        );
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }
            Some(ArmedFor::Foe4) => {
                self.armed = None;
                let date = if self.today.is_empty() {
                    "an unrecorded day".to_string()
                } else {
                    self.today.clone()
                };
                match self.expedition.floor4_foe_confirm_strike(&date) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("The Crooked Foundation fell on {date}."));
                        self.expedition.log_push(
                            "The Brass Eye Stone entered the codex; the Plumb \
                             Line, the pack.",
                        );
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }
            Some(ArmedFor::Foe2) => {
                self.armed = None;
                let date = if self.today.is_empty() {
                    "an unrecorded day".to_string()
                } else {
                    self.today.clone()
                };
                match self.expedition.floor2_foe_confirm_strike(&date) {
                    Ok(text) => {
                        self.expedition
                            .log_push(format!("The Menagerie fell on {date}."));
                        self.expedition.log_push(
                            "The Lens Stone entered the codex; the Plugin \
                             Menagerie, the pack.",
                        );
                        self.say(StatusKind::Good, text);
                    }
                    Err(refusal) => self.say(StatusKind::Refusal, refusal.message()),
                }
                self.changed();
            }
            None => {}
        }
    }

    // ------------------------------------------------------------------
    // The editor.
    // ------------------------------------------------------------------

    /// The draft the editor edits on this screen: the Admission
    /// Desk at the Menagerie's desk, the Interner's Hoard
    /// everywhere else. The two drafts are separate records; the
    /// editor machinery below speaks only through these.
    pub fn editor_text(&self) -> &str {
        if self.screen == Screen::Floor2Foe {
            self.expedition.floor2_foe_draft()
        } else if self.screen == Screen::Floor3Foe {
            self.expedition.floor3_foe_draft()
        } else {
            &self.expedition.editor_code
        }
    }

    /// Editing the Release's draft douses its lanterns:
    /// they were lit for the draft as it stood.
    fn douse_if_release(&mut self) {
        if self.screen == Screen::Floor3Foe {
            self.expedition.floor3_foe_douse_lanterns();
        }
    }

    /// The draft the editor edits, writable.
    fn editor_text_mut(&mut self) -> &mut String {
        if self.screen == Screen::Floor2Foe {
            self.expedition.floor2_foe_draft_mut()
        } else if self.screen == Screen::Floor3Foe {
            self.expedition.floor3_foe_draft_mut()
        } else {
            &mut self.expedition.editor_code
        }
    }

    /// The current screen's gate record: the Desk's verdict at
    /// the Menagerie's desk, the Hoard's elsewhere — for the
    /// editor's verdict line and the gate record view.
    pub fn gate_record(&self) -> Option<&str> {
        if self.screen == Screen::Floor2Foe {
            self.expedition.floor2.foe.gate_record.as_deref()
        } else if self.screen == Screen::Floor3Foe {
            self.expedition.floor3.foe.gate_record.as_deref()
        } else {
            self.expedition.gate_record.as_deref()
        }
    }
    /// Position the cursor from an editor-area click.
    fn editor_click(&mut self, x: f32, y: f32) {
        let area = rects::EDITOR_AREA;
        let line = self.editor_scroll + (((y - area.y) / EDITOR_LINE_H).floor().max(0.0) as usize);
        let col = ((x - area.x - EDITOR_PAD_X) / EDITOR_CHAR_W)
            .floor()
            .max(0.0) as usize;
        self.editor_cursor = byte_at_line_col(self.editor_text(), line, col);
        self.editor_anchor = None;
        self.focus = Some(El::EditorArea);
        self.bump();
    }

    /// Insert text at the cursor (replacing any selection).
    pub fn editor_insert(&mut self, text: &str) {
        self.douse_if_release();
        self.editor_delete_selection();
        let at = clamp_boundary(self.editor_text(), self.editor_cursor);
        self.editor_text_mut().insert_str(at, text);
        self.editor_cursor = at + text.len();
        self.editor_anchor = None;
        self.editor_sync_scroll();
        self.bump();
    }

    /// Delete the selection, if one stands. Returns whether it did.
    fn editor_delete_selection(&mut self) -> bool {
        let Some(anchor) = self.editor_anchor else {
            return false;
        };
        let (start, end) = ordered(anchor, self.editor_cursor);
        let (start, end) = (
            clamp_boundary(self.editor_text(), start),
            clamp_boundary(self.editor_text(), end),
        );
        if start == end {
            self.editor_anchor = None;
            return false;
        }
        self.editor_text_mut().drain(start..end);
        self.editor_cursor = start;
        self.editor_anchor = None;
        true
    }

    /// Backspace: delete the selection, or one char back.
    pub fn editor_backspace(&mut self) {
        self.douse_if_release();
        if self.editor_delete_selection() {
            self.editor_sync_scroll();
            self.bump();
            return;
        }
        let cursor = clamp_boundary(self.editor_text(), self.editor_cursor);
        if cursor == 0 {
            return;
        }
        let prev = prev_boundary(self.editor_text(), cursor);
        self.editor_text_mut().drain(prev..cursor);
        self.editor_cursor = prev;
        self.editor_sync_scroll();
        self.bump();
    }

    /// Delete: one char forward.
    pub fn editor_delete(&mut self) {
        self.douse_if_release();
        if self.editor_delete_selection() {
            self.editor_sync_scroll();
            self.bump();
            return;
        }
        let cursor = clamp_boundary(self.editor_text(), self.editor_cursor);
        if cursor >= self.editor_text().len() {
            return;
        }
        let next = next_boundary(self.editor_text(), cursor);
        self.editor_text_mut().drain(cursor..next);
        self.editor_sync_scroll();
        self.bump();
    }

    /// Select the whole draft.
    pub fn editor_select_all(&mut self) {
        self.editor_anchor = Some(0);
        self.editor_cursor = self.editor_text().len();
        self.bump();
    }

    /// Move the editor cursor. `extend` grows a selection.
    pub fn editor_move(&mut self, key: Key, extend: bool) {
        let cursor = clamp_boundary(self.editor_text(), self.editor_cursor);
        if extend && self.editor_anchor.is_none() {
            self.editor_anchor = Some(cursor);
        }
        if !extend {
            self.editor_anchor = None;
        }
        let text = self.editor_text();
        let (line, col) = line_col_of_byte(text, cursor);
        let new = match key {
            Key::Left => prev_boundary(text, cursor),
            Key::Right => next_boundary(text, cursor),
            Key::Up => byte_at_line_col(text, line.saturating_sub(1), col),
            Key::Down => byte_at_line_col(text, line + 1, col),
            _ => cursor,
        };
        self.editor_cursor = new;
        self.editor_sync_scroll();
        self.bump();
    }

    /// Home/End for the editor: line start / line end.
    pub fn editor_line_edge(&mut self, end: bool) {
        let text = self.editor_text();
        let cursor = clamp_boundary(text, self.editor_cursor);
        let (line, _) = line_col_of_byte(text, cursor);
        self.editor_cursor = if end {
            let starts = line_starts(text);
            let start = starts.get(line).copied().unwrap_or(0);
            let next = starts.get(line + 1).copied().unwrap_or(text.len());
            // Line end is just before the newline, if one is there.
            if next > start && text.as_bytes().get(next - 1) == Some(&b'\n') {
                next - 1
            } else {
                next
            }
        } else {
            line_starts(text).get(line).copied().unwrap_or(0)
        };
        self.editor_anchor = None;
        self.editor_sync_scroll();
        self.bump();
    }

    /// Keep the cursor's line inside the visible window.
    fn editor_sync_scroll(&mut self) {
        let (line, _) = line_col_of_byte(self.editor_text(), self.editor_cursor);
        if line < self.editor_scroll {
            self.editor_scroll = line;
        } else if line >= self.editor_scroll + EDITOR_VISIBLE_LINES {
            self.editor_scroll = line + 1 - EDITOR_VISIBLE_LINES;
        }
    }

    /// The cursor's (line, column in chars), 0-based.
    pub fn editor_line_col(&self) -> (usize, usize) {
        line_col_of_byte(self.editor_text(), self.editor_cursor)
    }

    /// The draft's line count.
    pub fn editor_line_count(&self) -> usize {
        line_starts(self.editor_text()).len()
    }

    // ------------------------------------------------------------------
    // Reading views.
    // ------------------------------------------------------------------

    /// The current reading overlay's title.
    pub fn reading_title(&self, kind: ReadingKind) -> String {
        match kind {
            ReadingKind::Inscription => match self.screen {
                Screen::Chamber(index) => CHAMBERS[index].inscription_title.to_string(),
                Screen::Floor2Chamber(index) => content::MENAGERIE_CHAMBERS[index]
                    .inscription_title
                    .to_string(),
                Screen::Floor3Chamber(index) => format!(
                    "Chamber {} — {}",
                    index + 1,
                    content::GATEHOUSE_CHAMBERS[index].name
                ),
                Screen::Floor4Chamber(index) => format!(
                    "Chamber {} — {}",
                    index + 1,
                    content::FOUNDATION_CHAMBERS[index].name
                ),
                Screen::Floor5Chamber(index) => format!(
                    "Chamber {} — {}",
                    index + 1,
                    content::VEIL_CHAMBERS[index].name
                ),
                Screen::Floor6Chamber(index) => format!(
                    "Chamber {} — {}",
                    index + 1,
                    content::LOCK_CHAMBERS[index].name
                ),
                Screen::Foe => "The Hoard's Inscription".to_string(),
                Screen::Floor2Foe => "The Menagerie's Inscription".to_string(),
                Screen::Floor3Foe => "The Release's Inscription".to_string(),
                Screen::Floor4Foe => "The Foundation's Inscription".to_string(),
                Screen::Floor5Foe => "The Thing's Inscription".to_string(),
                Screen::Floor6Foe => "The Heart's Inscription".to_string(),
                _ => "The Inscription".to_string(),
            },
            ReadingKind::Demonstration => "The Demonstration".to_string(),
            ReadingKind::Hint => "The Hint Ladder".to_string(),
            ReadingKind::Stairs => "The Stairs Gate — Stratum I to Stratum II".to_string(),
            ReadingKind::FloorIntro => match self.screen {
                Screen::Floor2Hub => format!("Floor 2 — {}", content::FLOOR2_NAME),
                Screen::Floor3Hub => format!("Floor 3 — {}", content::FLOOR3_NAME),
                Screen::Floor4Hub => format!("Floor 4 — {}", content::FLOOR4_NAME),
                Screen::Floor5Hub => format!("Floor 5 — {}", content::FLOOR5_NAME),
                Screen::Floor6Hub => format!("Floor 6 — {}", content::FLOOR6_NAME),
                _ => format!("Floor 1 — {}", content::FLOOR_NAME),
            },
            ReadingKind::MasterPlan => "The Master Plan".to_string(),
        }
    }

    /// The current reading overlay's body text.
    pub fn reading_text(&self, kind: ReadingKind) -> String {
        match kind {
            ReadingKind::Inscription => match self.screen {
                Screen::Chamber(index) => CHAMBERS[index].inscription.to_string(),
                Screen::Floor2Chamber(index) => {
                    content::MENAGERIE_CHAMBERS[index].inscription.to_string()
                }
                Screen::Floor3Chamber(index) => content::FLOOR3_CHAMBER_INTROS[index].to_string(),
                Screen::Floor4Chamber(index) => content::FLOOR4_CHAMBER_INTROS[index].to_string(),
                Screen::Floor5Chamber(index) => content::FLOOR5_CHAMBER_INTROS[index].to_string(),
                Screen::Floor6Chamber(index) => content::FLOOR6_CHAMBER_INTROS[index].to_string(),
                Screen::Foe => content::FOE_INSCRIPTION.to_string(),
                Screen::Floor2Foe => content::FOE2_INSCRIPTION.to_string(),
                Screen::Floor3Foe => content::FOE3_INSCRIPTION.to_string(),
                Screen::Floor4Foe => content::FOE4_INSCRIPTION.to_string(),
                Screen::Floor5Foe => content::FOE5_INSCRIPTION.to_string(),
                Screen::Floor6Foe => content::FOE6_INSCRIPTION.to_string(),
                _ => content::FLOOR_INTRO.to_string(),
            },
            ReadingKind::Demonstration => match self.screen {
                Screen::Chamber(index) => CHAMBERS[index].demonstration.to_string(),
                Screen::Floor2Chamber(index) => {
                    content::MENAGERIE_CHAMBERS[index].demonstration.to_string()
                }
                Screen::Floor3Chamber(index) => match index {
                    0 => content::FAILURE_CHARTER.to_string(),
                    1 => content::CONVEYOR_CHARTER.to_string(),
                    2 => content::SHOUT_CHARTER.to_string(),
                    3 => content::RAIL_CHARTER.to_string(),
                    _ => content::FLOOR3_CHAMBER_INTROS[index].to_string(),
                },
                Screen::Floor4Chamber(index) => content::FOUNDATION_CHAMBERS[index]
                    .demonstration
                    .to_string(),
                Screen::Floor5Chamber(index) => {
                    content::VEIL_CHAMBERS[index].demonstration.to_string()
                }
                Screen::Floor6Chamber(index) => {
                    content::LOCK_CHAMBERS[index].demonstration.to_string()
                }
                Screen::Foe => content::FOE_DEMONSTRATION.to_string(),
                Screen::Floor2Foe => content::FOE2_DEMONSTRATION.to_string(),
                Screen::Floor3Foe => content::FOE3_INTRO.to_string(),
                Screen::Floor4Foe => content::FOE4_DEMONSTRATION.to_string(),
                Screen::Floor5Foe => content::FOE5_DEMONSTRATION.to_string(),
                Screen::Floor6Foe => content::FOE6_DEMONSTRATION.to_string(),
                _ => content::FLOOR_INTRO.to_string(),
            },
            ReadingKind::Hint => {
                let rung = self.hint_showing.max(1) as usize;
                let text = match self.screen {
                    Screen::Floor2Chamber(index) => content::floor2_hints(index)[rung - 1],
                    Screen::Floor3Chamber(index) => content::floor3_chamber_hints(index)[rung - 1],
                    Screen::Floor4Chamber(index) => content::floor4_chamber_hints(index)[rung - 1],
                    Screen::Floor5Chamber(index) => content::floor5_chamber_hints(index)[rung - 1],
                    Screen::Floor6Chamber(index) => content::floor6_chamber_hints(index)[rung - 1],
                    Screen::Floor2Foe => content::floor2_foe_hints()[rung - 1],
                    Screen::Floor3Foe => content::floor3_foe_hints()[rung - 1],
                    Screen::Floor4Foe => content::floor4_foe_hints()[rung - 1],
                    Screen::Floor5Foe => content::floor5_foe_hints()[rung - 1],
                    Screen::Floor6Foe => content::floor6_foe_hints()[rung - 1],
                    _ => {
                        let challenge = self.current_challenge().unwrap_or(0);
                        content::hints(challenge)[rung - 1]
                    }
                };
                let mut body = format!("Rung {rung} of 4.\n\n{text}");
                if rung >= 4 {
                    body.push_str(
                        "\n\nThe seam is named. This challenge's facet is forfeit — \
                         the floor still completes, and the record keeps it honestly.",
                    );
                }
                body
            }
            ReadingKind::Stairs => {
                let mut body = String::from(
                    "The stairs out of Stratum I open when every clause stands lit:\n",
                );
                for (lit, text) in self.expedition.stairs_clauses() {
                    body.push_str(&format!(
                        "\n{} {}",
                        if lit { "[lit]" } else { "[unlit]" },
                        text
                    ));
                }
                body.push_str(
                    "\n\nThe stairs are shut — for now. A chart is drawn one verified \
                     floor at a time.",
                );
                body
            }
            ReadingKind::FloorIntro => match self.screen {
                Screen::Floor2Hub => content::FLOOR2_INTRO.to_string(),
                Screen::Floor3Hub => content::FLOOR3_INTRO.to_string(),
                Screen::Floor4Hub => content::FLOOR4_INTRO.to_string(),
                Screen::Floor5Hub => content::FLOOR5_INTRO.to_string(),
                Screen::Floor6Hub => content::FLOOR6_INTRO.to_string(),
                _ => content::FLOOR_INTRO.to_string(),
            },
            ReadingKind::MasterPlan => content::MASTER_PLAN.to_string(),
        }
    }
}

/// Per-chamber door narrations, Floor 2, for the chambers whose
/// builds have landed.
const F2_DOOR_TEXTS: [&str; 5] = [
    "Chamber 1 — Traits and impls. The forge where every beast is \
     marked: watch the call, strike the sigil, name the body.",
    "Chamber 2 — Generics and bounds. The Generic Stable: loosen the \
     posted bound, service by service, until it demands exactly what \
     each service's work asks — no more, no less.",
    "Chamber 3 — Associated types. The Two Kennels: run the same \
     errands from both houses, and find the one that runs contorted \
     where the kennel chooses.",
    "Chamber 4 — Trait objects and the Wide Door. The registry: drive \
     the Dawn Round through the one door that admits any beast by \
     sigil, and name the bars on the three it refuses.",
    "Chamber 5 — Coherence. Foreign Marks: a foreign beast, a foreign \
     sigil, and the one lawful joining — a collar of your own making.",
];

/// Per-chamber Examine narrations, Floor 2.
const F2_EXAMINE_TEXTS: [&str; 5] = [
    "The forge: an anvil, a screen of hammered copper, and the sigil-book \
     open at the Anvil Call.",
    "The Generic Stable: three postings, one posted bound demanding \
     every sigil of every beast, and the Sigilwright waiting with her \
     reasons.",
    "The Two Kennels: two courier houses, one service, and a wall of \
     delivery records waiting to be read.",
    "The registry: six beasts in the book, one service routine, and \
     a door wide enough for any beast whose charter can be answered \
     without knowing its kind.",
    "The forge and the desk: a river-horse with the floods' mud on \
     her legs, a foreign sigil, and two collars claiming the same \
     mark.",
];

/// Per-chamber Examine narrations.
const EXAMINE_TEXTS: [&str; 5] = [
    "The issue desk: eight things to pack, three porters, and a plan that already \
     failed once at the vault door.",
    "The reading room: one chart, five people, and a pen that must never write \
     over a held page.",
    "The long lease: a charter on the wall, its term drawn as one brass bracket \
     of thirty-six months.",
    "The steward's desk: a delivery book, a rope coil, and a crate with chalk \
     marks still fresh.",
    "The gauntlet yard: ten placards on ten posts, and the yard's gate watching \
     your hands.",
];

/// Editor text metrics, shared with the shell's rendering.
pub const EDITOR_LINE_H: f32 = 19.0;
/// Editor character advance (Liberation Mono at 14px ≈ 0.6em).
pub const EDITOR_CHAR_W: f32 = 8.4;
/// Editor text left padding inside its area (past the line-number
/// gutter the shell draws).
pub const EDITOR_PAD_X: f32 = 52.0;

/// Byte offsets of every line start in `text`.
fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

/// The (line, column in chars) of a byte index.
fn line_col_of_byte(text: &str, byte: usize) -> (usize, usize) {
    let byte = clamp_boundary(text, byte.min(text.len()));
    let starts = line_starts(text);
    let line = starts.iter().rposition(|start| *start <= byte).unwrap_or(0);
    let col = text[starts[line]..byte].chars().count();
    (line, col)
}

/// The byte index at (line, column in chars), clamped to the line.
fn byte_at_line_col(text: &str, line: usize, col: usize) -> usize {
    let starts = line_starts(text);
    let Some(start) = starts.get(line).copied() else {
        return text.len();
    };
    let line_end = starts.get(line + 1).copied().unwrap_or(text.len());
    let mut byte = start;
    let mut seen = 0;
    while byte < line_end && seen < col {
        byte = next_boundary(text, byte);
        seen += 1;
    }
    // Do not rest the cursor on the newline itself when the column
    // overshoots a short line: the line's last char boundary before
    // '\n' is the resting place.
    if byte > start && text.as_bytes().get(byte - 1) == Some(&b'\n') && byte == line_end {
        return byte - 1;
    }
    byte
}

/// Clamp an index to a char boundary at or before it.
fn clamp_boundary(text: &str, index: usize) -> usize {
    let mut at = index.min(text.len());
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// The char boundary before `index`.
fn prev_boundary(text: &str, index: usize) -> usize {
    let mut at = clamp_boundary(text, index);
    if at > 0 {
        at -= 1;
        at = clamp_boundary(text, at);
    }
    at
}

/// The char boundary after `index`.
fn next_boundary(text: &str, index: usize) -> usize {
    let mut at = clamp_boundary(text, index);
    if at < text.len() {
        at += 1;
        while at < text.len() && !text.is_char_boundary(at) {
            at += 1;
        }
    }
    at
}

/// Order a pair of indices.
fn ordered(a: usize, b: usize) -> (usize, usize) {
    if a <= b { (a, b) } else { (b, a) }
}

// ---------------------------------------------------------------------------
// Layout and hit-testing.
// ---------------------------------------------------------------------------

/// Every interactive element on screen, with its rectangle, in
/// drawing order (hit-testing walks it in reverse).
pub fn layout(game: &Game) -> Vec<(El, Rect)> {
    let mut els = Vec::new();
    match game.screen {
        Screen::Title => {
            els.push((El::Begin, rects::BEGIN));
            if game.save_present {
                els.push((El::Continue, rects::CONTINUE));
            }
            for index in 0..3 {
                els.push((El::Setting(index), rects::title_setting(index)));
            }
            return els;
        }
        Screen::Hub
        | Screen::Floor2Hub
        | Screen::Chamber(_)
        | Screen::Floor2Chamber(_)
        | Screen::Floor3Chamber(_)
        | Screen::Floor4Chamber(_)
        | Screen::Floor5Chamber(_)
        | Screen::Floor6Chamber(_)
        | Screen::Foe
        | Screen::Floor2Foe
        | Screen::Floor3Foe
        | Screen::Floor4Foe
        | Screen::Floor5Foe
        | Screen::Floor6Foe
        | Screen::Floor3Hub
        | Screen::Floor4Hub
        | Screen::Floor5Hub
        | Screen::Floor6Hub => {
            els.push((El::TabFloor, rects::TAB_FLOOR));
            els.push((El::TabMap, rects::TAB_MAP));
            els.push((El::TabLog, rects::TAB_LOG));
            els.push((El::TabCodex, rects::TAB_CODEX));
            els.push((El::PauseBtn, rects::PAUSE_BTN));
        }
    }
    match game.screen {
        Screen::Hub => {
            for index in 0..5 {
                els.push((El::DoorChamber(index as u8), geom::door_chamber(index)));
            }
            els.push((El::DoorFoe, geom::door_foe()));
            els.push((El::DoorStairs, geom::door_stairs()));
            els.push((El::DoorFloor2, geom::door_floor2()));
            els.push((El::DoorFloor4, geom::door_floor4()));
            els.push((El::CampStone, geom::camp_stone()));
        }
        Screen::Floor3Hub => {
            els.push((El::CampStone, geom::camp_stone()));
            for index in 0..5 {
                els.push((El::Floor3Door(index as u8), geom::door_chamber(index)));
            }
            els.push((El::DoorFoe3, geom::door_foe3()));
        }
        Screen::Floor4Hub => {
            els.push((El::CampStone, geom::camp_stone()));
            for index in 0..5 {
                els.push((El::Floor4Door(index as u8), geom::door_chamber(index)));
            }
            els.push((El::DoorFoe4, geom::door_foe4()));
            els.push((El::DoorFloor5, geom::door_floor5()));
        }
        Screen::Floor5Hub => {
            els.push((El::CampStone, geom::camp_stone()));
            for index in 0..5 {
                els.push((El::Floor5Door(index as u8), geom::door_chamber(index)));
            }
            els.push((El::DoorFoe5, geom::door_foe5()));
            els.push((El::DoorFloor6, geom::door_floor6()));
        }
        Screen::Floor6Hub => {
            els.push((El::CampStone, geom::camp_stone()));
            for index in 0..5 {
                els.push((El::Floor6Door(index as u8), geom::door_chamber(index)));
            }
            els.push((El::DoorFoe6, geom::door_foe6()));
        }

        Screen::Floor2Hub => {
            els.push((El::CampStone, geom::camp_stone()));
            for index in 0..5 {
                els.push((El::Floor2Door(index as u8), geom::door_chamber(index)));
            }
            els.push((El::DoorFoe2, geom::door_foe2()));
            els.push((El::DoorFloor3, geom::door_floor3()));
            for group in 0..content::ROSTER_GROUPS.len() {
                els.push((El::RosterToggle(group as u8), geom::roster_group(group)));
            }
            for letter in 0..content::HERALD_LETTERS.len() {
                els.push((El::F2HeraldAct(letter as u8), geom::f2_herald_act(letter)));
            }
        }
        Screen::Chamber(index) => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            layout_chamber_panel(&mut els, game, index);
            layout_seal(&mut els, game);
        }
        Screen::Floor4Chamber(index) => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            layout_floor4_chamber_panel(&mut els, game, index);
            layout_floor4_chamber34(&mut els, index);
            layout_floor4_chamber5(&mut els, index);
            layout_seal(&mut els, game);
        }

        Screen::Floor5Chamber(index) => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            layout_floor5_chamber_panel(&mut els, game, index);
            layout_seal(&mut els, game);
        }

        Screen::Floor6Chamber(index) => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            layout_floor6_chamber_panel(&mut els, game, index);
            layout_seal(&mut els, game);
        }

        Screen::Floor3Chamber(index) => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            layout_floor3_chamber_panel(&mut els, game, index);
            layout_seal(&mut els, game);
        }

        Screen::Floor2Chamber(index) => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            layout_floor2_chamber_panel(&mut els, game, index);
            layout_seal(&mut els, game);
        }
        Screen::Foe => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            els.push((El::EditorOpen, geom::editor_open()));
            if game.expedition.foe.enumerated {
                let uncopiable = puzzles::foe_uncopiable_rows();
                for (pos, row) in uncopiable.iter().enumerate() {
                    els.push((El::CensusCard(*row as u8), geom::foe_row(pos)));
                    let verified = game
                        .expedition
                        .foe
                        .verified
                        .get(*row)
                        .copied()
                        .unwrap_or(false);
                    if !verified {
                        els.push((El::VerifyRow(*row as u8), geom::verify_btn(pos)));
                    }
                }
            }
            layout_seal(&mut els, game);
        }
        Screen::Floor3Foe => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            els.push((El::EditorOpen, geom::editor_open()));
            if !game.expedition.floor3.foe.slate_taken {
                els.push((El::F3SlateTake, geom::f3_slate_take()));
            } else {
                for entry in 0..content::LOG_SLATE.len() {
                    els.push((El::F3SlateEntry(entry as u8), geom::f3_slate_entry(entry)));
                }
                if !game.expedition.floor3.foe.slate_done {
                    els.push((El::F3Reconcile, geom::f3_reconcile()));
                }
                els.push((El::F3Stamp, geom::f3_stamp()));
                els.push((El::F3Rite, geom::f3_rite_field()));
                els.push((El::F3Lantern(0), geom::f3_lantern(0)));
                els.push((El::F3Lantern(1), geom::f3_lantern(1)));
            }
            layout_seal(&mut els, game);
        }
        Screen::Floor4Foe => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            if !game.expedition.floor4.foe.book_taken {
                els.push((El::F4BookTake, geom::f4_book_take()));
            } else {
                for entry in 0..content::DIMENSION_BOOK.len() {
                    els.push((El::F4Measure(entry as u8), geom::f4_measure_btn(entry)));
                    if !game.expedition.floor4.foe.exhibited[entry] {
                        els.push((El::F4Exhibit(entry as u8), geom::f4_exhibit_btn(entry)));
                    }
                }
                for stone in 0..content::RECUT_FOREIGN.len() {
                    els.push((El::F4ForeignSeat(stone as u8), geom::f4_foreign_row(stone)));
                }
                els.push((El::F4ForeignProve, geom::f4_foreign_prove()));
                els.push((El::F4ForeignReset, geom::f4_foreign_reset()));
                for stone in 0..content::RECUT_FREE.len() {
                    els.push((El::F4FreeSeat(stone as u8), geom::f4_free_row(stone)));
                }
                els.push((El::F4FreeProve, geom::f4_free_prove()));
                els.push((El::F4FreeReset, geom::f4_free_reset()));
            }
            layout_seal(&mut els, game);
        }

        Screen::Floor5Foe => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            for exhibit in 0..content::FOE5_EXHIBITS.len() {
                els.push((
                    El::F5Exhibit(exhibit as u8),
                    geom::f5foe_exhibit_btn(exhibit),
                ));
            }
            for case in 0..content::FOE5_CASES.len() {
                els.push((El::F5Case(case as u8), geom::f5foe_case_btn(case)));
            }
            for line in 0..content::FOE5_CARD.len() {
                els.push((El::F5FoeCard(line as u8), geom::f5foe_card_btn(line)));
            }
            layout_seal(&mut els, game);
        }
        Screen::Floor6Foe => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            for exhibit in 0..content::FOE6_EXHIBIT_NAMES.len() {
                els.push((
                    El::F6Exhibit(exhibit as u8),
                    geom::f6foe_exhibit_btn(exhibit),
                ));
            }
            for case in 0..content::FOE6_CASES.len() {
                els.push((El::F6Loom(case as u8), geom::f6foe_case_btn(case)));
            }
            for line in 0..content::FOE6_CARD.len() {
                els.push((El::F6FoeCard(line as u8), geom::f6foe_card_btn(line)));
            }
            layout_seal(&mut els, game);
        }

        Screen::Floor2Foe => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            els.push((El::EditorOpen, geom::editor_open()));
            if game.expedition.floor2.foe.audit_taken {
                for row in 0..content::FOE_ADMISSIONS.len() {
                    els.push((El::F2FoeName(row as u8), geom::f2foe_row(row)));
                    let exhibited = game
                        .expedition
                        .floor2
                        .foe
                        .audit_exhibited
                        .get(row)
                        .copied()
                        .unwrap_or(false);
                    if !exhibited {
                        els.push((El::F2FoeExhibit(row as u8), geom::f2foe_exhibit_btn(row)));
                    }
                }
                els.push((El::F2FoeCalibrate, geom::f2foe_calibrate()));
            }
            layout_seal(&mut els, game);
        }
        Screen::Title => {}
    }
    let verbs = game.verbs();
    let count = verbs.len();
    for (index, verb) in verbs.into_iter().enumerate() {
        els.push((El::Chip(verb), rects::chip(index, count)));
    }
    // While an overlay stands open, it owns input entirely: the
    // layout for hit-testing and focus is the overlay's alone.
    let mut over: Vec<(El, Rect)> = Vec::new();
    match game.overlay {
        Overlay::None => return els,
        Overlay::Reading(kind) => {
            over.push((El::ReadingScale, rects::READING_SCALE));
            over.push((El::OverlayClose, rects::OVERLAY_CLOSE));
            if kind == ReadingKind::Hint && game.hint_showing < 4 {
                over.push((El::HintDeeper, rects::HINT_DEEPER));
            }
        }
        Overlay::Map => {
            over.push((El::OverlayClose, rects::MAP_CLOSE));
        }
        Overlay::Log | Overlay::Codex | Overlay::GateRecord => {
            over.push((El::OverlayClose, rects::OVERLAY_CLOSE));
        }
        Overlay::Pause => {
            for index in 0..3 {
                over.push((El::Setting(index), rects::setting(index)));
            }
            over.push((El::Export, rects::setting(3)));
            over.push((El::Import, rects::setting(4)));
            over.push((El::Resume, rects::setting(5)));
        }
        Overlay::Editor => {
            over.push((El::EditorArea, rects::EDITOR_AREA));
            over.push((El::RunGate, rects::RUN_GATE));
            over.push((El::ResetProof, rects::RESET_PROOF));
            over.push((El::GateRecordBtn, rects::GATE_RECORD_BTN));
            over.push((El::OverlayClose, rects::EDITOR_CLOSE));
        }
    }
    over
}

/// The seal elements for the current screen, when relevant.
fn layout_seal(els: &mut Vec<(El, Rect)>, game: &Game) {
    let armed = match game.screen {
        Screen::Chamber(index) => game.armed == Some(ArmedFor::Chamber(index)),
        Screen::Floor2Chamber(index) => game.armed == Some(ArmedFor::Floor2Chamber(index)),
        Screen::Floor3Chamber(index) => game.armed == Some(ArmedFor::Floor3Chamber(index)),
        Screen::Floor4Chamber(index) => game.armed == Some(ArmedFor::Floor4Chamber(index)),
        Screen::Floor5Chamber(index) => game.armed == Some(ArmedFor::Floor5Chamber(index)),
        Screen::Floor6Chamber(index) => game.armed == Some(ArmedFor::Floor6Chamber(index)),
        Screen::Foe => game.expedition.foe.strike == StrikePhase::Armed,
        Screen::Floor2Foe => game.expedition.floor2.foe.strike == StrikePhase::Armed,
        Screen::Floor3Foe => game.expedition.floor3.foe.strike == StrikePhase::Armed,
        Screen::Floor4Foe => game.expedition.floor4.foe.strike == StrikePhase::Armed,
        Screen::Floor5Foe => game.expedition.floor5.foe.strike == StrikePhase::Armed,
        Screen::Floor6Foe => game.expedition.floor6.foe.strike == StrikePhase::Armed,
        _ => false,
    };
    els.push((El::Seal, rects::SEAL));
    if armed {
        els.push((El::SealConfirm, rects::SEAL_CONFIRM));
        els.push((El::SealCancel, rects::SEAL_CANCEL));
    }
}

/// The Floor 6 chambers' panel elements: the watch; each
/// chamber's own controls land with its step.
fn layout_floor6_chamber_panel(els: &mut Vec<(El, Rect)>, game: &Game, chamber: usize) {
    let _ = game;
    els.push((El::F6Watch, geom::f2_watch()));
    match chamber {
        0 => {
            for op in 0..6 {
                els.push((El::F6DayOp(op as u8), geom::f6_op_row(op)));
            }
        }
        1 => {
            for call in 0..5 {
                els.push((El::F6BrazierCall(call as u8), geom::f6_op_row(call)));
            }
        }
        2 => {
            for claim in 0..4 {
                els.push((El::F6Annotate(claim as u8), geom::f6_op_row(claim)));
            }
            els.push((El::F6BoundDown, geom::f6_op_row(4)));
            els.push((El::F6BoundUp, geom::f6_op_row(5)));
            els.push((El::F6SpinRun, geom::f6_op_row(6)));
        }
        3 => {
            for class in 0..3 {
                els.push((El::F6CensusCycle(class as u8), geom::f6_op_row(class)));
            }
            els.push((El::F6CensusSubmit, geom::f6_op_row(3)));
        }
        4 => {
            for datum in 0..3 {
                els.push((El::F6Choose(datum as u8), geom::f6_datum_row(datum)));
                els.push((El::F6Invariant(datum as u8), geom::f6_sentence_btn(datum)));
            }
        }
        _ => {}
    }
}

/// The Floor 5 chambers' panel elements: the watch; each
/// chamber's own controls land with its step.
fn layout_floor5_chamber_panel(els: &mut Vec<(El, Rect)>, game: &Game, chamber: usize) {
    els.push((El::F5Watch, geom::f2_watch()));
    if chamber == 0 {
        for step in 0..content::STEP_CARDS.len() {
            els.push((El::F5Card(step as u8), geom::f5_card_row(step)));
        }
        els.push((El::F5Walk, geom::f5_walk_btn()));
        els.push((El::F5RefuseStep, geom::f5_refuse_btn()));
    }
    if chamber == 1 {
        for index in 0..content::TENANCIES.len() {
            els.push((El::F5Placard(index as u8), geom::f5_placard_row(index)));
            let work = &game.expedition.floor5.chambers[1].work;
            if work.judged && puzzles::tenancy_missed(&work.predictions, index) {
                els.push((El::F5Annotate(index as u8), geom::f5_annotate_row(index)));
            }
        }
        els.push((El::F5Judge, geom::f5_judge_btn()));
    }
    if chamber == 2 {
        for tool in 0..content::CARD_TOOLS.len() {
            for line in 0..3 {
                els.push((
                    El::F5CardLine(tool as u8, line as u8),
                    geom::f5_cardline_row(tool, line),
                ));
            }
        }
    }
    if chamber == 4 {
        for station in 0..6 {
            els.push((El::F5Verdict(station as u8), geom::f5_station_row(station)));
            els.push((El::F5Lens(station as u8), geom::f5_lens_btn(station)));
            let work = &game.expedition.floor5.chambers[4].work;
            if work.verdicts[station] == 2 && crate::content::STATIONS[station].fault.is_some() {
                els.push((El::F5FixCard(station as u8), geom::f5_fix_btn(station)));
            }
        }
    }
    if chamber == 3 {
        els.push((El::F5SplitDown, geom::f5_split_down()));
        els.push((El::F5SplitUp, geom::f5_split_up()));
        els.push((El::F5Seam, geom::f5_seam_btn()));
        els.push((El::F5Reach, geom::f5_reach_btn()));
        els.push((El::F5WorkHalves, geom::f5_work_btn()));
        els.push((El::F5DropLeft, geom::f5_drop_left_btn()));
        els.push((El::F5DropRight, geom::f5_drop_right_btn()));
    }
}

/// The Floor 4 chambers' panel elements: the watch, Chamber
/// 1's stone rows, steppers, line drop, and annotation;
/// Chamber 2's seat rows and the wall's reset.
fn layout_floor4_chamber_panel(els: &mut Vec<(El, Rect)>, game: &Game, chamber: usize) {
    let _ = game;
    els.push((El::F4Watch, geom::f2_watch()));
    if chamber == 0 {
        for stone in 0..content::BENCH_STONES.len() {
            els.push((El::F4Stone(stone as u8), geom::f4_stone_row(stone)));
        }
        els.push((El::F4SizeDown, geom::f4_size_down()));
        els.push((El::F4SizeUp, geom::f4_size_up()));
        els.push((El::F4AlignDown, geom::f4_align_down()));
        els.push((El::F4AlignUp, geom::f4_align_up()));
        els.push((El::F4DropLine, geom::f4_drop_line()));
        els.push((El::F4Annotate, geom::f4_annotate()));
    }
    if chamber == 1 {
        for stone in 0..content::WALL_STONES.len() {
            els.push((El::F4Seat(stone as u8), geom::f4_seat_row(stone)));
        }
        els.push((El::F4WallReset, geom::f4_wall_reset()));
    }
}

/// Floor 4 Chambers 3–4 panel elements: the crests' cuts
/// and proofs; the treaty tray, the line, the amendment.
fn layout_floor4_chamber34(els: &mut Vec<(El, Rect)>, chamber: usize) {
    if chamber == 2 {
        for crest in 0..3u8 {
            els.push((El::F4Crest(crest), geom::f4_crest_row(crest as usize)));
            els.push((
                El::F4CrestProve(crest),
                geom::f4_crest_prove(crest as usize),
            ));
        }
    }
    if chamber == 3 {
        for stone in 0..5u8 {
            els.push((El::F4TreatySeat(stone), geom::f4_treaty_row(stone as usize)));
        }
        els.push((El::F4PromiseDown, geom::f4_size_down()));
        els.push((El::F4PromiseUp, geom::f4_size_up()));
        els.push((El::F4TreatyProve, geom::f4_treaty_prove()));
        els.push((El::F4TreatyReset, geom::f4_treaty_reset()));
    }
}

/// Floor 4 Chamber 5 panel elements: the hollow stones'
/// fillings, tags, and readings; the blank's cells and use.
fn layout_floor4_chamber5(els: &mut Vec<(El, Rect)>, chamber: usize) {
    if chamber == 4 {
        for stone in 0..2u8 {
            els.push((El::F4Fill(stone), geom::f4_fill_btn(stone as usize)));
            els.push((El::F4Tag(stone), geom::f4_tag_btn(stone as usize)));
            els.push((El::F4Read(stone), geom::f4_read_btn(stone as usize)));
        }
        for cell in 0..4u8 {
            els.push((El::F4Cell(cell), geom::f4_cell(cell as usize)));
        }
        els.push((El::F4UseBlank, geom::f4_use_blank()));
    }
}

/// The Floor 3 chambers' panel elements: the watch, Chamber 1's
/// four pair buttons, Chamber 2's three station buttons.
fn layout_floor3_chamber_panel(els: &mut Vec<(El, Rect)>, game: &Game, chamber: usize) {
    let _ = game;
    els.push((El::F3Watch, geom::f2_watch()));
    if chamber == 0 {
        for report in 0..content::FAILURE_REPORTS.len() {
            els.push((El::F3Pair(report as u8), geom::f3_pair(report)));
        }
    }
    if chamber == 1 {
        for slot in 0..3 {
            els.push((El::F3Station(slot as u8), geom::f3_station(slot)));
        }
    }
    if chamber == 2 {
        for case in 0..content::SHOUT_CASES.len() {
            els.push((El::F3Shout(case as u8), geom::f3_shout(case)));
            els.push((El::F3Invariant(case as u8), geom::f3_invariant(case)));
        }
    }
    if chamber == 3 {
        for slip in 0..content::REQUEST_SLIPS.len() {
            for action in 0..3 {
                els.push((
                    El::F3Slip(slip as u8, action as u8),
                    geom::f3_slip(slip, action),
                ));
            }
        }
    }
    if chamber == 4 {
        for item in 0..content::INSPECTION_ITEMS.len() {
            els.push((El::F3Check(item as u8), geom::f3_check(item)));
        }
        els.push((El::F3NaReason, geom::f3_na_reason()));
        els.push((El::F3Msrv, geom::f3_msrv()));
        els.push((El::F3Rite, geom::f3_rite_field()));
    }
}

/// The Floor 3 chamber facet lines for the Examine verb.
const F3_EXAMINE_TEXTS: [&str; 5] = [
    "The Named Failures desk: four reports on the wall, five pairs \
     on the desk, and the great bin chalked BROKEN standing fuller \
     than every named door together.",
    "The Conveyor: gear, gear, spring, a cracked gear, spring, \
     cam, cam, bell — and three stations between the belt and \
     the office.",
    "The Audit of Shouts: a soundproof booth, a ledger of four \
     shouts, and two chairs — one for the failures a traveller \
     can cause, one for the ones only the house can.",
    "The Sealed Request: a press, a wax pot, and three slips on \
     a rail that runs unaddressed, addressed, bodied, sealed — \
     and nowhere else.",
    "The Inspection: the Guidelines wall and the Hoard open under \
     audit. This desk is not open yet.",
];
/// The Floor 2 chamber panels' elements. Chamber 1: the watch
/// button, the three charter candidates, and the two bodies the
/// blind run may be named as. Chamber 2: one sigil toggle per
/// sigil per stable service. Chamber 3: the run button, the three
/// errands, and the two kennels of the finding. Chamber 4: the
/// drive button and the bar-naming chips. Chamber 5: the two
/// panel tabs, the three routes, and the collision readings.
fn layout_floor2_chamber_panel(els: &mut Vec<(El, Rect)>, game: &Game, chamber: usize) {
    let _ = game;
    if chamber == 0 {
        els.push((El::F2Watch, geom::f2_watch()));
        for choice in 0..content::CHARTER_CANDIDATES.len() {
            els.push((El::F2Charter(choice as u8), geom::f2_charter(choice)));
        }
        for choice in 0..content::TRICK_PERFORMERS.len() {
            els.push((El::F2RunChoice(choice as u8), geom::f2_run(choice)));
        }
    }
    if chamber == 1 {
        for service in 0..content::STABLE_SERVICES.len() {
            for sigil in 0..content::STABLE_SIGILS.len() {
                els.push((
                    El::F2BoundToggle(service as u8, sigil as u8),
                    geom::f2_bound(service, sigil),
                ));
            }
        }
    }
    if chamber == 2 {
        els.push((El::F2RunDeliveries, geom::f2_run_deliveries()));
        for errand in 0..content::ERRANDS.len() {
            els.push((El::F2Errand(errand as u8), geom::f2_errand(errand)));
        }
        for kennel in 0..content::KENNELS.len() {
            els.push((El::F2Kennel(kennel as u8), geom::f2_kennel(kennel)));
        }
    }
    if chamber == 3 {
        els.push((El::F2DriveRound, geom::f2_drive()));
        let barred = content::REGISTRY_BEASTS
            .iter()
            .filter(|entry| !entry.passes)
            .count();
        for slot in 0..barred {
            for label in 0..content::BAR_REASON_LABELS.len() {
                els.push((
                    El::F2BarLabel(slot as u8, label as u8),
                    geom::f2_bar(slot, label),
                ));
            }
        }
    }
    if chamber == 4 {
        els.push((El::F2PanelTab(0), geom::panel_tab(0)));
        els.push((El::F2PanelTab(1), geom::panel_tab(1)));
        for route in 0..content::COLLAR_ROUTES.len() {
            els.push((El::F2Route(route as u8), geom::f2_route(route)));
        }
        for reading in 0..content::COLLISION_READINGS.len() {
            els.push((El::F2Reading(reading as u8), geom::f2_reading(reading)));
        }
    }
}

/// The per-chamber panel elements.
fn layout_chamber_panel(els: &mut Vec<(El, Rect)>, game: &Game, chamber: usize) {
    let work = &game.expedition.chambers[chamber].work;
    match chamber {
        0 => {
            els.push((El::PanelTab(0), geom::panel_tab(0)));
            els.push((El::PanelTab(1), geom::panel_tab(1)));
            if game.c1_view == 0 {
                for item in 0..content::PACK_ITEMS.len() {
                    for porter in 0..3 {
                        els.push((
                            El::HolderToggle(item as u8, porter as u8),
                            geom::holder_btn(item, porter),
                        ));
                    }
                }
            } else {
                for choice in 0..content::TOLL_CANDIDATES.len() {
                    els.push((El::TollChoice(choice as u8), geom::toll_btn(choice)));
                }
            }
        }
        1 => {
            for person in 0..content::READERS.len() {
                els.push((El::PickToggle(person as u8), geom::pick_btn(person)));
            }
        }
        2 => {
            for lease in 0..3 {
                for field in 0..2 {
                    els.push((
                        El::LeaseStep(lease as u8, field as u8, -1),
                        geom::lease_step(lease, field, -1),
                    ));
                    els.push((
                        El::LeaseStep(lease as u8, field as u8, 1),
                        geom::lease_step(lease, field, 1),
                    ));
                }
            }
        }
        3 => {
            els.push((El::RunDelivery, geom::run_btn()));
            els.push((El::CommitToggle, geom::commit_btn()));
            for record in 0..work.records.len() {
                els.push((El::RecordCard(record as u8), geom::record_card(record)));
            }
            els.push((El::ProofSlot(0), rects::SLOT));
            els.push((El::ProofSlot(1), rects::SLOT2));
        }
        4 => {
            els.push((El::PagePrev, geom::page_prev()));
            els.push((El::PageNext, geom::page_next()));
            let page = game.c5_page as usize;
            for local in 0..4 {
                let global = page * 4 + local;
                if global < content::PLACARDS.len() {
                    els.push((El::JudgeToggle(global as u8), geom::judge_btn(local)));
                }
            }
        }
        _ => {}
    }
}

/// The element at a point, topmost first.
pub fn hit_test(game: &Game, x: f32, y: f32) -> Option<El> {
    layout(game)
        .iter()
        .rev()
        .find(|(_, rect)| rect.contains(x, y))
        .map(|(el, _)| *el)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor_game() -> Game {
        let mut game = Game::new();
        game.begin();
        game.io = None; // the shell drains the begin-autosave at once
        game.screen = Screen::Foe;
        game.overlay = Overlay::Editor;
        game.expedition.editor_code = "line one\nline two\nline three\nline four\n".to_string();
        game.editor_cursor = 0;
        game
    }

    /// Regression (found by the 2026-11-09 scripted playthrough):
    /// while the editor overlay was open, pointer_up returned
    /// early and NO editor button could ever be pressed — the
    /// FOE's gate was unreachable by mouse or touch.
    #[test]
    fn editor_buttons_answer_the_pointer() {
        let mut game = editor_game();
        let rect = rects::RUN_GATE;
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert!(
            matches!(game.io, Some(IoRequest::RunGate { .. })),
            "Run the gate must answer the pointer while the editor is open"
        );
        assert!(game.gate_running);
    }

    /// The other half of that regression: pressing a button must
    /// not also place the editor cursor (the playthrough watched
    /// the cursor drift four lines on the Run-the-gate click).
    #[test]
    fn editor_button_press_keeps_the_cursor() {
        let mut game = editor_game();
        let rect = rects::RUN_GATE;
        game.pointer_down(rect.cx(), rect.cy());
        assert_eq!(
            game.editor_cursor, 0,
            "button press must not place the cursor"
        );
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(game.editor_cursor, 0);
    }

    /// Adversarial: clicking the text area places the cursor and
    /// files no request of any kind.
    #[test]
    fn editor_area_click_places_cursor_and_files_nothing() {
        let mut game = editor_game();
        let area = rects::EDITOR_AREA;
        let x = area.x + EDITOR_PAD_X + 4.0;
        let y = area.y + EDITOR_LINE_H * 2.5;
        game.pointer_down(x, y);
        game.pointer_up(x, y);
        assert!(game.io.is_none(), "a text click must not file a request");
        assert!(game.editor_cursor > 0, "the click must place the cursor");
        assert_eq!(game.focus, Some(El::EditorArea));
    }

    /// Floor 2's hub door answers the pointer: the Menagerie's
    /// chamber opens from the grounds.
    #[test]
    fn floor2_foe_door_answers_the_pointer() {
        let mut game = Game::new();
        game.begin();
        game.screen = Screen::Floor2Hub;
        let rect = geom::door_foe2();
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(game.screen, Screen::Floor2Foe);
    }

    /// The editor at the Menagerie's desk edits the Desk draft,
    /// never the Hoard: the two drafts are separate records.
    #[test]
    fn editor_at_the_desk_edits_the_desk_draft() {
        let mut game = Game::new();
        game.begin();
        game.io = None; // the shell drains the begin-autosave at once
        game.screen = Screen::Floor2Foe;
        game.overlay = Overlay::Editor;
        let hoard_before = game.expedition.editor_code.clone();
        game.editor_insert("x");
        assert_eq!(game.expedition.editor_code, hoard_before);
        assert!(game.expedition.floor2_foe_draft().starts_with('x'));
    }

    /// Step 5 integration: the whole of Floor 2 driven through
    /// the GUI surface itself — layout hit-testing and pointer
    /// presses, the game's own editor, and the real Desk gate
    /// answered through the io slot exactly as the shell answers
    /// it. Fixture: Floor 1's FOE is felled at the state level
    /// first (the way down is barred until the Hoard falls);
    /// everything on Floor 2 itself is played, not set.
    #[test]
    fn floor2_gui_playthrough_fells_the_menagerie() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn repair(game: &mut Game, index: usize) {
            click(game, El::Chip(Verb::Prove));
            click(game, El::Seal);
            click(game, El::SealConfirm);
            assert_eq!(
                game.expedition.floor2.chambers[index].state,
                ChamberState::Repaired,
                "chamber {index} repaired through the seal"
            );
            click(game, El::TabFloor);
            click(game, El::DoorFloor2);
        }

        let mut game = Game::new();
        game.begin();
        game.io = None; // the shell drains the begin-autosave at once
        // Fixture: fell Floor 1's FOE at the state level.
        {
            let expedition = &mut game.expedition;
            expedition.foe_enumerate();
            let double = crate::puzzles::foe_double_row().expect("double row");
            expedition.foe_exhibit_row(double).expect("exhibit");
            for row in crate::puzzles::foe_uncopiable_rows() {
                expedition.foe_verify_row(row).expect("verify");
            }
            expedition.foe_mark_gate_passed();
            expedition.foe_arm_strike().expect("arm");
            expedition.foe_confirm_strike("2026-11-10").expect("strike");
        }
        click(&mut game, El::DoorFloor2);
        assert_eq!(game.screen, Screen::Floor2Hub);

        // Chamber 1 — Traits and impls.
        click(&mut game, El::Floor2Door(0));
        click(&mut game, El::F2Watch);
        let charter = (0..content::CHARTER_CANDIDATES.len())
            .find(|choice| crate::puzzles::trick_charter_ok(content::CHARTER_CANDIDATES[*choice]))
            .expect("a true charter");
        click(&mut game, El::F2Charter(charter as u8));
        let runner = (0..content::TRICK_PERFORMERS.len())
            .find(|choice| crate::puzzles::identification_ok(*choice))
            .expect("the blind run named");
        click(&mut game, El::F2RunChoice(runner as u8));
        repair(&mut game, 0);

        // Chamber 2 — Generics and bounds: toggle each service's
        // bound to exactly the sigils its work requires.
        click(&mut game, El::Floor2Door(1));
        for (service, spec) in content::STABLE_SERVICES.iter().enumerate() {
            let current = game.expedition.floor2.chambers[1]
                .work
                .bound_masks
                .get(service)
                .copied()
                .unwrap_or(content::POSTED_BOUND);
            for sigil in 0..content::STABLE_SIGILS.len() {
                let want = spec.required & (1 << sigil) != 0;
                let has = current & (1 << sigil) != 0;
                if want != has {
                    click(&mut game, El::F2BoundToggle(service as u8, sigil as u8));
                }
            }
        }
        repair(&mut game, 1);

        // Chamber 3 — Associated types.
        click(&mut game, El::Floor2Door(2));
        click(&mut game, El::F2RunDeliveries);
        let errand = (0..content::ERRANDS.len())
            .find(|choice| crate::puzzles::errand_choice_ok(*choice))
            .expect("the contorted errand");
        click(&mut game, El::F2Errand(errand as u8));
        let kennel = (0..content::KENNELS.len())
            .find(|kennel| crate::puzzles::kennel_choice_ok(*kennel))
            .expect("the found kennel");
        click(&mut game, El::F2Kennel(kennel as u8));
        repair(&mut game, 2);

        // Chamber 4 — Trait objects and the Wide Door.
        click(&mut game, El::Floor2Door(3));
        click(&mut game, El::F2DriveRound);
        for (slot, label) in [(0u8, 0u8), (1, 1), (2, 2)] {
            click(&mut game, El::F2BarLabel(slot, label));
        }
        repair(&mut game, 3);

        // Chamber 5 — Coherence.
        click(&mut game, El::Floor2Door(4));
        click(&mut game, El::F2Route(1));
        click(&mut game, El::F2Reading(2));
        repair(&mut game, 4);

        // The Herald's letters, acted from the hub strip.
        for letter in 0..content::HERALD_LETTERS.len() {
            click(&mut game, El::F2HeraldAct(letter as u8));
        }
        assert!(
            game.expedition
                .floor2
                .herald_filed
                .iter()
                .all(|filed| *filed)
        );

        // The FOE: the audit, the exhibits, the naming, the
        // brazier calibration.
        click(&mut game, El::DoorFoe2);
        assert_eq!(game.screen, Screen::Floor2Foe);
        click(&mut game, El::Chip(Verb::Enumerate));
        assert!(game.expedition.floor2.foe.audit_taken);
        for row in 0..content::FOE_ADMISSIONS.len() {
            click(&mut game, El::F2FoeExhibit(row as u8));
        }
        let seam = crate::puzzles::foe2_disagreement_row().expect("the seam");
        click(&mut game, El::F2FoeName(seam as u8));
        click(&mut game, El::F2FoeCalibrate);
        assert_eq!(game.expedition.floor2.foe.state, FoeState::Exposed);

        // The Desk, written in the game's own editor and judged
        // by the real gate through the io slot.
        click(&mut game, El::EditorOpen);
        let reference = content::GATE2_SCAFFOLD
            .replacen("todo!()", "beast", 1)
            .replacen("todo!()", "beast", 1)
            .replacen(
                "todo!()",
                "!bound.is_empty() && bound.iter().all(|sigil| sigils.contains(sigil))",
                1,
            );
        game.editor_select_all();
        for ch in reference.chars() {
            let mut buf = [0u8; 4];
            game.editor_insert(ch.encode_utf8(&mut buf));
        }
        click(&mut game, El::RunGate);
        let code = match game.io.take() {
            Some(IoRequest::RunGate { code, floor2, .. }) => {
                assert!(floor2, "the desk files the second gate");
                code
            }
            other => panic!("expected a gate filing, got {other:?}"),
        };
        let scratch = std::env::temp_dir().join("niflheim-step5-playthrough-gate");
        let report = crate::gate::run_gate2(&scratch, &code, &crate::gate::GateLimits::default());
        assert!(report.passed(), "the Desk gate: {}", report.summary);
        game.on_gate_result(report);
        assert!(game.expedition.floor2.foe.gate_record.is_some());
        click(&mut game, El::OverlayClose);

        // The strike, through the seal.
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor2.foe.state, FoeState::Fallen);
        assert!(
            game.expedition
                .codex
                .iter()
                .any(|s| s == content::LENS_STONE)
        );
        assert!(
            game.expedition
                .pack
                .iter()
                .any(|s| s == content::PLUGIN_MENAGERIE)
        );
        assert!(game.expedition.floor2_complete());

        // The stairs recite the floor's standing: clause 3 reads
        // both Stratum I lanterns lit.
        let clauses = game.expedition.stairs_clauses();
        assert!(
            clauses[2].1.contains("Floor 1's lantern is lit")
                && clauses[2].1.contains("Floor 2's lantern is lit"),
            "clause 3: {}",
            clauses[2].1
        );

        // The whole record round-trips through the save.
        let save = game.expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&save).expect("load");
        assert_eq!(loaded, game.expedition);
    }

    /// Floor 3's door answers the pointer when both valley
    /// FOEs have fallen, and refuses it while either stands.
    #[test]
    fn floor3_door_answers_the_pointer_when_reachable() {
        let mut game = Game::new();
        game.begin();
        game.io = None;
        game.screen = Screen::Floor2Hub;
        let rect = geom::door_floor3();
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(
            game.screen,
            Screen::Floor2Hub,
            "barred while the FOEs stand"
        );
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(game.screen, Screen::Floor3Hub);
    }

    /// Driven playthrough: Floor 3's first two chambers played
    /// on the GUI surface — the sort judged pair by pair (the
    /// Broken bin refused by the filing), the belt set station
    /// by station, both proofs filed, both seals struck.
    #[test]
    fn floor3_chambers_1_and_2_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.screen = Screen::Floor3Hub;

        // Chamber 1 — Named Failures.
        click(&mut game, El::Floor3Door(0));
        assert_eq!(game.screen, Screen::Floor3Chamber(0));
        click(&mut game, El::F3Watch);
        assert!(game.expedition.floor3.chambers[0].work.watched);
        // Sort each report: from unsorted, the pair button cycles
        // 0,1,2,3,4 — report r's own pair takes pair+1 presses.
        for (report, entry) in content::FAILURE_REPORTS.iter().enumerate() {
            for _ in 0..=entry.pair {
                click(&mut game, El::F3Pair(report as u8));
            }
        }
        assert!(game.expedition.floor3_diagnosed(0));
        // One report re-sorted under Broken unmakes the proof.
        click(&mut game, El::F3Pair(0));
        assert!(!game.expedition.floor3_diagnosed(0));
        assert!(game.expedition.floor3_file_proof(0).is_err());
        for _ in 0..4 {
            click(&mut game, El::F3Pair(0));
        }
        assert!(game.expedition.floor3_diagnosed(0));
        game.do_verb(Verb::Prove);
        assert_eq!(
            game.expedition.floor3.chambers[0].state,
            ChamberState::Diagnosed
        );
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor3.chambers[0].state,
            ChamberState::Repaired
        );

        // Chamber 2 — the Conveyor.
        game.screen = Screen::Floor3Hub;
        click(&mut game, El::Floor3Door(1));
        assert_eq!(game.screen, Screen::Floor3Chamber(1));
        click(&mut game, El::F3Watch);
        // Stations cycle from unset: portion 2 takes 3 presses,
        // dedup 1 takes 2, halt 1 takes 2.
        for _ in 0..3 {
            click(&mut game, El::F3Station(0));
        }
        for _ in 0..2 {
            click(&mut game, El::F3Station(1));
        }
        for _ in 0..2 {
            click(&mut game, El::F3Station(2));
        }
        assert!(game.expedition.floor3_diagnosed(1));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor3.chambers[1].state,
            ChamberState::Repaired
        );
        assert_eq!(game.expedition.floor3_chambers_repaired(), 2);
    }

    /// Driven playthrough: Floor 3's third and fourth chambers
    /// on the GUI surface — the audit judged (one shout first
    /// left invariant-less, and the filing refused), and the
    /// rail worked against its interlock (the press refused
    /// early, twice, before it may fall).
    #[test]
    fn floor3_chambers_3_and_4_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.screen = Screen::Floor3Hub;

        // Chamber 3 — the Audit of Shouts.
        click(&mut game, El::Floor3Door(2));
        assert_eq!(game.screen, Screen::Floor3Chamber(2));
        click(&mut game, El::F3Watch);
        // Verdicts cycle unjudged → refusal → stands: cases 0,2
        // take one press, cases 1,3 take two.
        click(&mut game, El::F3Shout(0));
        click(&mut game, El::F3Shout(1));
        click(&mut game, El::F3Shout(1));
        click(&mut game, El::F3Shout(2));
        click(&mut game, El::F3Shout(3));
        click(&mut game, El::F3Shout(3));
        // Invariant buttons cycle unjudged → 1 → 2 → 3 → 4:
        // case 1 carries invariant 1 (one press), case 3
        // invariant 2 (two presses).
        click(&mut game, El::F3Invariant(1));
        click(&mut game, El::F3Invariant(3));
        assert!(!game.expedition.floor3_diagnosed(2));
        click(&mut game, El::F3Invariant(3));
        assert!(game.expedition.floor3_diagnosed(2));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor3.chambers[2].state,
            ChamberState::Repaired
        );

        // Chamber 4 — the Sealed Request.
        game.screen = Screen::Floor3Hub;
        click(&mut game, El::Floor3Door(3));
        assert_eq!(game.screen, Screen::Floor3Chamber(3));
        click(&mut game, El::F3Watch);
        // The press refuses the first slip unbodied (state
        // unchanged), then the order is worked honestly.
        click(&mut game, El::F3Slip(0, 1));
        assert_eq!(game.expedition.floor3.chambers[3].work.slip_stages[0], 0);
        click(&mut game, El::F3Slip(0, 0));
        click(&mut game, El::F3Slip(0, 0));
        click(&mut game, El::F3Slip(0, 1));
        click(&mut game, El::F3Slip(1, 0));
        click(&mut game, El::F3Slip(1, 1));
        // The stranger's slip cannot advance; it returns.
        click(&mut game, El::F3Slip(2, 0));
        assert_eq!(game.expedition.floor3.chambers[3].work.slip_stages[2], 1);
        click(&mut game, El::F3Slip(2, 2));
        assert!(game.expedition.floor3_diagnosed(3));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor3.chambers[3].state,
            ChamberState::Repaired
        );
        assert_eq!(game.expedition.floor3_chambers_repaired(), 2);
    }

    /// Driven playthrough: Chamber 5 and the Release on the
    /// GUI surface — the inspection answered, the rite gating
    /// the seal on the record, and the FOE's arc: slate,
    /// stamp, rite, lanterns lit through the game's own
    /// result handlers, an edit dousing them, and the strike.
    #[test]
    fn floor3_chamber5_and_the_release_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        // Chambers 1–4 stand repaired (their own tests played
        // them); this test plays 5 and the FOE.
        for index in 0..4 {
            game.expedition.floor3.chambers[index].state = ChamberState::Repaired;
        }
        game.screen = Screen::Floor3Hub;

        // Chamber 5 — the Inspection.
        click(&mut game, El::Floor3Door(4));
        assert_eq!(game.screen, Screen::Floor3Chamber(4));
        click(&mut game, El::F3Watch);
        // Verdicts cycle unanswered → conforms → repaired →
        // not applicable: item truths are [1, 0, 2, 0, 1].
        for (item, presses) in [(0, 2), (1, 1), (2, 3), (3, 1), (4, 2)] {
            for _ in 0..presses {
                click(&mut game, El::F3Check(item));
            }
        }
        click(&mut game, El::F3NaReason); // — unnamed — → reason 0
        click(&mut game, El::F3Msrv); // — unstamped — → stamp 0
        assert!(game.expedition.floor3_diagnosed(4));
        game.do_verb(Verb::Prove);
        // The seal refuses while the rite stands undone.
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor3.chambers[4].state,
            ChamberState::Diagnosed,
            "the rite gates the repair"
        );
        click(&mut game, El::F3Rite);
        click(&mut game, El::F3Rite);
        click(&mut game, El::F3Rite);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor3.chambers[4].state,
            ChamberState::Repaired
        );

        // The Release: back up through the hubs to Floor 3's.
        click(&mut game, El::TabFloor);
        click(&mut game, El::DoorFloor2);
        click(&mut game, El::DoorFloor3);
        assert_eq!(game.screen, Screen::Floor3Hub);
        click(&mut game, El::DoorFoe3);
        assert_eq!(game.screen, Screen::Floor3Foe);
        click(&mut game, El::F3SlateTake);
        assert_eq!(game.expedition.floor3.foe.state, FoeState::Exposed);
        // Verdicts cycle unjudged → ran → never ran: the slate
        // reads [never, never, ran].
        click(&mut game, El::F3SlateEntry(0));
        click(&mut game, El::F3SlateEntry(0));
        click(&mut game, El::F3SlateEntry(1));
        click(&mut game, El::F3SlateEntry(1));
        click(&mut game, El::F3SlateEntry(2));
        click(&mut game, El::F3Reconcile);
        assert!(game.expedition.floor3.foe.slate_done);
        click(&mut game, El::F3Stamp);
        click(&mut game, El::F3Rite);
        click(&mut game, El::F3Rite);
        click(&mut game, El::F3Rite);
        assert!(game.expedition.floor3_foe_proof_complete());
        // The lanterns light through the result handlers, as
        // the shell delivers them. The gate flag stands as the
        // RunGate press left it: this room's gate is Floor 3's.
        game.gate_floor3 = true;
        game.on_lantern_result(crate::gate::LanternReport {
            kind: 0,
            passed: true,
            text: game.editor_text().to_string(),
        });
        game.on_lantern_result(crate::gate::LanternReport {
            kind: 1,
            passed: true,
            text: String::new(),
        });
        game.on_gate_result(crate::gate::GateReport {
            tests_passed: true,
            tests_run: 6,
            tests_failed: 0,
            clippy_passed: true,
            fmt_checked: true,
            fmt_passed: true,
            timed_out: false,
            summary: "The gate stood open.".to_string(),
            detail: String::new(),
        });
        assert_eq!(game.expedition.floor3.foe.lanterns, [true, true, true]);
        assert!(game.expedition.floor3.foe.gate_passed);
        // An edit douses all three lanterns and the gate.
        game.editor_insert(" ");
        assert_eq!(game.expedition.floor3.foe.lanterns, [false, false, false]);
        assert!(!game.expedition.floor3.foe.gate_passed);
        game.editor_backspace();
        // Light them again and strike.
        game.on_lantern_result(crate::gate::LanternReport {
            kind: 0,
            passed: true,
            text: game.editor_text().to_string(),
        });
        game.on_lantern_result(crate::gate::LanternReport {
            kind: 1,
            passed: true,
            text: String::new(),
        });
        game.on_gate_result(crate::gate::GateReport {
            tests_passed: true,
            tests_run: 6,
            tests_failed: 0,
            clippy_passed: true,
            fmt_checked: true,
            fmt_passed: true,
            timed_out: false,
            summary: "The gate stood open.".to_string(),
            detail: String::new(),
        });
        game.today = "2026-11-10".to_string();
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor3.foe.state, FoeState::Fallen);
        assert!(game.expedition.floor3_complete());
        assert!(
            game.expedition
                .codex
                .iter()
                .any(|stone| *stone == content::LANTERN_STONE)
        );
        assert!(
            game.expedition
                .pack
                .iter()
                .any(|loot| *loot == content::GATEHOUSE_KEY)
        );
    }

    /// Driven playthrough: the whole of Floor 3 on the GUI
    /// surface — both prerequisite FOEs felled at the state
    /// level, the doors walked honestly, all five chambers
    /// played and sealed through their desks, the Release's
    /// lanterns lit by the real engines through the io slot,
    /// the real third gate run on the typed draft, and the
    /// strike through the seal.
    #[test]
    fn floor3_gui_playthrough_fells_the_release() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn back_to_gatehouse(game: &mut Game) {
            click(game, El::TabFloor);
            click(game, El::DoorFloor2);
            click(game, El::DoorFloor3);
            assert_eq!(game.screen, Screen::Floor3Hub);
        }
        fn repair(game: &mut Game, index: usize) {
            click(game, El::Chip(Verb::Prove));
            click(game, El::Seal);
            click(game, El::SealConfirm);
            assert_eq!(
                game.expedition.floor3.chambers[index].state,
                ChamberState::Repaired,
                "chamber {index} repaired through the seal"
            );
            back_to_gatehouse(game);
        }

        let mut game = Game::new();
        game.begin();
        game.io = None; // the shell drains the begin-autosave at once
        // Fixture: fell both prerequisite FOEs at the state level.
        {
            let expedition = &mut game.expedition;
            expedition.foe_enumerate();
            let double = crate::puzzles::foe_double_row().expect("double row");
            expedition.foe_exhibit_row(double).expect("exhibit");
            for row in crate::puzzles::foe_uncopiable_rows() {
                expedition.foe_verify_row(row).expect("verify");
            }
            expedition.foe_mark_gate_passed();
            expedition.foe_arm_strike().expect("arm");
            expedition.foe_confirm_strike("2026-11-10").expect("strike");
            expedition.floor2_foe_take_audit();
            for row in 0..content::FOE_ADMISSIONS.len() {
                expedition.floor2_foe_exhibit(row).expect("exhibit 2");
            }
            let seam = crate::puzzles::foe2_disagreement_row().expect("seam row");
            expedition
                .floor2_foe_name_disagreement(seam)
                .expect("name the seam");
            expedition.floor2_foe_calibrate().expect("calibrate");
            expedition.floor2_foe_mark_gate_passed();
            expedition.floor2_foe_arm_strike().expect("arm 2");
            expedition
                .floor2_foe_confirm_strike("2026-11-10")
                .expect("strike 2");
        }
        click(&mut game, El::DoorFloor2);
        click(&mut game, El::DoorFloor3);
        assert_eq!(game.screen, Screen::Floor3Hub);

        // Chamber 1 — Named Failures: the true sort.
        click(&mut game, El::Floor3Door(0));
        click(&mut game, El::F3Watch);
        for (report, entry) in content::FAILURE_REPORTS.iter().enumerate() {
            for _ in 0..=entry.pair {
                click(&mut game, El::F3Pair(report as u8));
            }
        }
        assert!(game.expedition.floor3_diagnosed(0));
        repair(&mut game, 0);

        // Chamber 2 — the Conveyor at its one true setting.
        click(&mut game, El::Floor3Door(1));
        click(&mut game, El::F3Watch);
        for _ in 0..3 {
            click(&mut game, El::F3Station(0));
        }
        for _ in 0..2 {
            click(&mut game, El::F3Station(1));
        }
        for _ in 0..2 {
            click(&mut game, El::F3Station(2));
        }
        assert!(game.expedition.floor3_diagnosed(1));
        repair(&mut game, 1);

        // Chamber 3 — the Audit of Shouts: the true sort.
        click(&mut game, El::Floor3Door(2));
        click(&mut game, El::F3Watch);
        click(&mut game, El::F3Shout(0));
        click(&mut game, El::F3Shout(1));
        click(&mut game, El::F3Shout(1));
        click(&mut game, El::F3Shout(2));
        click(&mut game, El::F3Shout(3));
        click(&mut game, El::F3Shout(3));
        click(&mut game, El::F3Invariant(1));
        click(&mut game, El::F3Invariant(3));
        click(&mut game, El::F3Invariant(3));
        assert!(game.expedition.floor3_diagnosed(2));
        repair(&mut game, 2);

        // Chamber 4 — the Sealed Request, worked in its order.
        click(&mut game, El::Floor3Door(3));
        click(&mut game, El::F3Watch);
        click(&mut game, El::F3Slip(0, 0));
        click(&mut game, El::F3Slip(0, 0));
        click(&mut game, El::F3Slip(0, 1));
        click(&mut game, El::F3Slip(1, 0));
        click(&mut game, El::F3Slip(1, 1));
        click(&mut game, El::F3Slip(2, 2));
        assert!(game.expedition.floor3_diagnosed(3));
        repair(&mut game, 3);

        // Chamber 5 — the Inspection, its rite before its seal.
        click(&mut game, El::Floor3Door(4));
        click(&mut game, El::F3Watch);
        for (item, presses) in [(0u8, 2), (1, 1), (2, 3), (3, 1), (4, 2)] {
            for _ in 0..presses {
                click(&mut game, El::F3Check(item));
            }
        }
        click(&mut game, El::F3NaReason);
        click(&mut game, El::F3Msrv);
        assert!(game.expedition.floor3_diagnosed(4));
        click(&mut game, El::Chip(Verb::Prove));
        click(&mut game, El::F3Rite);
        click(&mut game, El::F3Rite);
        click(&mut game, El::F3Rite);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor3.chambers[4].state,
            ChamberState::Repaired
        );
        assert_eq!(game.expedition.floor3_chambers_repaired(), 5);
        back_to_gatehouse(&mut game);

        // The Release — slate, stamp, and rite on the GUI.
        click(&mut game, El::DoorFoe3);
        assert_eq!(game.screen, Screen::Floor3Foe);
        click(&mut game, El::F3SlateTake);
        click(&mut game, El::F3SlateEntry(0));
        click(&mut game, El::F3SlateEntry(0));
        click(&mut game, El::F3SlateEntry(1));
        click(&mut game, El::F3SlateEntry(1));
        click(&mut game, El::F3SlateEntry(2));
        click(&mut game, El::F3Reconcile);
        assert!(game.expedition.floor3.foe.slate_done);
        click(&mut game, El::F3Stamp);
        click(&mut game, El::F3Rite);
        click(&mut game, El::F3Rite);
        click(&mut game, El::F3Rite);
        assert!(game.expedition.floor3_foe_proof_complete());

        // The draft, written in the game's own editor.
        click(&mut game, El::EditorOpen);
        game.editor_select_all();
        for ch in content::GATE3_REFERENCE.chars() {
            let mut buf = [0u8; 4];
            game.editor_insert(ch.encode_utf8(&mut buf));
        }

        click(&mut game, El::OverlayClose);

        // The format lantern, lit by the real rustfmt through
        // the io slot, as the shell runs it.
        click(&mut game, El::F3Lantern(0));
        let code = match game.io.take() {
            Some(IoRequest::RunLantern { kind: 0, code }) => code,
            other => panic!("expected the format lantern filing, got {other:?}"),
        };
        let scratch = std::env::temp_dir().join("niflheim-step5-playthrough-fmt");
        let report = crate::gate::run_fmt(&scratch, &code, &crate::gate::GateLimits::default());
        assert!(report.passed, "the format lantern: {}", report.text);
        game.on_lantern_result(report);
        assert!(game.expedition.floor3.foe.lanterns[0]);

        // The lint lantern, lit by the real clippy.
        click(&mut game, El::F3Lantern(1));
        let code = match game.io.take() {
            Some(IoRequest::RunLantern { kind: 1, code }) => code,
            other => panic!("expected the lint lantern filing, got {other:?}"),
        };
        let scratch = std::env::temp_dir().join("niflheim-step5-playthrough-lint");
        let report = crate::gate::run_lint(&scratch, &code, &crate::gate::GateLimits::default());
        assert!(report.passed, "the lint lantern: {}", report.text);
        game.on_lantern_result(report);
        assert!(game.expedition.floor3.foe.lanterns[1]);

        // The Release's own gate, run for real on the draft,
        // filed from the editor as the Desk's was.
        click(&mut game, El::EditorOpen);
        click(&mut game, El::RunGate);
        let code = match game.io.take() {
            Some(IoRequest::RunGate { code, floor3, .. }) => {
                assert!(floor3, "the Release files the third gate");
                code
            }
            other => panic!("expected a gate filing, got {other:?}"),
        };
        let scratch = std::env::temp_dir().join("niflheim-step5-playthrough-gate");
        let report = crate::gate::run_gate3(&scratch, &code, &crate::gate::GateLimits::default());
        assert!(report.passed(), "the Release gate: {}", report.summary);
        assert!(report.fmt_passed, "the gate's own format verdict");
        game.on_gate_result(report);
        assert_eq!(game.expedition.floor3.foe.lanterns, [true, true, true]);
        assert!(game.expedition.floor3.foe.gate_record.is_some());
        click(&mut game, El::OverlayClose);

        // The strike, through the seal.
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor3.foe.state, FoeState::Fallen);
        assert!(
            game.expedition
                .codex
                .iter()
                .any(|s| s == content::LANTERN_STONE)
        );
        assert!(
            game.expedition
                .pack
                .iter()
                .any(|s| s == content::GATEHOUSE_KEY)
        );
        assert!(game.expedition.floor3_complete());

        // The stairs recite the descent's standing: clause 3
        // reads all three Stratum I lanterns lit.
        let clauses = game.expedition.stairs_clauses();
        assert!(
            clauses[2].1.contains("Floor 1's lantern is lit")
                && clauses[2].1.contains("Floor 2's lantern is lit")
                && clauses[2].1.contains("Floor 3's lantern is lit"),
            "clause 3: {}",
            clauses[2].1
        );

        // The whole record round-trips through the save.
        let save = game.expedition.save_json().expect("save");
        let loaded = Expedition::load_json(&save).expect("load");
        assert_eq!(loaded, game.expedition);
    }

    /// Floor 4's door answers the pointer when the Stratum I
    /// stairs stand lit, and refuses it while any clause waits.
    #[test]
    fn floor4_door_answers_the_pointer_when_the_stairs_stand_lit() {
        let mut game = Game::new();
        game.begin();
        game.io = None;
        game.screen = Screen::Hub;
        let rect = geom::door_floor4();
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(game.screen, Screen::Hub, "barred while the stairs wait");
        // Light the stairs: FOEs 1 and 2 fallen; the lanterns
        // by Floor 1's gate, Floor 2's chambers, Floor 3's gate.
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.expedition.floor3.foe.gate_passed = true;
        for chamber in game.expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        assert!(game.expedition.floor4_reachable());
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(game.screen, Screen::Floor4Hub);
    }

    /// Floor 5's door answers the pointer when Floor 4 stands
    /// complete, and refuses it while any part waits.
    #[test]
    fn floor5_door_answers_the_pointer_when_floor4_stands_complete() {
        let mut game = Game::new();
        game.begin();
        game.io = None;
        game.screen = Screen::Hub;
        // Light the Stratum I stairs (Floor 4 Step 1's fixture).
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.expedition.floor3.foe.gate_passed = true;
        for chamber in game.expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        let door4 = geom::door_floor4();
        game.pointer_down(door4.cx(), door4.cy());
        game.pointer_up(door4.cx(), door4.cy());
        assert_eq!(game.screen, Screen::Floor4Hub);
        let rect = geom::door_floor5();
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(
            game.screen,
            Screen::Floor4Hub,
            "barred while Floor 4 stands incomplete"
        );
        for chamber in game.expedition.floor4.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        game.expedition.floor4.foe.state = FoeState::Fallen;
        assert!(game.expedition.floor5_reachable());
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(game.screen, Screen::Floor5Hub);
    }

    /// Floor 6 Step 5 — the shipped whole-floor playthrough:
    /// every chamber and the FOE driven end-to-end on the
    /// GUI surface from a fresh expedition, every press a
    /// real press handler, closing on the completed floor.
    #[test]
    fn floor6_whole_floor_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn to_floor6_hub(game: &mut Game) {
            game.screen = Screen::Hub;
            game.expedition.foe.state = FoeState::Fallen;
            game.expedition.foe.gate_passed = true;
            game.expedition.floor2.foe.state = FoeState::Fallen;
            game.expedition.floor3.foe.gate_passed = true;
            for chamber in game.expedition.floor2.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            for chamber in game.expedition.floor4.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            game.expedition.floor4.foe.state = FoeState::Fallen;
            for chamber in game.expedition.floor5.chambers.iter_mut().take(3) {
                chamber.state = ChamberState::Repaired;
            }
            click(game, El::DoorFloor4);
            click(game, El::DoorFloor5);
            click(game, El::DoorFloor6);
            assert_eq!(game.screen, Screen::Floor6Hub);
        }
        fn seal_chamber(game: &mut Game, index: usize) {
            game.do_verb(Verb::Prove);
            click(game, El::Seal);
            click(game, El::SealConfirm);
            assert_eq!(
                game.expedition.floor6.chambers[index].state,
                ChamberState::Repaired
            );
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        to_floor6_hub(&mut game);
        // Chamber 1 — the Cell Heart.
        click(&mut game, El::Floor6Door(0));
        click(&mut game, El::F6Watch);
        click(&mut game, El::F6DayOp(2));
        assert_eq!(
            game.expedition.floor6.chambers[0].work.day_walked,
            [false; 6]
        );
        for op in 0..6u8 {
            click(&mut game, El::F6DayOp(op));
        }
        assert!(game.expedition.floor6.chambers[0].work.bell_struck);
        assert!(game.expedition.floor6_diagnosed(0));
        seal_chamber(&mut game, 0);
        // Chamber 2 — the Borrow-Flag Brazier.
        click(&mut game, El::TabFloor);
        to_floor6_hub(&mut game);
        click(&mut game, El::Floor6Door(1));
        click(&mut game, El::F6Watch);
        for call in 0..5u8 {
            click(&mut game, El::F6BrazierCall(call));
        }
        assert!(game.expedition.floor6_diagnosed(1));
        seal_chamber(&mut game, 1);
        // Chamber 3 — the Spin Heart.
        click(&mut game, El::TabFloor);
        to_floor6_hub(&mut game);
        click(&mut game, El::Floor6Door(2));
        click(&mut game, El::F6Watch);
        click(&mut game, El::F6SpinRun);
        assert!(!game.expedition.floor6.chambers[2].work.gave_up);
        click(&mut game, El::F6BoundUp);
        click(&mut game, El::F6BoundUp);
        click(&mut game, El::F6BoundUp);
        click(&mut game, El::F6SpinRun);
        assert!(game.expedition.floor6.chambers[2].work.gave_up);
        for claim in 0..4u8 {
            click(&mut game, El::F6Annotate(claim));
        }
        assert!(game.expedition.floor6_diagnosed(2));
        seal_chamber(&mut game, 2);
        // Chamber 4 — the Day Census.
        click(&mut game, El::TabFloor);
        to_floor6_hub(&mut game);
        click(&mut game, El::Floor6Door(3));
        click(&mut game, El::F6Watch);
        click(&mut game, El::F6CensusSubmit);
        assert!(!game.expedition.floor6.chambers[3].work.census_judged);
        click(&mut game, El::F6CensusCycle(0));
        for _ in 0..3 {
            click(&mut game, El::F6CensusCycle(1));
        }
        click(&mut game, El::F6CensusCycle(2));
        click(&mut game, El::F6CensusSubmit);
        assert!(game.expedition.floor6_diagnosed(3));
        seal_chamber(&mut game, 3);
        // Chamber 5 — Choosing the Cell.
        click(&mut game, El::TabFloor);
        to_floor6_hub(&mut game);
        click(&mut game, El::Floor6Door(4));
        click(&mut game, El::F6Watch);
        for (datum, datum_meta) in content::CELL_DATA.iter().enumerate() {
            for _ in 0..=datum_meta.true_kind {
                click(&mut game, El::F6Choose(datum as u8));
            }
            for _ in 0..=datum_meta.true_sentence {
                click(&mut game, El::F6Invariant(datum as u8));
            }
        }
        assert!(game.expedition.floor6_diagnosed(4));
        seal_chamber(&mut game, 4);
        // The FOE — the Heart Locked Twice.
        click(&mut game, El::TabFloor);
        to_floor6_hub(&mut game);
        click(&mut game, El::DoorFoe6);
        assert_eq!(game.screen, Screen::Floor6Foe);
        click(&mut game, El::F6Exhibit(1));
        assert!(!game.expedition.floor6.foe.exhibits[1]);
        for exhibit in 0..3u8 {
            click(&mut game, El::F6Exhibit(exhibit));
        }
        assert!(game.expedition.floor6_foe_proof_complete());
        for case in 0..4u8 {
            click(&mut game, El::F6Loom(case));
        }
        assert!(game.expedition.floor6.foe.gate_passed);
        click(&mut game, El::Seal);
        assert!(game.armed.is_none());
        click(&mut game, El::F6FoeCard(0));
        click(&mut game, El::F6FoeCard(1));
        click(&mut game, El::F6FoeCard(1));
        assert!(game.expedition.floor6_foe_repair_ok());
        click(&mut game, El::Seal);
        assert!(game.armed.is_some());
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor6.foe.state, FoeState::Fallen);
        // The floor stands complete. (The save round-trip of
        // a fully earned Floor 6 state is the Step 5 probe's
        // closing assertion: this test's earlier-floor
        // fixtures mark chambers repaired without their
        // work, which the loader's invariant check rightly
        // refuses — the earned state is the probe's to load.)
        assert!(game.expedition.floor6_complete());
        assert_eq!(game.expedition.floor6_chambers_repaired(), 5);
        game.expedition.save_json().expect("the save serializes");
    }
    /// Floor 6 Step 4, driven on the GUI surface: Choosing
    /// the Cell houses the three data and writes their
    /// doors; the Heart Locked Twice is exhibited in order,
    /// walked on the loom, rewritten, and struck — the
    /// stone and the pair change hands.
    #[test]
    fn floor6_chamber_five_and_foe_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn to_floor6_hub(game: &mut Game) {
            game.screen = Screen::Hub;
            game.expedition.foe.state = FoeState::Fallen;
            game.expedition.foe.gate_passed = true;
            game.expedition.floor2.foe.state = FoeState::Fallen;
            game.expedition.floor3.foe.gate_passed = true;
            for chamber in game.expedition.floor2.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            for chamber in game.expedition.floor4.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            game.expedition.floor4.foe.state = FoeState::Fallen;
            for chamber in game.expedition.floor5.chambers.iter_mut().take(3) {
                chamber.state = ChamberState::Repaired;
            }
            click(game, El::DoorFloor4);
            click(game, El::DoorFloor5);
            click(game, El::DoorFloor6);
            assert_eq!(game.screen, Screen::Floor6Hub);
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        to_floor6_hub(&mut game);
        // Chamber 5 — Choosing the Cell.
        click(&mut game, El::Floor6Door(4));
        click(&mut game, El::F6Watch);
        // House each datum: presses cycle unhoused → plain →
        // many-readers → borrow-flag, so the press count is
        // the true kind + 1.
        for (datum, datum_meta) in content::CELL_DATA.iter().enumerate() {
            for _ in 0..=datum_meta.true_kind {
                click(&mut game, El::F6Choose(datum as u8));
            }
            for _ in 0..=datum_meta.true_sentence {
                click(&mut game, El::F6Invariant(datum as u8));
            }
        }
        assert!(game.expedition.floor6_diagnosed(4));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor6.chambers[4].state,
            ChamberState::Repaired
        );
        // The FOE — the Heart Locked Twice.
        click(&mut game, El::TabFloor);
        to_floor6_hub(&mut game);
        click(&mut game, El::DoorFoe6);
        assert_eq!(game.screen, Screen::Floor6Foe);
        // Out of order: the worked day cannot precede the
        // census.
        click(&mut game, El::F6Exhibit(1));
        assert!(!game.expedition.floor6.foe.exhibits[1]);
        for exhibit in 0..3u8 {
            click(&mut game, El::F6Exhibit(exhibit));
        }
        assert!(game.expedition.floor6_foe_proof_complete());
        for case in 0..4u8 {
            click(&mut game, El::F6Loom(case));
        }
        assert!(game.expedition.floor6.foe.gate_passed);
        // The card: line 1 once, line 2 twice, line 3 never.
        click(&mut game, El::F6FoeCard(0));
        click(&mut game, El::F6FoeCard(1));
        click(&mut game, El::F6FoeCard(1));
        assert!(game.expedition.floor6_foe_repair_ok());
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor6.foe.state, FoeState::Fallen);
        assert!(
            game.expedition
                .codex
                .iter()
                .any(|stone| stone == content::LOCKHEART_STONE)
        );
        assert!(
            game.expedition
                .pack
                .iter()
                .any(|loot| loot == content::LOCKHEART_PAIR)
        );
    }
    /// Floor 6 Step 3, driven on the GUI surface: the Spin
    /// Heart's bound is set, the heart gives up honestly,
    /// four claims are recorded provisional; the Day Census
    /// is submitted wrong, recounted, and judged true; both
    /// chambers through the seal.
    #[test]
    fn floor6_chambers_three_and_four_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn to_floor6_hub(game: &mut Game) {
            game.screen = Screen::Hub;
            game.expedition.foe.state = FoeState::Fallen;
            game.expedition.foe.gate_passed = true;
            game.expedition.floor2.foe.state = FoeState::Fallen;
            game.expedition.floor3.foe.gate_passed = true;
            for chamber in game.expedition.floor2.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            for chamber in game.expedition.floor4.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            game.expedition.floor4.foe.state = FoeState::Fallen;
            for chamber in game.expedition.floor5.chambers.iter_mut().take(3) {
                chamber.state = ChamberState::Repaired;
            }
            click(game, El::DoorFloor4);
            click(game, El::DoorFloor5);
            click(game, El::DoorFloor6);
            assert_eq!(game.screen, Screen::Floor6Hub);
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        to_floor6_hub(&mut game);
        // Chamber 3 — the Spin Heart.
        click(&mut game, El::Floor6Door(2));
        click(&mut game, El::F6Watch);
        // No bound, no run: the heart refuses to spin free.
        click(&mut game, El::F6SpinRun);
        assert!(!game.expedition.floor6.chambers[2].work.gave_up);
        click(&mut game, El::F6BoundUp);
        click(&mut game, El::F6BoundUp);
        click(&mut game, El::F6BoundUp);
        assert_eq!(game.expedition.floor6.chambers[2].work.spin_bound, 3);
        click(&mut game, El::F6SpinRun);
        assert!(game.expedition.floor6.chambers[2].work.gave_up);
        for claim in 0..4u8 {
            click(&mut game, El::F6Annotate(claim));
        }
        assert!(game.expedition.floor6_diagnosed(2));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor6.chambers[2].state,
            ChamberState::Repaired
        );
        // Chamber 4 — the Day Census.
        click(&mut game, El::TabFloor);
        to_floor6_hub(&mut game);
        click(&mut game, El::Floor6Door(3));
        click(&mut game, El::F6Watch);
        // A wrong census is refused nothing — it is simply
        // not judged, and may be submitted again.
        click(&mut game, El::F6CensusSubmit);
        assert!(!game.expedition.floor6.chambers[3].work.census_judged);
        click(&mut game, El::F6CensusCycle(0));
        for _ in 0..3 {
            click(&mut game, El::F6CensusCycle(1));
        }
        click(&mut game, El::F6CensusCycle(2));
        click(&mut game, El::F6CensusSubmit);
        assert!(game.expedition.floor6.chambers[3].work.census_judged);
        assert!(game.expedition.floor6_diagnosed(3));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor6.chambers[3].state,
            ChamberState::Repaired
        );
    }
    /// Floor 6 Step 2, driven on the GUI surface: the Cell
    /// Heart's day is walked in order — out-of-order refused,
    /// the exclusive refused with the bell — and the
    /// Brazier's five calls kindle once; both chambers
    /// through the seal.
    #[test]
    fn floor6_chambers_one_and_two_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn to_floor6_hub(game: &mut Game) {
            game.screen = Screen::Hub;
            game.expedition.foe.state = FoeState::Fallen;
            game.expedition.foe.gate_passed = true;
            game.expedition.floor2.foe.state = FoeState::Fallen;
            game.expedition.floor3.foe.gate_passed = true;
            for chamber in game.expedition.floor2.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            for chamber in game.expedition.floor4.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            game.expedition.floor4.foe.state = FoeState::Fallen;
            for chamber in game.expedition.floor5.chambers.iter_mut().take(3) {
                chamber.state = ChamberState::Repaired;
            }
            click(game, El::DoorFloor4);
            click(game, El::DoorFloor5);
            assert_eq!(game.screen, Screen::Floor5Hub);
            click(game, El::DoorFloor6);
            assert_eq!(game.screen, Screen::Floor6Hub);
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        to_floor6_hub(&mut game);
        // Chamber 1 — the Cell Heart.
        click(&mut game, El::Floor6Door(0));
        click(&mut game, El::F6Watch);
        // Out of order: the day refuses to skip ahead.
        click(&mut game, El::F6DayOp(2));
        assert_eq!(
            game.expedition.floor6.chambers[0].work.day_walked,
            [false; 6]
        );
        for op in 0..6u8 {
            click(&mut game, El::F6DayOp(op));
        }
        assert!(game.expedition.floor6.chambers[0].work.bell_struck);
        assert!(game.expedition.floor6_diagnosed(0));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor6.chambers[0].state,
            ChamberState::Repaired
        );
        // Chamber 2 — the Borrow-Flag Brazier.
        click(&mut game, El::TabFloor);
        to_floor6_hub(&mut game);
        click(&mut game, El::Floor6Door(1));
        click(&mut game, El::F6Watch);
        for call in 0..5u8 {
            click(&mut game, El::F6BrazierCall(call));
        }
        assert!(game.expedition.floor6_diagnosed(1));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor6.chambers[1].state,
            ChamberState::Repaired
        );
    }
    /// Floor 6's door answers the pointer when the Veilworks
    /// stands charted through its third chamber, and refuses
    /// it while any of the first three chambers waits —
    /// the fourth chamber is not the third, and the door
    /// does not wait on the Thing.
    #[test]
    fn floor6_door_answers_the_pointer_when_floor5_stands_charted_through_chamber_three() {
        let mut game = Game::new();
        game.begin();
        game.io = None;
        game.screen = Screen::Hub;
        // Light the Stratum I stairs (Floor 4 Step 1's fixture).
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.expedition.floor3.foe.gate_passed = true;
        for chamber in game.expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        let door4 = geom::door_floor4();
        game.pointer_down(door4.cx(), door4.cy());
        game.pointer_up(door4.cx(), door4.cy());
        assert_eq!(game.screen, Screen::Floor4Hub);
        // Floor 4 complete opens the Veilworks.
        for chamber in game.expedition.floor4.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        game.expedition.floor4.foe.state = FoeState::Fallen;
        let door5 = geom::door_floor5();
        game.pointer_down(door5.cx(), door5.cy());
        game.pointer_up(door5.cx(), door5.cy());
        assert_eq!(game.screen, Screen::Floor5Hub);
        // The Lockheart's door: barred with nothing charted.
        let rect = geom::door_floor6();
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(
            game.screen,
            Screen::Floor5Hub,
            "barred while the Veilworks stands uncharted"
        );
        game.expedition.floor5.chambers[0].state = ChamberState::Repaired;
        game.expedition.floor5.chambers[1].state = ChamberState::Repaired;
        game.expedition.floor5.chambers[3].state = ChamberState::Repaired;
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(
            game.screen,
            Screen::Floor5Hub,
            "the fourth chamber is not the third"
        );
        game.expedition.floor5.chambers[2].state = ChamberState::Repaired;
        assert!(game.expedition.floor6_reachable());
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(game.screen, Screen::Floor6Hub);
    }
    /// Floor 5 Step 2, driven on the GUI surface: the Pointer
    /// Walks is read, walked, and refused at the cracked
    /// stone, and the Two Tenants are called, stamped, and
    /// annotated — both chambers through the seal.
    #[test]
    fn floor5_chambers_one_and_two_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn to_floor5_hub(game: &mut Game) {
            game.screen = Screen::Hub;
            game.expedition.foe.state = FoeState::Fallen;
            game.expedition.foe.gate_passed = true;
            game.expedition.floor2.foe.state = FoeState::Fallen;
            game.expedition.floor3.foe.gate_passed = true;
            for chamber in game.expedition.floor2.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            for chamber in game.expedition.floor4.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            game.expedition.floor4.foe.state = FoeState::Fallen;
            click(game, El::DoorFloor4);
            click(game, El::DoorFloor5);
            assert_eq!(game.screen, Screen::Floor5Hub);
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        to_floor5_hub(&mut game);
        // Chamber 1 — the Pointer Walks.
        click(&mut game, El::Floor5Door(0));
        click(&mut game, El::F5Watch);
        for step in 0..6u8 {
            click(&mut game, El::F5Card(step));
            if step == 3 {
                click(&mut game, El::F5RefuseStep);
            } else {
                click(&mut game, El::F5Walk);
            }
        }
        assert!(game.expedition.floor5_diagnosed(0));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor5.chambers[0].state,
            ChamberState::Repaired
        );
        // Chamber 2 — the Two Tenants.
        click(&mut game, El::TabFloor);
        to_floor5_hub(&mut game);
        click(&mut game, El::Floor5Door(1));
        click(&mut game, El::F5Watch);
        for (index, (_, sound, _)) in content::TENANCIES.iter().enumerate() {
            // One deliberate miss: the repainting writer (index
            // 1) is called sound.
            let presses = if index == 1 || *sound { 1 } else { 2 };
            for _ in 0..presses {
                click(&mut game, El::F5Placard(index as u8));
            }
        }
        click(&mut game, El::F5Judge);
        assert!(game.expedition.floor5.chambers[1].work.judged);
        assert!(!game.expedition.floor5_diagnosed(1));
        for _ in 0..3 {
            click(&mut game, El::F5Annotate(1));
        }
        assert_eq!(
            game.expedition.floor5.chambers[1].work.annotations[1],
            Some(2)
        );
        assert!(game.expedition.floor5_diagnosed(1));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor5.chambers[1].state,
            ChamberState::Repaired
        );
    }

    /// Floor 5 Step 5 — the shipped whole-floor playthrough:
    /// every chamber and the FOE driven end-to-end on the GUI
    /// surface from a fresh expedition, every press a real
    /// press handler, closing on the completed floor and a
    /// save that round-trips.
    #[test]
    fn floor5_whole_floor_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn to_floor5_hub(game: &mut Game) {
            game.screen = Screen::Hub;
            game.expedition.foe.state = FoeState::Fallen;
            game.expedition.foe.gate_passed = true;
            game.expedition.floor2.foe.state = FoeState::Fallen;
            game.expedition.floor3.foe.gate_passed = true;
            for chamber in game.expedition.floor2.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            for chamber in game.expedition.floor4.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            game.expedition.floor4.foe.state = FoeState::Fallen;
            click(game, El::DoorFloor4);
            click(game, El::DoorFloor5);
            assert_eq!(game.screen, Screen::Floor5Hub);
        }
        fn seal_chamber(game: &mut Game, index: usize) {
            game.do_verb(Verb::Prove);
            click(game, El::Seal);
            click(game, El::SealConfirm);
            assert_eq!(
                game.expedition.floor5.chambers[index].state,
                ChamberState::Repaired
            );
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        to_floor5_hub(&mut game);
        // Chamber 1 — the Pointer Walks.
        click(&mut game, El::Floor5Door(0));
        click(&mut game, El::F5Watch);
        for step in 0..6u8 {
            click(&mut game, El::F5Card(step));
            if step == 3 {
                click(&mut game, El::F5RefuseStep);
            } else {
                click(&mut game, El::F5Walk);
            }
        }
        assert!(game.expedition.floor5_diagnosed(0));
        seal_chamber(&mut game, 0);
        // Chamber 2 — the Two Tenants.
        click(&mut game, El::TabFloor);
        to_floor5_hub(&mut game);
        click(&mut game, El::Floor5Door(1));
        click(&mut game, El::F5Watch);
        for (index, (_, sound, _)) in content::TENANCIES.iter().enumerate() {
            let presses = if index == 1 || *sound { 1 } else { 2 };
            for _ in 0..presses {
                click(&mut game, El::F5Placard(index as u8));
            }
        }
        click(&mut game, El::F5Judge);
        assert!(!game.expedition.floor5_diagnosed(1));
        for _ in 0..3 {
            click(&mut game, El::F5Annotate(1));
        }
        assert!(game.expedition.floor5_diagnosed(1));
        seal_chamber(&mut game, 1);
        // Chamber 3 — the Contract Cards.
        click(&mut game, El::TabFloor);
        to_floor5_hub(&mut game);
        click(&mut game, El::Floor5Door(2));
        click(&mut game, El::F5Watch);
        for (tool, (_, _, key)) in content::CARD_TOOLS.iter().enumerate() {
            for (line, checkable) in key.iter().enumerate() {
                for _ in 0..*checkable {
                    click(&mut game, El::F5CardLine(tool as u8, line as u8));
                }
            }
        }
        assert!(game.expedition.floor5_diagnosed(2));
        seal_chamber(&mut game, 2);
        // Chamber 4 — the Split Beam.
        click(&mut game, El::TabFloor);
        to_floor5_hub(&mut game);
        click(&mut game, El::Floor5Door(3));
        click(&mut game, El::F5Watch);
        click(&mut game, El::F5DropRight);
        assert_eq!(
            game.expedition.floor5.chambers[3].work.dropped,
            [false, false]
        );
        click(&mut game, El::F5SplitUp);
        click(&mut game, El::F5SplitUp);
        click(&mut game, El::F5SplitUp);
        click(&mut game, El::F5Seam);
        click(&mut game, El::F5Reach);
        click(&mut game, El::F5WorkHalves);
        click(&mut game, El::F5DropRight);
        assert_eq!(
            game.expedition.floor5.chambers[3].work.dropped,
            [false, false]
        );
        click(&mut game, El::F5DropLeft);
        click(&mut game, El::F5DropRight);
        assert!(game.expedition.floor5_diagnosed(3));
        seal_chamber(&mut game, 3);
        // Chamber 5 — the UB Hunt.
        click(&mut game, El::TabFloor);
        to_floor5_hub(&mut game);
        click(&mut game, El::Floor5Door(4));
        click(&mut game, El::F5Watch);
        for station in 0..6u8 {
            click(&mut game, El::F5Lens(station));
        }
        for station in [0u8, 3, 5] {
            click(&mut game, El::F5Verdict(station));
        }
        for station in [1u8, 2, 4] {
            click(&mut game, El::F5Verdict(station));
            click(&mut game, El::F5Verdict(station));
        }
        click(&mut game, El::F5FixCard(1));
        click(&mut game, El::F5FixCard(1));
        click(&mut game, El::F5FixCard(4));
        assert!(game.expedition.floor5_diagnosed(4));
        seal_chamber(&mut game, 4);
        // The FOE — the Thing Behind the Veil.
        click(&mut game, El::TabFloor);
        to_floor5_hub(&mut game);
        click(&mut game, El::DoorFoe5);
        assert_eq!(game.screen, Screen::Floor5Foe);
        click(&mut game, El::F5Exhibit(1));
        assert!(!game.expedition.floor5.foe.exhibits[1]);
        click(&mut game, El::F5Exhibit(0));
        click(&mut game, El::F5Exhibit(1));
        click(&mut game, El::F5Exhibit(2));
        assert!(game.expedition.floor5_foe_proof_complete());
        for case in 0..5u8 {
            click(&mut game, El::F5Case(case));
        }
        assert!(game.expedition.floor5.foe.gate_passed);
        click(&mut game, El::Seal);
        assert!(game.armed.is_none());
        click(&mut game, El::F5FoeCard(0));
        click(&mut game, El::F5FoeCard(1));
        click(&mut game, El::F5FoeCard(1));
        click(&mut game, El::F5FoeCard(2));
        assert!(game.expedition.floor5_foe_repair_ok());
        click(&mut game, El::Seal);
        assert!(game.armed.is_some());
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor5.foe.state, FoeState::Fallen);
        // The floor stands complete. (The save round-trip of
        // a fully earned Floor 5 state is the Step 5 probe's
        // closing assertion: this test's earlier-floor
        // fixtures mark chambers repaired without their
        // work, which the loader's invariant check rightly
        // refuses — the earned state is the probe's to load.)
        assert!(game.expedition.floor5_complete());
        assert_eq!(game.expedition.floor5_chambers_repaired(), 5);
        game.expedition.save_json().expect("the save serializes");
    }

    /// Floor 5 Step 4, driven on the GUI surface: the UB
    /// Hunt is lensed, judged, and carded through the seal,
    /// and the Thing Behind the Veil is diagnosed in order,
    /// walked through the referee's five cases, given a
    /// rewritten card, and struck — completing the floor.
    /// (Chambers 1–4 are set up at the state level; their
    /// own steps' playthroughs covered their GUI.)
    #[test]
    fn floor5_chamber_5_and_the_thing_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.expedition.floor3.foe.gate_passed = true;
        for chamber in game.expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        for chamber in game.expedition.floor3.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        game.expedition.floor3.foe.state = FoeState::Fallen;
        for chamber in game.expedition.floor4.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        game.expedition.floor4.foe.state = FoeState::Fallen;
        // Chambers 1–4 of this floor, worked at state level.
        {
            let work = &mut game.expedition.floor5.chambers[0].work;
            work.watched = true;
            work.cards_read = [true; 6];
            work.walked = 6;
            work.refused = true;
        }
        {
            let work = &mut game.expedition.floor5.chambers[1].work;
            work.watched = true;
            for (index, (_, sound, _)) in content::TENANCIES.iter().enumerate() {
                work.predictions[index] = if *sound { 1 } else { 2 };
            }
            work.judged = true;
        }
        {
            let work = &mut game.expedition.floor5.chambers[2].work;
            work.watched = true;
            work.card_lines = [[1, 0, 1], [0, 2, 1], [2, 0, 1], [0, 1, 2]];
        }
        {
            let work = &mut game.expedition.floor5.chambers[3].work;
            work.watched = true;
            work.split = 2;
            work.seamed = true;
            work.access_refused = true;
            work.worked = [true, true];
            work.dropped = [true, true];
        }
        for index in 0..4 {
            game.expedition.floor5_file_proof(index).expect("filed");
            game.expedition
                .floor5_repair_chamber(index)
                .expect("repaired");
        }
        game.screen = Screen::Floor5Hub;
        // Chamber 5 — the UB Hunt.
        click(&mut game, El::Floor5Door(4));
        assert_eq!(game.screen, Screen::Floor5Chamber(4));
        click(&mut game, El::F5Watch);
        for station in 0..6u8 {
            click(&mut game, El::F5Lens(station));
        }
        for station in [0u8, 3, 5] {
            click(&mut game, El::F5Verdict(station));
        }
        for station in [1u8, 2, 4] {
            click(&mut game, El::F5Verdict(station));
            click(&mut game, El::F5Verdict(station));
        }
        click(&mut game, El::F5FixCard(1));
        click(&mut game, El::F5FixCard(1));
        click(&mut game, El::F5FixCard(4));
        assert!(game.expedition.floor5_diagnosed(4));
        click(&mut game, El::Chip(Verb::Prove));
        click(&mut game, El::Seal);
        assert!(game.armed.is_some());
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor5.chambers[4].state,
            ChamberState::Repaired
        );
        // The FOE — the Thing Behind the Veil.
        click(&mut game, El::TabFloor);
        click(&mut game, El::DoorFloor4);
        click(&mut game, El::DoorFloor5);
        click(&mut game, El::DoorFoe5);
        assert_eq!(game.screen, Screen::Floor5Foe);
        // Out of order: the growth cannot precede the loan.
        click(&mut game, El::F5Exhibit(1));
        assert!(!game.expedition.floor5.foe.exhibits[1]);
        assert_eq!(game.status_kind, StatusKind::Refusal);
        click(&mut game, El::F5Exhibit(0));
        assert_eq!(game.expedition.floor5.foe.state, FoeState::Exposed);
        click(&mut game, El::F5Exhibit(1));
        click(&mut game, El::F5Exhibit(2));
        assert!(game.expedition.floor5_foe_proof_complete());
        for case in 0..5u8 {
            click(&mut game, El::F5Case(case));
        }
        assert!(game.expedition.floor5.foe.gate_passed);
        // The seal refuses until the card stands rewritten.
        click(&mut game, El::Seal);
        assert!(game.armed.is_none());
        click(&mut game, El::F5FoeCard(0));
        click(&mut game, El::F5FoeCard(1));
        click(&mut game, El::F5FoeCard(1));
        click(&mut game, El::F5FoeCard(2));
        assert!(game.expedition.floor5_foe_repair_ok());
        click(&mut game, El::Seal);
        assert!(game.armed.is_some());
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor5.foe.state, FoeState::Fallen);
        assert!(game.expedition.floor5_complete());
        assert!(
            game.expedition
                .codex
                .iter()
                .any(|stone| stone == content::ICE_STONE)
        );
        assert!(
            game.expedition
                .pack
                .iter()
                .any(|loot| loot == content::VEILWALKERS_BUFFER)
        );
    }

    /// Floor 5 Step 3, driven on the GUI surface: the Contract
    /// Cards are cycled line by line to the checkable
    /// phrasings, and the Split Beam is seamed, refused at the
    /// over-reach, worked on both halves, and dropped in the
    /// card's order — both chambers through the seal.
    #[test]
    fn floor5_chambers_three_and_four_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn to_floor5_hub(game: &mut Game) {
            game.screen = Screen::Hub;
            game.expedition.foe.state = FoeState::Fallen;
            game.expedition.foe.gate_passed = true;
            game.expedition.floor2.foe.state = FoeState::Fallen;
            game.expedition.floor3.foe.gate_passed = true;
            for chamber in game.expedition.floor2.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            for chamber in game.expedition.floor4.chambers.iter_mut() {
                chamber.state = ChamberState::Repaired;
            }
            game.expedition.floor4.foe.state = FoeState::Fallen;
            click(game, El::DoorFloor4);
            click(game, El::DoorFloor5);
            assert_eq!(game.screen, Screen::Floor5Hub);
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        to_floor5_hub(&mut game);
        // Chamber 3 — the Contract Cards.
        click(&mut game, El::Floor5Door(2));
        click(&mut game, El::F5Watch);
        for (tool, (_, _, key)) in content::CARD_TOOLS.iter().enumerate() {
            for (line, checkable) in key.iter().enumerate() {
                for _ in 0..*checkable {
                    click(&mut game, El::F5CardLine(tool as u8, line as u8));
                }
            }
        }
        assert!(game.expedition.floor5_diagnosed(2));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor5.chambers[2].state,
            ChamberState::Repaired
        );
        // Chamber 4 — the Split Beam.
        click(&mut game, El::TabFloor);
        to_floor5_hub(&mut game);
        click(&mut game, El::Floor5Door(3));
        click(&mut game, El::F5Watch);
        // A right-first drop before the seam is refused outright.
        click(&mut game, El::F5DropRight);
        assert_eq!(
            game.expedition.floor5.chambers[3].work.dropped,
            [false, false]
        );
        click(&mut game, El::F5SplitUp);
        click(&mut game, El::F5SplitUp);
        click(&mut game, El::F5SplitUp);
        click(&mut game, El::F5Seam);
        assert!(game.expedition.floor5.chambers[3].work.seamed);
        // The right half cannot come down before the left.
        click(&mut game, El::F5Reach);
        click(&mut game, El::F5WorkHalves);
        click(&mut game, El::F5DropRight);
        assert_eq!(
            game.expedition.floor5.chambers[3].work.dropped,
            [false, false]
        );
        click(&mut game, El::F5DropLeft);
        click(&mut game, El::F5DropRight);
        assert!(game.expedition.floor5_diagnosed(3));
        game.do_verb(Verb::Prove);
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor5.chambers[3].state,
            ChamberState::Repaired
        );
    }
    /// Floor 4 Step 3, driven on the GUI surface: the crests
    /// are cut to their niches and measured, and the treaty
    /// wall is seated, proven, and its book amended — both
    /// chambers through the seal.
    /// Regression (D-1189 fold-in): Floor 3's chambers lay
    /// out a ResetChamber button whose press used to fall
    /// through to the match's catch-all — the reset now
    /// works, and hint usage stays on the record.
    #[test]
    fn floor3_chamber_reset_button_resets_the_work() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        // Floor 3's prerequisite: both earlier FOEs fallen.
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.screen = Screen::Hub;
        click(&mut game, El::DoorFloor2);
        click(&mut game, El::DoorFloor3);
        assert_eq!(game.screen, Screen::Floor3Hub);
        click(&mut game, El::Floor3Door(0));
        click(&mut game, El::F3Watch);
        click(&mut game, El::F3Pair(0));
        {
            let work = &game.expedition.floor3.chambers[0].work;
            assert!(work.watched);
            assert_eq!(work.pair_choices[0], Some(0));
        }
        game.expedition.floor3_view_hint(0);
        click(&mut game, El::ResetChamber);
        let progress = &game.expedition.floor3.chambers[0];
        assert_eq!(progress.state, ChamberState::Unworked);
        assert!(!progress.work.watched);
        assert_eq!(progress.work.pair_choices[0], None);
        assert_eq!(progress.hints_used, 1, "hint rungs stay on the record");
    }

    #[test]
    fn floor4_chambers_3_and_4_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        // The stairs stand lit (the Step 1 fixture).
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.expedition.floor3.foe.gate_passed = true;
        for chamber in game.expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        game.screen = Screen::Hub;
        click(&mut game, El::DoorFloor4);
        assert_eq!(game.screen, Screen::Floor4Hub);
        // Chamber 3 — the Seal Stones.
        click(&mut game, El::Floor4Door(2));
        assert_eq!(game.screen, Screen::Floor4Chamber(2));
        click(&mut game, El::F4Watch);
        for _ in 0..3 {
            click(&mut game, El::F4Crest(0));
        }
        click(&mut game, El::F4CrestProve(0));
        for _ in 0..2 {
            click(&mut game, El::F4Crest(1));
        }
        click(&mut game, El::F4CrestProve(1));
        for _ in 0..3 {
            click(&mut game, El::F4Crest(2));
        }
        click(&mut game, El::F4CrestProve(2));
        assert!(game.expedition.floor4_diagnosed(2));
        click(&mut game, El::Chip(Verb::Prove));
        click(&mut game, El::Seal);
        assert!(game.armed.is_some());
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor4.chambers[2].state,
            ChamberState::Repaired
        );
        // Chamber 4 — the treaty wall.
        click(&mut game, El::TabFloor);
        click(&mut game, El::DoorFloor4);
        click(&mut game, El::Floor4Door(3));
        assert_eq!(game.screen, Screen::Floor4Chamber(3));
        click(&mut game, El::F4Watch);
        for stone in 0..4u8 {
            click(&mut game, El::F4TreatySeat(stone));
        }
        click(&mut game, El::F4TreatyProve);
        assert!(game.expedition.floor4.chambers[3].work.treaty_proven);
        assert!(!game.expedition.floor4_diagnosed(3));
        for _ in 0..6 {
            click(&mut game, El::F4PromiseUp);
        }
        assert!(game.expedition.floor4_diagnosed(3));
        click(&mut game, El::Chip(Verb::Prove));
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor4_chambers_repaired(), 2);
    }

    /// Driven playthrough: Floor 4's first two chambers on
    /// the GUI surface — the slate committed by steppers and
    /// graded by the dropped line (two surprises annotated),
    /// and the wall seated stone by stone to its least span.
    #[test]
    fn floor4_chambers_1_and_2_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn back_to_galleries(game: &mut Game) {
            click(game, El::TabFloor);
            click(game, El::DoorFloor4);
            assert_eq!(game.screen, Screen::Floor4Hub);
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        // The stairs stand lit (the Step 1 fixture).
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.expedition.floor3.foe.gate_passed = true;
        for chamber in game.expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        game.screen = Screen::Hub;
        click(&mut game, El::DoorFloor4);
        assert_eq!(game.screen, Screen::Floor4Hub);

        // Chamber 1 — the bench. True slates are (size,
        // align-presses): the align stepper walks 1,2,4,8,16,
        // so alignment A takes its position + 1 presses.
        click(&mut game, El::Floor4Door(0));
        assert_eq!(game.screen, Screen::Floor4Chamber(0));
        click(&mut game, El::F4Watch);
        // (stone, size presses, align presses, annotate presses)
        let slate: [(u8, u32, u32, u32); 6] = [
            (0, 1, 1, 0),  // u8: 1 · 1, true
            (1, 5, 3, 2),  // (u8, u32): slate 5 · 4 — surprise, annotate to 1
            (2, 16, 4, 0), // &str: 16 · 8, true
            (3, 2, 1, 0),  // Option<u8>: 2 · 1, true
            (4, 16, 4, 2), // Option<&u64>: slate 16 · 8 — surprise, annotate to 1
            (5, 24, 4, 0), // Vec<u8>: 24 · 8, true
        ];
        for (stone, sizes, aligns, annotates) in slate {
            click(&mut game, El::F4Stone(stone));
            for _ in 0..sizes {
                click(&mut game, El::F4SizeUp);
            }
            for _ in 0..aligns {
                click(&mut game, El::F4AlignUp);
            }
            click(&mut game, El::F4DropLine);
            for _ in 0..annotates {
                click(&mut game, El::F4Annotate);
            }
        }
        assert!(game.expedition.floor4_diagnosed(0));
        click(&mut game, El::Chip(Verb::Prove));
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor4.chambers[0].state,
            ChamberState::Repaired
        );
        back_to_galleries(&mut game);

        // Chamber 2 — the wall seated to its least span:
        // beam, plate, rod, shim, pin.
        click(&mut game, El::Floor4Door(1));
        assert_eq!(game.screen, Screen::Floor4Chamber(1));
        click(&mut game, El::F4Watch);
        for stone in [1u8, 2, 4, 3, 0] {
            click(&mut game, El::F4Seat(stone));
        }
        assert!(game.expedition.floor4_diagnosed(1));
        click(&mut game, El::Chip(Verb::Prove));
        click(&mut game, El::Seal);
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor4.chambers[1].state,
            ChamberState::Repaired
        );
        assert_eq!(game.expedition.floor4_chambers_repaired(), 2);
    }

    /// The general pointer path still works outside overlays:
    /// a hub door opens its chamber.
    #[test]
    fn hub_door_answers_the_pointer() {
        let mut game = Game::new();
        game.begin();
        let rect = geom::door_chamber(0);
        game.pointer_down(rect.cx(), rect.cy());
        game.pointer_up(rect.cx(), rect.cy());
        assert_eq!(game.screen, Screen::Chamber(0));
    }

    /// Floor 4 Step 4, driven on the GUI surface: the hollow
    /// stones are filled, tagged, read, and repaired through
    /// the seal — with the referee's catch landing on the way —
    /// and the Crooked Foundation is taken down, re-surveyed,
    /// exhibited, re-cut in both sets, and struck through its
    /// door on Floor 4's hub.
    #[test]
    fn floor4_chamber_5_and_the_foundation_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        // The stairs stand lit (the Step 1 fixture).
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.expedition.floor3.foe.gate_passed = true;
        for chamber in game.expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        game.screen = Screen::Hub;
        click(&mut game, El::DoorFloor4);
        assert_eq!(game.screen, Screen::Floor4Hub);
        // Chamber 5 — the Hollow Stones.
        click(&mut game, El::Floor4Door(4));
        assert_eq!(game.screen, Screen::Floor4Chamber(4));
        click(&mut game, El::F4Watch);
        // A use on the part-filled blank is caught, and the
        // catch stays on the record.
        click(&mut game, El::F4Cell(0));
        click(&mut game, El::F4UseBlank);
        assert_eq!(game.expedition.floor4.chambers[4].work.referee_catches, 1);
        assert!(!game.expedition.floor4.chambers[4].work.blank_used);
        for cell in 1..4u8 {
            click(&mut game, El::F4Cell(cell));
        }
        click(&mut game, El::F4UseBlank);
        assert!(game.expedition.floor4.chambers[4].work.blank_used);
        // Stone 0 takes the number, tagged a number; stone 1
        // takes the ratio, tagged a ratio.
        click(&mut game, El::F4Fill(0));
        click(&mut game, El::F4Tag(0));
        click(&mut game, El::F4Fill(1));
        click(&mut game, El::F4Fill(1));
        click(&mut game, El::F4Tag(1));
        click(&mut game, El::F4Tag(1));
        click(&mut game, El::F4Read(0));
        click(&mut game, El::F4Read(1));
        assert!(game.expedition.floor4_diagnosed(4));
        click(&mut game, El::Chip(Verb::Prove));
        click(&mut game, El::Seal);
        assert!(game.armed.is_some());
        click(&mut game, El::SealConfirm);
        assert_eq!(
            game.expedition.floor4.chambers[4].state,
            ChamberState::Repaired
        );
        // The FOE — the Crooked Foundation.
        click(&mut game, El::TabFloor);
        click(&mut game, El::DoorFloor4);
        click(&mut game, El::DoorFoe4);
        assert_eq!(game.screen, Screen::Floor4Foe);
        click(&mut game, El::F4BookTake);
        assert_eq!(game.expedition.floor4.foe.state, FoeState::Exposed);
        for entry in 0..5u8 {
            click(&mut game, El::F4Measure(entry));
        }
        for entry in [1u8, 3, 4] {
            click(&mut game, El::F4Exhibit(entry));
        }
        assert!(game.expedition.floor4_foe_proof_complete());
        // The seal refuses before the re-cut stands verified.
        click(&mut game, El::Seal);
        assert!(game.armed.is_none());
        for stone in 0..4u8 {
            click(&mut game, El::F4ForeignSeat(stone));
        }
        click(&mut game, El::F4ForeignProve);
        assert!(game.expedition.floor4.foe.foreign_proven);
        assert!(!game.expedition.floor4.foe.gate_passed);
        for stone in [1u8, 2, 3, 0] {
            click(&mut game, El::F4FreeSeat(stone));
        }
        click(&mut game, El::F4FreeProve);
        assert!(game.expedition.floor4.foe.gate_passed);
        click(&mut game, El::Seal);
        assert!(game.armed.is_some());
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor4.foe.state, FoeState::Fallen);
        assert!(
            game.expedition
                .codex
                .iter()
                .any(|stone| stone == content::BRASS_EYE_STONE)
        );
        assert!(
            game.expedition
                .pack
                .iter()
                .any(|loot| loot == content::PLUMB_LINE)
        );
    }

    /// Floor 4 Step 5 — the shipped whole-floor playthrough:
    /// all five galleries and the Crooked Foundation, played
    /// end to end on the GUI surface from the stairs-lit
    /// fixture, with no state touched except through clicks.
    /// The floor completes here: five chambers repaired at
    /// full standard (no hint spent anywhere), the FOE
    /// fallen, its stone and its loot in hand.
    #[test]
    fn floor4_whole_floor_played_on_the_gui() {
        fn click(game: &mut Game, el: El) {
            let rect = layout(game)
                .iter()
                .find(|(found, _)| *found == el)
                .map(|(_, rect)| *rect)
                .unwrap_or_else(|| panic!("{el:?} not laid out on {:?}", game.screen));
            game.pointer_down(rect.cx(), rect.cy());
            game.pointer_up(rect.cx(), rect.cy());
        }
        fn back_to_galleries(game: &mut Game) {
            click(game, El::TabFloor);
            click(game, El::DoorFloor4);
            assert_eq!(game.screen, Screen::Floor4Hub);
        }
        fn seal_the_chamber(game: &mut Game, index: usize) {
            click(game, El::Chip(Verb::Prove));
            click(game, El::Seal);
            assert!(game.armed.is_some());
            click(game, El::SealConfirm);
            assert_eq!(
                game.expedition.floor4.chambers[index].state,
                ChamberState::Repaired
            );
        }
        let mut game = Game::new();
        game.begin();
        game.io = None;
        // The stairs stand lit (the Step 1 fixture).
        game.expedition.foe.state = FoeState::Fallen;
        game.expedition.foe.gate_passed = true;
        game.expedition.floor2.foe.state = FoeState::Fallen;
        game.expedition.floor3.foe.gate_passed = true;
        for chamber in game.expedition.floor2.chambers.iter_mut() {
            chamber.state = ChamberState::Repaired;
        }
        game.screen = Screen::Hub;
        click(&mut game, El::DoorFloor4);
        assert_eq!(game.screen, Screen::Floor4Hub);

        // Chamber 1 — the Surveyor's Bench.
        click(&mut game, El::Floor4Door(0));
        assert_eq!(game.screen, Screen::Floor4Chamber(0));
        click(&mut game, El::F4Watch);
        // (stone, size presses, align presses, annotate presses)
        let slate: [(u8, u32, u32, u32); 6] = [
            (0, 1, 1, 0),  // u8: 1 · 1, true
            (1, 5, 3, 2),  // (u8, u32): slate 5 · 4 — surprise, annotate to 1
            (2, 16, 4, 0), // &str: 16 · 8, true
            (3, 2, 1, 0),  // Option<u8>: 2 · 1, true
            (4, 16, 4, 2), // Option<&u64>: slate 16 · 8 — surprise, annotate to 1
            (5, 24, 4, 0), // Vec<u8>: 24 · 8, true
        ];
        for (stone, sizes, aligns, annotates) in slate {
            click(&mut game, El::F4Stone(stone));
            for _ in 0..sizes {
                click(&mut game, El::F4SizeUp);
            }
            for _ in 0..aligns {
                click(&mut game, El::F4AlignUp);
            }
            click(&mut game, El::F4DropLine);
            for _ in 0..annotates {
                click(&mut game, El::F4Annotate);
            }
        }
        assert!(game.expedition.floor4_diagnosed(0));
        seal_the_chamber(&mut game, 0);
        back_to_galleries(&mut game);

        // Chamber 2 — the Alignment Wall at its least span.
        click(&mut game, El::Floor4Door(1));
        assert_eq!(game.screen, Screen::Floor4Chamber(1));
        click(&mut game, El::F4Watch);
        for stone in [1u8, 2, 4, 3, 0] {
            click(&mut game, El::F4Seat(stone));
        }
        assert!(game.expedition.floor4_diagnosed(1));
        seal_the_chamber(&mut game, 1);
        back_to_galleries(&mut game);

        // Chamber 3 — the Seal Stones.
        click(&mut game, El::Floor4Door(2));
        assert_eq!(game.screen, Screen::Floor4Chamber(2));
        click(&mut game, El::F4Watch);
        for _ in 0..3 {
            click(&mut game, El::F4Crest(0));
        }
        click(&mut game, El::F4CrestProve(0));
        for _ in 0..2 {
            click(&mut game, El::F4Crest(1));
        }
        click(&mut game, El::F4CrestProve(1));
        for _ in 0..3 {
            click(&mut game, El::F4Crest(2));
        }
        click(&mut game, El::F4CrestProve(2));
        assert!(game.expedition.floor4_diagnosed(2));
        seal_the_chamber(&mut game, 2);
        back_to_galleries(&mut game);

        // Chamber 4 — the Treaty Wall.
        click(&mut game, El::Floor4Door(3));
        assert_eq!(game.screen, Screen::Floor4Chamber(3));
        click(&mut game, El::F4Watch);
        for stone in 0..4u8 {
            click(&mut game, El::F4TreatySeat(stone));
        }
        click(&mut game, El::F4TreatyProve);
        assert!(!game.expedition.floor4_diagnosed(3));
        for _ in 0..6 {
            click(&mut game, El::F4PromiseUp);
        }
        assert!(game.expedition.floor4_diagnosed(3));
        seal_the_chamber(&mut game, 3);
        back_to_galleries(&mut game);

        // Chamber 5 — the Hollow Stones.
        click(&mut game, El::Floor4Door(4));
        assert_eq!(game.screen, Screen::Floor4Chamber(4));
        click(&mut game, El::F4Watch);
        click(&mut game, El::F4Cell(0));
        click(&mut game, El::F4UseBlank);
        assert_eq!(game.expedition.floor4.chambers[4].work.referee_catches, 1);
        for cell in 1..4u8 {
            click(&mut game, El::F4Cell(cell));
        }
        click(&mut game, El::F4UseBlank);
        click(&mut game, El::F4Fill(0));
        click(&mut game, El::F4Tag(0));
        click(&mut game, El::F4Fill(1));
        click(&mut game, El::F4Fill(1));
        click(&mut game, El::F4Tag(1));
        click(&mut game, El::F4Tag(1));
        assert!(game.expedition.floor4_diagnosed(4));
        seal_the_chamber(&mut game, 4);
        back_to_galleries(&mut game);
        assert_eq!(game.expedition.floor4_chambers_repaired(), 5);

        // The FOE — the Crooked Foundation.
        click(&mut game, El::DoorFoe4);
        assert_eq!(game.screen, Screen::Floor4Foe);
        click(&mut game, El::F4BookTake);
        for entry in 0..5u8 {
            click(&mut game, El::F4Measure(entry));
        }
        for entry in [1u8, 3, 4] {
            click(&mut game, El::F4Exhibit(entry));
        }
        assert!(game.expedition.floor4_foe_proof_complete());
        click(&mut game, El::Seal);
        assert!(game.armed.is_none(), "the re-cut is not yet verified");
        for stone in 0..4u8 {
            click(&mut game, El::F4ForeignSeat(stone));
        }
        click(&mut game, El::F4ForeignProve);
        for stone in [1u8, 2, 3, 0] {
            click(&mut game, El::F4FreeSeat(stone));
        }
        click(&mut game, El::F4FreeProve);
        assert!(game.expedition.floor4.foe.gate_passed);
        click(&mut game, El::Seal);
        assert!(game.armed.is_some());
        click(&mut game, El::SealConfirm);
        assert_eq!(game.expedition.floor4.foe.state, FoeState::Fallen);

        // The floor stands complete, at full standard.
        assert!(game.expedition.floor4_complete());
        assert!(
            game.expedition
                .floor4
                .chambers
                .iter()
                .all(|chamber| chamber.facet),
            "no hint was spent: every chamber keeps its facet"
        );
        assert!(
            game.expedition
                .codex
                .iter()
                .any(|stone| stone == content::BRASS_EYE_STONE)
        );
        assert!(
            game.expedition
                .pack
                .iter()
                .any(|loot| loot == content::PLUMB_LINE)
        );
    }
}
