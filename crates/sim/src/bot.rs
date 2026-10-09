//! Bot kiểm thử (combat-progression-balance.md §11). Bot đi qua cùng đường input như người chơi,
//! không gọi thẳng hàm gây damage.
//!
//! - [`RandomBot`] (B0): input ngẫu nhiên để tìm crash, trạng thái kẹt và input lặp.
//! - [`PatternReader`] (B1 tối giản): đánh boss bằng cách đọc tín hiệu.

use crate::boss::MIN_TELEGRAPH;
use crate::fighter::{ARENA_WIDTH, DASH_STAMINA, Fighter, FighterId, State};
use crate::input::{Buttons, InputFrame};
use crate::kit::{PX, Phase, Rect};
use crate::rng::Rng;
use crate::tick::ms_to_ticks;
use crate::world::World;

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

/// Khoảng đệm khi rời vùng đòn.
const MARGIN: i32 = 12 * PX;
/// Đạn còn cách thân bao xa thì bắt đầu giữ đỡ (startup đỡ 6 tick).
const GUARD_RANGE: i32 = 120 * PX;
/// Khoảng cách tối đa để đòn nhẹ chạm thân boss.
const STRIKE_RANGE: i32 = 100 * PX;
/// Số tick dự phòng để đi bộ ra khỏi vùng đòn kế tiếp.
const ESCAPE_BUDGET: u32 = 35;

/// Bot B1 tối giản: rời vùng đòn đang báo theo lối ngắn nhất (lướt nếu đi bộ không kịp), đỡ đạn
/// đang bay tới, và chỉ ra đòn khi boss còn bận đủ lâu để kịp né đòn sau. Bot chỉ đọc những gì
/// người chơi thấy trên màn hình, không đọc bộ não boss.
///
/// Thời gian phản ứng: bot chỉ nhận ra một đòn sau khi tín hiệu đã hiện đủ lâu. combat.md §10
/// dự trù 250 ms phản ứng trong 700 ms tín hiệu tối thiểu.
#[derive(Clone, Debug)]
pub struct PatternReader {
    seq: u32,
    reaction: u32,
}

impl Default for PatternReader {
    fn default() -> Self {
        Self::with_reaction_ms(0)
    }
}

impl PatternReader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_reaction_ms(ms: u32) -> Self {
        Self {
            seq: 0,
            reaction: ms_to_ticks(ms),
        }
    }

    pub fn next_frame(&mut self, world: &World, me: FighterId, boss: FighterId) -> InputFrame {
        self.seq += 1;
        let mut frame = InputFrame {
            seq: self.seq,
            ..InputFrame::default()
        };
        let me = world.fighter(me);
        let boss_fighter = world.fighter(boss);
        if me.state == State::Downed || boss_fighter.state == State::Downed {
            return frame;
        }

        if let Some(zone) = threat_zone(boss_fighter, self.reaction) {
            let body = me.hurtbox();
            let padded = Rect {
                x0: body.x0 - MARGIN,
                x1: body.x1 + MARGIN,
                ..body
            };
            if zone.overlaps(&padded) {
                let (dir, distance) = escape_route(me, &zone);
                frame.move_x = dir;
                let walk_ticks = (distance.max(0) as u32).div_ceil(me.kit.body.walk_speed as u32);
                if walk_ticks >= ticks_until_active(boss_fighter)
                    && me.stamina.can_spend(DASH_STAMINA)
                {
                    frame.pressed = Buttons::DASH;
                }
                return frame;
            }
        }

        let body = me.hurtbox();
        for projectile in world.projectiles().iter().filter(|p| p.owner == boss) {
            let gap = if projectile.dir > 0 {
                body.x0 - projectile.rect.x1
            } else {
                projectile.rect.x0 - body.x1
            };
            if (-MARGIN..GUARD_RANGE).contains(&gap) {
                // Quay về phía đạn bay tới rồi giữ đỡ.
                if me.facing != -projectile.dir {
                    frame.move_x = -projectile.dir;
                }
                frame.held = Buttons::GUARD;
                return frame;
            }
        }

        let toward = if boss_fighter.x >= me.x { 1 } else { -1 };
        if (boss_fighter.x - me.x).abs() > STRIKE_RANGE {
            frame.move_x = toward;
        } else if safe_to_commit(me, boss_fighter) {
            frame.move_x = toward;
            frame.pressed = Buttons::LIGHT;
        }
        frame
    }
}

/// Vùng boss sắp đánh trúng: hộp đòn trong startup/active, hoặc cả đường bay nếu là đạn sắp bắn.
/// Đòn mới hiện chưa tới `reaction` tick thì bot chưa nhận ra.
fn threat_zone(boss: &Fighter, reaction: u32) -> Option<Rect> {
    let (_, spec, phase) = boss.action()?;
    let State::Attack { elapsed, .. } = boss.state else {
        return None;
    };
    if elapsed < reaction {
        return None;
    }
    match (spec.projectile, phase) {
        (Some(projectile), Phase::Startup) => {
            let start = projectile.hitbox.place(boss.x, boss.y, boss.facing);
            let reach = projectile.speed * projectile.lifetime as i32;
            Some(if boss.facing > 0 {
                Rect {
                    x1: start.x1 + reach,
                    ..start
                }
            } else {
                Rect {
                    x0: start.x0 - reach,
                    ..start
                }
            })
        }
        (None, Phase::Startup | Phase::Active) if spec.damage > 0 => {
            Some(spec.hitbox.place(boss.x, boss.y, boss.facing))
        }
        _ => None,
    }
}

/// Hướng và quãng đường ngắn nhất để cả thân ra khỏi `zone` mà vẫn trong arena.
fn escape_route(me: &Fighter, zone: &Rect) -> (i8, i32) {
    let half = me.kit.body.half_width;
    let left_target = zone.x0 - half - MARGIN;
    let right_target = zone.x1 + half + MARGIN;
    let left = me.x - left_target;
    let right = right_target - me.x;
    match (left_target >= half, right_target <= ARENA_WIDTH - half) {
        (true, false) => (-1, left),
        (false, true) => (1, right),
        _ if left <= right => (-1, left),
        _ => (1, right),
    }
}

fn ticks_until_active(boss: &Fighter) -> u32 {
    match (boss.state, boss.action()) {
        (State::Attack { elapsed, .. }, Some((_, spec, _))) => spec.startup.saturating_sub(elapsed),
        _ => 0,
    }
}

/// Boss còn bận đủ lâu để đánh xong một đòn nhẹ rồi đi ra khỏi vùng đòn kế tiếp.
fn safe_to_commit(me: &Fighter, boss: &Fighter) -> bool {
    let (State::Attack { elapsed, .. }, Some((_, spec, _))) = (boss.state, boss.action()) else {
        return false;
    };
    let remaining = spec.total() - elapsed;
    remaining + MIN_TELEGRAPH >= me.kit.light[0].total() + ESCAPE_BUDGET
}
