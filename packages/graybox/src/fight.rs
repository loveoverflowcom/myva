//! Trận trong client. `GameplayPlugin` (D04) chạy tick cố định 60 Hz trên `Session` của lõi;
//! module này gom input thiết bị vào `LocalInput`, cho bot lái hộ người chơi, làm trọng tài sau
//! mỗi tick (kết quả, log, kiểm chứng replay khi trận lắng) và áp lệnh host như đấu lại.
//!
//! Thứ tự mỗi khung hình:
//! 1. `RunFixedMainLoop` trước vòng fixed: phím tắt → lệnh host → thiết bị → `LocalInput`.
//! 2. `FixedUpdate`, 0..n lần theo thời gian đã trôi: bot lái hộ (nếu bật) →
//!    `GameplaySet::{Input, Simulate, Mirror, Events}`; trọng tài chạy trong `Events`.
//! 3. Sau vòng fixed: [`ArenaView`] đọc lại component mirror cho cảnh, HUD và host.
//!
//! Tốc độ luật không phụ thuộc FPS; khung hình chậm chỉ làm nhiều tick chạy dồn trong một khung.
//! Client không có `&mut World`: mọi thay đổi trận đi qua lệnh và `Session::step`.

use std::collections::VecDeque;

use bevy::prelude::*;
use myva_gameplay::components::Projectile;
use myva_gameplay::view::{self, FighterMirror, ProjectileMirror};
use myva_gameplay::{GameplaySet, LastTick, LoadSession, LocalInput, SimSession, session_running};
use myva_sim::battle::{Battle, Bout, Mode, PLAYER_EPOCH, SparringBot, Verification};
use myva_sim::boss::BossPhase;
use myva_sim::protocol::CommandFrame;
use myva_sim::snapshot::{FighterView, ProjectileView};
use myva_sim::tick::Tick;
use myva_sim::{Buttons, Event, FighterId, InputFrame};

use crate::text::ascii;

/// Số dòng log HUD giữ lại.
const LOG_LINES: usize = 6;

/// Trận đang chơi phía client: trạng thái trận ngoài phiên và tùy chọn hiển thị. Phiên nằm trong
/// `SimSession`; chỉ lệnh host thay `bout`.
#[derive(Resource)]
pub struct Match {
    pub bout: Bout,
    pub show_boxes: bool,
    /// Kết quả chạy lại replay của trận, tính một lần khi trận lắng.
    pub verification: Option<Result<Verification, String>>,
    /// Số tick có cú bấm của người chơi vào mô phỏng; host dùng để đo trễ input → tick.
    pub applied_presses: u32,
}

impl Match {
    pub fn new(bout: Bout) -> Self {
        Self {
            bout,
            show_boxes: false,
            verification: None,
            applied_presses: 0,
        }
    }

    /// Tên ngắn trên HUD và log.
    pub fn name(&self, id: FighterId) -> &'static str {
        match (id == self.bout.player(), self.bout.mode()) {
            (true, _) => "P1",
            (false, Mode::Boss) => "Boss",
            (false, Mode::Duel) => "Bot",
        }
    }
}

/// Đưa trận `bout` với phiên của nó vào app: phiên cho `GameplayPlugin`, `LocalInput` cho người
/// chơi, `Match` cho phần còn lại của client.
pub fn insert(app: &mut App, bout: Bout, session: myva_sim::Session) {
    app.insert_resource(SimSession::new(session))
        .insert_resource(LocalInput::new(bout.player(), PLAYER_EPOCH, None))
        .insert_resource(Match::new(bout));
}

/// Ý định thiết bị trong khung hình hiện tại: mọi thiết bị cộng dồn hướng, gộp nút giữ và nút
/// vừa nhấn, rồi chuyển một lần vào `LocalInput`, nơi cú bấm được chốt tới tick kế tiếp.
#[derive(Resource, Default, Debug)]
pub struct Intent {
    pub move_x: i32,
    pub held: Buttons,
    pub pressed: Buttons,
}

impl Intent {
    pub fn add(&mut self, move_x: i8, held: Buttons, pressed: Buttons) {
        self.move_x += i32::from(move_x);
        self.held |= held;
        self.pressed |= pressed;
    }

    /// Lệnh tương đương: hướng, đỡ và một hành động theo ưu tiên của lõi.
    pub fn command(&self) -> CommandFrame {
        CommandFrame::from_input(&InputFrame {
            seq: 0,
            move_x: self.move_x.signum() as i8,
            held: self.held,
            pressed: self.pressed,
        })
    }
}

