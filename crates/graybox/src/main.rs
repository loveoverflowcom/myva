//! Bản native của graybox Bevy: `cargo run -p myva-graybox`.
//!
//! Hai chế độ: đánh boss Kẻ Giữ Đập (mặc định) và đấu tập với bot B0 (`--duel`). Phím theo
//! combat.md §12; thêm Enter đấu lại, M đổi chế độ, B bật/tắt bot đấu tập, H hitbox. Riêng bản
//! native: F9 lưu replay vào thư mục làm việc, Esc thoát.
//!
//! `--demo` để bot lái người chơi (B1 khi đánh boss, B0 khi đấu tập);
//! `--screenshot <file.png>` chạy demo tới lúc boss đang báo đòn, lưu ảnh màn hình cùng replay
//! rồi thoát.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use myva_graybox::{CombatLog, GrayboxPlugin, Mode, Session};
use myva_sim::Phase;

/// Đợi qua nhịp mở màn và vài đòn để ảnh có HUD, log và vùng báo.
const SCREENSHOT_AFTER_TICK: u32 = 900;
/// Số khung chờ ảnh ghi xong trước khi thoát.
const EXIT_AFTER_FRAMES: u32 = 30;

#[derive(Resource)]
struct Capture {
    path: String,
    taken_at: Option<u32>,
    frames: u32,
}

fn main() -> AppExit {
    let mut args = std::env::args().skip(1);
    let mut mode = Mode::Boss;
    let mut demo = false;
    let mut capture = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--duel" => mode = Mode::Duel,
            "--demo" => demo = true,
            "--screenshot" => match args.next() {
                Some(path) => {
                    capture = Some(Capture {
                        path,
                        taken_at: None,
                        frames: 0,
                    });
                    demo = true;
                }
                None => eprintln!("--screenshot cần đường dẫn file .png"),
            },
            other => eprintln!("bỏ qua tham số lạ: {other}"),
        }
    }

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MyVa — Thần Mạch · graybox Bevy".into(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(GrayboxPlugin {
        mode,
        autopilot: demo,
    })
    .add_systems(Update, native_keys);
    if let Some(capture) = capture {
        app.insert_resource(capture)
            .add_systems(Update, take_screenshot);
    }
    app.run()
}

fn save_replay(session: &Session, log: &mut CombatLog) -> Option<String> {
    let battle = &session.battle;
    let path = format!(
        "graybox-r{}-t{}.myva-replay",
        battle.round(),
        battle.world().tick()
    );
    match std::fs::write(&path, battle.replay().to_text()) {
        Ok(()) => {
            log.note(format!("Da luu replay: {path}"));
            Some(path)
        }
        Err(error) => {
            log.note(format!("Khong luu duoc replay: {error}"));
            None
        }
    }
}

fn native_keys(
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<Session>,
    mut log: ResMut<CombatLog>,
    mut exit: MessageWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::F9) {
        save_replay(&session, &mut log);
    }
}

fn take_screenshot(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    session: Res<Session>,
    mut log: ResMut<CombatLog>,
    mut exit: MessageWriter<AppExit>,
) {
    let battle = &session.battle;
    match capture.taken_at {
        None => {
            let boss = battle.world().fighter(battle.rival());
            let winding_up = matches!(boss.action(), Some((_, _, Phase::Startup)));
            if battle.world().tick() >= SCREENSHOT_AFTER_TICK && winding_up {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(capture.path.clone()));
                if let Some(path) = save_replay(&session, &mut log) {
                    println!("ảnh: {}; replay: {path}", capture.path);
                }
                capture.taken_at = Some(battle.world().tick());
            }
        }
        Some(_) => {
            capture.frames += 1;
            if capture.frames >= EXIT_AFTER_FRAMES {
                exit.write(AppExit::Success);
            }
        }
    }
}
