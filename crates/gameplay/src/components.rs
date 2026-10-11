//! Component miền MyVa trên entity Bevy.
//!
//! **Built-in và miền.** Crate này không dùng component built-in nào của Bevy: `Transform`,
//! sprite, camera hay animation thuộc client presentation và được suy ra từ [`Position`] (mili-pixel,
//! trục y hướng lên) ở `Update`. Component ở đây là **mirror** trạng thái authoritative của
//! `myva-sim`: số nguyên, chỉ hệ mirror ghi, và đều `#[component(immutable)]` nên system khác
//! không lấy được `&mut` để sửa luật qua ECS. Đổi giá trị là thay cả component, nên change
//! detection chỉ báo khi lõi thật sự đổi.

use bevy_ecs::component::Component;
use myva_sim::kit::{ActionKind, Kit, Rect};
use myva_sim::monster::AiState;
use myva_sim::npc::NpcSpec;
use myva_sim::snapshot::{ActionView, Gauge};
use myva_sim::status::StatusEffect;
use myva_sim::{EntityRef, FighterId, State};

/// ID miền của entity; ổn định suốt `World`, khác handle `Entity` cục bộ của Bevy.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[component(immutable)]
pub struct SimId(pub EntityRef);

/// Nhân vật do người chơi điều khiển.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[component(immutable)]
pub struct Player;

/// Nhân vật do bộ não AI phía server điều khiển.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[component(immutable)]
pub struct Monster;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[component(immutable)]
pub struct Npc;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[component(immutable)]
pub struct Projectile;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct KitRef(pub &'static Kit);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct NpcKind(pub &'static NpcSpec);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Faction(pub Option<myva_sim::Team>);

/// Mili-pixel, y hướng lên; với đạn là tâm ngang và mép dưới của hộp va chạm.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Facing(pub i8);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Health {
    pub hp: u32,
    pub max: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Stamina(pub Gauge);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Energy(pub Gauge);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Mach(pub Gauge);

/// Tư thế chiến đấu: trung tính, đòn, đỡ, lướt, hitstun, vỡ thế, bị hạ.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Stance(pub State);

/// Đòn đang thực hiện và pha của nó, cho animation/telegraph.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct CurrentAction(pub Option<ActionView>);

/// Tick hồi chiêu còn lại của ba ô thuật.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Cooldowns(pub [u32; 3]);

/// Buff/debuff đang hiệu lực, tối đa một mỗi loại.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct StatusEffects(pub Vec<StatusEffect>);

/// Hurt volume của thân.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Hurtbox(pub Rect);

/// Hit volume cận chiến khi đòn đang active; hit volume của đạn là [`Volume`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct AttackBox(pub Option<Rect>);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Invulnerable(pub bool);

/// Đứng trên nền (không nhảy, không rơi).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Grounded(pub bool);

/// Thế đỡ đã có hiệu lực: `Some(true)` trong cửa sổ đỡ hoàn hảo.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct GuardWindow(pub Option<bool>);

/// `seq` lớn nhất lõi đã áp dụng, để client hòa giải input dự đoán.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct AckedSeq(pub Option<u32>);

/// Trạng thái AI công khai; luật không đọc component này.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Ai(pub AiState);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Owner(pub FighterId);

/// Đòn đã sinh ra viên đạn.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct SourceAction(pub ActionKind);

/// Hộp va chạm của đạn.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Volume(pub Rect);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub struct Heading(pub i8);
