//! Ngân sách mất quyền điều khiển theo mục tiêu, mọi nguồn dùng chung (GT #19).
//! Không phụ thuộc người đánh, loại đòn, độ cao hay tiếp đất. Chỉ reset chuỗi sau một cửa
//! phản ứng đủ dài; lịch sử DR vẫn tồn tại qua cửa đó.

use crate::tick::ms_to_ticks;

pub const LOCK_LIMIT: u32 = ms_to_ticks(800);
pub const RESPONSE_WINDOW: u32 = ms_to_ticks(200);
pub const STUN_LIMIT: u32 = ms_to_ticks(300);
pub const DR_WINDOW: u32 = ms_to_ticks(8_000);
pub const DR_IMMUNITY: u32 = ms_to_ticks(1_000);
pub const DAMAGE_PERCENT: u32 = 25;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ControlBudget {
    history: [u32; 3],
    pub immune: u32,
    pub locked_ticks: u32,
    pub damage: u32,
    response_ticks: u32,
    chain: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Granted {
    pub damage: u32,
    pub ticks: u32,
    pub release: bool,
}

impl ControlBudget {
    /// Gọi cho cả damage không gây stun khi mục tiêu đang trong chuỗi; không có kẽ hở đạn
    /// hay đổi người đánh. `requested` đã được giới hạn theo loại control ở caller.
    pub(crate) fn grant(&mut self, damage: u32, requested: u32, max_hp: u32) -> Granted {
        let mut ticks = 0;
        if requested > 0 && self.immune == 0 {
            let count = self.history.iter().filter(|&&ttl| ttl > 0).count();
            if count == 3 {
                self.immune = DR_IMMUNITY;
            } else {
                let scale = [100, 70, 40][count];
                ticks = (requested * scale / 100).min(LOCK_LIMIT.saturating_sub(self.locked_ticks));
                *self.history.iter_mut().find(|ttl| **ttl == 0).unwrap() = DR_WINDOW;
            }
        }
        self.chain |= ticks > 0;
        let damage = if self.chain {
            let allowed = (max_hp * DAMAGE_PERCENT / 100).saturating_sub(self.damage);
            damage.min(allowed)
        } else {
            damage
        };
        let mut release = false;
        if self.chain {
            self.damage += damage;
            if self.damage >= max_hp * DAMAGE_PERCENT / 100 || self.locked_ticks >= LOCK_LIMIT {
                ticks = 0;
                release = true;
                self.immune = self.immune.max(RESPONSE_WINDOW + 1);
            }
        }
        Granted {
            damage,
            ticks,
            release,
        }
    }

    /// Một tick bị khóa hoặc có thể phản ứng. Một khoảng trống 1 tick không reset chuỗi.
    pub(crate) fn tick(&mut self, locked: bool) -> bool {
        for ttl in &mut self.history {
            *ttl = ttl.saturating_sub(1);
        }
        self.immune = self.immune.saturating_sub(1);
        if locked || self.chain {
            if locked {
                self.response_ticks = 0;
            } else {
                self.response_ticks += 1;
            }
            if self.response_ticks >= RESPONSE_WINDOW {
                self.chain = false;
                self.damage = 0;
                self.locked_ticks = 0;
            } else {
                // Khoảng trống ngắn chưa phải counterplay: vẫn tính vào 800 ms của chuỗi.
                self.locked_ticks = (self.locked_ticks + 1).min(LOCK_LIMIT);
                if self.locked_ticks >= LOCK_LIMIT {
                    self.immune = self.immune.max(RESPONSE_WINDOW);
                    return locked;
                }
            }
        }
        false
    }
}
