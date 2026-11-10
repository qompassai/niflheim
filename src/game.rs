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
    /// Floor 2's hub: grounds, doors, roster.
    Floor2Hub,
    /// A Floor 2 chamber, by index (0–4).
    Floor2Chamber(usize),
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
    /// Hub: the stairs door.
    DoorStairs,
    /// Hub: the Floor 2 door.
    DoorFloor2,
    /// Floor 2 hub: a chamber door, by index.
    Floor2Door(u8),
    /// Floor 2 hub: collapse or expand a roster group, by index.
    RosterToggle(u8),
    /// Floor 2 Chamber 1: watch both beasts perform the call.
    F2Watch,
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
    /// Run the FOE gate on this code.
    RunGate {
        /// The player's submitted draft.
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
    /// The FOE's strike.
    Foe,
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
                | Screen::Chamber(_)
                | Screen::Floor2Chamber(_)
                | Screen::Foe
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
            Screen::Foe => self.expedition.foe.strike == StrikePhase::Armed,
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
    pub fn on_gate_result(&mut self, report: GateReport) {
        self.gate_running = false;
        self.gate_detail = report.detail.clone();
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
            Screen::Floor2Hub => vec![Verb::Read, Verb::Map],
            Screen::Chamber(_) | Screen::Floor2Chamber(_) => vec![
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
        }
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
            Screen::Hub | Screen::Floor2Hub => cy == 0 || cell == (5, 4),
            Screen::Chamber(_) | Screen::Floor2Chamber(_) | Screen::Foe => {
                (8..10).contains(&cx) && (2..4).contains(&cy) || cell == (2, 7)
            }
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
                } else if self.armed.is_some() || self.expedition.foe.strike == StrikePhase::Armed {
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
                    Screen::Foe => {
                        "The hoard-engine: a ledger the size of a door, turning its \
                         pages by itself, smiling with every leaf."
                    }
                    Screen::Hub => {
                        "The vaults: five chamber doors, the Hoard's door, and the \
                         stairs — shut, for now."
                    }
                    Screen::Floor2Hub => {
                        "The Menagerie Grounds: five chamber doors, the roster \
                         board, and the Menagerie watching from its wide door."
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
            }
            Verb::Hint => {
                if let Screen::Floor2Chamber(index) = self.screen {
                    if self.expedition.floor2_hints_used(index) == 0 {
                        self.expedition.floor2_view_hint(index);
                    }
                    self.hint_showing = self.expedition.floor2_hints_used(index).max(1);
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
            El::DoorStairs => {
                self.overlay = Overlay::Reading(ReadingKind::Stairs);
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
            El::Floor2Door(index) => {
                let index = index as usize;
                if index <= 2
                    && (index == 0
                        || self.expedition.floor2.chambers[index - 1].state
                            == ChamberState::Repaired)
                {
                    self.enter_screen(Screen::Floor2Chamber(index));
                    self.say(StatusKind::Info, F2_DOOR_TEXTS[index]);
                } else if index <= 2 {
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
                _ => {}
            },
            El::EditorOpen => {
                self.overlay = Overlay::Editor;
                self.editor_cursor = self.editor_cursor.min(self.expedition.editor_code.len());
                self.say(
                    StatusKind::Info,
                    "The Interner's Hoard, in your own hand. Run the gate when it is \
                     written; the draft is kept whatever the verdict.",
                );
            }
            El::EditorArea => {
                self.focus = Some(El::EditorArea);
                self.bump();
            }
            El::RunGate => {
                if self.gate_running {
                    self.say(StatusKind::Info, "The gate is already running.");
                } else if self.expedition.editor_code.trim().is_empty() {
                    self.say(
                        StatusKind::Refusal,
                        "The editor is empty — there is nothing to judge.",
                    );
                } else {
                    self.io = Some(IoRequest::RunGate {
                        code: self.expedition.editor_code.clone(),
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
                self.expedition.editor_code = content::GATE_SCAFFOLD.to_string();
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
            None => {}
        }
    }

    // ------------------------------------------------------------------
    // The editor.
    // ------------------------------------------------------------------

    /// Position the cursor from an editor-area click.
    fn editor_click(&mut self, x: f32, y: f32) {
        let area = rects::EDITOR_AREA;
        let line = self.editor_scroll + (((y - area.y) / EDITOR_LINE_H).floor().max(0.0) as usize);
        let col = ((x - area.x - EDITOR_PAD_X) / EDITOR_CHAR_W)
            .floor()
            .max(0.0) as usize;
        self.editor_cursor = byte_at_line_col(&self.expedition.editor_code, line, col);
        self.editor_anchor = None;
        self.focus = Some(El::EditorArea);
        self.bump();
    }

    /// Insert text at the cursor (replacing any selection).
    pub fn editor_insert(&mut self, text: &str) {
        self.editor_delete_selection();
        let at = clamp_boundary(&self.expedition.editor_code, self.editor_cursor);
        self.expedition.editor_code.insert_str(at, text);
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
            clamp_boundary(&self.expedition.editor_code, start),
            clamp_boundary(&self.expedition.editor_code, end),
        );
        if start == end {
            self.editor_anchor = None;
            return false;
        }
        self.expedition.editor_code.drain(start..end);
        self.editor_cursor = start;
        self.editor_anchor = None;
        true
    }

    /// Backspace: delete the selection, or one char back.
    pub fn editor_backspace(&mut self) {
        if self.editor_delete_selection() {
            self.editor_sync_scroll();
            self.bump();
            return;
        }
        let cursor = clamp_boundary(&self.expedition.editor_code, self.editor_cursor);
        if cursor == 0 {
            return;
        }
        let prev = prev_boundary(&self.expedition.editor_code, cursor);
        self.expedition.editor_code.drain(prev..cursor);
        self.editor_cursor = prev;
        self.editor_sync_scroll();
        self.bump();
    }

    /// Delete: one char forward.
    pub fn editor_delete(&mut self) {
        if self.editor_delete_selection() {
            self.editor_sync_scroll();
            self.bump();
            return;
        }
        let cursor = clamp_boundary(&self.expedition.editor_code, self.editor_cursor);
        if cursor >= self.expedition.editor_code.len() {
            return;
        }
        let next = next_boundary(&self.expedition.editor_code, cursor);
        self.expedition.editor_code.drain(cursor..next);
        self.editor_sync_scroll();
        self.bump();
    }

    /// Select the whole draft.
    pub fn editor_select_all(&mut self) {
        self.editor_anchor = Some(0);
        self.editor_cursor = self.expedition.editor_code.len();
        self.bump();
    }

    /// Move the editor cursor. `extend` grows a selection.
    pub fn editor_move(&mut self, key: Key, extend: bool) {
        let text = &self.expedition.editor_code;
        let cursor = clamp_boundary(text, self.editor_cursor);
        if extend && self.editor_anchor.is_none() {
            self.editor_anchor = Some(cursor);
        }
        if !extend {
            self.editor_anchor = None;
        }
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
        let text = &self.expedition.editor_code;
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
        let (line, _) = line_col_of_byte(&self.expedition.editor_code, self.editor_cursor);
        if line < self.editor_scroll {
            self.editor_scroll = line;
        } else if line >= self.editor_scroll + EDITOR_VISIBLE_LINES {
            self.editor_scroll = line + 1 - EDITOR_VISIBLE_LINES;
        }
    }

    /// The cursor's (line, column in chars), 0-based.
    pub fn editor_line_col(&self) -> (usize, usize) {
        line_col_of_byte(&self.expedition.editor_code, self.editor_cursor)
    }

    /// The draft's line count.
    pub fn editor_line_count(&self) -> usize {
        line_starts(&self.expedition.editor_code).len()
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
                Screen::Foe => "The Hoard's Inscription".to_string(),
                _ => "The Inscription".to_string(),
            },
            ReadingKind::Demonstration => "The Demonstration".to_string(),
            ReadingKind::Hint => "The Hint Ladder".to_string(),
            ReadingKind::Stairs => "The Stairs Gate — Stratum I to Stratum II".to_string(),
            ReadingKind::FloorIntro => match self.screen {
                Screen::Floor2Hub => format!("Floor 2 — {}", content::FLOOR2_NAME),
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
                Screen::Foe => content::FOE_INSCRIPTION.to_string(),
                _ => content::FLOOR_INTRO.to_string(),
            },
            ReadingKind::Demonstration => match self.screen {
                Screen::Chamber(index) => CHAMBERS[index].demonstration.to_string(),
                Screen::Floor2Chamber(index) => {
                    content::MENAGERIE_CHAMBERS[index].demonstration.to_string()
                }
                Screen::Foe => content::FOE_DEMONSTRATION.to_string(),
                _ => content::FLOOR_INTRO.to_string(),
            },
            ReadingKind::Hint => {
                let rung = self.hint_showing.max(1) as usize;
                let text = match self.screen {
                    Screen::Floor2Chamber(index) => content::floor2_hints(index)[rung - 1],
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
                _ => content::FLOOR_INTRO.to_string(),
            },
            ReadingKind::MasterPlan => content::MASTER_PLAN.to_string(),
        }
    }
}

/// Per-chamber door narrations, Floor 2, for the chambers whose
/// builds have landed.
const F2_DOOR_TEXTS: [&str; 3] = [
    "Chamber 1 — Traits and impls. The forge where every beast is \
     marked: watch the call, strike the sigil, name the body.",
    "Chamber 2 — Generics and bounds. The Generic Stable: loosen the \
     posted bound, service by service, until it demands exactly what \
     each service's work asks — no more, no less.",
    "Chamber 3 — Associated types. The Two Kennels: run the same \
     errands from both houses, and find the one that runs contorted \
     where the kennel chooses.",
];

/// Per-chamber Examine narrations, Floor 2. Chambers beyond the
/// third are described by their own steps when they land.
const F2_EXAMINE_TEXTS: [&str; 5] = [
    "The forge: an anvil, a screen of hammered copper, and the sigil-book \
     open at the Anvil Call.",
    "The Generic Stable: three postings, one posted bound demanding \
     every sigil of every beast, and the Sigilwright waiting with her \
     reasons.",
    "The Two Kennels: two courier houses, one service, and a wall of \
     delivery records waiting to be read.",
    "The Sigilwright is still building this chamber.",
    "The Sigilwright is still building this chamber.",
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
        | Screen::Foe => {
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
            els.push((El::CampStone, geom::camp_stone()));
        }
        Screen::Floor2Hub => {
            els.push((El::CampStone, geom::camp_stone()));
            for index in 0..5 {
                els.push((El::Floor2Door(index as u8), geom::door_chamber(index)));
            }
            for group in 0..content::ROSTER_GROUPS.len() {
                els.push((El::RosterToggle(group as u8), geom::roster_group(group)));
            }
        }
        Screen::Chamber(index) => {
            els.push((El::Desk, geom::desk()));
            els.push((El::Stone, geom::stone()));
            els.push((El::ResetChamber, rects::RESET_BTN));
            layout_chamber_panel(&mut els, game, index);
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
        Screen::Foe => game.expedition.foe.strike == StrikePhase::Armed,
        _ => false,
    };
    els.push((El::Seal, rects::SEAL));
    if armed {
        els.push((El::SealConfirm, rects::SEAL_CONFIRM));
        els.push((El::SealCancel, rects::SEAL_CANCEL));
    }
}

/// The Floor 2 chamber panels' elements. Chamber 1: the watch
/// button, the three charter candidates, and the two bodies the
/// blind run may be named as. Chamber 2: one sigil toggle per
/// sigil per stable service. Chamber 3: the run button, the three
/// errands, and the two kennels of the finding.
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
}
