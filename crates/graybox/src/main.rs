//! Graybox combat của MyVa — Thần Mạch (work-plan 020): một arena phẳng, Long Lưu do người chơi
//! điều khiển. Chỉ vẽ hình khối; mọi luật nằm trong `myva-sim`, client chỉ gom input và thể hiện
//! trạng thái.
//!
//! Hai chế độ: đánh boss Kẻ Giữ Đập (mặc định) và đấu tập với bot B0 (`--duel`).
//!
//! Phím theo combat.md §12: A/D hoặc ←/→ di chuyển, Space nhảy, Shift lướt, giữ L để đỡ, J/K đòn
//! nhẹ/nặng, Q/E/R thuật 1–3. M đổi chế độ, B bật/tắt bot đấu tập, H bật/tắt hitbox, F5 đấu
//! lại, F9 lưu replay, Esc thoát.
//!
//! `--demo` để bot điều khiển người chơi (B1 khi đánh boss, B0 khi đấu tập);
//! `--screenshot <file.png>` chạy demo vài giây, lưu ảnh màn hình cùng replay rồi thoát.

mod draw;

use std::collections::VecDeque;

use macroquad::prelude::*;
use myva_sim::boss::{BossBrain, BossPhase, KE_GIU_DAP};
use myva_sim::bot::{PatternReader, RandomBot};
use myva_sim::replay::Recorder;
use myva_sim::tick::TICK_HZ;
use myva_sim::{Buttons, Event, FighterId, InputFrame, LONG_LUU, PX, State, World};

const DT: f32 = 1.0 / TICK_HZ as f32;
/// Giới hạn thời gian bù mỗi khung để không chạy dồn hàng trăm tick sau khi cửa sổ bị treo.
const MAX_FRAME_TIME: f32 = 0.25;
const LOG_LINES: usize = 6;
const SCREENSHOT_FRAMES: u32 = 600;
/// Bot B1 trong demo phản ứng như ngân sách combat.md §10.
const DEMO_REACTION_MS: u32 = 250;
const KO_RESET_TICKS: u32 = 2 * TICK_HZ;
const CHECKPOINT_EVERY: u32 = TICK_HZ;

const PRESS_KEYS: [(KeyCode, Buttons); 8] = [
    (KeyCode::Space, Buttons::JUMP),
    (KeyCode::LeftShift, Buttons::DASH),
    (KeyCode::RightShift, Buttons::DASH),
    (KeyCode::J, Buttons::LIGHT),
    (KeyCode::K, Buttons::HEAVY),
    (KeyCode::Q, Buttons::SKILL1),
    (KeyCode::E, Buttons::SKILL2),
    (KeyCode::R, Buttons::SKILL3),
];

