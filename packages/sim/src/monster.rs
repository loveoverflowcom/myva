//! Quái thường ở phòng thử: một đòn cận chiến và một thuật gây Slow (combat.md §6, §8).
//!
//! Quái là một `Fighter` có kit riêng; [`MonsterBrain`] sinh input như người chơi, nên server
//! kiểm tra lệnh quái bằng cùng luật. Thêm loại quái mới là thêm kit và bộ não, không sửa vòng
//! lặp tick. Tên, chỉ số và timing là giả thuyết (GT) hư cấu, chưa cân bằng.

use crate::fighter::{Fighter, FighterId, State};
use crate::input::{Buttons, InputFrame};
use crate::kit::{ActionKind, ActionSpec, Body, Hitbox, Kit, PX, ProjectileSpec};
use crate::rng::Rng;
use crate::status::{StatusKind, StatusSpec};
use crate::tick::ms_to_ticks;
use crate::world::World;

/// Quái thường dùng tín hiệu ngắn hơn boss nhưng vẫn đủ phản ứng (combat.md §10, GT).
pub const MONSTER_MIN_TELEGRAPH: u32 = ms_to_ticks(450);

const CLAW: ActionSpec = ActionSpec {
    damage: 60,
    guard_pressure: 12,
    hitstun: ms_to_ticks(150),
    hitbox: Hitbox::px(5, 45, 10, 40),
    ..ActionSpec::timed("Cào Bùn", 450, 100, 400)
};

/// Thuật duy nhất: cục bùn bay chậm, trúng thì Slow 20% trong 1,5 giây.
const MUD_SHOT: ActionSpec = ActionSpec {
    energy_cost: 30,
    cooldown: ms_to_ticks(4_000),
    damage: 35,
    guard_pressure: 8,
    hitstun: ms_to_ticks(120),
    projectile: Some(ProjectileSpec {
        hitbox: Hitbox::px(20, 20, 20, 20),
        speed: 8 * PX,
        lifetime: ms_to_ticks(900),
        pierce: false,
    }),
    on_hit: Some(StatusSpec {
        kind: StatusKind::Slow,
        percent: 20,
        duration: ms_to_ticks(1_500),
    }),
    ..ActionSpec::timed("Phun Bùn", 550, 100, 450)
};

/// Ô đòn: nhẹ = Cào Bùn, thuật 1 = Phun Bùn. Ô còn lại lặp lại vì bộ não không dùng.
pub const QUAI_BUN: Kit = Kit {
    id: "quai-bun",
    lineage: "Quái bùn lệch nhịp",
    body: Body {
        max_hp: 300,
        half_width: 22 * PX,
        height: 56 * PX,
        walk_speed: 3 * PX,
        armored: false,
    },
    light: [CLAW; 3],
    heavy: CLAW,
    skills: [MUD_SHOT; 3],
};

/// Trạng thái AI công khai cho presentation/debug; luật không đọc nó.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AiState {
    /// Không có mục tiêu trong tầm phát hiện.
    Idle,
    Chase {
        target: FighterId,
    },
    /// Đang thực hiện đòn đã chọn (startup/active/recovery do luật quyết định).
    Attack {
        target: FighterId,
    },
    /// Nghỉ sau đòn hoặc bị choáng; độ dài nghỉ lấy từ RNG có seed.
    Recover,
    Defeated,
}

/// Tầm phát hiện mục tiêu.
const AGGRO_RANGE: i32 = 420 * PX;
/// Khoảng cách tâm–tâm tối đa để bắt đầu Cào Bùn.
const CLAW_RANGE: i32 = 70 * PX;
/// Ngoài tầm cào nhưng trong tầm này thì có thể Phun Bùn.
const SHOT_RANGE: i32 = 360 * PX;
const REST_MIN: u32 = ms_to_ticks(300);
const REST_SPREAD: u32 = ms_to_ticks(500);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonsterBrain {
    seq: u32,
    rng: Rng,
    rest: u32,
    state: AiState,
}

impl MonsterBrain {
    pub const fn new(seed: u64) -> Self {
        Self {
            seq: 0,
            rng: Rng::new(seed),
            rest: 0,
            state: AiState::Idle,
        }
    }

    pub fn state(&self) -> AiState {
        self.state
    }

    /// Khung input cho quái `me`, đọc trạng thái đầu tick như người chơi thấy trên màn hình.
    pub fn next_frame(&mut self, world: &World, me: FighterId) -> InputFrame {
        self.seq += 1;
        let mut frame = InputFrame {
            seq: self.seq,
            ..InputFrame::default()
        };
        let body = world.fighter(me);
        match body.state {
            State::Downed => {
                self.state = AiState::Defeated;
                return frame;
            }
            State::Attack { .. } => return frame,
            State::Neutral => {}
            _ => {
                self.state = AiState::Recover;
                return frame;
            }
        }
        if self.rest > 0 {
            self.rest -= 1;
            self.state = AiState::Recover;
            return frame;
        }

        let Some(target) = nearest_hostile(world, body) else {
            self.state = AiState::Idle;
            return frame;
        };
        let dx = target.x - body.x;
        let distance = dx.abs();
        // Hướng mặt chỉ đổi ở trạng thái trung tính, ngay trước startup.
        frame.move_x = dx.signum() as i8;
        let shot = body.kit.spec(ActionKind::Skill(0));
        let can_shoot = body.cooldowns[0] == 0 && body.energy.can_spend(shot.energy_cost);
        let attack = if distance <= CLAW_RANGE {
            Some(Buttons::LIGHT)
        } else if distance <= SHOT_RANGE && can_shoot && self.rng.chance(1, 20) {
            Some(Buttons::SKILL1)
        } else {
            None
        };
        match attack {
            Some(button) => {
                frame.pressed = button;
                self.rest = REST_MIN + self.rng.below(REST_SPREAD);
                self.state = AiState::Attack { target: target.id };
            }
            None => self.state = AiState::Chase { target: target.id },
        }
        frame
    }
}