/// Lệnh từ host hoặc phím tắt; không đi qua mô phỏng và không ghi vào replay.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostCommand {
    Rematch,
    SwitchMode,
    ToggleHitboxes,
    ToggleSparringBot,
    SetAutopilot(bool),
    SetPaused(bool),
}

/// Log HUD của các tick gần đây.
#[derive(Resource, Default)]
pub struct CombatLog {
    pub lines: VecDeque<String>,
}

impl CombatLog {
    pub fn note(&mut self, line: String) {
        self.lines.push_back(line);
        while self.lines.len() > LOG_LINES {
            self.lines.pop_front();
        }
    }
}

/// Những gì cảnh, HUD và host cần trong một khung hình, đọc từ component mirror sau vòng fixed.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct ArenaView {
    /// Theo `FighterId` tăng dần.
    pub fighters: Vec<FighterView>,
    /// Theo `ProjectileId` tăng dần.
    pub projectiles: Vec<ProjectileView>,
    /// Số tick đã chạy.
    pub tick: Tick,
    pub hash: u64,
    pub boss_phase: Option<BossPhase>,
    /// `Some(bật)` ở chế độ đấu tập.
    pub sparring: Option<bool>,
}

impl ArenaView {
    pub fn fighter(&self, id: FighterId) -> Option<&FighterView> {
        self.fighters.get(usize::from(id.0)).filter(|f| f.id == id)
    }

