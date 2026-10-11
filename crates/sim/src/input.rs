//! Input theo tick của người chơi hoặc bot (combat.md §4, §12).
//!
//! Bot đi qua cùng kiểu input này; không gọi thẳng hàm gây damage
//! (combat-progression-balance.md §11).

use std::ops::{BitOr, BitOrAssign};

/// Tập nút theo hành động, không theo phím vật lý; mapping phím nằm ở client.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Buttons(u16);

impl Buttons {
    pub const NONE: Self = Self(0);
    pub const JUMP: Self = Self(1 << 0);
    pub const DASH: Self = Self(1 << 1);
    pub const GUARD: Self = Self(1 << 2);
    pub const LIGHT: Self = Self(1 << 3);
    pub const HEAVY: Self = Self(1 << 4);
    pub const SKILL1: Self = Self(1 << 5);
    pub const SKILL2: Self = Self(1 << 6);
    pub const SKILL3: Self = Self(1 << 7);
    pub const INTERACT: Self = Self(1 << 8);

    /// Mọi nút đã định nghĩa; bit khác bị coi là dữ liệu hỏng.
    pub const ALL: [Self; 9] = [
        Self::JUMP,
        Self::DASH,
        Self::GUARD,
        Self::LIGHT,
        Self::HEAVY,
        Self::SKILL1,
        Self::SKILL2,
        Self::SKILL3,
        Self::INTERACT,
    ];

    /// Dựng từ bitflag, `None` nếu có bit lạ.
    pub fn from_bits(bits: u16) -> Option<Self> {
        let known = Self::ALL.iter().fold(0, |acc, b| acc | b.0);
        (bits & !known == 0).then_some(Self(bits))
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        other.0 != 0 && self.0 & other.0 == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for Buttons {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Buttons {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Một khung input. `seq` tăng dần; khung trùng hoặc cũ bị từ chối để mỗi input chỉ gây một
/// hành động.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InputFrame {
    pub seq: u32,
    /// -1 trái, 0 đứng, 1 phải.
    pub move_x: i8,
    /// Nút đang giữ (đỡ).
    pub held: Buttons,
    /// Nút vừa nhấn trong khung này (đánh, thuật, nhảy, lướt).
    pub pressed: Buttons,
}

/// Ý định hành động rút từ `pressed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Intent {
    /// Ô thuật `0..=2`.
    Skill(u8),
    Heavy,
    Light,
    Dash,
    Jump,
    /// Tương tác NPC trong tầm; ưu tiên thấp nhất nên không chặn đòn đánh.
    Interact,
}

const PRIORITY: [(Buttons, Intent); 8] = [
    (Buttons::SKILL1, Intent::Skill(0)),
    (Buttons::SKILL2, Intent::Skill(1)),
    (Buttons::SKILL3, Intent::Skill(2)),
    (Buttons::HEAVY, Intent::Heavy),
    (Buttons::LIGHT, Intent::Light),
    (Buttons::DASH, Intent::Dash),
    (Buttons::JUMP, Intent::Jump),
    (Buttons::INTERACT, Intent::Interact),
];

impl InputFrame {
    /// Một ý định mỗi khung, theo thứ tự ưu tiên cố định; các nút còn lại bị bỏ qua.
    pub fn intent(&self) -> Option<Intent> {
        PRIORITY
            .iter()
            .find(|(button, _)| self.pressed.contains(*button))
            .map(|&(_, intent)| intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_one_intent_by_priority() {
        let frame = InputFrame {
            pressed: Buttons::JUMP | Buttons::LIGHT | Buttons::SKILL3,
            ..InputFrame::default()
        };
        assert_eq!(frame.intent(), Some(Intent::Skill(2)));
        assert_eq!(InputFrame::default().intent(), None);
    }

    #[test]
    fn guard_alone_is_not_an_intent() {
        let frame = InputFrame {
            held: Buttons::GUARD,
            pressed: Buttons::GUARD,
            ..InputFrame::default()
        };
        assert_eq!(frame.intent(), None);
        assert!(!Buttons::NONE.contains(Buttons::NONE));
    }

    #[test]
    fn interact_never_beats_combat_buttons() {
        let frame = InputFrame {
            pressed: Buttons::INTERACT | Buttons::JUMP,
            ..InputFrame::default()
        };
        assert_eq!(frame.intent(), Some(Intent::Jump));
        assert_eq!(Buttons::from_bits(1 << 15), None);
        assert_eq!(Buttons::from_bits(0x1FF).map(Buttons::bits), Some(0x1FF));
    }
}
