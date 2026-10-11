//! Đạn đang bay (combat.md §7): không homing, một hit mỗi mục tiêu, biến mất khi hết tầm hoặc
//! chạm mép arena. Mép arena là vật cản duy nhất của graybox.

use crate::fighter::{ARENA_WIDTH, Fighter, FighterId};
use crate::kit::{ActionKind, Phase, Rect};

/// ID miền của một viên đạn: tăng dần, không tái sử dụng trong một `World`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectileId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Projectile {
    pub id: ProjectileId,
    pub owner: FighterId,
    pub action: ActionKind,
    pub rect: Rect,
    pub dir: i8,
    pub(crate) speed: i32,
    pub(crate) remaining: u32,
    pub(crate) pierce: bool,
    pub(crate) hit_set: Vec<FighterId>,
}

impl Projectile {
    /// Đạn sinh ra ở tick này nếu `fighter` vừa vào tick active đầu tiên của một đòn bắn đạn.
    pub(crate) fn launch(fighter: &Fighter, id: ProjectileId) -> Option<Self> {
        let (action, spec, phase) = fighter.action()?;
        let projectile = spec.projectile?;
        let crate::fighter::State::Attack { elapsed, .. } = fighter.state else {
            return None;
        };
        (phase == Phase::Active && elapsed == spec.startup).then(|| Self {
            id,
            owner: fighter.id,
            action,
            rect: projectile
                .hitbox
                .place(fighter.x, fighter.y, fighter.facing),
            dir: fighter.facing,
            speed: projectile.speed,
            remaining: projectile.lifetime,
            pierce: projectile.pierce,
            hit_set: Vec::new(),
        })
    }

    /// Bay một tick; trả `false` khi đạn hết tầm hoặc ra khỏi arena.
    pub(crate) fn fly(&mut self) -> bool {
        self.rect = self.rect.shifted(i32::from(self.dir) * self.speed);
        self.remaining = self.remaining.saturating_sub(1);
        self.is_alive()
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.remaining > 0 && self.rect.x1 > 0 && self.rect.x0 < ARENA_WIDTH
    }

    /// Điểm đạn bay tới từ đó, để luật đỡ "chỉ chặn phía trước" áp dụng như đòn cận chiến.
    pub(crate) fn origin_for(&self, target_x: i32) -> i32 {
        target_x - i32::from(self.dir)
    }

    pub fn remaining(&self) -> u32 {
        self.remaining
    }
}
