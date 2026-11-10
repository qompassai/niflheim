//! The Bevy shell: window, input translation, save IO, the gate
//! thread, and a Bevy-UI rendering of the shared game's state.
//!
//! Deliberate split (carried from the D-803 spike): every rule
//! lives in `game`/`state`; this module only translates. Pointer
//! and touch route through the shared `game::hit_test`, keyboard
//! through the abstract `game::Key`, and the editor through the
//! game's own text model — so input semantics are shell-independent
//! and the playthrough can be scripted end to end.
//!
//! Presentation binds design §6.7: no flashing sequences at all
//! (scene changes are rebuilds, not inversions), reduced motion is
//! a first-class setting, state is always carried by shape and
//! text as well as colour, and body text never goes below the
//! legibility floor.

use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::core::SaveError;
use crate::game::{DESIGN_H, DESIGN_W, Game, IoRequest, Key as GameKey, Overlay, Screen};
use crate::gate::GateReport;
use crate::platform;
use crate::state::Expedition;

/// The whole game state as a Bevy resource.
#[derive(Resource)]
struct GameRes(Game);

/// Run measurements (cold start, floor load), printed once each.
#[derive(Resource)]
struct Metrics {
    started: Instant,
    frames: u32,
    cold_done: bool,
    floor_at: Option<Instant>,
    floor_done: bool,
}

/// The bundled faces.
#[derive(Resource)]
struct Fonts {
    regular: Handle<Font>,
    bold: Handle<Font>,
    mono: Handle<Font>,
}

/// The fixed 1280x800 stage node.
#[derive(Resource)]
struct Stage(pub Entity);

/// Nodes rebuilt from game state.
#[derive(Component)]
struct DynNode;

/// The persistent delver token node.
#[derive(Component)]
struct TokenNode;

/// The last built visible-state fingerprint.
#[derive(Resource, Default)]
struct LastFingerprint(Option<u64>);

/// The in-flight gate run's result channel, if any. Behind a
/// mutex because an mpsc Receiver is Send but not Sync.
#[derive(Resource, Default)]
struct GateChannel(std::sync::Mutex<Option<Receiver<GateReport>>>);

/// Whether a touch contact is currently down (mouse/touch arbiter).
#[derive(Resource, Default)]
struct TouchActive(bool);

/// Entry point for both the desktop binary and `#[bevy_main]`.
pub fn run_app() {
    let started = Instant::now();
    let mut game = Game::new();
    game.save_present = platform::save_path().exists();
    game.today = platform::today_string();
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Niflheim — Floor 1 · The Lending Vaults".to_string(),
                        resolution: (1280u32, 800u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "wgpu=error,bevy_render=warn,bevy_winit=warn,niflheim=info".to_string(),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb_u8(32, 36, 44)))
        .insert_resource(GameRes(game))
        .insert_resource(Metrics {
            started,
            frames: 0,
            cold_done: false,
            floor_at: None,
            floor_done: false,
        })
        .insert_resource(LastFingerprint::default())
        .insert_resource(TouchActive::default())
        .insert_resource(GateChannel::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_scale,
                keyboard_input,
                editor_input,
                touch_input,
                pointer_input,
                game_tick,
                io_system,
                rebuild_ui,
                token_system,
                metrics_system,
            )
                .chain(),
        )
        .run();
}

/// Spawn the camera, load the fonts, build the stage + token.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    let regular: Handle<Font> = asset_server.load("fonts/LiberationSans-Regular.ttf");
    let bold: Handle<Font> = asset_server.load("fonts/LiberationSans-Bold.ttf");
    let mono: Handle<Font> = asset_server.load("fonts/LiberationMono-Regular.ttf");
    commands.insert_resource(Fonts {
        regular,
        bold,
        mono,
    });
    let pal = palette(false);
    let stage = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Px(DESIGN_W),
                height: Val::Px(DESIGN_H),
                position_type: PositionType::Relative,
                ..default()
            })
            .with_children(|stage| {
                // The delver token: three concentric discs, moved per
                // frame, above the room and below the overlays.
                stage
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Px(34.0),
                            height: Val::Px(34.0),
                            ..default()
                        },
                        TokenNode,
                        ZIndex(5),
                        Visibility::Hidden,
                    ))
                    .with_children(|token| {
                        for (inset, diameter, fill) in [
                            (0.0, 34.0, pal.accent),
                            (5.0, 24.0, pal.card),
                            (12.0, 10.0, pal.ink),
                        ] {
                            token.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(inset),
                                    top: Val::Px(inset),
                                    width: Val::Px(diameter),
                                    height: Val::Px(diameter),
                                    border_radius: BorderRadius::all(Val::Px(diameter / 2.0)),
                                    ..default()
                                },
                                BackgroundColor(fill),
                            ));
                        }
                    });
            });
        })
        .id();
    commands.insert_resource(Stage(stage));
}

/// The stage box entity (child of the centred root).
fn stage_box(world_children: &Query<&Children>, stage: &Stage) -> Option<Entity> {
    world_children.get(stage.0).ok()?.first().copied()
}

/// Keep `UiScale` matched to the window so design units are pixels
/// at 1280x800, and the clear colour matched to the palette.
fn update_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
    mut clear: ResMut<ClearColor>,
    game: Res<GameRes>,
) {
    if let Ok(window) = windows.single() {
        let scale = (window.width() / DESIGN_W)
            .min(window.height() / DESIGN_H)
            .max(0.05);
        if (ui_scale.0 - scale).abs() > 0.001 {
            ui_scale.0 = scale;
        }
    }
    let pal = palette(game.0.expedition.settings.high_contrast);
    if clear.0 != pal.bg {
        clear.0 = pal.bg;
    }
}

/// Convert window logical pixels to design coordinates.
fn to_design(window: &Window, scale: f32, x: f32, y: f32) -> (f32, f32) {
    let off_x = (window.width() / scale - DESIGN_W) / 2.0;
    let off_y = (window.height() / scale - DESIGN_H) / 2.0;
    (x / scale - off_x, y / scale - off_y)
}

/// Keyboard: Bevy key codes to the shared game's abstract keys.
/// Silent while the editor stands open — the editor has its own
/// channel, and text keys must never leak into verbs.
fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<GameRes>) {
    let game = &mut game.0;
    if game.overlay == Overlay::Editor {
        return;
    }
    let pressed = |codes: &[KeyCode]| codes.iter().any(|code| keys.just_pressed(*code));
    if pressed(&[KeyCode::ArrowUp, KeyCode::KeyW]) {
        game.key(GameKey::Up);
    }
    if pressed(&[KeyCode::ArrowDown, KeyCode::KeyS]) {
        game.key(GameKey::Down);
    }
    if pressed(&[KeyCode::ArrowLeft, KeyCode::KeyA]) {
        game.key(GameKey::Left);
    }
    if pressed(&[KeyCode::ArrowRight, KeyCode::KeyD]) {
        game.key(GameKey::Right);
    }
    if keys.just_pressed(KeyCode::KeyE) {
        game.key(GameKey::Examine);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        game.key(GameKey::Read);
    }
    if keys.just_pressed(KeyCode::KeyN) {
        game.key(GameKey::Enumerate);
    }
    if keys.just_pressed(KeyCode::KeyX) {
        game.key(GameKey::Exhibit);
    }
    if keys.just_pressed(KeyCode::KeyH) {
        game.key(GameKey::Hint);
    }
    if keys.just_pressed(KeyCode::KeyM) {
        game.key(GameKey::Map);
    }
    if keys.just_pressed(KeyCode::KeyL) {
        game.key(GameKey::Log);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        game.key(GameKey::Codex);
    }
    if keys.just_pressed(KeyCode::Tab) {
        game.key(GameKey::Tab);
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        game.key(GameKey::Confirm);
    }
    if keys.just_pressed(KeyCode::Escape) {
        game.key(GameKey::Cancel);
    }
}