/// Mục tiêu còn sống gần nhất không cùng phe, trong tầm phát hiện; hòa thì ID nhỏ hơn.
fn nearest_hostile<'a>(world: &'a World, me: &Fighter) -> Option<&'a Fighter> {
    world
        .fighters()
        .iter()
        .filter(|f| f.id != me.id && !me.is_ally(f) && f.state != State::Downed)
        .map(|f| ((f.x - me.x).abs(), f))
        .filter(|(distance, _)| *distance <= AGGRO_RANGE)
        .min_by_key(|(distance, f)| (*distance, f.id))
        .map(|(_, f)| f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fighter::Team;
    use crate::kit::LONG_LUU;
    use crate::world::Event;

    const PLAYERS: Option<Team> = Some(Team(1));
    const HOSTILE: Option<Team> = Some(Team(2));

    #[test]
    fn every_monster_attack_is_telegraphed() {
        for spec in QUAI_BUN.light.iter().chain(&QUAI_BUN.skills) {
            assert!(
                spec.startup >= MONSTER_MIN_TELEGRAPH,
                "{} báo quá ngắn",
                spec.name
            );
        }
    }

    #[test]
    fn chases_then_claws_a_player_in_range() {
        let mut world = World::new();
        let player = world.spawn_in_team(&LONG_LUU, 600 * PX, 1, PLAYERS);
        let monster = world.spawn_in_team(&QUAI_BUN, 900 * PX, -1, HOSTILE);
        let mut brain = MonsterBrain::new(1);
        let mut events = Vec::new();
        let mut seen = Vec::new();
        for _ in 0..400 {
            let frame = brain.next_frame(&world, monster);
            seen.push(brain.state());
            events.extend(world.step(&[(monster, frame)]));
        }
        assert!(seen.contains(&AiState::Chase { target: player }));
        assert!(seen.contains(&AiState::Attack { target: player }));
        assert!(events.contains(&Event::Hit {
            attacker: monster,
            target: player,
            action: ActionKind::Light(0),
            damage: 60,
        }));
    }

    #[test]
    fn ignores_allies_and_targets_out_of_range() {
        let mut world = World::new();
        world.spawn_in_team(&QUAI_BUN, 300 * PX, 1, HOSTILE);
        let monster = world.spawn_in_team(&QUAI_BUN, 340 * PX, -1, HOSTILE);
        world.spawn_in_team(&LONG_LUU, 1_500 * PX, 1, PLAYERS);
        let mut brain = MonsterBrain::new(9);
        for _ in 0..120 {
            let frame = brain.next_frame(&world, monster);
            assert_eq!(frame.pressed, Buttons::NONE);
            assert_eq!(brain.state(), AiState::Idle);
            world.step(&[(monster, frame)]);
        }
    }

    #[test]
    fn mud_shot_slows_the_target_walk() {
        let mut world = World::new();
        let player = world.spawn_in_team(&LONG_LUU, 400 * PX, 1, PLAYERS);
        let monster = world.spawn_in_team(&QUAI_BUN, 600 * PX, -1, HOSTILE);
        world.step(&[(
            monster,
            InputFrame {
                seq: 1,
                pressed: Buttons::SKILL1,
                ..InputFrame::default()
            },
        )]);
        let mut events = Vec::new();
        while !events
            .iter()
            .any(|e| matches!(e, Event::StatusApplied { .. }))
        {
            assert!(world.tick() < 120, "Phun Bùn phải trúng");
            events.extend(world.step(&[]));
        }
        assert_eq!(world.fighter(player).statuses.slow_percent(), 20);
        // Hết hitstun rồi đi bộ: 80% tốc độ chuẩn.
        while world.fighter(player).state != State::Neutral {
            world.step(&[]);
        }
        let before = world.fighter(player).x;
        let seq = world.fighter(player).last_seq().unwrap_or(0) + 1;
        world.step(&[(
            player,
            InputFrame {
                seq,
                move_x: -1,
                ..InputFrame::default()
            },
        )]);
        assert_eq!(
            before - world.fighter(player).x,
            LONG_LUU.body.walk_speed * 80 / 100
        );
    }

    #[test]
    fn same_seed_same_decisions() {
        let run = |seed| {
            let mut world = World::new();
            world.spawn_in_team(&LONG_LUU, 500 * PX, 1, PLAYERS);
            let monster = world.spawn_in_team(&QUAI_BUN, 800 * PX, -1, HOSTILE);
            let mut brain = MonsterBrain::new(seed);
            (0..900)
                .map(|_| {
                    let frame = brain.next_frame(&world, monster);
                    world.step(&[(monster, frame)]);
                    world.state_hash()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(run(3), run(3));
        assert_ne!(run(3), run(4));
    }
}
