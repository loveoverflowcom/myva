//! Graybox combat của MyVa — Thần Mạch (work-plan 020): một arena phẳng, Long Lưu do người chơi
//! điều khiển và một đối thủ bot B0. Chỉ vẽ hình khối; mọi luật nằm trong `myva-sim`, client chỉ
//! gom input và thể hiện trạng thái.
//!
//! Phím theo combat.md §12: A/D hoặc ←/→ di chuyển, Space nhảy, Shift lướt, giữ L để đỡ, J/K đòn
//! nhẹ/nặng, Q/E/R thuật 1–3. B bật/tắt bot, H bật/tắt hitbox, F5 đấu lại, Esc thoát.
//!
//! `--demo` để bot điều khiển cả hai bên; `--screenshot <file.png>` chạy demo vài giây, lưu ảnh
//! màn hình rồi thoát.

mod draw;

use std::collections::VecDeque;

use macroquad::prelude::*;
use myva_sim::bot::RandomBot;
use myva_sim::tick::TICK_HZ;
use myva_sim::{Buttons, Event, FighterId, InputFrame, LONG_LUU, PX, State, World};

const DT: f32 = 1.0 / TICK_HZ as f32;
/// Giới hạn thời gian bù mỗi khung để không chạy dồn hàng trăm tick sau khi cửa sổ bị treo.
const MAX_FRAME_TIME: f32 = 0.25;
const LOG_LINES: usize = 6;
const SCREENSHOT_FRAMES: u32 = 600;
const KO_RESET_TICKS: u32 = 2 * TICK_HZ;

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

pub(crate) struct Match {
    pub world: World,
    pub player: FighterId,
    pub rival: FighterId,
    pub rival_bot: Option<RandomBot>,
    /// Ở chế độ demo, bot điều khiển cả người chơi.
    pub player_bot: Option<RandomBot>,
    pub log: VecDeque<String>,
    ko_ticks: u32,
    round: u64,
}

impl Match {
    fn new(round: u64, demo: bool) -> Self {
        let mut world = World::new();
        let player = world.spawn(&LONG_LUU, 640 * PX, 1);
        let rival = world.spawn(&LONG_LUU, 960 * PX, -1);
        Self {
            world,
            player,
            rival,
            rival_bot: Some(RandomBot::new(round)),
            player_bot: demo.then(|| RandomBot::new(round ^ 0x5EED)),
            log: VecDeque::new(),
            ko_ticks: 0,
            round,
        }
    }

    fn rematch(&self) -> Self {
        Self::new(self.round + 1, self.player_bot.is_some())
    }

    fn step(&mut self, keyboard: InputFrame) {
        let player_frame = match &mut self.player_bot {
            Some(bot) => bot.next_frame(),
            None => keyboard,
        };
        let mut inputs = vec![(self.player, player_frame)];
        if let Some(bot) = &mut self.rival_bot {
            inputs.push((self.rival, bot.next_frame()));
        }
        let tick = self.world.tick();
        for event in self.world.step(&inputs) {
            if let Some(line) = self.describe(&event) {
                self.log.push_back(format!("t{tick:>5}  {line}"));
                if self.log.len() > LOG_LINES {
                    self.log.pop_front();
                }
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

    pub fn is_ko(&self) -> bool {
        self.ko_ticks > 0
    }

    pub fn name(&self, id: FighterId) -> &'static str {
        if id == self.player { "P1" } else { "Bot" }
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
            Event::InputRejected { .. } | Event::ActionStarted { .. } => return None,
        })
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut args = std::env::args().skip(1);
    let mut demo = false;
    let mut screenshot = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--demo" => demo = true,
            "--screenshot" => {
                screenshot = args.next();
                demo = true;
            }
            other => eprintln!("bỏ qua tham số lạ: {other}"),
        }
    }

    let mut game = Match::new(1, demo);
    let mut keyboard = KeyboardInput::default();
    let mut show_boxes = true;
    let mut accumulator = 0.0;
    let mut frames = 0u32;
    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::F5) || (demo && game.ko_ticks > KO_RESET_TICKS) {
            game = game.rematch();
        }
        if is_key_pressed(KeyCode::B) {
            game.rival_bot = match game.rival_bot {
                Some(_) => None,
                None => Some(RandomBot::new(game.round)),
            };
        }
        if is_key_pressed(KeyCode::H) {
            show_boxes = !show_boxes;
        }

        keyboard.poll();
        accumulator += get_frame_time().min(MAX_FRAME_TIME);
        while accumulator >= DT {
            game.step(keyboard.frame());
            accumulator -= DT;
        }

        draw::frame(&game, show_boxes);
        frames += 1;
        if let Some(path) = &screenshot {
            if frames >= SCREENSHOT_FRAMES {
                get_screen_data().export_png(path);
                break;
            }
        }
        next_frame().await;
    }
}
