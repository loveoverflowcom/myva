//! Bot B0: chọn input ngẫu nhiên để tìm crash, trạng thái kẹt và input lặp
//! (combat-progression-balance.md §11). Bot đi qua cùng đường input như người chơi.

use crate::input::{Buttons, InputFrame};
use crate::rng::Rng;

const ACTIONS: [Buttons; 7] = [
    Buttons::JUMP,
    Buttons::DASH,
    Buttons::LIGHT,
    Buttons::HEAVY,
    Buttons::SKILL1,
    Buttons::SKILL2,
    Buttons::SKILL3,
];

#[derive(Clone, Debug)]
pub struct RandomBot {
    rng: Rng,
    seq: u32,
    move_x: i8,
    guarding: bool,
}

impl RandomBot {
    pub const fn new(seed: u64) -> Self {
        Self {
            rng: Rng::new(seed),
            seq: 0,
            move_x: 0,
            guarding: false,
        }
    }

    pub fn next_frame(&mut self) -> InputFrame {
        self.seq += 1;
        if self.rng.chance(1, 20) {
            self.move_x = self.rng.below(3) as i8 - 1;
        }
        if self.rng.chance(1, 30) {
            self.guarding = !self.guarding;
        }
        let pressed = if self.rng.chance(1, 8) {
            ACTIONS[self.rng.below(ACTIONS.len() as u32) as usize]
        } else {
            Buttons::NONE
        };
        InputFrame {
            seq: self.seq,
            move_x: self.move_x,
            held: if self.guarding {
                Buttons::GUARD
            } else {
                Buttons::NONE
            },
            pressed,
        }
    }
}
