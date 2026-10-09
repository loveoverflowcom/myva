//! Boss graybox Kẻ Giữ Đập (combat.md §9): đấu trường phẳng, ba pha theo sinh lực.
//!
//! Boss là một `Fighter` có cơ thể giáp và kit riêng; [`BossBrain`] sinh input như một người
//! chơi, nên server sau này kiểm tra lệnh boss bằng cùng luật. Mọi đòn có tín hiệu báo trước
//! (startup) ít nhất 700 ms; đòn nào cũng có cách né: nhảy, lướt, đứng sát hoặc ra sau lưng.
//!
//! Chưa có: hai bệ cao, van điều tiết làm lộ điểm yếu, co-op và gợi ý sau ba lần thất bại.

use crate::fighter::{FighterId, State};
use crate::input::{Buttons, InputFrame};
use crate::kit::{ActionSpec, Body, Hitbox, Kit, PX, ProjectileSpec};
use crate::tick::ms_to_ticks;
use crate::world::World;

/// Tín hiệu tối thiểu cho đòn boss bắt buộc người mới phản ứng (combat.md §10).
pub const MIN_TELEGRAPH: u32 = ms_to_ticks(700);

/// Nhịp giới thiệu an toàn khi vào pha mới.
pub const PHASE_INTRO: u32 = ms_to_ticks(1_500);

const SWEEP: ActionSpec = ActionSpec {
    damage: 120,
    guard_pressure: 30,
    hitstun: ms_to_ticks(300),
    knockback: 40 * PX,
    // Thấp: nhảy qua được.
    hitbox: Hitbox::px(0, 220, 0, 50),
    ..ActionSpec::timed("Quét Ngang", 800, 200, 600)
};

const SLAM: ActionSpec = ActionSpec {
    damage: 180,
    guard_pressure: 60,
    hitstun: ms_to_ticks(400),
    knockback: 80 * PX,
    // Phủ cả hai bên thân boss: phải lướt hoặc đi ra xa.
    hitbox: Hitbox::px(-120, 240, 0, 70),
    ..ActionSpec::timed("Đập Đất", 1_000, 150, 800)
};

const FLOOD: ActionSpec = ActionSpec {
    damage: 100,
    guard_pressure: 20,
    hitstun: ms_to_ticks(300),
    // Nước dâng một phần nền phía trước; sát chân boss là vùng an toàn.
    hitbox: Hitbox::px(160, 640, 0, 30),
    ..ActionSpec::timed("Nước Dâng", 1_600, 400, 700)
};

const ARC: ActionSpec = ActionSpec {
    damage: 90,
    guard_pressure: 20,
    hitstun: ms_to_ticks(250),
    projectile: Some(ProjectileSpec {
        hitbox: Hitbox::px(60, 28, 20, 28),
        speed: 7 * PX,
        lifetime: ms_to_ticks(2_000),
        pierce: false,
    }),
    ..ActionSpec::timed("Đạn Vòng Cung", 900, 100, 600)
};