fn window_conf() -> Conf {
    Conf {
        window_title: "MyVa — Thần Mạch · graybox".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

/// Gom các nút vừa nhấn giữa hai tick: FPS cao hơn tick rate không làm mất input, và mỗi lần
/// nhấn chỉ vào đúng một khung.
#[derive(Default)]
struct KeyboardInput {
    pending: Buttons,
    seq: u32,
}

impl KeyboardInput {
    fn poll(&mut self) {
        for (key, button) in PRESS_KEYS {
            if is_key_pressed(key) {
                self.pending |= button;
            }
        }
    }

    fn frame(&mut self) -> InputFrame {
        self.seq += 1;
        let left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
        let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
        InputFrame {
            seq: self.seq,
            move_x: i8::from(right) - i8::from(left),
            held: if is_key_down(KeyCode::L) {
                Buttons::GUARD
            } else {
                Buttons::NONE
            },
            pressed: std::mem::take(&mut self.pending),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Boss,
    Duel,
}

enum PlayerControl {
    Keyboard,
    Reader(PatternReader),
    Random(RandomBot),
}

enum RivalControl {
    Boss(BossBrain),
    Random(RandomBot),
    Idle,
}

pub(crate) struct Match {
    pub mode: Mode,
    pub world: World,
    pub player: FighterId,
    pub rival: FighterId,
    pub log: VecDeque<String>,
    player_control: PlayerControl,
    rival_control: RivalControl,
    recorder: Recorder,
    ko_ticks: u32,
    round: u64,
    demo: bool,
}

impl Match {
    fn new(round: u64, mode: Mode, demo: bool) -> Self {
        let mut world = World::new();
        let (player, rival, player_control, rival_control) = match mode {
            Mode::Boss => (
                world.spawn(&LONG_LUU, 400 * PX, 1),
                world.spawn(&KE_GIU_DAP, 1_100 * PX, -1),
                if demo {
                    PlayerControl::Reader(PatternReader::with_reaction_ms(DEMO_REACTION_MS))
                } else {
                    PlayerControl::Keyboard
                },
                RivalControl::Boss(BossBrain::new()),
            ),
            Mode::Duel => (
                world.spawn(&LONG_LUU, 640 * PX, 1),
                world.spawn(&LONG_LUU, 960 * PX, -1),
                if demo {
                    PlayerControl::Random(RandomBot::new(round ^ 0x5EED))
                } else {
                    PlayerControl::Keyboard
                },
                RivalControl::Random(RandomBot::new(round)),
            ),
        };
        Self {
            mode,
            recorder: Recorder::new(&world, CHECKPOINT_EVERY),
            world,
            player,
            rival,
            log: VecDeque::new(),
            player_control,
            rival_control,
            ko_ticks: 0,
            round,
            demo,
        }
    }

    fn rematch(&self, mode: Mode) -> Self {
        Self::new(self.round + 1, mode, self.demo)
    }

    fn toggle_sparring_bot(&mut self) {
        self.rival_control = match self.rival_control {
            RivalControl::Random(_) => RivalControl::Idle,
            RivalControl::Idle => RivalControl::Random(RandomBot::new(self.round)),
            RivalControl::Boss(_) => return,
        };
    }

    pub fn sparring_bot(&self) -> Option<bool> {
        match self.rival_control {
            RivalControl::Random(_) => Some(true),
            RivalControl::Idle => Some(false),
            RivalControl::Boss(_) => None,
        }
    }

    pub fn boss_phase(&self) -> Option<BossPhase> {
        match &self.rival_control {
            RivalControl::Boss(brain) => Some(brain.phase()),
            _ => None,
        }
    }

    fn step(&mut self, keyboard: InputFrame) {
        let world = &self.world;
        let player_frame = match &mut self.player_control {
            PlayerControl::Keyboard => keyboard,
            PlayerControl::Reader(reader) => reader.next_frame(world, self.player, self.rival),
            PlayerControl::Random(bot) => bot.next_frame(),
        };
        let mut inputs = vec![(self.player, player_frame)];
        match &mut self.rival_control {
            RivalControl::Boss(brain) => {
                inputs.push((self.rival, brain.next_frame(world, self.rival, self.player)));
            }
            RivalControl::Random(bot) => inputs.push((self.rival, bot.next_frame())),
            RivalControl::Idle => {}
        }
        let tick = self.world.tick();
        for event in self.recorder.step(&mut self.world, &inputs) {
            if let Some(line) = self.describe(&event) {
                self.note(format!("t{tick:>5}  {line}"));
            }
        }
        if self
            .world
            .fighters()
            .iter()
            .any(|f| f.state == State::Downed)
        {
            self.ko_ticks += 1;
        }
    }

    fn note(&mut self, line: String) {
        self.log.push_back(line);
        if self.log.len() > LOG_LINES {
            self.log.pop_front();
        }
    }

    /// Ghi replay của trận hiện tại ra thư mục làm việc; kiểm tra bằng `myva-replay`.
    #[cfg(not(target_arch = "wasm32"))]
    fn save_replay(&mut self) {
        let path = format!("graybox-r{}-t{}.myva-replay", self.round, self.world.tick());
        let text = self.recorder.snapshot(&self.world).to_text();
        let line = match std::fs::write(&path, text) {
            Ok(()) => format!("da luu replay: {path}"),
            Err(error) => format!("khong luu duoc replay: {error}"),
        };
        self.note(line);
    }

    #[cfg(target_arch = "wasm32")]
    fn save_replay(&mut self) {
        self.note("ban web chua luu duoc replay".to_owned());
    }

    pub fn is_ko(&self) -> bool {
        self.ko_ticks > 0
    }

    pub fn player_won(&self) -> bool {
        self.world.fighter(self.rival).state == State::Downed
            && self.world.fighter(self.player).state != State::Downed
    }

    pub fn name(&self, id: FighterId) -> &'static str {
        match (id == self.player, self.mode) {
            (true, _) => "P1",
            (false, Mode::Boss) => "Boss",
            (false, Mode::Duel) => "Bot",
        }
    }

    fn describe(&self, event: &Event) -> Option<String> {
        let action_name =
            |id: FighterId, action| draw::ascii(self.world.fighter(id).kit.spec(action).name);
        Some(match *event {
            Event::Hit {
                attacker,
                target,
                action,
                damage,
            } => format!(
                "{} trung {}: {} -{damage}",
                self.name(attacker),
                self.name(target),
                action_name(attacker, action)
            ),
            Event::Blocked {
                attacker,
                target,
                action,
                perfect,
            } => format!(
                "{} do {}{}",
                self.name(target),
                action_name(attacker, action),
                if perfect { " (hoan hao)" } else { "" }
            ),
            Event::Countered {
                counterer,
                attacker,
                damage,
            } => format!(
                "{} phan cong {} -{damage}",
                self.name(counterer),
                self.name(attacker)
            ),
            Event::GuardBroken { fighter } => format!("{} vo the do", self.name(fighter)),
            Event::Downed { fighter } => format!("{} bi ha", self.name(fighter)),
            Event::InputRejected { .. }
            | Event::ActionStarted { .. }
            | Event::StatusApplied { .. }
            | Event::StatusEnded { .. }
            | Event::Interacted { .. } => return None,
        })
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut args = std::env::args().skip(1);
    let mut demo = false;
    let mut mode = Mode::Boss;
    let mut screenshot = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => demo = true,
            "--duel" => mode = Mode::Duel,
            "--screenshot" => {
                screenshot = args.next();
                demo = true;
            }
            other => eprintln!("bỏ qua tham số lạ: {other}"),
        }
    }

    let mut game = Match::new(1, mode, demo);
    let mut keyboard = KeyboardInput::default();
    let mut show_boxes = true;
    let mut accumulator = 0.0;
    let mut frames = 0u32;
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::F5) || (demo && game.ko_ticks > KO_RESET_TICKS) {
            game = game.rematch(game.mode);
        }
        if is_key_pressed(KeyCode::M) {
            let other = match game.mode {
                Mode::Boss => Mode::Duel,
                Mode::Duel => Mode::Boss,
            };
            game = game.rematch(other);
        }
        if is_key_pressed(KeyCode::B) {
            game.toggle_sparring_bot();
        }
        if is_key_pressed(KeyCode::H) {
            show_boxes = !show_boxes;
        }
        if is_key_pressed(KeyCode::F9) {
            game.save_replay();
        }

        keyboard.poll();
        accumulator += get_frame_time().min(MAX_FRAME_TIME);
        while accumulator >= DT {
            game.step(keyboard.frame());
            accumulator -= DT;
        }

        draw::frame(&game, show_boxes);
        frames += 1;
        if let Some(path) = &screenshot
            && frames >= SCREENSHOT_FRAMES
        {
            get_screen_data().export_png(path);
            game.save_replay();
            break;
        }
        next_frame().await;
    }
}
