//! Thanh tài nguyên chiến đấu bằng số nguyên (combat.md §3).

use crate::tick::{TICK_HZ, ms_to_ticks};

/// Mỗi điểm chia thành `TICK_HZ` đơn vị con, nên tốc độ "x điểm/giây" cộng hoặc trừ đúng x đơn vị
/// con mỗi tick mà không cần float.
pub const SUB_PER_POINT: u32 = TICK_HZ;

/// Một thanh tài nguyên có trần, độ trễ trước khi hồi và tốc độ hồi. Không bao giờ âm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Meter {
    sub: u32,
    max_sub: u32,
    regen_per_sec: u32,
    regen_delay: u32,
    idle: u32,
}

impl Meter {
    pub const fn new(max: u32, start: u32, regen_per_sec: u32, regen_delay_ms: u32) -> Self {
        let max_sub = max * SUB_PER_POINT;
        let start_sub = start * SUB_PER_POINT;
        Self {
            sub: if start_sub < max_sub {
                start_sub
            } else {
                max_sub
            },
            max_sub,
            regen_per_sec,
            regen_delay: ms_to_ticks(regen_delay_ms),
            idle: 0,
        }
    }

    /// Sức bền: 100, hồi 20/giây sau 600 ms không tiêu.
    pub const fn stamina() -> Self {
        Self::new(100, 100, 20, 600)
    }

    /// Năng lượng: 100, hồi 6/giây sau 1.000 ms không dùng thuật.
    pub const fn energy() -> Self {
        Self::new(100, 100, 6, 1_000)
    }

    /// Mạch: 0–100, không tự hồi; nhận khi tác động hợp lệ trong chiến đấu.
    pub const fn mach() -> Self {
        Self::new(100, 0, 0, 0)
    }

    pub const fn points(&self) -> u32 {
        self.sub / SUB_PER_POINT
    }

    pub const fn max_points(&self) -> u32 {
        self.max_sub / SUB_PER_POINT
    }

    pub const fn sub(&self) -> u32 {
        self.sub
    }

    pub const fn max_sub(&self) -> u32 {
        self.max_sub
    }

    pub const fn is_empty(&self) -> bool {
        self.sub == 0
    }

    pub const fn can_spend(&self, points: u32) -> bool {
        Self::covers(self.sub, points)
    }

    /// `sub` đơn vị con có đủ trả `points` điểm không; dùng chung cho khung nhìn snapshot.
    pub const fn covers(sub: u32, points: u32) -> bool {
        sub >= points * SUB_PER_POINT
    }

    /// Tiêu đủ `points` hoặc không tiêu gì; trả `false` khi thiếu.
    pub fn try_spend(&mut self, points: u32) -> bool {
        if !self.can_spend(points) {
            return false;
        }
        self.sub -= points * SUB_PER_POINT;
        self.idle = 0;
        true
    }

    /// Rút tối đa `sub` đơn vị con; trả `true` khi thanh cạn.
    pub fn drain_sub(&mut self, sub: u32) -> bool {
        self.sub = self.sub.saturating_sub(sub);
        self.idle = 0;
        self.sub == 0
    }

    /// Rút theo tốc độ điểm/giây trong một tick; trả `true` khi thanh cạn.
    pub fn drain_per_sec(&mut self, points_per_sec: u32) -> bool {
        self.drain_sub(points_per_sec)
    }

    pub fn gain(&mut self, points: u32) {
        self.sub = (self.sub + points * SUB_PER_POINT).min(self.max_sub);
    }

    /// Tiến một tick. `regen_allowed = false` khi luật cấm hồi, ví dụ đang đỡ.
    pub fn tick(&mut self, regen_allowed: bool) {
        if !regen_allowed {
            return;
        }
        if self.idle < self.regen_delay {
            self.idle += 1;
            return;
        }
        self.sub = (self.sub + self.regen_per_sec).min(self.max_sub);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spend_is_all_or_nothing() {
        let mut m = Meter::new(100, 10, 0, 0);
        assert!(!m.try_spend(11));
        assert_eq!(m.points(), 10);
        assert!(m.try_spend(10));
        assert!(m.is_empty());
    }

    #[test]
    fn stamina_waits_600ms_then_regens_20_per_second() {
        let mut m = Meter::stamina();
        assert!(m.try_spend(50));
        for _ in 0..ms_to_ticks(600) {
            m.tick(true);
        }
        assert_eq!(m.points(), 50);
        for _ in 0..TICK_HZ {
            m.tick(true);
        }
        assert_eq!(m.points(), 70);
    }

    #[test]
    fn no_regen_when_blocked_and_never_negative() {
        let mut m = Meter::new(100, 1, 20, 0);
        for _ in 0..600 {
            m.tick(false);
        }
        assert_eq!(m.points(), 1);
        assert!(m.drain_sub(10_000));
        assert_eq!(m.sub(), 0);
    }

    #[test]
    fn gain_is_capped() {
        let mut m = Meter::mach();
        m.gain(70);
        m.gain(70);
        assert_eq!(m.points(), 100);
    }
}