/// Ô đòn: nhẹ = Quét Ngang, nặng = Đập Đất, thuật 1 = Nước Dâng, thuật 2 = Đạn Vòng Cung.
/// Thuật 3 để trống (lặp Quét Ngang): pha 3 nối các đòn đã học thay vì thêm đòn mới.
pub const KE_GIU_DAP: Kit = Kit {
    lineage: "Kẻ Giữ Đập",
    body: Body {
        max_hp: 2_400,
        half_width: 50 * PX,
        height: 140 * PX,
        walk_speed: 2 * PX,
        armored: true,
    },
    light: [SWEEP; 3],
    heavy: SLAM,
    skills: [FLOOD, ARC, SWEEP],
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BossPhase {
    /// 100–70% HP: Quét Ngang và Đập Đất, nhịp nghỉ đủ để thử phản công.
    Recognize,
    /// 70–35% HP: thêm Nước Dâng và Đạn Vòng Cung.
    Terrain,
    /// Dưới 35% HP: nối hai đòn đã học, mỗi tín hiệu vẫn đủ dài.
    Combine,
}

impl BossPhase {
    pub fn for_hp(hp: u32, max_hp: u32) -> Self {
        let percent = u64::from(hp) * 100;
        let max = u64::from(max_hp);
        if percent > max * 70 {
            BossPhase::Recognize
        } else if percent >= max * 35 {
            BossPhase::Terrain
        } else {
            BossPhase::Combine
        }
    }

    fn pattern(self) -> &'static [Move] {
        use Move::*;
        match self {
            BossPhase::Recognize => &[Sweep, Slam],
            BossPhase::Terrain => &[Sweep, Arc, Slam, Flood],
            BossPhase::Combine => &[Sweep, SlamChained, Arc, Flood],
        }
    }

    /// Nhịp nghỉ sau mỗi đòn, ngoài recovery của chính đòn.
    fn rest(self) -> u32 {
        match self {
            BossPhase::Recognize => ms_to_ticks(500),
            BossPhase::Terrain => ms_to_ticks(350),
            BossPhase::Combine => ms_to_ticks(200),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Move {
    Sweep,
    Slam,
    /// Đập Đất ngay sau đòn trước, không nghỉ thêm.
    SlamChained,
    Flood,
    Arc,
}

impl Move {
    fn button(self) -> Buttons {
        match self {
            Move::Sweep => Buttons::LIGHT,
            Move::Slam | Move::SlamChained => Buttons::HEAVY,
            Move::Flood => Buttons::SKILL1,
            Move::Arc => Buttons::SKILL2,
        }
    }

    /// Khoảng cách tối đa tới mục tiêu để bắt đầu đòn; xa hơn thì boss đi tới.
    fn reach(self) -> i32 {
        match self {
            Move::Sweep => 200 * PX,
            Move::Slam | Move::SlamChained => 110 * PX,
            Move::Flood | Move::Arc => 900 * PX,
        }
    }
}

/// Bộ não theo pattern: cố định, không ngẫu nhiên, để người chơi học được.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BossBrain {
    seq: u32,
    phase: BossPhase,
    step: usize,
    rest: u32,
}

impl Default for BossBrain {
    fn default() -> Self {
        Self::new()
    }
}

impl BossBrain {
    pub fn new() -> Self {
        Self {
            seq: 0,
            phase: BossPhase::Recognize,
            step: 0,
            rest: PHASE_INTRO,
        }
    }

    pub fn phase(&self) -> BossPhase {
        self.phase
    }

    /// Khung input cho boss `me` nhắm vào `target`. Chỉ quyết định khi boss ở trạng thái trung
    /// tính, nên đổi pha không bao giờ cắt ngang một đòn đang báo.
    pub fn next_frame(&mut self, world: &World, me: FighterId, target: FighterId) -> InputFrame {
        self.seq += 1;
        let mut frame = InputFrame {
            seq: self.seq,
            ..InputFrame::default()
        };
        let boss = world.fighter(me);
        let target = world.fighter(target);
        if boss.state != State::Neutral || target.state == State::Downed {
            return frame;
        }

        let phase = BossPhase::for_hp(boss.hp, boss.kit.body.max_hp);
        if phase != self.phase {
            self.phase = phase;
            self.step = 0;
            self.rest = PHASE_INTRO;
        }
        if self.rest > 0 {
            self.rest -= 1;
            return frame;
        }

        let pattern = self.phase.pattern();
        let next = pattern[self.step];
        let dx = target.x - boss.x;
        frame.move_x = dx.signum() as i8;
        if dx.abs() > next.reach() {
            return frame;
        }
        // Hướng mặt đổi theo `move_x` ngay trước startup, rồi giữ nguyên suốt đòn.
        frame.pressed = next.button();
        self.step = (self.step + 1) % pattern.len();
        self.rest = match pattern[self.step] {
            Move::SlamChained => 0,
            _ => self.phase.rest(),
        };
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{ActionKind, LONG_LUU, Phase};
    use crate::world::Event;

    #[test]
    fn every_boss_attack_is_telegraphed() {
        let kit = &KE_GIU_DAP;
        for spec in kit.light.iter().chain([&kit.heavy]).chain(&kit.skills) {
            assert!(spec.startup >= MIN_TELEGRAPH, "{} báo quá ngắn", spec.name);
            assert_eq!(
                (spec.energy_cost, spec.stamina_cost, spec.cooldown),
                (0, 0, 0)
            );
        }
    }

    #[test]
    fn phases_follow_hp_thresholds() {
        let max = KE_GIU_DAP.body.max_hp;
        assert_eq!(BossPhase::for_hp(max, max), BossPhase::Recognize);
        assert_eq!(BossPhase::for_hp(max * 70 / 100, max), BossPhase::Terrain);
        assert_eq!(BossPhase::for_hp(max * 35 / 100, max), BossPhase::Terrain);
        assert_eq!(
            BossPhase::for_hp(max * 35 / 100 - 1, max),
            BossPhase::Combine
        );
    }

    #[test]
    fn armor_keeps_boss_attacking_when_hit() {
        let mut world = World::new();
        let player = world.spawn(&LONG_LUU, 800 * PX, 1);
        let boss = world.spawn(&KE_GIU_DAP, 900 * PX, -1);
        let mut brain = BossBrain {
            rest: 0,
            ..BossBrain::new()
        };
        let frame = brain.next_frame(&world, boss, player);
        assert_eq!(
            frame.pressed,
            Buttons::LIGHT,
            "mục tiêu trong tầm thì Quét Ngang"
        );
        let jab = InputFrame {
            seq: 1,
            pressed: Buttons::LIGHT,
            ..InputFrame::default()
        };
        world.step(&[(boss, frame), (player, jab)]);
        let events: Vec<Event> = (0..15).flat_map(|_| world.step(&[])).collect();
        assert!(events.contains(&Event::Hit {
            attacker: player,
            target: boss,
            action: ActionKind::Light(0),
            damage: 45,
        }));
        assert!(
            matches!(world.fighter(boss).action(), Some((_, _, Phase::Startup))),
            "boss giáp vẫn tiếp tục báo đòn sau khi bị trúng"
        );
    }

    #[test]
    fn phase_change_waits_for_neutral_and_grants_intro() {
        let mut world = World::new();
        let player = world.spawn(&LONG_LUU, 800 * PX, 1);
        let boss = world.spawn(&KE_GIU_DAP, 900 * PX, -1);
        let mut brain = BossBrain {
            rest: 0,
            ..BossBrain::new()
        };
        let frame = brain.next_frame(&world, boss, player);
        world.step(&[(boss, frame)]);

        // Mất nửa máu khi đang báo đòn: pha chưa đổi cho tới khi đòn kết thúc.
        world.fighters_mut()[usize::from(boss.0)].hp = KE_GIU_DAP.body.max_hp / 2;
        brain.next_frame(&world, boss, player);
        assert_eq!(brain.phase(), BossPhase::Recognize);

        while world.fighter(boss).state != State::Neutral {
            world.step(&[]);
        }
        let frame = brain.next_frame(&world, boss, player);
        assert_eq!(brain.phase(), BossPhase::Terrain);
        assert!(
            frame.pressed.is_empty(),
            "vào pha mới bằng nhịp nghỉ, không ra đòn ngay"
        );
        assert_eq!(brain.rest, PHASE_INTRO - 1);
    }
}
