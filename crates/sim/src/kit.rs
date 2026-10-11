//! Dữ liệu kit truyền thừa. Graybox chỉ có Long Lưu (combat.md §7); Sơn Cốt và Phong Vũ được thêm
//! khi kit đầu đạt nhịp (work-plan 020). Mọi giá trị là giả thuyết (GT) ở phòng thử 1.000 HP.

use crate::status::StatusSpec;
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

/// Hộp va chạm chữ nhật, mili-pixel, trục y hướng lên.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x0: i32,
    pub x1: i32,
    pub y0: i32,
    pub y1: i32,
}

impl Rect {
    pub const fn overlaps(&self, other: &Rect) -> bool {
        self.x0 < other.x1 && other.x0 < self.x1 && self.y0 < other.y1 && other.y0 < self.y1
    }

    pub const fn center_x(&self) -> i32 {
        self.x0 + (self.x1 - self.x0) / 2
    }

    pub const fn shifted(self, dx: i32) -> Self {
        Self {
            x0: self.x0 + dx,
            x1: self.x1 + dx,
            ..self
        }
    }
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
    /// Đặt hộp vào thế giới cho nhân vật đứng tại `(x, y)` quay về `facing`.
    pub const fn place(&self, x: i32, y: i32, facing: i8) -> Rect {
        let (x0, x1) = if facing > 0 {
            (x + self.front, x + self.front + self.width)
        } else {
            (x - self.front - self.width, x - self.front)
        };
        let y0 = y + self.bottom;
        Rect {
            x0,
            x1,
            y0,
            y1: y0 + self.height,
        }
    }

    pub(crate) const fn px(front: i32, width: i32, bottom: i32, height: i32) -> Self {
        Self {
            front: front * PX,
            width: width * PX,
            bottom: bottom * PX,
            height: height * PX,
        }
    }
}

/// Đạn bay thẳng, sinh ở tick active đầu tiên của đòn; không homing (combat.md §7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectileSpec {
    /// Hộp va chạm lúc sinh, đặt như `Hitbox` từ chân người bắn.
    pub hitbox: Hitbox,
    /// Mili-pixel mỗi tick.
    pub speed: i32,
    /// Số tick tồn tại, tính cả tick sinh; giới hạn tầm bắn.
    pub lifetime: u32,
    /// `true`: xuyên qua, mỗi mục tiêu trúng một lần. `false`: biến mất ở mục tiêu đầu tiên.
    pub pierce: bool,
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
    /// Đòn bắn đạn: damage đi theo đạn, đòn không có hitbox cận chiến.
    pub projectile: Option<ProjectileSpec>,
    /// Hiệu ứng áp lên mục tiêu khi trúng (cả cận chiến lẫn đạn); không áp khi bị đỡ.
    pub on_hit: Option<StatusSpec>,
}

impl ActionSpec {
    /// Đòn chỉ có timing (ms); các trường khác bằng 0 để ghi đè bằng struct update.
    pub(crate) const fn timed(
        name: &'static str,
        startup_ms: u32,
        active_ms: u32,
        recovery_ms: u32,
    ) -> Self {
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
            projectile: None,
            on_hit: None,
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

/// Thông số cơ thể theo kit: người chơi dùng `HUMAN`, boss có cơ thể riêng.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Body {
    pub max_hp: u32,
    pub half_width: i32,
    pub height: i32,
    /// Mili-pixel mỗi tick.
    pub walk_speed: i32,
    /// Nhận damage nhưng không bị choáng hay đẩy lùi; boss dùng luật riêng (combat.md §6).
    pub armored: bool,
}

/// Cơ thể chuẩn ở phòng thử 1.000 HP.
pub const HUMAN: Body = Body {
    max_hp: 1_000,
    half_width: 20 * PX,
    height: 80 * PX,
    walk_speed: 4 * PX,
    armored: false,
};

/// 2 đòn cơ bản (đòn nhẹ là chuỗi 3 nhịp) + 3 thuật được trang bị.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Kit {
    /// Mã ASCII ổn định dùng trong replay và cấu hình.
    pub id: &'static str,
    pub lineage: &'static str,
    pub body: Body,
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
    id: "long-luu",
    lineage: "Long Lưu",
    body: HUMAN,
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
        // Đạn thẳng, một hit mỗi mục tiêu, tầm khoảng 500 px (GT).
        ActionSpec {
            energy_cost: 20,
            cooldown: ms_to_ticks(3_000),
            damage: 80,
            guard_pressure: 12,
            hitstun: ms_to_ticks(180),
            projectile: Some(ProjectileSpec {
                hitbox: Hitbox::px(20, 24, 30, 24),
                speed: 16 * PX,
                lifetime: ms_to_ticks(500),
                pierce: true,
            }),
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