/// The editor's keyboard channel: real text input, fed from the
/// `KeyboardInput` event stream (the reliable shape, per the
/// spike's touch lesson). Only live while the editor is open.
fn editor_input(
    mut events: MessageReader<KeyboardInput>,
    mut mods: Local<(bool, bool)>,
    mut game: ResMut<GameRes>,
) {
    if game.0.overlay != Overlay::Editor {
        *mods = (false, false);
        return;
    }
    let game = &mut game.0;
    for event in events.read() {
        // Modifiers are tracked from the event stream itself, in
        // order: a polled ButtonInput state can miss a fast
        // ctrl+key chord entirely (press and release inside one
        // frame), while the stream cannot.
        match event.key_code {
            KeyCode::ControlLeft | KeyCode::ControlRight => {
                mods.0 = event.state == ButtonState::Pressed;
                continue;
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                mods.1 = event.state == ButtonState::Pressed;
                continue;
            }
            _ => {}
        }
        if event.state != ButtonState::Pressed {
            continue;
        }
        let (ctrl, shift) = *mods;
        // Editing keys are matched on the physical key code first:
        // the logical key degrades to a control-character text
        // payload under keymap-poor displays (the Xvfb harness
        // proved it), while the code is layout-independent.
        match event.key_code {
            KeyCode::Escape => game.key(GameKey::Cancel),
            KeyCode::Backspace => game.editor_backspace(),
            KeyCode::Delete => game.editor_delete(),
            KeyCode::ArrowLeft => game.editor_move(GameKey::Left, shift),
            KeyCode::ArrowRight => game.editor_move(GameKey::Right, shift),
            KeyCode::ArrowUp => game.editor_move(GameKey::Up, shift),
            KeyCode::ArrowDown => game.editor_move(GameKey::Down, shift),
            KeyCode::Home => game.editor_line_edge(false),
            KeyCode::End => game.editor_line_edge(true),
            KeyCode::Enter | KeyCode::NumpadEnter => game.editor_insert("\n"),
            KeyCode::Tab => game.editor_insert("    "),
            KeyCode::KeyA if ctrl => game.editor_select_all(),
            _ => {
                if let Some(text) = &event.text {
                    match text.as_str() {
                        // The same keys, arrived as text.
                        "\r" | "\n" | "\r\n" => game.editor_insert("\n"),
                        "\u{7f}" | "\u{8}" => game.editor_backspace(),
                        "\t" => game.editor_insert("    "),
                        _ if !ctrl && !text.chars().any(char::is_control) => {
                            let text = text.to_string();
                            game.editor_insert(&text);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Mouse: press/release through the shared pointer handlers.
fn pointer_input(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    touch_active: Res<TouchActive>,
    mut game: ResMut<GameRes>,
    mut last: Local<Option<Vec2>>,
) {
    if touch_active.0 {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let game = &mut game.0;
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let (x, y) = to_design(window, ui_scale.0, cursor.x, cursor.y);
    if buttons.just_pressed(MouseButton::Left) {
        game.pointer_down(x, y);
    } else if buttons.just_released(MouseButton::Left) {
        game.pointer_up(x, y);
    } else if game.pointer_down && *last != Some(cursor) {
        game.pointer_move(x, y);
    }
    *last = Some(cursor);
}

/// Touch: the same shared handlers, fed from the `TouchInput`
/// event stream (per the spike's finding, the polled `Touches`
/// resource drops completed drags; the event stream does not).
fn touch_input(
    mut events: MessageReader<TouchInput>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut touch_active: ResMut<TouchActive>,
    mut game: ResMut<GameRes>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let game = &mut game.0;
    for event in events.read() {
        let (x, y) = to_design(window, ui_scale.0, event.position.x, event.position.y);
        match event.phase {
            TouchPhase::Started => {
                touch_active.0 = true;
                game.pointer_down(x, y);
            }
            TouchPhase::Moved => game.pointer_move(x, y),
            TouchPhase::Ended | TouchPhase::Canceled => {
                touch_active.0 = false;
                game.pointer_up(x, y);
            }
        }
    }
}

/// Advance the game clock, the inscription date, and note the
/// first floor entry for the load metric.
fn game_tick(time: Res<Time>, mut game: ResMut<GameRes>, mut metrics: ResMut<Metrics>) {
    game.0.today = platform::today_string();
    let was_title = game.0.screen == Screen::Title;
    game.0.update(time.delta_secs().min(0.05));
    if was_title && game.0.screen != Screen::Title && metrics.floor_at.is_none() {
        metrics.floor_at = Some(Instant::now());
    }
}

/// Perform the game's file and gate requests. Gate runs happen on
/// a worker thread (owned by the channel); the verdict is polled
/// back here so the game never blocks on cargo.
fn io_system(mut game: ResMut<GameRes>, gate_channel: ResMut<GateChannel>) {
    let game = &mut game.0;
    while let Some(request) = game.io.take() {
        match request {
            IoRequest::WriteSave { announce } => match game.expedition.save_json() {
                Ok(json) => {
                    let path = platform::save_path();
                    match platform::write_atomic(&path, json.as_bytes()) {
                        Ok(()) => {
                            game.save_present = true;
                            platform::log_line(format!("save_written bytes={}", json.len()));
                            if announce {
                                game.on_save_written(json.len());
                            }
                        }
                        Err(error) => {
                            let reason = error.to_string();
                            platform::log_line(format!("save_write_failed reason={reason}"));
                            if announce {
                                game.on_save_failed(reason);
                            }
                        }
                    }
                }
                Err(error) => {
                    if announce {
                        game.on_save_failed(error.message());
                    }
                }
            },
            IoRequest::ReadSave => {
                let path = platform::save_path();
                let result = std::fs::read_to_string(&path)
                    .map_err(|_| SaveError::NotASave)
                    .and_then(|text| Expedition::load_json(&text));
                match &result {
                    Ok(expedition) => {
                        platform::log_line(format!(
                            "save_loaded chambers_repaired={} foe={}",
                            expedition.chambers_repaired(),
                            match expedition.foe.state {
                                crate::state::FoeState::Standing => "standing",
                                crate::state::FoeState::Exposed => "exposed",
                                crate::state::FoeState::Fallen => "fallen",
                            }
                        ));
                    }
                    Err(error) => {
                        platform::log_line(format!("save_load_failed reason={}", error.message()));
                    }
                }
                game.on_load_result(result);
            }
            IoRequest::RunGate { code } => {
                let (sender, receiver): (Sender<GateReport>, Receiver<GateReport>) =
                    std::sync::mpsc::channel();
                *gate_channel
                    .0
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner()) = Some(receiver);
                let root = platform::gate_scratch_root();
                platform::log_line("gate_run_started".to_string());
                std::thread::spawn(move || {
                    let report =
                        crate::gate::run_gate(&root, &code, &crate::gate::GateLimits::default());
                    let _ = sender.send(report);
                });
            }
        }
    }
    let finished = {
        let mut slot = gate_channel
            .0
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        match slot.as_ref().map(|receiver| receiver.try_recv()) {
            Some(Ok(report)) => {
                *slot = None;
                Some(report)
            }
            _ => None,
        }
    };
    if let Some(report) = finished {
        platform::log_line(format!("gate_run_finished passed={}", report.passed()));
        game.on_gate_result(report);
    }
}

/// Print the cold-start line once the third frame has run.
fn metrics_system(mut metrics: ResMut<Metrics>) {
    metrics.frames += 1;
    if !metrics.cold_done && metrics.frames >= 3 {
        metrics.cold_done = true;
        platform::log_line(format!(
            "cold_start_ms={}",
            metrics.started.elapsed().as_millis()
        ));
    }
}

/// The delver token follows the game's player position every frame.
fn token_system(
    game: Res<GameRes>,
    mut tokens: Query<(&mut Node, &mut Visibility), With<TokenNode>>,
) {
    for (mut node, mut visibility) in &mut tokens {
        let (px, py) = game.0.player_px();
        node.left = Val::Px(px - 17.0);
        node.top = Val::Px(py - 17.0);
        let want = if game.0.token_visible() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
    }
}

// ---------------------------------------------------------------------------
// Palette (the spike's values, unchanged).
// ---------------------------------------------------------------------------

/// The shell palette as Bevy colours.
struct Pal {
    bg: Color,
    panel: Color,
    edge: Color,
    floor_a: Color,
    floor_b: Color,
    ink: Color,
    ink_dim: Color,
    accent: Color,
    card: Color,
    good: Color,
    bad: Color,
    wax: Color,
    dim: Color,
}

/// The palette for the current settings.
fn palette(high_contrast: bool) -> Pal {
    if high_contrast {
        Pal {
            bg: Color::srgb_u8(0, 0, 0),
            panel: Color::srgb_u8(10, 10, 10),
            edge: Color::srgb_u8(255, 255, 255),
            floor_a: Color::srgb_u8(24, 24, 24),
            floor_b: Color::srgb_u8(38, 38, 38),
            ink: Color::srgb_u8(255, 255, 255),
            ink_dim: Color::srgb_u8(200, 200, 200),
            accent: Color::srgb_u8(255, 215, 0),
            card: Color::srgb_u8(20, 20, 20),
            good: Color::srgb_u8(125, 255, 138),
            bad: Color::srgb_u8(255, 154, 138),
            wax: Color::srgb_u8(255, 120, 90),
            dim: Color::srgba_u8(0, 0, 0, 184),
        }
    } else {
        Pal {
            bg: Color::srgb_u8(32, 36, 44),
            panel: Color::srgb_u8(242, 232, 208),
            edge: Color::srgb_u8(138, 109, 63),
            floor_a: Color::srgb_u8(232, 220, 192),
            floor_b: Color::srgb_u8(223, 208, 172),
            ink: Color::srgb_u8(43, 33, 24),
            ink_dim: Color::srgb_u8(109, 92, 68),
            accent: Color::srgb_u8(176, 125, 42),
            card: Color::srgb_u8(255, 248, 230),
            good: Color::srgb_u8(61, 107, 53),
            bad: Color::srgb_u8(156, 43, 30),
            wax: Color::srgb_u8(138, 51, 36),
            dim: Color::srgba_u8(10, 8, 4, 140),
        }
    }
}

// ---------------------------------------------------------------------------
// Node-building helpers (all design-space, absolute positioned).
// ---------------------------------------------------------------------------

use crate::game::Rect;

/// An absolutely positioned box in design coordinates.
fn box_node(rect: Rect) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(rect.x),
        top: Val::Px(rect.y),
        width: Val::Px(rect.w),
        height: Val::Px(rect.h),
        ..default()
    }
}

/// Spawn a filled (optionally bordered, optionally rounded) box.
fn spawn_box(
    commands: &mut Commands,
    parent: Entity,
    rect: Rect,
    fill: Option<Color>,
    border: Option<(f32, Color)>,
    radius: f32,
) -> Entity {
    let mut node = box_node(rect);
    if let Some((width, _)) = border {
        node.border = UiRect::all(Val::Px(width));
    }
    if radius > 0.0 {
        node.border_radius = BorderRadius::all(Val::Px(radius));
    }
    let mut entity = commands.spawn((node, DynNode, ChildOf(parent)));
    if let Some(fill) = fill {
        entity.insert(BackgroundColor(fill));
    }
    if let Some((_, color)) = border {
        entity.insert(BorderColor::all(color));
    }
    entity.id()
}

/// Spawn a text run in one of the bundled faces.
#[allow(clippy::too_many_arguments)]
fn spawn_text_face(
    commands: &mut Commands,
    parent: Entity,
    x: f32,
    y: f32,
    width: Option<f32>,
    text: &str,
    size: f32,
    face: &Handle<Font>,
    color: Color,
    justify: Justify,
) -> Entity {
    let mut node = Node {
        position_type: PositionType::Absolute,
        left: Val::Px(x),
        top: Val::Px(y),
        ..default()
    };
    if let Some(width) = width {
        node.width = Val::Px(width);
    }
    commands
        .spawn((
            node,
            DynNode,
            ChildOf(parent),
            Text::new(text),
            TextFont {
                font: face.clone().into(),
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            TextLayout {
                justify,
                linebreak: LineBreak::WordBoundary,
            },
        ))
        .id()
}

/// Spawn a text run (regular or bold).
#[allow(clippy::too_many_arguments)]
fn spawn_text(
    commands: &mut Commands,
    parent: Entity,
    x: f32,
    y: f32,
    width: Option<f32>,
    text: &str,
    size: f32,
    bold: bool,
    color: Color,
    justify: Justify,
    fonts: &Fonts,
) -> Entity {
    let face = if bold {
        fonts.bold.clone()
    } else {
        fonts.regular.clone()
    };
    spawn_text_face(
        commands, parent, x, y, width, text, size, &face, color, justify,
    )
}

/// Spawn a monospaced text run.
#[allow(clippy::too_many_arguments)]
fn spawn_text_mono(
    commands: &mut Commands,
    parent: Entity,
    x: f32,
    y: f32,
    width: Option<f32>,
    text: &str,
    size: f32,
    color: Color,
    fonts: &Fonts,
) -> Entity {
    let face = fonts.mono.clone();
    spawn_text_face(
        commands,
        parent,
        x,
        y,
        width,
        text,
        size,
        &face,
        color,
        Justify::Left,
    )
}

/// Centred text over a rect's horizontal centre at a top offset.
#[allow(clippy::too_many_arguments)]
fn spawn_text_centered(
    commands: &mut Commands,
    parent: Entity,
    cx: f32,
    y: f32,
    width: f32,
    text: &str,
    size: f32,
    bold: bool,
    color: Color,
    fonts: &Fonts,
) -> Entity {
    spawn_text(
        commands,
        parent,
        cx - width / 2.0,
        y,
        Some(width),
        text,
        size,
        bold,
        color,
        Justify::Center,
        fonts,
    )
}

/// Right-aligned text ending at `right`.
#[allow(clippy::too_many_arguments)]
fn spawn_text_right(
    commands: &mut Commands,
    parent: Entity,
    right: f32,
    y: f32,
    width: f32,
    text: &str,
    size: f32,
    bold: bool,
    color: Color,
    fonts: &Fonts,
) -> Entity {
    spawn_text(
        commands,
        parent,
        right - width,
        y,
        Some(width),
        text,
        size,
        bold,
        color,
        Justify::Right,
        fonts,
    )
}

/// A button: face, edge, centred bold label.
fn spawn_button(
    commands: &mut Commands,
    parent: Entity,
    rect: Rect,
    label: &str,
    pal: &Pal,
    active: bool,
    fonts: &Fonts,
) {
    let face = if active { pal.accent } else { pal.card };
    spawn_box(
        commands,
        parent,
        rect,
        Some(face),
        Some((2.0, pal.edge)),
        0.0,
    );
    let dark_bg = pal.bg == Color::srgb_u8(0, 0, 0);
    let ink = if active && !dark_bg {
        Color::srgb_u8(255, 250, 238)
    } else {
        pal.ink
    };
    spawn_text_centered(
        commands,
        parent,
        rect.cx(),
        rect.y + 9.0,
        rect.w,
        label,
        15.0,
        true,
        ink,
        fonts,
    );
}

/// A filled circle given by centre and radius.
fn spawn_circle(
    commands: &mut Commands,
    parent: Entity,
    cx: f32,
    cy: f32,
    radius: f32,
    fill: Color,
) {
    spawn_box(
        commands,
        parent,
        Rect::new(cx - radius, cy - radius, radius * 2.0, radius * 2.0),
        Some(fill),
        None,
        radius,
    );
}

// ---------------------------------------------------------------------------
// The rebuild: dynamic UI mirrors game state whenever the
// fingerprint moves.
// ---------------------------------------------------------------------------

use crate::content::{self, CHAMBERS};
use crate::game::{ROOM_COLS, ROOM_ROWS, ReadingKind, StatusKind, cell_rect, geom, layout, rects};
use crate::puzzles;
use crate::state::{ChamberState, FoeState};

/// A cheap hash of everything that changes what is on screen.
fn fingerprint(game: &Game) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    game.revision().hash(&mut hasher);
    game.gate_running.hash(&mut hasher);
    game.editor_cursor.hash(&mut hasher);
    game.editor_scroll.hash(&mut hasher);
    game.editor_anchor.is_some().hash(&mut hasher);
    game.hint_showing.hash(&mut hasher);
    if let Some(focused) = game.focused_element() {
        std::mem::discriminant(&focused).hash(&mut hasher);
    }
    hasher.finish()
}

/// Rebuild the dynamic UI when the visible state changed.
#[allow(clippy::too_many_arguments)]
fn rebuild_ui(
    mut commands: Commands,
    game: Res<GameRes>,
    fonts: Option<Res<Fonts>>,
    stage: Option<Res<Stage>>,
    children: Query<&Children>,
    dynamic: Query<Entity, With<DynNode>>,
    mut last: ResMut<LastFingerprint>,
    mut metrics: ResMut<Metrics>,
) {
    let Some(fonts) = fonts else { return };
    let Some(stage) = stage else { return };
    let Some(stage_box) = stage_box(&children, &stage) else {
        return;
    };
    let fp = fingerprint(&game.0);
    if last.0 == Some(fp) {
        return;
    }
    last.0 = Some(fp);
    rebuild_now(&mut commands, &game.0, &fonts, stage_box, &dynamic);
    if !metrics.floor_done
        && let Some(at) = metrics.floor_at
        && game.0.screen != Screen::Title
    {
        metrics.floor_done = true;
        platform::log_line(format!("floor_load_ms={}", at.elapsed().as_millis()));
    }
}

/// The actual rebuild.
fn rebuild_now(
    commands: &mut Commands,
    game: &Game,
    fonts: &Fonts,
    stage: Entity,
    dynamic: &Query<Entity, With<DynNode>>,
) {
    for entity in dynamic.iter() {
        commands.entity(entity).despawn();
    }
    let pal = palette(game.expedition.settings.high_contrast);
    spawn_box(
        commands,
        stage,
        Rect::new(0.0, 0.0, DESIGN_W, DESIGN_H),
        Some(pal.bg),
        None,
        0.0,
    );
    match game.screen {
        Screen::Title => build_title(commands, game, &pal, fonts, stage),
        Screen::Hub => build_hub(commands, game, &pal, fonts, stage),
        Screen::Chamber(index) => build_chamber(commands, game, &pal, fonts, stage, index),
        Screen::Foe => build_foe(commands, game, &pal, fonts, stage),
    }
    match game.overlay {
        Overlay::None => {}
        Overlay::Reading(kind) => build_reading(commands, game, &pal, fonts, stage, kind),
        Overlay::Map => build_map(commands, game, &pal, fonts, stage),
        Overlay::Log => build_log(commands, game, &pal, fonts, stage),
        Overlay::Pause => build_pause(commands, game, &pal, fonts, stage),
        Overlay::Codex => build_codex(commands, game, &pal, fonts, stage),
        Overlay::Editor => build_editor(commands, game, &pal, fonts, stage),
        Overlay::GateRecord => build_gate_record(commands, game, &pal, fonts, stage),
    }
    build_focus(commands, game, &pal, stage);
}

/// The keyboard focus outline around the focused element.
fn build_focus(commands: &mut Commands, game: &Game, pal: &Pal, stage: Entity) {
    if let Some(focused) = game.focused_element() {
        let rects = layout(game);
        if let Some((_, rect)) = rects.iter().find(|(el, _)| *el == focused) {
            spawn_box(
                commands,
                stage,
                Rect::new(rect.x - 3.0, rect.y - 3.0, rect.w + 6.0, rect.h + 6.0),
                None,
                Some((3.0, pal.accent)),
                0.0,
            );
        }
    }
}

/// Title screen: the descent, the begin/continue, and the settings
/// strip the design asks for at first launch.
fn build_title(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    use rects::*;
    spawn_box(
        commands,
        stage,
        TITLE_PANEL,
        Some(pal.panel),
        Some((3.0, pal.edge)),
        0.0,
    );
    spawn_text_centered(
        commands, stage, 640.0, 150.0, 600.0, "NIFLHEIM", 44.0, true, pal.ink, fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        640.0,
        220.0,
        600.0,
        "A descent in seventeen floors",
        17.0,
        false,
        pal.ink_dim,
        fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        640.0,
        262.0,
        500.0,
        "Floor 1 — The Lending Vaults. One thing, one holder: audit the plan that \
         promises the same chart twice, prove it, write the Hoard that replaces \
         it, and strike.",
        15.0,
        false,
        pal.ink,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        BEGIN,
        "Begin the descent",
        pal,
        false,
        fonts,
    );
    if game.save_present {
        spawn_button(
            commands,
            stage,
            CONTINUE,
            "Continue the expedition",
            pal,
            false,
            fonts,
        );
    }
    let settings = &game.expedition.settings;
    let labels = [
        format!("Text: {}", settings.text_scale.label()),
        format!(
            "Contrast: {}",
            if settings.high_contrast {
                "High"
            } else {
                "Standard"
            }
        ),
        format!(
            "Motion: {}",
            if settings.reduced_motion {
                "Reduced"
            } else {
                "Full"
            }
        ),
    ];
    for (index, label) in labels.iter().enumerate() {
        spawn_button(
            commands,
            stage,
            title_setting(index),
            label,
            pal,
            false,
            fonts,
        );
    }
    spawn_text_centered(
        commands,
        stage,
        640.0,
        630.0,
        600.0,
        "Production build — Floor 1 complete; the deeper floors stand charted, unbuilt.",
        12.0,
        false,
        pal.ink_dim,
        fonts,
    );
}

/// The header band for room screens.
fn build_header(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    use rects::*;
    spawn_box(commands, stage, HEADER, Some(pal.panel), None, 0.0);
    let title = match game.screen {
        Screen::Hub => "Niflheim — Floor 1 · The Lending Vaults".to_string(),
        Screen::Chamber(index) => {
            format!("Floor 1 · Chamber {} — {}", index + 1, CHAMBERS[index].name)
        }
        Screen::Foe => "Floor 1 · The Borrowed Hoard".to_string(),
        Screen::Title => "Niflheim".to_string(),
    };
    spawn_text(
        commands,
        stage,
        20.0,
        13.0,
        None,
        &title,
        20.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        TAB_FLOOR,
        "Floor",
        pal,
        game.screen == Screen::Hub && game.overlay == Overlay::None,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        TAB_MAP,
        "Map",
        pal,
        game.overlay == Overlay::Map,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        TAB_LOG,
        "Log",
        pal,
        game.overlay == Overlay::Log,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        TAB_CODEX,
        "Codex",
        pal,
        game.overlay == Overlay::Codex,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        PAUSE_BTN,
        "Pause",
        pal,
        game.overlay == Overlay::Pause,
        fonts,
    );
}

/// The room: floor checker and caption. Dressing is per screen.
fn build_room_base(
    commands: &mut Commands,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
    caption: &str,
) {
    use rects::*;
    spawn_box(
        commands,
        stage,
        ROOM,
        Some(pal.panel),
        Some((2.0, pal.edge)),
        0.0,
    );
    for cy in 0..ROOM_ROWS {
        for cx in 0..ROOM_COLS {
            let tone = if (cx + cy) % 2 == 0 {
                pal.floor_a
            } else {
                pal.floor_b
            };
            spawn_box(commands, stage, cell_rect(cx, cy), Some(tone), None, 0.0);
        }
    }
    spawn_text(
        commands,
        stage,
        32.0,
        646.0,
        None,
        caption,
        13.0,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
}

/// A labelled block in the room (desk, engine, stone).
#[allow(clippy::too_many_arguments)]
fn build_block(
    commands: &mut Commands,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
    rect: Rect,
    fill: Color,
    label: &str,
    sub: &str,
) {
    spawn_box(commands, stage, rect, Some(fill), Some((3.0, pal.ink)), 0.0);
    spawn_text_centered(
        commands,
        stage,
        rect.cx(),
        rect.y + 8.0,
        rect.w,
        label,
        12.0,
        true,
        pal.ink,
        fonts,
    );
    if !sub.is_empty() {
        spawn_text_centered(
            commands,
            stage,
            rect.cx(),
            rect.y + 26.0,
            rect.w,
            sub,
            10.5,
            false,
            pal.ink,
            fonts,
        );
    }
}

/// The bottom bar: verb chips and the status line. The status
/// carries a glyph as well as a colour — never colour alone.
fn build_bottom(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    use rects::*;
    spawn_box(
        commands,
        stage,
        BOTTOM,
        Some(pal.panel),
        Some((2.0, pal.edge)),
        0.0,
    );
    let verbs = game.verbs();
    let count = verbs.len();
    for (index, verb) in verbs.iter().enumerate() {
        spawn_button(
            commands,
            stage,
            rects::chip(index, count),
            verb.label(),
            pal,
            false,
            fonts,
        );
    }
    let (glyph, color) = match game.status_kind {
        StatusKind::Info => ("", pal.ink),
        StatusKind::Refusal => ("× ", pal.bad),
        StatusKind::Good => ("● ", pal.good),
    };
    spawn_text(
        commands,
        stage,
        716.0,
        688.0,
        Some(532.0),
        &format!("{glyph}{}", game.status),
        13.5,
        false,
        color,
        Justify::Left,
        fonts,
    );
}

/// The strike seal and its caption, for chamber and FOE screens.
fn build_seal(
    commands: &mut Commands,
    game: &Game,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
    active: bool,
    caption: &str,
) {
    use rects::*;
    let armed = game.seal_armed();
    let seal_color = if active || armed {
        pal.wax
    } else {
        pal.ink_dim.with_alpha(0.5)
    };
    spawn_circle(commands, stage, SEAL.cx(), SEAL.cy(), 38.0, seal_color);
    spawn_circle(commands, stage, SEAL.cx(), SEAL.cy(), 31.0, pal.panel);
    spawn_circle(commands, stage, SEAL.cx(), SEAL.cy(), 28.0, seal_color);
    spawn_text_centered(
        commands,
        stage,
        SEAL.cx(),
        SEAL.cy() - 8.0,
        76.0,
        "STRIKE",
        11.0,
        true,
        pal.ink,
        fonts,
    );
    if armed {
        spawn_button(
            commands,
            stage,
            SEAL_CONFIRM,
            "Confirm the strike",
            pal,
            true,
            fonts,
        );
        spawn_button(
            commands,
            stage,
            SEAL_CANCEL,
            "Stand down",
            pal,
            false,
            fonts,
        );
    } else {
        spawn_text_centered(
            commands,
            stage,
            1028.0,
            598.0,
            420.0,
            caption,
            11.5,
            false,
            pal.ink_dim,
            fonts,
        );
    }
}

/// The panel's title block and the reset-proof control.
fn build_panel_header(
    commands: &mut Commands,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
    title: &str,
    subtitle: &str,
) {
    spawn_box(
        commands,
        stage,
        rects::PANEL,
        Some(pal.panel),
        Some((2.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        808.0,
        84.0,
        None,
        title,
        19.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        808.0,
        112.0,
        None,
        subtitle,
        12.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        rects::RESET_BTN,
        "Reset proof",
        pal,
        false,
        fonts,
    );
}

/// The proof slot for non-C4 chambers.
fn build_slot(
    commands: &mut Commands,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
    state: ChamberState,
    wants: &str,
    facet: bool,
) {
    use rects::*;
    spawn_box(
        commands,
        stage,
        SLOT,
        Some(pal.card),
        Some((2.0, pal.edge)),
        0.0,
    );
    match state {
        ChamberState::Unworked => {
            spawn_text(
                commands,
                stage,
                820.0,
                SLOT.y + 6.0,
                Some(416.0),
                &format!("The slot wants: {wants}."),
                12.0,
                false,
                pal.ink_dim,
                Justify::Left,
                fonts,
            );
        }
        ChamberState::Diagnosed => {
            spawn_text_centered(
                commands,
                stage,
                SLOT.cx(),
                SLOT.y + 18.0,
                SLOT.w,
                "The proof stands filed — the seal wakes.",
                14.0,
                true,
                pal.good,
                fonts,
            );
        }
        ChamberState::Repaired => {
            spawn_text_centered(
                commands,
                stage,
                SLOT.cx(),
                SLOT.y + 18.0,
                SLOT.w,
                if facet {
                    "Repaired at full standard — a facet earned."
                } else {
                    "Repaired."
                },
                14.0,
                true,
                pal.good,
                fonts,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The floor hub.
// ---------------------------------------------------------------------------

/// The hub: camp stone, five chamber doors, the FOE's door, the
/// stairs, and the floor's progress in the panel.
fn build_hub(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    build_header(commands, game, pal, fonts, stage);
    build_room_base(
        commands,
        pal,
        fonts,
        stage,
        "The Vaults — five chambers, the Hoard, and the stairs",
    );
    // Chamber doors along the top wall.
    for (index, meta) in CHAMBERS.iter().enumerate() {
        let rect = geom::door_chamber(index);
        let state = game.expedition.chambers[index].state;
        let locked =
            index > 0 && game.expedition.chambers[index - 1].state != ChamberState::Repaired;
        spawn_box(
            commands,
            stage,
            rect,
            Some(pal.card),
            Some((2.0, pal.edge)),
            0.0,
        );
        spawn_text_centered(
            commands,
            stage,
            rect.cx(),
            rect.y + 6.0,
            rect.w,
            &format!("CHAMBER {}", index + 1),
            10.5,
            true,
            pal.ink,
            fonts,
        );
        spawn_text_centered(
            commands,
            stage,
            rect.cx(),
            rect.y + 24.0,
            rect.w - 8.0,
            meta.name,
            10.5,
            false,
            pal.ink,
            fonts,
        );
        let (tag, color) = match state {
            ChamberState::Repaired => ("REPAIRED", pal.good),
            ChamberState::Diagnosed => ("PROOF FILED", pal.accent),
            ChamberState::Unworked => {
                if locked {
                    ("BARRED", pal.ink_dim)
                } else {
                    ("", pal.ink_dim)
                }
            }
        };
        if !tag.is_empty() {
            spawn_text_centered(
                commands,
                stage,
                rect.cx(),
                rect.y + 64.0,
                rect.w,
                tag,
                10.5,
                true,
                color,
                fonts,
            );
        }
    }
    // The camp stone.
    let stone = geom::camp_stone();
    spawn_box(
        commands,
        stage,
        stone,
        Some(Color::srgb_u8(140, 140, 148)),
        Some((2.0, pal.ink_dim)),
        0.0,
    );
    spawn_text_centered(
        commands,
        stage,
        stone.cx(),
        stone.y + 12.0,
        stone.w,
        "THE",
        11.0,
        true,
        pal.ink,
        fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        stone.cx(),
        stone.y + 26.0,
        stone.w,
        "FLOOR",
        11.0,
        true,
        pal.ink,
        fonts,
    );
    // The stairs door.
    let stairs = geom::door_stairs();
    spawn_box(
        commands,
        stage,
        stairs,
        Some(pal.card),
        Some((2.0, pal.edge)),
        0.0,
    );
    spawn_text_centered(
        commands,
        stage,
        stairs.cx(),
        stairs.y + 34.0,
        stairs.w,
        "STAIRS",
        13.0,
        true,
        pal.ink,
        fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        stairs.cx(),
        stairs.y + 56.0,
        stairs.w,
        "to Stratum II",
        11.0,
        false,
        pal.ink_dim,
        fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        stairs.cx(),
        stairs.y + 76.0,
        stairs.w,
        "SHUT",
        11.0,
        true,
        pal.bad,
        fonts,
    );
    // The FOE's door.
    let foe = geom::door_foe();
    spawn_box(
        commands,
        stage,
        foe,
        Some(pal.card),
        Some((2.5, pal.wax)),
        0.0,
    );
    spawn_text_centered(
        commands,
        stage,
        foe.cx(),
        foe.y + 26.0,
        foe.w,
        "THE BORROWED",
        14.0,
        true,
        pal.ink,
        fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        foe.cx(),
        foe.y + 46.0,
        foe.w,
        "HOARD",
        14.0,
        true,
        pal.ink,
        fonts,
    );
    let (foe_tag, foe_color) = match game.expedition.foe.state {
        FoeState::Standing => ("FOE · standing", pal.wax),
        FoeState::Exposed => ("FOE · EXPOSED", pal.accent),
        FoeState::Fallen => ("FOE · FALLEN", pal.good),
    };
    spawn_text_centered(
        commands,
        stage,
        foe.cx(),
        foe.y + 82.0,
        foe.w,
        foe_tag,
        12.0,
        true,
        foe_color,
        fonts,
    );
    // The panel: the floor's progress.
    build_panel_header(
        commands,
        pal,
        fonts,
        stage,
        "The Lending Vaults",
        "Keeper: the Oathwarden · Domain 1 — Ownership",
    );
    let mut y = 150.0;
    for (index, meta) in CHAMBERS.iter().enumerate() {
        let chamber = &game.expedition.chambers[index];
        let (mark, color) = match chamber.state {
            ChamberState::Repaired => ("■", pal.good),
            ChamberState::Diagnosed => ("●", pal.accent),
            ChamberState::Unworked => ("○", pal.ink_dim),
        };
        spawn_text(
            commands,
            stage,
            808.0,
            y,
            None,
            mark,
            14.0,
            true,
            color,
            Justify::Left,
            fonts,
        );
        spawn_text(
            commands,
            stage,
            834.0,
            y,
            None,
            &format!("Chamber {} — {}", index + 1, meta.name),
            13.5,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
        if chamber.facet {
            spawn_text_right(
                commands, stage, 1240.0, y, 120.0, "facet", 12.0, true, pal.accent, fonts,
            );
        }
        y += 30.0;
    }
    let (foe_mark, foe_color) = match game.expedition.foe.state {
        FoeState::Fallen => ("■", pal.good),
        FoeState::Exposed => ("●", pal.accent),
        FoeState::Standing => ("○", pal.ink_dim),
    };
    spawn_text(
        commands,
        stage,
        808.0,
        y,
        None,
        foe_mark,
        14.0,
        true,
        foe_color,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        834.0,
        y,
        None,
        "The Borrowed Hoard — FOE",
        13.5,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    y += 44.0;
    spawn_text(
        commands,
        stage,
        808.0,
        y,
        None,
        "THE STAIRS GATE",
        12.0,
        true,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    y += 24.0;
    for (lit, text) in game.expedition.stairs_clauses() {
        spawn_text(
            commands,
            stage,
            808.0,
            y,
            Some(432.0),
            &format!("{} {}", if lit { "[lit]" } else { "[unlit]" }, text),
            11.5,
            false,
            if lit { pal.good } else { pal.ink_dim },
            Justify::Left,
            fonts,
        );
        y += 34.0;
    }
    if game.expedition.floor_complete() {
        spawn_text(
            commands,
            stage,
            808.0,
            560.0,
            Some(432.0),
            "FLOOR 1 CHARTED — every chamber repaired, the Hoard fallen, the chart drawn.",
            13.0,
            true,
            pal.good,
            Justify::Left,
            fonts,
        );
    }
    build_bottom(commands, game, pal, fonts, stage);
}

// ---------------------------------------------------------------------------
// Chambers.
// ---------------------------------------------------------------------------

/// One chamber screen: room, proof panel, seal, bottom bar.
fn build_chamber(
    commands: &mut Commands,
    game: &Game,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
    index: usize,
) {
    let meta = &CHAMBERS[index];
    build_header(commands, game, pal, fonts, stage);
    build_room_base(
        commands,
        pal,
        fonts,
        stage,
        &format!("Chamber {} — {} · {}", index + 1, meta.name, meta.room),
    );
    let subjects = [
        "ISSUE DESK",
        "THE CHART",
        "THE CHARTER",
        "STEWARD'S DESK",
        "THE YARD GATE",
    ];
    build_block(
        commands,
        pal,
        fonts,
        stage,
        geom::desk(),
        Color::srgb_u8(166, 124, 58),
        subjects[index],
        "the demonstration",
    );
    let stone = geom::stone();
    spawn_box(
        commands,
        stage,
        stone,
        Some(Color::srgb_u8(140, 140, 148)),
        Some((2.0, pal.ink_dim)),
        0.0,
    );
    spawn_text_centered(
        commands,
        stage,
        stone.cx(),
        stone.y + 12.0,
        stone.w,
        "THE",
        11.0,
        true,
        pal.ink,
        fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        stone.cx(),
        stone.y + 26.0,
        stone.w,
        "RULE",
        11.0,
        true,
        pal.ink,
        fonts,
    );
    build_panel_header(commands, pal, fonts, stage, meta.name, meta.room);
    let progress = &game.expedition.chambers[index];
    if index == 3 {
        build_c4_slots(commands, game, pal, fonts, stage);
    } else {
        build_slot(
            commands,
            pal,
            fonts,
            stage,
            progress.state,
            meta.slot_wants,
            progress.facet,
        );
    }
    match index {
        0 => build_c1(commands, game, pal, fonts, stage),
        1 => build_c2(commands, game, pal, fonts, stage),
        2 => build_c3(commands, game, pal, fonts, stage),
        3 => build_c4(commands, game, pal, fonts, stage),
        _ => build_c5(commands, game, pal, fonts, stage),
    }
    let (active, caption) = match progress.state {
        ChamberState::Diagnosed => (true, "The seal wakes — press it to repair."),
        ChamberState::Repaired => (false, "Repaired. This chamber's square fills."),
        ChamberState::Unworked => (false, "The seal sleeps until the proof is complete."),
    };
    build_seal(commands, game, pal, fonts, stage, active, caption);
    build_bottom(commands, game, pal, fonts, stage);
}

/// C1 — the packing list and the Toll of Tallies (two sub-views).
fn build_c1(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    let work = &game.expedition.chambers[0].work;
    spawn_button(
        commands,
        stage,
        geom::panel_tab(0),
        "The packing",
        pal,
        game.c1_view() == 0,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        geom::panel_tab(1),
        "The toll",
        pal,
        game.c1_view() == 1,
        fonts,
    );
    if game.c1_view() == 0 {
        spawn_text(
            commands,
            stage,
            808.0,
            216.0,
            None,
            "Porters: A = Anke · B = Boro · C = Cato — toggle who carries each thing.",
            11.0,
            false,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
        for (index, item) in content::PACK_ITEMS.iter().enumerate() {
            let row = geom::c1_row(index);
            let mask = work.assignments.get(index).copied().unwrap_or(0);
            spawn_text(
                commands,
                stage,
                row.x + 4.0,
                row.y + 8.0,
                None,
                item.name,
                13.5,
                true,
                pal.ink,
                Justify::Left,
                fonts,
            );
            spawn_text_right(
                commands,
                stage,
                1090.0,
                row.y + 10.0,
                130.0,
                if item.copiable { "copiable" } else { "unique" },
                10.5,
                false,
                pal.ink_dim,
                fonts,
            );
            for porter in 0..3 {
                let held = mask & (1 << porter) != 0;
                let label = if held {
                    format!("[{}]", content::PORTERS[porter])
                } else {
                    content::PORTERS[porter].to_string()
                };
                spawn_button(
                    commands,
                    stage,
                    geom::holder_btn(index, porter),
                    &label,
                    pal,
                    held,
                    fonts,
                );
            }
        }
    } else {
        let overlap = puzzles::toll_overlap();
        let body = format!(
            "Two tally-keepers counted the vault's crates by name, and their counts \
             disagree.\n\nAnsel counted {}: {}.\n\nBrin counted {}: {}.\n\nSix names \
             stand on both lists: {}. A name counted twice is still one crate. \
             Reconcile the toll by name, then file it.",
            content::TOLL_KEEPER_A.len(),
            content::TOLL_KEEPER_A.join(", "),
            content::TOLL_KEEPER_B.len(),
            content::TOLL_KEEPER_B.join(", "),
            overlap.join(", ")
        );
        spawn_text(
            commands,
            stage,
            808.0,
            234.0,
            Some(432.0),
            &body,
            12.5,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
        for (index, count) in content::TOLL_CANDIDATES.iter().enumerate() {
            let chosen = work.toll_choice == Some(index);
            spawn_button(
                commands,
                stage,
                geom::toll_btn(index),
                &format!("{count} crates"),
                pal,
                chosen,
                fonts,
            );
        }
    }
    let progress = &game.expedition.chambers[0];
    spawn_text(
        commands,
        stage,
        808.0,
        508.0,
        None,
        &format!(
            "Packing: {} · Toll: {}",
            if work.packing_ok { "filed" } else { "unfiled" },
            if work.toll_ok { "filed" } else { "unfiled" }
        ),
        11.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    let _ = progress;
}

/// C2 — the Reading Room day-plan.
fn build_c2(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    let work = &game.expedition.chambers[1].work;
    spawn_text(
        commands,
        stage,
        808.0,
        204.0,
        None,
        "One chart. The annotator's pen must never share an hour with a reader.",
        11.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    for (index, person) in content::READERS.iter().enumerate() {
        let row = geom::c2_row(index);
        let pick = work.picks.get(index).copied().unwrap_or(0).min(1) as usize;
        let (start, end) = person.blocks[pick];
        let (other_start, other_end) = person.blocks[1 - pick];
        spawn_text(
            commands,
            stage,
            row.x + 4.0,
            row.y + 4.0,
            None,
            person.name,
            13.5,
            true,
            pal.ink,
            Justify::Left,
            fonts,
        );
        spawn_text(
            commands,
            stage,
            row.x + 4.0,
            row.y + 24.0,
            None,
            &format!("or {other_start}–{other_end}"),
            10.5,
            false,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
        if person.annotator {
            spawn_text_right(
                commands,
                stage,
                990.0,
                row.y + 6.0,
                90.0,
                "✎ writes",
                11.0,
                true,
                pal.accent,
                fonts,
            );
        }
        spawn_button(
            commands,
            stage,
            geom::pick_btn(index),
            &format!("{start}–{end}"),
            pal,
            false,
            fonts,
        );
    }
}

/// C3 — the Long Lease: brackets drawn, steppers below.
fn build_c3(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    let work = &game.expedition.chambers[2].work;
    let leases: [(i32, i32); 3] = [
        work.leases.first().map(|p| (p[0], p[1])).unwrap_or((0, 0)),
        work.leases.get(1).map(|p| (p[0], p[1])).unwrap_or((0, 0)),
        work.leases.get(2).map(|p| (p[0], p[1])).unwrap_or((0, 0)),
    ];
    let lines = puzzles::bracket_lines(&leases);
    let mut y = 210.0;
    for line in &lines {
        spawn_text_mono(commands, stage, 808.0, y, None, line, 12.5, pal.ink, fonts);
        y += 24.0;
    }
    spawn_text(
        commands,
        stage,
        936.0,
        332.0,
        None,
        "begins",
        10.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        1096.0,
        332.0,
        None,
        "ends",
        10.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    for (lease, pair) in leases.iter().enumerate() {
        let row_y = 356.0 + lease as f32 * 44.0;
        spawn_text(
            commands,
            stage,
            808.0,
            row_y + 7.0,
            None,
            &format!("Lease {}", lease + 1),
            13.0,
            true,
            pal.ink,
            Justify::Left,
            fonts,
        );
        for field in 0..2 {
            spawn_button(
                commands,
                stage,
                geom::lease_step(lease, field, -1),
                "−",
                pal,
                false,
                fonts,
            );
            spawn_button(
                commands,
                stage,
                geom::lease_step(lease, field, 1),
                "+",
                pal,
                false,
                fonts,
            );
        }
        spawn_text_centered(
            commands,
            stage,
            1004.0,
            row_y + 7.0,
            30.0,
            &format!("{}", pair.0),
            13.0,
            true,
            pal.ink,
            fonts,
        );
        spawn_text_centered(
            commands,
            stage,
            1162.0,
            row_y + 7.0,
            30.0,
            &format!("{}", pair.1),
            13.0,
            true,
            pal.ink,
            fonts,
        );
    }
    spawn_text(
        commands,
        stage,
        808.0,
        496.0,
        None,
        "Months, in steps of three. Each lease must nest inside the one before it.",
        11.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
}

/// C4 — the two slots (kept / restored).
fn build_c4_slots(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    use rects::*;
    let work = &game.expedition.chambers[3].work;
    let kind_name = |raw: u8| match raw {
        0 => "the kept record",
        1 => "the kept record (second room)",
        _ => "the restored record",
    };
    spawn_box(
        commands,
        stage,
        SLOT,
        Some(pal.card),
        Some((2.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        820.0,
        SLOT.y + 5.0,
        None,
        "WHAT THE BOOK KEPT",
        10.0,
        true,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        820.0,
        SLOT.y + 24.0,
        Some(416.0),
        match work.slot_kept {
            Some(record) => kind_name(work.records.get(record).copied().unwrap_or(0)),
            None => "— empty —",
        },
        12.5,
        false,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_box(
        commands,
        stage,
        SLOT2,
        Some(pal.card),
        Some((2.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        820.0,
        SLOT2.y + 3.0,
        None,
        "WHAT THE ROOM RESTORED",
        10.0,
        true,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        1000.0,
        SLOT2.y + 3.0,
        Some(240.0),
        match work.slot_restored {
            Some(record) => kind_name(work.records.get(record).copied().unwrap_or(0)),
            None => "— empty —",
        },
        12.0,
        false,
        pal.ink,
        Justify::Left,
        fonts,
    );
}

/// C4 — the delivery: run it, then exhibit the two records.
fn build_c4(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    let work = &game.expedition.chambers[3].work;
    let body = format!(
        "The delivery carries {} — committed in the book — and {}, arranged for \
         the room only. Run it through two rooms, then exhibit each record where \
         it belongs.",
        content::DELIVERY_COMMITTED,
        content::DELIVERY_TEMPORARY
    );
    spawn_text(
        commands,
        stage,
        808.0,
        240.0,
        Some(432.0),
        &body,
        12.5,
        false,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        geom::run_btn(),
        "Run the delivery",
        pal,
        false,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        geom::commit_btn(),
        if work.commit_second {
            "2nd room: committed"
        } else {
            "2nd room: uncommitted"
        },
        pal,
        work.commit_second,
        fonts,
    );
    let records = puzzles::run_delivery(work.commit_second);
    for (index, _) in work.records.iter().enumerate() {
        let rect = geom::record_card(index);
        let record = &records[index];
        let selected = game.selected_record() == Some(index);
        let exhibited = work.slot_kept == Some(index) || work.slot_restored == Some(index);
        let border = if selected {
            (3.0, pal.accent)
        } else if exhibited {
            (2.5, pal.good)
        } else {
            (1.5, pal.edge)
        };
        spawn_box(commands, stage, rect, Some(pal.card), Some(border), 0.0);
        let tag = match record.kind {
            puzzles::RecordKind::Kept => "RECORD — KEPT",
            puzzles::RecordKind::KeptSecond => "RECORD — KEPT (SECOND ROOM)",
            puzzles::RecordKind::Restored => "RECORD — RESTORED",
        };
        spawn_text(
            commands,
            stage,
            rect.x + 10.0,
            rect.y + 4.0,
            None,
            tag,
            10.0,
            true,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
        spawn_text(
            commands,
            stage,
            rect.x + 10.0,
            rect.y + 18.0,
            Some(420.0),
            record.text,
            11.0,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
        if exhibited {
            spawn_text_right(
                commands,
                stage,
                rect.x + rect.w - 10.0,
                rect.y + 4.0,
                110.0,
                "exhibited",
                10.5,
                true,
                pal.good,
                fonts,
            );
        }
    }
    if !work.ran {
        spawn_text(
            commands,
            stage,
            808.0,
            356.0,
            None,
            "No records yet — the delivery has not run.",
            12.0,
            false,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
    }
}

/// C5 — the Gauntlet Yard: ten placards, judged.
fn build_c5(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    let work = &game.expedition.chambers[4].work;
    let page = game.c5_page() as usize;
    let first = page * 4 + 1;
    let last = (page * 4 + 4).min(content::PLACARDS.len());
    spawn_text(
        commands,
        stage,
        808.0,
        204.0,
        None,
        &format!("Placards {first}–{last} of 10 — judge each by the rule, not its face."),
        11.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        geom::page_prev(),
        "< Prev",
        pal,
        false,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        geom::page_next(),
        "Next >",
        pal,
        false,
        fonts,
    );
    for local in 0..4 {
        let global = page * 4 + local;
        if global >= content::PLACARDS.len() {
            break;
        }
        let placard = &content::PLACARDS[global];
        let row = geom::c5_row(local);
        let marked = work.wrong_marks.contains(&global);
        spawn_box(
            commands,
            stage,
            row,
            Some(pal.card),
            Some(if marked {
                (2.5, pal.bad)
            } else {
                (1.5, pal.edge)
            }),
            0.0,
        );
        spawn_text(
            commands,
            stage,
            row.x + 10.0,
            row.y + 4.0,
            None,
            &format!("{}#{}", if marked { "× " } else { "" }, global + 1),
            12.0,
            true,
            if marked { pal.bad } else { pal.ink },
            Justify::Left,
            fonts,
        );
        let judgement = work.judgements.get(global).copied().flatten();
        let label = match judgement {
            None => "— judge —",
            Some(true) => "LEGAL",
            Some(false) => "BARRED",
        };
        spawn_button(
            commands,
            stage,
            geom::judge_btn(local),
            label,
            pal,
            judgement.is_some(),
            fonts,
        );
        spawn_text(
            commands,
            stage,
            row.x + 10.0,
            row.y + 28.0,
            Some(424.0),
            placard.text,
            11.0,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
    }
    let judged = work.judgements.iter().filter(|j| j.is_some()).count();
    spawn_text(
        commands,
        stage,
        808.0,
        496.0,
        None,
        &format!("Judged: {judged} of 10. Prove (X) faces the yard's gate."),
        11.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
}

// ---------------------------------------------------------------------------
// The FOE.
// ---------------------------------------------------------------------------

/// The Borrowed Hoard's chamber: the audit, the census, the gate.
fn build_foe(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    build_header(commands, game, pal, fonts, stage);
    build_room_base(
        commands,
        pal,
        fonts,
        stage,
        "The Borrowed Hoard — the audit chamber",
    );
    build_block(
        commands,
        pal,
        fonts,
        stage,
        geom::desk(),
        Color::srgb_u8(120, 84, 96),
        "HOARD-ENGINE",
        "the demonstration",
    );
    let stone = geom::stone();
    spawn_box(
        commands,
        stage,
        stone,
        Some(Color::srgb_u8(140, 140, 148)),
        Some((2.0, pal.ink_dim)),
        0.0,
    );
    spawn_text_centered(
        commands,
        stage,
        stone.cx(),
        stone.y + 12.0,
        stone.w,
        "THE",
        11.0,
        true,
        pal.ink,
        fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        stone.cx(),
        stone.y + 26.0,
        stone.w,
        "RULE",
        11.0,
        true,
        pal.ink,
        fonts,
    );
    build_panel_header(
        commands,
        pal,
        fonts,
        stage,
        "The Borrowed Hoard",
        "FOE of Floor 1 — audit, expose, replace, strike",
    );
    // The proof slot.
    use rects::*;
    spawn_box(
        commands,
        stage,
        SLOT,
        Some(pal.card),
        Some((2.0, pal.edge)),
        0.0,
    );
    let foe = &game.expedition.foe;
    match foe.state {
        FoeState::Fallen => {
            spawn_text_centered(
                commands,
                stage,
                SLOT.cx(),
                SLOT.y + 18.0,
                SLOT.w,
                "Fallen — the plan re-written, the chart in one column.",
                13.5,
                true,
                pal.good,
                fonts,
            );
        }
        _ if foe.exhibited => {
            spawn_text_centered(
                commands,
                stage,
                SLOT.cx(),
                SLOT.y + 10.0,
                SLOT.w,
                "Exhibited: the Heartstone Chart — both columns shown.",
                13.5,
                true,
                pal.good,
                fonts,
            );
            spawn_text_centered(
                commands,
                stage,
                SLOT.cx(),
                SLOT.y + 32.0,
                SLOT.w,
                if foe.state == FoeState::Exposed {
                    "The Hoard stands EXPOSED."
                } else {
                    "The census is not yet complete."
                },
                12.0,
                false,
                pal.good,
                fonts,
            );
        }
        _ => {
            spawn_text(
                commands,
                stage,
                820.0,
                SLOT.y + 6.0,
                Some(416.0),
                "The slot wants: an uncopiable thing promised twice, exhibited with both columns shown.",
                12.0,
                false,
                pal.ink_dim,
                Justify::Left,
                fonts,
            );
        }
    }
    spawn_button(
        commands,
        stage,
        geom::editor_open(),
        if foe.gate_passed {
            "The Hoard — gate passed"
        } else {
            "Write the Hoard"
        },
        pal,
        foe.gate_passed,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        808.0,
        202.0,
        None,
        if foe.gate_passed {
            "The gate: PASSED — the strike may be armed."
        } else if game.gate_running {
            "The gate: running…"
        } else {
            "The gate: not yet passed."
        },
        11.5,
        false,
        if foe.gate_passed {
            pal.good
        } else {
            pal.ink_dim
        },
        Justify::Left,
        fonts,
    );
    if !foe.enumerated {
        spawn_text(
            commands,
            stage,
            808.0,
            240.0,
            Some(432.0),
            "The great ledger is closed. Enumerate the master plan (N) and the census \
             becomes cards: verify every uncopiable row's holders, and exhibit the \
             row that lies.\n\nThe strike needs both proofs — the completed census, \
             and a Hoard of your own that passes the vault's gate.",
            13.0,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
    } else {
        let census = puzzles::foe_census();
        let copiable: Vec<&str> = census
            .rows
            .iter()
            .filter(|row| row.copiable)
            .map(|row| row.item.as_str())
            .collect();
        spawn_text(
            commands,
            stage,
            808.0,
            216.0,
            Some(432.0),
            &format!(
                "Copiable — copies need no verification: {}.",
                copiable.join(" · ")
            ),
            11.0,
            false,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
        let uncopiable = puzzles::foe_uncopiable_rows();
        for (pos, row_index) in uncopiable.iter().enumerate() {
            let row = &census.rows[*row_index];
            let rect = geom::foe_row(pos);
            let verified = foe.verified.get(*row_index).copied().unwrap_or(false);
            let border = if row.double_promised() {
                (2.5, pal.wax)
            } else {
                (1.5, pal.edge)
            };
            spawn_box(commands, stage, rect, Some(pal.card), Some(border), 0.0);
            spawn_text(
                commands,
                stage,
                rect.x + 10.0,
                rect.y + 6.0,
                None,
                &row.item,
                13.0,
                true,
                pal.ink,
                Justify::Left,
                fonts,
            );
            spawn_text_right(
                commands,
                stage,
                1146.0,
                rect.y + 7.0,
                300.0,
                &format!(
                    "{} holder{}: {}",
                    row.holders.len(),
                    if row.holders.len() == 1 { "" } else { "s" },
                    row.holders.join(" + ")
                ),
                11.0,
                false,
                pal.ink_dim,
                fonts,
            );
            if verified {
                spawn_text_right(
                    commands,
                    stage,
                    rect.x + rect.w - 10.0,
                    rect.y + 6.0,
                    110.0,
                    "verified",
                    11.5,
                    true,
                    pal.good,
                    fonts,
                );
            } else {
                spawn_button(
                    commands,
                    stage,
                    geom::verify_btn(pos),
                    "Verify",
                    pal,
                    false,
                    fonts,
                );
            }
        }
    }
    let (active, caption) = match foe.state {
        FoeState::Fallen => (false, "Fallen — the chart is drawn with its date."),
        FoeState::Exposed => {
            if foe.gate_passed {
                (true, "The seal wakes — press it to strike.")
            } else {
                (false, "Exposed — but the Hoard first: pass the gate.")
            }
        }
        FoeState::Standing => (false, "The seal sleeps until the proof is complete."),
    };
    build_seal(commands, game, pal, fonts, stage, active, caption);
    build_bottom(commands, game, pal, fonts, stage);
}

// ---------------------------------------------------------------------------
// Overlays.
// ---------------------------------------------------------------------------

/// The reading overlay (inscriptions, demonstrations, hints,
/// the stairs gate, the floor intro, the master plan).
fn build_reading(
    commands: &mut Commands,
    game: &Game,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
    kind: ReadingKind,
) {
    use rects::*;
    spawn_box(
        commands,
        stage,
        Rect::new(0.0, 0.0, DESIGN_W, DESIGN_H),
        Some(pal.dim),
        None,
        0.0,
    );
    spawn_box(
        commands,
        stage,
        READING,
        Some(pal.panel),
        Some((3.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        262.0,
        106.0,
        None,
        &game.reading_title(kind),
        20.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        READING_SCALE,
        &format!("Text: {}", game.expedition.settings.text_scale.label()),
        pal,
        false,
        fonts,
    );
    spawn_button(commands, stage, OVERLAY_CLOSE, "Close", pal, false, fonts);
    let factor = game.expedition.settings.text_scale.factor();
    let body = game.reading_text(kind);
    if kind == ReadingKind::MasterPlan {
        spawn_text_mono(
            commands,
            stage,
            262.0,
            168.0,
            Some(756.0),
            &body,
            13.5,
            pal.ink,
            fonts,
        );
        spawn_text(
            commands,
            stage,
            262.0,
            660.0,
            None,
            "Read it, then enumerate it (N) — the census turns these lines into countable cards.",
            12.5,
            false,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
    } else {
        spawn_text(
            commands,
            stage,
            262.0,
            168.0,
            Some(756.0),
            &body,
            16.0 * factor,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
    }
    if kind == ReadingKind::Hint {
        if game.hint_showing < 4 {
            spawn_button(
                commands,
                stage,
                HINT_DEEPER,
                &format!("Climb a rung deeper (rung {})", game.hint_showing + 1),
                pal,
                false,
                fonts,
            );
        }
        spawn_text(
            commands,
            stage,
            262.0,
            620.0,
            Some(700.0),
            "The ladder is free. Rungs 1–3 cost nothing; rung 4 names the seam and forfeits this challenge's facet.",
            12.0,
            false,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
    }
    if kind == ReadingKind::Inscription {
        spawn_text(
            commands,
            stage,
            262.0,
            660.0,
            None,
            "The desk shows the mechanism working — click it for the demonstration.",
            12.5,
            false,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
    }
}

/// One map square.
#[allow(clippy::too_many_arguments)]
fn build_square(
    commands: &mut Commands,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
    rect: Rect,
    number: usize,
    domain: &str,
    class: &str,
    status: &str,
    filled: bool,
) {
    spawn_box(
        commands,
        stage,
        rect,
        Some(if filled { pal.floor_b } else { pal.panel }),
        Some((2.0, pal.edge)),
        0.0,
    );
    if filled {
        spawn_box(
            commands,
            stage,
            Rect::new(rect.x + 5.0, rect.y + 5.0, rect.w - 10.0, rect.h - 10.0),
            None,
            Some((2.0, pal.ink)),
            0.0,
        );
    }
    spawn_text(
        commands,
        stage,
        rect.x + 12.0,
        rect.y + 8.0,
        None,
        &format!("{number}"),
        16.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        rect.x + 40.0,
        rect.y + 10.0,
        Some(rect.w - 52.0),
        domain,
        11.5,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        rect.x + 40.0,
        rect.y + 40.0,
        None,
        class,
        10.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        rect.x + 12.0,
        rect.y + rect.h - 22.0,
        Some(rect.w - 24.0),
        status,
        11.0,
        true,
        if filled { pal.good } else { pal.ink_dim },
        Justify::Left,
        fonts,
    );
}

/// The map overlay: the Ferronia chart. A cell fills only on a
/// verified gate; everything else stands honestly uncharted.
fn build_map(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    use rects::*;
    spawn_box(
        commands,
        stage,
        Rect::new(0.0, 0.0, DESIGN_W, DESIGN_H),
        Some(pal.dim),
        None,
        0.0,
    );
    spawn_box(
        commands,
        stage,
        MAP_PANEL,
        Some(pal.panel),
        Some((3.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        48.0,
        88.0,
        None,
        "The Ferronia Chart — seventeen floors",
        22.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        48.0,
        122.0,
        None,
        "A square fills only on proof seen. Blank parchment is honest: it claims nothing.",
        13.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    spawn_button(commands, stage, MAP_CLOSE, "Close", pal, false, fonts);
    let repaired = game.expedition.chambers_repaired();
    let floor1_status = match game.expedition.foe.state {
        FoeState::Fallen => format!(
            "FOE FALLEN — {}",
            game.expedition.foe.fallen_date.clone().unwrap_or_default()
        ),
        _ if repaired > 0
            || game
                .expedition
                .chambers
                .iter()
                .any(|c| c.state == ChamberState::Diagnosed) =>
        {
            format!("CHARTING ({repaired}/5)")
        }
        _ => "UNCHARTED".to_string(),
    };
    let mut y = 156.0;
    for (name, first, count) in content::STRATA {
        spawn_text(
            commands,
            stage,
            48.0,
            y,
            None,
            name,
            13.0,
            true,
            pal.ink,
            Justify::Left,
            fonts,
        );
        y += 24.0;
        for offset in 0..count {
            let index = first + offset;
            let floor = &content::FLOOR_CHART[index];
            let rect = Rect::new(48.0 + offset as f32 * 236.0, y, 224.0, 72.0);
            if index == 0 {
                build_square(
                    commands,
                    pal,
                    fonts,
                    stage,
                    rect,
                    1,
                    floor.domain,
                    floor.class,
                    &floor1_status,
                    game.expedition.foe.state == FoeState::Fallen,
                );
            } else {
                build_square(
                    commands,
                    pal,
                    fonts,
                    stage,
                    rect,
                    index + 1,
                    floor.domain,
                    floor.class,
                    "UNCHARTED",
                    false,
                );
            }
        }
        y += 88.0;
    }
    let niflhel = Rect::new(48.0, y + 4.0, 1184.0, 48.0);
    spawn_box(
        commands,
        stage,
        niflhel,
        Some(pal.card),
        Some((2.5, pal.wax)),
        0.0,
    );
    spawn_text_centered(
        commands,
        stage,
        niflhel.cx(),
        niflhel.y + 15.0,
        niflhel.w,
        "NIFLHEL — SEALED · unseals when all four Stratum IV FOEs have fallen",
        13.0,
        true,
        pal.ink,
        fonts,
    );
    spawn_text(
        commands,
        stage,
        48.0,
        y + 62.0,
        Some(1180.0),
        "Stairs to Stratum II: FOE 1 fallen, plus FOE 2 or FOE 3, with every Stratum I boss crate clippy-clean. Only Floor 1 is built in this build; its square fills on the verified gate, with the date inscribed.",
        12.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
}

/// The expedition log overlay.
fn build_log(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    spawn_box(
        commands,
        stage,
        Rect::new(0.0, 0.0, DESIGN_W, DESIGN_H),
        Some(pal.dim),
        None,
        0.0,
    );
    let panel = Rect::new(230.0, 84.0, 820.0, 632.0);
    spawn_box(
        commands,
        stage,
        panel,
        Some(pal.panel),
        Some((3.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        262.0,
        106.0,
        None,
        "The Expedition Log",
        20.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        rects::OVERLAY_CLOSE,
        "Close",
        pal,
        false,
        fonts,
    );
    let factor = game.expedition.settings.text_scale.factor();
    let lines: Vec<&String> = game.expedition.log.iter().rev().take(16).collect();
    let mut y = 160.0;
    for line in lines.iter().rev() {
        let shown = if line.chars().count() > 92 {
            let cut: String = line.chars().take(91).collect();
            format!("{cut}…")
        } else {
            (*line).clone()
        };
        spawn_text(
            commands,
            stage,
            262.0,
            y,
            None,
            &shown,
            13.0 * factor,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
        y += 24.0 * factor;
    }
    if game.expedition.log.is_empty() {
        spawn_text(
            commands,
            stage,
            262.0,
            y,
            None,
            "Nothing is written yet.",
            14.0,
            false,
            pal.ink_dim,
            Justify::Left,
            fonts,
        );
    }
}

/// The pause & settings overlay.
fn build_pause(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    use rects::*;
    spawn_box(
        commands,
        stage,
        Rect::new(0.0, 0.0, DESIGN_W, DESIGN_H),
        Some(pal.dim),
        None,
        0.0,
    );
    spawn_box(
        commands,
        stage,
        PAUSE_PANEL,
        Some(pal.panel),
        Some((3.0, pal.edge)),
        0.0,
    );
    spawn_text_centered(
        commands,
        stage,
        640.0,
        138.0,
        500.0,
        "Pause & Settings",
        22.0,
        true,
        pal.ink,
        fonts,
    );
    let settings = &game.expedition.settings;
    let labels = [
        format!("Text size: {}", settings.text_scale.label()),
        format!(
            "High contrast: {}",
            if settings.high_contrast { "On" } else { "Off" }
        ),
        format!(
            "Reduced motion: {}",
            if settings.reduced_motion { "On" } else { "Off" }
        ),
        "Export the save file".to_string(),
        "Import a save file".to_string(),
        "Resume".to_string(),
    ];
    for (index, label) in labels.iter().enumerate() {
        spawn_button(
            commands,
            stage,
            rects::setting(index),
            label,
            pal,
            false,
            fonts,
        );
    }
    spawn_text_centered(
        commands,
        stage,
        640.0,
        560.0,
        500.0,
        "Settings travel with the expedition save.",
        12.0,
        false,
        pal.ink_dim,
        fonts,
    );
    spawn_text_centered(
        commands,
        stage,
        640.0,
        584.0,
        500.0,
        "Reduced motion is offered here and at the title — demonstrations step, never flash.",
        11.5,
        false,
        pal.ink_dim,
        fonts,
    );
}

/// The codex overlay: principle stones, facets, and the pack.
fn build_codex(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    spawn_box(
        commands,
        stage,
        Rect::new(0.0, 0.0, DESIGN_W, DESIGN_H),
        Some(pal.dim),
        None,
        0.0,
    );
    spawn_box(
        commands,
        stage,
        rects::READING,
        Some(pal.panel),
        Some((3.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        262.0,
        106.0,
        None,
        "The Principle Codex & the Pack",
        20.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        rects::OVERLAY_CLOSE,
        "Close",
        pal,
        false,
        fonts,
    );
    let mut y = 160.0;
    spawn_text(
        commands,
        stage,
        262.0,
        y,
        None,
        "PRINCIPLE STONES",
        13.0,
        true,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    y += 28.0;
    if game.expedition.codex.is_empty() {
        spawn_text(
            commands,
            stage,
            262.0,
            y,
            None,
            "No stone yet. A floor's stone is its FOE's to give.",
            14.0,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
        y += 26.0;
    } else {
        for stone in &game.expedition.codex {
            spawn_text(
                commands,
                stage,
                262.0,
                y,
                None,
                stone,
                16.0,
                true,
                pal.ink,
                Justify::Left,
                fonts,
            );
            y += 24.0;
            if stone == content::PLANK_STONE {
                spawn_text(
                    commands,
                    stage,
                    262.0,
                    y,
                    None,
                    content::PLANK_STONE_TEXT,
                    14.0,
                    false,
                    pal.good,
                    Justify::Left,
                    fonts,
                );
                y += 28.0;
            }
        }
    }
    y += 12.0;
    spawn_text(
        commands,
        stage,
        262.0,
        y,
        None,
        "FACETS — chambers repaired at full standard",
        13.0,
        true,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    y += 28.0;
    let facets: Vec<String> = game
        .expedition
        .chambers
        .iter()
        .enumerate()
        .filter(|(_, chamber)| chamber.facet)
        .map(|(index, _)| CHAMBERS[index].name.to_string())
        .collect();
    let facets_text: String = if facets.is_empty() {
        "None yet — a facet is a chamber repaired without ever naming the seam (hint rung 4)."
            .to_string()
    } else {
        facets.join(" · ")
    };
    spawn_text(
        commands,
        stage,
        262.0,
        y,
        Some(756.0),
        &facets_text,
        14.0,
        false,
        pal.ink,
        Justify::Left,
        fonts,
    );
    y += 52.0;
    spawn_text(
        commands,
        stage,
        262.0,
        y,
        None,
        "THE PACK — loot, wielded by relevance",
        13.0,
        true,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    y += 28.0;
    if game.expedition.pack.is_empty() {
        spawn_text(
            commands,
            stage,
            262.0,
            y,
            None,
            "The pack is empty. Loot is a FOE's other gift.",
            14.0,
            false,
            pal.ink,
            Justify::Left,
            fonts,
        );
    } else {
        for item in &game.expedition.pack {
            spawn_text(
                commands,
                stage,
                262.0,
                y,
                None,
                item,
                16.0,
                true,
                pal.ink,
                Justify::Left,
                fonts,
            );
            y += 24.0;
            if item == content::INTERNERS_HOARD {
                spawn_text(
                    commands,
                    stage,
                    262.0,
                    y,
                    Some(756.0),
                    content::INTERNERS_HOARD_TEXT,
                    13.0,
                    false,
                    pal.ink_dim,
                    Justify::Left,
                    fonts,
                );
                y += 44.0;
            }
        }
    }
}

/// The editor overlay: the Interner's Hoard draft, in mono, with
/// its line gutter, cursor block, and gate controls.
fn build_editor(commands: &mut Commands, game: &Game, pal: &Pal, fonts: &Fonts, stage: Entity) {
    use crate::game::{EDITOR_CHAR_W, EDITOR_LINE_H, EDITOR_PAD_X, EDITOR_VISIBLE_LINES};
    use rects::*;
    spawn_box(
        commands,
        stage,
        Rect::new(0.0, 0.0, DESIGN_W, DESIGN_H),
        Some(pal.dim),
        None,
        0.0,
    );
    spawn_box(
        commands,
        stage,
        EDITOR_PANEL,
        Some(pal.panel),
        Some((3.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        64.0,
        62.0,
        None,
        "The Interner's Hoard — in your own hand",
        20.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    let verdict = if game.gate_running {
        "The gate is running: cargo test first, then the clippy lanterns…".to_string()
    } else if let Some(record) = &game.expedition.gate_record {
        record.clone()
    } else {
        "No verdict yet. The scaffold stands unfinished — write the Hoard, then run the gate."
            .to_string()
    };
    spawn_text(
        commands,
        stage,
        64.0,
        92.0,
        Some(900.0),
        &verdict,
        12.5,
        false,
        pal.ink_dim,
        Justify::Left,
        fonts,
    );
    let (cursor_line, cursor_col) = game.editor_line_col();
    spawn_text_right(
        commands,
        stage,
        1216.0,
        92.0,
        240.0,
        &format!("line {} of {}", cursor_line + 1, game.editor_line_count()),
        12.0,
        false,
        pal.ink_dim,
        fonts,
    );
    spawn_box(
        commands,
        stage,
        EDITOR_AREA,
        Some(pal.card),
        Some((2.0, pal.edge)),
        0.0,
    );
    let text = &game.expedition.editor_code;
    let lines: Vec<&str> = text.lines().collect();
    let scroll = game.editor_scroll;
    // Selection range, as (start, end) byte offsets, if any.
    let selection = game.editor_anchor.map(|anchor| {
        if anchor <= game.editor_cursor {
            (anchor, game.editor_cursor)
        } else {
            (game.editor_cursor, anchor)
        }
    });
    // Byte offset of each visible line's start.
    let mut line_starts: Vec<usize> = Vec::with_capacity(lines.len() + 1);
    let mut offset = 0usize;
    for line in &lines {
        line_starts.push(offset);
        offset += line.len() + 1;
    }
    for visible in 0..EDITOR_VISIBLE_LINES {
        let line_index = scroll + visible;
        let Some(line) = lines.get(line_index) else {
            break;
        };
        let y = EDITOR_AREA.y + 4.0 + visible as f32 * EDITOR_LINE_H;
        spawn_text_right(
            commands,
            stage,
            EDITOR_AREA.x + 40.0,
            y + 1.0,
            34.0,
            &format!("{}", line_index + 1),
            11.0,
            false,
            pal.ink_dim,
            fonts,
        );
        let shown: String = line.chars().take(128).collect();
        spawn_text_mono(
            commands,
            stage,
            EDITOR_AREA.x + EDITOR_PAD_X,
            y,
            None,
            &shown,
            14.0,
            pal.ink,
            fonts,
        );
        // Selection wash for this line.
        if let Some((sel_start, sel_end)) = selection {
            let line_start = line_starts[line_index];
            let line_end = line_start + line.len();
            if sel_start < line_end && sel_end > line_start {
                let from = sel_start.max(line_start) - line_start;
                let to = sel_end.min(line_end) - line_start;
                let from_chars = line[..from].chars().count();
                let to_chars = line[..to].chars().count();
                spawn_box(
                    commands,
                    stage,
                    Rect::new(
                        EDITOR_AREA.x + EDITOR_PAD_X + from_chars as f32 * EDITOR_CHAR_W,
                        y,
                        (to_chars - from_chars) as f32 * EDITOR_CHAR_W,
                        EDITOR_LINE_H,
                    ),
                    Some(pal.accent.with_alpha(0.30)),
                    None,
                    0.0,
                );
            }
        }
    }
    // The cursor: a still block outline (never a blink — §6.7).
    if selection.is_none() && cursor_line >= scroll && cursor_line < scroll + EDITOR_VISIBLE_LINES {
        let y = EDITOR_AREA.y + 4.0 + (cursor_line - scroll) as f32 * EDITOR_LINE_H;
        spawn_box(
            commands,
            stage,
            Rect::new(
                EDITOR_AREA.x + EDITOR_PAD_X + cursor_col as f32 * EDITOR_CHAR_W - 1.0,
                y,
                EDITOR_CHAR_W + 2.0,
                EDITOR_LINE_H,
            ),
            None,
            Some((2.0, pal.accent)),
            0.0,
        );
    }
    spawn_button(
        commands,
        stage,
        RUN_GATE,
        if game.gate_running {
            "The gate is running…"
        } else {
            "Run the gate"
        },
        pal,
        false,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        RESET_PROOF,
        "Reset the proof",
        pal,
        false,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        GATE_RECORD_BTN,
        "The gate record",
        pal,
        false,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        EDITOR_CLOSE,
        "Close — draft kept",
        pal,
        false,
        fonts,
    );
}

/// The gate record overlay: the last verdict, in full.
fn build_gate_record(
    commands: &mut Commands,
    game: &Game,
    pal: &Pal,
    fonts: &Fonts,
    stage: Entity,
) {
    spawn_box(
        commands,
        stage,
        Rect::new(0.0, 0.0, DESIGN_W, DESIGN_H),
        Some(pal.dim),
        None,
        0.0,
    );
    spawn_box(
        commands,
        stage,
        rects::READING,
        Some(pal.panel),
        Some((3.0, pal.edge)),
        0.0,
    );
    spawn_text(
        commands,
        stage,
        262.0,
        106.0,
        None,
        "The Gate Record",
        20.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    spawn_button(
        commands,
        stage,
        rects::OVERLAY_CLOSE,
        "Close",
        pal,
        false,
        fonts,
    );
    let summary = game
        .expedition
        .gate_record
        .clone()
        .unwrap_or_else(|| "No gate has judged a Hoard yet.".to_string());
    spawn_text(
        commands,
        stage,
        262.0,
        160.0,
        Some(756.0),
        &summary,
        15.0,
        true,
        pal.ink,
        Justify::Left,
        fonts,
    );
    let detail = if game.gate_detail.is_empty() {
        "No gate has run this session, so there is no working to show. Run the gate \
         from the editor and the record will hold the compiler's and the suite's \
         own words."
            .to_string()
    } else {
        game.gate_detail.clone()
    };
    let detail: String = detail.chars().take(2400).collect();
    spawn_text_mono(
        commands,
        stage,
        262.0,
        230.0,
        Some(756.0),
        &detail,
        11.5,
        pal.ink,
        fonts,
    );
}