    /// Cùng khung nhìn dựng thẳng từ snapshot của lõi, cho test và công cụ không có ECS.
    pub fn of(battle: &Battle) -> Self {
        let session = battle.session();
        let snapshot = session.snapshot();
        Self {
            fighters: snapshot.fighters,
            projectiles: snapshot.projectiles,
            tick: snapshot.tick,
            hash: snapshot.hash,
            boss_phase: battle.bout().boss_phase(session),
            sparring: battle.bout().sparring_bot(session),
        }
    }
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GraySet {
    /// Phím tắt sinh lệnh host.
    Shortcuts,
    /// Áp lệnh host rồi làm mới `Intent`.
    Commands,
    /// Đọc từng thiết bị vào `Intent`.
    Devices,
    /// Chuyển `Intent` vào `LocalInput`.
    Flush,
}

/// Luật client của trận, không cần cửa sổ hay thiết bị; test headless dùng trực tiếp.
pub fn plugin(app: &mut App) {
    app.init_resource::<Intent>()
        .init_resource::<CombatLog>()
        .init_resource::<ArenaView>()
        .add_message::<HostCommand>()
        .configure_sets(
            RunFixedMainLoop,
            (
                GraySet::Shortcuts,
                GraySet::Commands,
                GraySet::Devices,
                GraySet::Flush,
            )
                .chain()
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(
            RunFixedMainLoop,
            (
                (apply_host_commands, begin_intent)
                    .chain()
                    .in_set(GraySet::Commands),
                flush_intent.in_set(GraySet::Flush),
                refresh_view.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                drive_player
                    .before(GameplaySet::Input)
                    .run_if(session_running),
                referee.in_set(GameplaySet::Events),
            ),
        );
}

fn begin_intent(mut intent: ResMut<Intent>) {
    *intent = Intent::default();
}

fn flush_intent(intent: Res<Intent>, mut input: ResMut<LocalInput>, time: Res<Time<Virtual>>) {
    // Nút nhấn trong lúc tạm dừng không được phát lại khi tiếp tục.
    if time.is_paused() {
        input.release_all();
        return;
    }
    let command = intent.command();
    input.set_move(command.move_x);
    input.set_guard(command.guard);
    if let Some(action) = command.action {
        input.press(action);
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_host_commands(
    mut commands: Commands,
    mut messages: MessageReader<HostCommand>,
    mut game: ResMut<Match>,
    mut sim: ResMut<SimSession>,
    mut input: ResMut<LocalInput>,
    mut log: ResMut<CombatLog>,
    mut time: ResMut<Time<Virtual>>,
) {
    for message in messages.read() {
        match *message {
            HostCommand::Rematch | HostCommand::SwitchMode => {
                let mode = match *message {
                    HostCommand::SwitchMode => game.bout.mode().other(),
                    _ => game.bout.mode(),
                };
                let (bout, session) = game.bout.rematch(mode);
                commands.queue(LoadSession(session));
                *input = LocalInput::new(bout.player(), PLAYER_EPOCH, None);
                log.note(match *message {
                    HostCommand::SwitchMode if mode == Mode::Boss => "Che do danh boss".to_owned(),
                    HostCommand::SwitchMode => "Che do dau tap".to_owned(),
                    _ => format!("Vong {} bat dau", bout.round()),
                });
                game.bout = bout;
                game.verification = None;
            }
            HostCommand::ToggleHitboxes => game.show_boxes = !game.show_boxes,
            HostCommand::ToggleSparringBot => {
                if let Some(bot) = sim.controller_mut::<SparringBot>(game.bout.rival()) {
                    bot.toggle();
                }
            }
            HostCommand::SetAutopilot(on) => game.bout.set_autopilot(on),
            HostCommand::SetPaused(paused) => {
                if paused {
                    time.pause();
                } else {
                    time.unpause();
                }
            }
        }
        // Đổi trận hoặc tạm dừng không mang theo nút đã nhấn cho trận cũ.
        input.release_all();
    }
}

/// Bot lái hộ thay toàn bộ ý định của tick; người chơi tự lái thì đếm cú bấm sắp vào tick.
fn drive_player(mut game: ResMut<Match>, sim: Res<SimSession>, mut input: ResMut<LocalInput>) {
    match game.bout.autopilot_command(sim.get().world()) {
        Some(frame) => input.set_frame(frame),
        None if input.has_press() => game.applied_presses = game.applied_presses.wrapping_add(1),
        None => {}
    }
}

/// Sau mỗi tick: số liệu, kết quả, log; trận lắng thì chạy lại replay để kiểm chứng rồi dừng phiên.
fn referee(
    mut game: ResMut<Match>,
    mut sim: ResMut<SimSession>,
    last: Res<LastTick>,
    mut log: ResMut<CombatLog>,
) {
    let Some(report) = last.report() else {
        return;
    };
    let world = sim.get().world();
    game.bout.observe(world, report);
    for record in &report.events {
        if let Some(line) = describe(&game, world, &record.event) {
            log.note(format!("t{:>5}  {line}", report.tick));
        }
    }
    if !game.bout.is_settled(world.tick()) {
        return;
    }
    let verification = game.bout.verify(sim.get()).map_err(|e| e.to_string());
    log.note(match &verification {
        Ok(v) => format!("Replay khop {} tick, hash {:016x}", v.ticks, v.hash),
        Err(error) => format!("Replay LECH: {}", ascii(error)),
    });
    game.verification = Some(verification);
    sim.halt();
}

fn describe(game: &Match, world: &myva_sim::World, event: &Event) -> Option<String> {
    let action = |id: FighterId, action| ascii(world.fighter(id).kit.spec(action).name);
    let name = |id| game.name(id);
    Some(match *event {
        Event::Hit {
            attacker,
            target,
            action: a,
            damage,
        } => format!(
            "{} trung {}: {} -{damage}",
            name(attacker),
            name(target),
            action(attacker, a)
        ),
        Event::Blocked {
            attacker,
            target,
            action: a,
            perfect,
        } => format!(
            "{} do {}{}",
            name(target),
            action(attacker, a),
            if perfect { " (hoan hao)" } else { "" }
        ),
        Event::Countered {
            counterer,
            attacker,
            damage,
        } => format!("{} phan cong {} -{damage}", name(counterer), name(attacker)),
        Event::GuardBroken { fighter } => format!("{} vo the do", name(fighter)),
        Event::Downed { fighter } => format!("{} bi ha", name(fighter)),
        Event::InputRejected { .. }
        | Event::ActionStarted { .. }
        | Event::StatusApplied { .. }
        | Event::StatusEnded { .. }
        | Event::Interacted { .. } => return None,
    })
}

fn refresh_view(
    mut arena: ResMut<ArenaView>,
    game: Res<Match>,
    sim: Res<SimSession>,
    fighters: Query<FighterMirror>,
    projectiles: Query<ProjectileMirror, With<Projectile>>,
) {
    let session = sim.get();
    *arena = ArenaView {
        fighters: view::fighters(&fighters),
        projectiles: view::projectiles(&projectiles),
        tick: session.world().tick(),
        hash: session.world().state_hash(),
        boss_phase: game.bout.boss_phase(session),
        sparring: game.bout.sparring_bot(session),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devices_merge_into_one_command_by_core_priority() {
        let mut intent = Intent::default();
        intent.add(1, Buttons::GUARD, Buttons::LIGHT);
        intent.add(-1, Buttons::NONE, Buttons::SKILL1);
        intent.add(1, Buttons::NONE, Buttons::NONE);
        let command = intent.command();
        assert_eq!(command.move_x, 1);
        assert!(command.guard);
        assert_eq!(
            command.action,
            Some(myva_sim::protocol::Action::Skill(0)),
            "thuật ưu tiên hơn đòn nhẹ như trong lõi"
        );
    }
}
