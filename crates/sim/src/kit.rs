//! Dữ liệu kit truyền thừa. Graybox chỉ có Long Lưu (combat.md §7); Sơn Cốt và Phong Vũ được thêm
//! khi kit đầu đạt nhịp (work-plan 020). Mọi giá trị là giả thuyết (GT) ở phòng thử 1.000 HP.

use crate::tick::ms_to_ticks;

/// Một pixel thiết kế tính bằng mili-pixel; vị trí và kích thước mô phỏng là số nguyên theo
/// đơn vị mili-pixel.
pub const PX: i32 = 1_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActionKind {
    /// Nhịp `0..=2` của chuỗi đòn nhẹ.
    Light(u8),
    Heavy,
    /// Ô thuật `0..=2`.
    Skill(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Startup,
    Active,
    Recovery,
}

/// Hộp đánh tính từ chân nhân vật theo hướng mặt, đơn vị mili-pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Hitbox {
    /// Khoảng cách từ tâm nhân vật tới mép gần của hộp.
    pub front: i32,
    pub width: i32,
    /// Độ cao mép dưới so với chân.
    pub bottom: i32,
    pub height: i32,
}

impl Hitbox {
    const fn px(front: i32, width: i32, bottom: i32, height: i32) -> Self {
        Self {
            front: front * PX,
            width: width * PX,
            bottom: bottom * PX,
            height: height * PX,
        }
    }
}

/// Một đòn đã lượng tử hóa timing sang tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ActionSpec {
    pub name: &'static str,
    pub startup: u32,
    pub active: u32,
    pub recovery: u32,
    pub stamina_cost: u32,
    pub energy_cost: u32,
    pub cooldown: u32,
    pub damage: u32,
    pub guard_pressure: u32,
    pub hitstun: u32,
    pub knockback: i32,
    pub hitbox: Hitbox,
    /// Tư thế phản công: bắt một đòn phía trước trong active thì gây lượng damage này lên
    /// người đánh, với `hitstun` của chính đòn.
    pub counter_damage: Option<u32>,
}

impl ActionSpec {
    /// Đòn chỉ có timing (ms); các trường khác bằng 0 để ghi đè bằng struct update.
    const fn timed(name: &'static str, startup_ms: u32, active_ms: u32, recovery_ms: u32) -> Self {
        Self {
            name,
            startup: ms_to_ticks(startup_ms),
            active: ms_to_ticks(active_ms),
            recovery: ms_to_ticks(recovery_ms),
            stamina_cost: 0,
            energy_cost: 0,
            cooldown: 0,
            damage: 0,
            guard_pressure: 0,
            hitstun: 0,
            knockback: 0,
            hitbox: Hitbox::px(0, 0, 0, 0),
            counter_damage: None,
        }
    }

    pub const fn total(&self) -> u32 {
        self.startup + self.active + self.recovery
    }

    pub const fn phase(&self, elapsed: u32) -> Phase {
        if elapsed < self.startup {
            Phase::Startup
        } else if elapsed < self.startup + self.active {
            Phase::Active
        } else {
            Phase::Recovery
        }
    }
}

/// 2 đòn cơ bản (đòn nhẹ là chuỗi 3 nhịp) + 3 thuật được trang bị.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Kit {
    pub lineage: &'static str,
    pub light: [ActionSpec; 3],
    pub heavy: ActionSpec,
    pub skills: [ActionSpec; 3],
}

impl Kit {
    pub fn spec(&self, action: ActionKind) -> &ActionSpec {
        match action {
            ActionKind::Light(step) => &self.light[usize::from(step)],
            ActionKind::Heavy => &self.heavy,
            ActionKind::Skill(slot) => &self.skills[usize::from(slot)],
        }
    }
}

const GON_SONG: ActionSpec = ActionSpec {
    damage: 45,
    guard_pressure: 10,
    hitstun: ms_to_ticks(120),
    hitbox: Hitbox::px(10, 60, 20, 50),
    ..ActionSpec::timed("Gợn Sóng", 180, 100, 240)
};

/// Long Lưu: điều tiết dòng chảy, giữ khoảng cách vừa và phản công (combat.md §7).
pub const LONG_LUU: Kit = Kit {
    lineage: "Long Lưu",
    // Nhịp 3 đẩy lùi nhẹ.
    light: [
        GON_SONG,
        GON_SONG,
        ActionSpec {
            knockback: 30 * PX,
            ..GON_SONG
        },
    ],
    heavy: ActionSpec {
        stamina_cost: 12,
        damage: 90,
        guard_pressure: 24,
        hitstun: ms_to_ticks(240),
        hitbox: Hitbox::px(10, 80, 10, 60),
        ..ActionSpec::timed("Phá Lưu", 400, 130, 360)
    },
    skills: [
        // Thiết kế là đạn thẳng; graybox tạm dùng hitbox dài cho tới khi có entity đạn.
        ActionSpec {
            energy_cost: 20,
            cooldown: ms_to_ticks(3_000),
            damage: 80,
            guard_pressure: 12,
            hitstun: ms_to_ticks(180),
            hitbox: Hitbox::px(20, 400, 30, 30),
            ..ActionSpec::timed("Lưu Tiễn", 300, 100, 300)
        },
        ActionSpec {
            energy_cost: 25,
            cooldown: ms_to_ticks(7_000),
            hitstun: ms_to_ticks(300),
            counter_damage: Some(65),
            ..ActionSpec::timed("Hồi Thế", 160, 300, 300)
        },
        ActionSpec {
            energy_cost: 35,
            cooldown: ms_to_ticks(10_000),
            damage: 100,
            guard_pressure: 30,
            hitstun: ms_to_ticks(300),
            knockback: 60 * PX,
            hitbox: Hitbox::px(0, 160, 0, 90),
            ..ActionSpec::timed("Triều Dâng", 450, 150, 450)
        },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_uses_quantized_doc_timing() {
        let light = LONG_LUU.spec(ActionKind::Light(0));
        assert_eq!((light.startup, light.active, light.recovery), (11, 6, 15));
        assert_eq!(light.phase(10), Phase::Startup);
        assert_eq!(light.phase(11), Phase::Active);
        assert_eq!(light.phase(17), Phase::Recovery);
    }

    #[test]
    fn only_the_third_light_pushes() {
        let pushes: Vec<_> = LONG_LUU.light.iter().map(|s| s.knockback > 0).collect();
        assert_eq!(pushes, [false, false, true]);
    }

    #[test]
    fn basic_attacks_never_cost_energy() {
        for spec in LONG_LUU.light.iter().chain([&LONG_LUU.heavy]) {
            assert_eq!(spec.energy_cost, 0, "{}", spec.name);
        }
        assert!(
            LONG_LUU
                .skills
                .iter()
                .all(|s| s.energy_cost > 0 && s.cooldown > 0)
        );
    }
}
