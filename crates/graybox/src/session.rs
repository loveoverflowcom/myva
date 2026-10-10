//! Adapter fixed tick (ADR 0003): ý định từ mọi thiết bị được gom trước vòng fixed, `Battle::step`
//! chạy đúng 60 lần mỗi giây luật trong `FixedUpdate`, rồi sự kiện chuyển sang phần thể hiện. Tốc
//! độ luật không phụ thuộc FPS; khung hình chậm chỉ làm nhiều tick chạy dồn trong một khung.

use std::collections::VecDeque;

use bevy::prelude::*;
use myva_sim::battle::{Battle, Mode, Verification};
use myva_sim::tick::Tick;
use myva_sim::{Buttons, Event, FighterId, InputFrame};

use crate::text::ascii;

/// Số dòng log HUD giữ lại.
const LOG_LINES: usize = 6;

/// Trận đang chơi và tùy chọn hiển thị. Chỉ hệ thống tick và lệnh host sửa `battle`.
#[derive(Resource)]
pub struct Session {
    pub battle: Battle,
    pub show_boxes: bool,
    /// Kết quả chạy lại replay của trận, tính một lần khi trận lắng.
    pub verification: Option<Result<Verification, String>>,
    /// Số khung input có nút vừa nhấn đã vào mô phỏng; host dùng để đo trễ input → tick.
    pub applied_presses: u32,
}

impl Session {
    pub fn new(mode: Mode, autopilot: bool) -> Self {
        let mut battle = Battle::new(mode, 1);
        battle.set_autopilot(autopilot);
        Self {
            battle,
            show_boxes: false,
            verification: None,
            applied_presses: 0,
        }
    }

    fn restart(&mut self, mode: Mode) {
        self.battle = self.battle.rematch(mode);
        self.verification = None;
    }

    /// Tên ngắn trên HUD và log.
    pub fn name(&self, id: FighterId) -> &'static str {
        match (id == self.battle.player(), self.battle.mode()) {
            (true, _) => "P1",
            (false, Mode::Boss) => "Boss",
            (false, Mode::Duel) => "Bot",
        }
    }
}

/// Ý định của người chơi. `move_x` và `held` được tính lại mỗi khung hình từ trạng thái thiết bị;
/// `pressed` dồn mọi nút vừa nhấn tới tick kế tiếp, nên FPS khác tick rate không làm mất hay nhân
/// đôi một lần nhấn.
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

    fn take_frame(&mut self) -> InputFrame {
        InputFrame {
            seq: 0,
            move_x: self.move_x.signum() as i8,
            held: self.held,
            pressed: std::mem::take(&mut self.pressed),
        }
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

/// Sự kiện gần đây: `fresh` cho phần thể hiện tiêu thụ trong khung hình, `lines` cho log HUD.
#[derive(Resource, Default)]
pub struct CombatLog {
    pub fresh: Vec<(Tick, Event)>,
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

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GraySet {
    /// Phím tắt sinh lệnh host.
    Shortcuts,
    /// Áp lệnh host rồi làm mới `Intent`.
    Commands,
    /// Đọc từng thiết bị vào `Intent`.
    Devices,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Intent>()
        .init_resource::<CombatLog>()
        .configure_sets(
            RunFixedMainLoop,
            (GraySet::Shortcuts, GraySet::Commands, GraySet::Devices)
                .chain()
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
        )
        .add_systems(
            RunFixedMainLoop,
            (apply_host_commands, begin_intent)
                .chain()
                .in_set(GraySet::Commands),
        )
        .add_systems(FixedUpdate, tick);
}

fn begin_intent(mut intent: ResMut<Intent>, time: Res<Time<Virtual>>) {
    intent.move_x = 0;
    intent.held = Buttons::NONE;
    // Nút nhấn trong lúc tạm dừng không được phát lại khi tiếp tục.
    if time.is_paused() {
        intent.pressed = Buttons::NONE;
    }
}

fn apply_host_commands(
    mut commands: MessageReader<HostCommand>,
    mut session: ResMut<Session>,
    mut intent: ResMut<Intent>,
    mut log: ResMut<CombatLog>,
    mut time: ResMut<Time<Virtual>>,
) {
    for command in commands.read() {
        match *command {
            HostCommand::Rematch => {
                let mode = session.battle.mode();
                session.restart(mode);
                log.fresh.clear();
                log.note(format!("Vong {} bat dau", session.battle.round()));
            }
            HostCommand::SwitchMode => {
                let mode = session.battle.mode().other();
                session.restart(mode);
                log.fresh.clear();
                log.note(match mode {
                    Mode::Boss => "Che do danh boss".to_owned(),
                    Mode::Duel => "Che do dau tap".to_owned(),
                });
            }
            HostCommand::ToggleHitboxes => session.show_boxes = !session.show_boxes,
            HostCommand::ToggleSparringBot => session.battle.toggle_sparring_bot(),
            HostCommand::SetAutopilot(on) => session.battle.set_autopilot(on),
            HostCommand::SetPaused(paused) => {
                if paused {
                    time.pause();
                } else {
                    time.unpause();
                }
            }
        }
        // Đổi trận hoặc tạm dừng không mang theo nút đã nhấn cho trận cũ.
        intent.pressed = Buttons::NONE;
    }
}

fn tick(mut session: ResMut<Session>, mut intent: ResMut<Intent>, mut log: ResMut<CombatLog>) {
    if session.battle.is_settled() {
        intent.pressed = Buttons::NONE;
        return;
    }
    let frame = intent.take_frame();
    if !frame.pressed.is_empty() && !session.battle.autopilot() {
        session.applied_presses = session.applied_presses.wrapping_add(1);
    }
    let tick = session.battle.world().tick();
    for event in session.battle.step(frame) {
        if let Some(line) = describe(&session, &event) {
            log.note(format!("t{tick:>5}  {line}"));
        }
        log.fresh.push((tick, event));
    }
    if session.battle.is_settled() {
        let verification = session.battle.verify().map_err(|error| error.to_string());
        log.note(match &verification {
            Ok(v) => format!("Replay khop {} tick, hash {:016x}", v.ticks, v.hash),
            Err(error) => format!("Replay LECH: {}", ascii(error)),
        });
        session.verification = Some(verification);
    }
}

fn describe(session: &Session, event: &Event) -> Option<String> {
    let world = session.battle.world();
    let action = |id: FighterId, action| ascii(world.fighter(id).kit.spec(action).name);
    let name = |id| session.name(id);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presses_are_taken_once_and_held_state_is_kept() {
        let mut intent = Intent::default();
        intent.add(1, Buttons::GUARD, Buttons::LIGHT);
        intent.add(-1, Buttons::NONE, Buttons::SKILL1);
        intent.add(1, Buttons::NONE, Buttons::NONE);
        let first = intent.take_frame();
        assert_eq!(first.move_x, 1);
        assert_eq!(first.held, Buttons::GUARD);
        assert_eq!(first.pressed, Buttons::LIGHT | Buttons::SKILL1);
        let second = intent.take_frame();
        assert!(second.pressed.is_empty());
        assert_eq!(second.held, Buttons::GUARD);
    }
}
