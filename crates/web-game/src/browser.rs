use std::cell::{Cell, RefCell};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitSettings;
use myva_gameplay::SimSession;
use myva_graybox::telemetry::{self, HostStatus};
use myva_graybox::touch::{self, ShowTouchControls, TouchUi};
use myva_graybox::{ArenaView, GraySet, GrayboxPlugin, HostCommand, Match};
use wasm_bindgen::prelude::*;

/// Hộp thư giữa JS và Bevy. JS chỉ xếp lệnh và đọc chuỗi đã chuẩn bị; mọi thay đổi trận xảy ra
/// trong schedule của Bevy ở khung hình kế tiếp.
#[derive(Default)]
struct Bridge {
    commands: Vec<HostCommand>,
    show_touch: Option<bool>,
    telemetry: String,
    touch_layout: String,
    replay_requested: bool,
    replay: Option<String>,
}

thread_local! {
    static BRIDGE: RefCell<Bridge> = RefCell::new(Bridge::default());
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

fn queue(command: HostCommand) {
    BRIDGE.with_borrow_mut(|bridge| bridge.commands.push(command));
}

/// Start exactly once per iframe. The iframe owns teardown of the WASM runtime.
#[wasm_bindgen]
pub fn start_game() {
    if STARTED.with(|started| started.replace(true)) {
        return;
    }

    App::new()
        .insert_resource(WinitSettings::continuous())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MyVa · trận graybox".into(),
                canvas: Some("#game-canvas".into()),
                resolution: (1280, 720).into(),
                fit_canvas_to_parent: true,
                // Tab, phím tắt trình duyệt và IME của shell vẫn hoạt động.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GrayboxPlugin::default())
        .add_systems(RunFixedMainLoop, pull_bridge.in_set(GraySet::Shortcuts))
        .add_systems(Last, push_bridge)
        .run();
}

#[wasm_bindgen]
pub fn set_paused(paused: bool) {
    queue(HostCommand::SetPaused(paused));
}

#[wasm_bindgen]
pub fn rematch() {
    queue(HostCommand::Rematch);
}

#[wasm_bindgen]
pub fn switch_mode() {
    queue(HostCommand::SwitchMode);
}

#[wasm_bindgen]
pub fn toggle_hitboxes() {
    queue(HostCommand::ToggleHitboxes);
}

/// Bot B1 lái người chơi để xem mẫu đánh boss; input tay bị bỏ qua khi bật.
#[wasm_bindgen]
pub fn set_autopilot(on: bool) {
    queue(HostCommand::SetAutopilot(on));
}

#[wasm_bindgen]
pub fn show_touch_controls(visible: bool) {
    BRIDGE.with_borrow_mut(|bridge| bridge.show_touch = Some(visible));
}

/// JSON ASCII một dòng; xem `myva_graybox::telemetry`.
#[wasm_bindgen]
pub fn telemetry() -> String {
    BRIDGE.with_borrow(|bridge| bridge.telemetry.clone())
}

/// Tọa độ nút cảm ứng theo px CSS của canvas.
#[wasm_bindgen]
pub fn touch_layout() -> String {
    BRIDGE.with_borrow(|bridge| bridge.touch_layout.clone())
}

/// Yêu cầu replay của trận hiện tại; lấy bằng `take_replay` sau một khung hình.
#[wasm_bindgen]
pub fn request_replay() {
    BRIDGE.with_borrow_mut(|bridge| bridge.replay_requested = true);
}

#[wasm_bindgen]
pub fn take_replay() -> Option<String> {
    BRIDGE.with_borrow_mut(|bridge| bridge.replay.take())
}

fn pull_bridge(
    mut commands: MessageWriter<HostCommand>,
    mut show_touch: MessageWriter<ShowTouchControls>,
) {
    BRIDGE.with_borrow_mut(|bridge| {
        commands.write_batch(bridge.commands.drain(..));
        if let Some(visible) = bridge.show_touch.take() {
            show_touch.write(ShowTouchControls(visible));
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn push_bridge(
    game: Res<Match>,
    arena: Res<ArenaView>,
    sim: Res<SimSession>,
    touch_ui: Res<TouchUi>,
    time: Res<Time<Virtual>>,
    gamepads: Query<(), With<Gamepad>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut updates: Local<u64>,
) {
    *updates = updates.saturating_add(1);
    let status = HostStatus {
        ready: true,
        paused: time.is_paused(),
        updates: *updates,
        touch: touch_ui.visible,
        gamepads: gamepads.iter().count(),
    };
    let json = telemetry::json(&game, &arena, status);
    let layout = touch::layout(window.size()).to_json();
    BRIDGE.with_borrow_mut(|bridge| {
        bridge.telemetry = json;
        bridge.touch_layout = layout;
        if std::mem::take(&mut bridge.replay_requested) {
            bridge.replay = Some(sim.get().replay().to_text());
        }
    });
}
