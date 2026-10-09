//! Thế giới phòng thử: bước tick tất định nhận input và trả sự kiện (architecture.md §2).
//!
//! Thứ tự mỗi tick: nhận input → di chuyển → đạn bay và sinh đạn → xử lý trúng đòn → tiến
//! bộ đếm.

use std::hash::{Hash, Hasher};

use crate::fighter::{Fighter, FighterId, State};
use crate::input::InputFrame;
use crate::kit::{ActionKind, Kit};
use crate::meter::SUB_PER_POINT;
use crate::projectile::Projectile;
use crate::tick::Tick;

/// Sự kiện do mô phỏng quyết định; client chỉ thể hiện, không tự tạo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    InputRejected {
        fighter: FighterId,
        seq: u32,
    },
    ActionStarted {
        fighter: FighterId,
        action: ActionKind,
    },
    Hit {
        attacker: FighterId,
        target: FighterId,
        action: ActionKind,
        damage: u32,
    },
    Blocked {
        attacker: FighterId,
        target: FighterId,
        action: ActionKind,
        perfect: bool,
    },
    Countered {
        counterer: FighterId,
        attacker: FighterId,
        damage: u32,
    },
    GuardBroken {
        fighter: FighterId,
    },
    Downed {
        fighter: FighterId,
    },
}

#[derive(Clone, Debug, Default, Hash)]
pub struct World {
    tick: Tick,
    fighters: Vec<Fighter>,
    projectiles: Vec<Projectile>,
    next_instance: u32,
}

/// Nguồn của một va chạm đang chờ áp dụng.
#[derive(Clone, Copy)]
enum Source {
    Melee,
    /// Chỉ số trong `World::projectiles`.
    Projectile(usize),
}

struct Strike {
    attacker: FighterId,
    target: FighterId,
    action: ActionKind,
    source: Source,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    /// Thêm nhân vật đứng trên nền tại `x` (mili-pixel).
    pub fn spawn(&mut self, kit: &'static Kit, x: i32, facing: i8) -> FighterId {
        let id = FighterId(u16::try_from(self.fighters.len()).expect("quá nhiều nhân vật"));
        self.fighters.push(Fighter::new(id, kit, x, facing));
        id
    }

    pub fn tick(&self) -> Tick {
        self.tick
    }

    pub fn fighters(&self) -> &[Fighter] {
        &self.fighters
    }

    pub fn fighter(&self, id: FighterId) -> &Fighter {
        &self.fighters[usize::from(id.0)]
    }

    pub fn projectiles(&self) -> &[Projectile] {
        &self.projectiles
    }

    #[cfg(test)]
    pub(crate) fn fighters_mut(&mut self) -> &mut [Fighter] {
        &mut self.fighters
    }

    /// Chạy một tick. Mỗi nhân vật nhận tối đa một khung input mỗi tick; khung thừa, trùng `seq`
    /// hoặc `seq` cũ bị từ chối.
    pub fn step(&mut self, inputs: &[(FighterId, InputFrame)]) -> Vec<Event> {
        let mut events = Vec::new();
        let mut frames: Vec<Option<InputFrame>> = vec![None; self.fighters.len()];
        for &(id, frame) in inputs {
            let index = usize::from(id.0);
            let accepted = match self.fighters.get_mut(index) {
                Some(fighter) => frames[index].is_none() && fighter.accept_seq(frame.seq),
                None => false,
            };
            if accepted {
                frames[index] = Some(frame);
            } else {
                events.push(Event::InputRejected {
                    fighter: id,
                    seq: frame.seq,
                });
            }
        }

        for (fighter, frame) in self.fighters.iter_mut().zip(frames) {
            fighter.apply_input(frame, &mut self.next_instance, &mut events);
        }
        for fighter in &mut self.fighters {
            fighter.integrate();
        }
        // Đạn cũ bay trước; đạn mới đứng tại chỗ sinh trong tick đầu tiên.
        self.projectiles.retain_mut(Projectile::fly);
        self.projectiles
            .extend(self.fighters.iter().filter_map(Projectile::launch));
        self.resolve_hits(&mut events);
        self.projectiles.retain(Projectile::is_alive);
        for fighter in &mut self.fighters {
            fighter.advance(&mut events);
        }
        self.tick += 1;
        events
    }

    /// Hash trạng thái để so replay trong cùng build; không dùng làm định dạng lưu trữ.
    pub fn state_hash(&self) -> u64 {
        let mut hasher = StableHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn resolve_hits(&mut self, events: &mut Vec<Event>) {
        // Gom va chạm từ cùng một trạng thái rồi mới áp dụng, để hai đòn trúng nhau cùng tick
        // đều được tính.
        let mut pending = Vec::new();
        for attacker in &self.fighters {
            let (Some(attack_box), Some((action, ..))) = (attacker.attack_box(), attacker.action())
            else {
                continue;
            };
            for target in self.targets(attacker.id, &attacker.hit_set) {
                if attack_box.overlaps(&target.hurtbox()) {
                    pending.push(Strike {
                        attacker: attacker.id,
                        target: target.id,
                        action,
                        source: Source::Melee,
                    });
                }
            }
        }
        for (index, projectile) in self.projectiles.iter().enumerate() {
            for target in self.targets(projectile.owner, &projectile.hit_set) {
                if projectile.rect.overlaps(&target.hurtbox()) {
                    pending.push(Strike {
                        attacker: projectile.owner,
                        target: target.id,
                        action: projectile.action,
                        source: Source::Projectile(index),
                    });
                    if !projectile.pierce {
                        break;
                    }
                }
            }
        }
        for strike in pending {
            self.apply_hit(strike, events);
        }
    }

    /// Những nhân vật có thể bị trúng bởi đòn của `owner` chưa trúng ai trong `hit_set`.
    fn targets<'a>(
        &'a self,
        owner: FighterId,
        hit_set: &'a [FighterId],
    ) -> impl Iterator<Item = &'a Fighter> {
        self.fighters.iter().filter(move |target| {
            target.id != owner && !hit_set.contains(&target.id) && !target.is_invulnerable()
        })
    }

    fn apply_hit(&mut self, strike: Strike, events: &mut Vec<Event>) {
        let Strike {
            attacker: attacker_id,
            target: target_id,
            action,
            source,
        } = strike;
        let (ai, ti) = (usize::from(attacker_id.0), usize::from(target_id.0));
        let kit: &'static Kit = self.fighters[ai].kit;
        let spec = kit.spec(action);
        // Một lần ra đòn hoặc một viên đạn chỉ trúng mỗi mục tiêu một lần.
        let origin_x = match source {
            Source::Melee => {
                self.fighters[ai].hit_set.push(target_id);
                self.fighters[ai].x
            }
            Source::Projectile(index) => {
                let projectile = &mut self.projectiles[index];
                if projectile.hit_set.contains(&target_id) {
                    return;
                }
                projectile.hit_set.push(target_id);
                if !projectile.pierce {
                    projectile.remaining = 0;
                }
                projectile.origin_for(self.fighters[ti].x)
            }
        };

        let target = &mut self.fighters[ti];
        let push_dir = if target.x >= origin_x { 1 } else { -1 };
        if target.faces(origin_x) {
            // Tư thế phản công chỉ bắt đòn cận chiến.
            if let (Source::Melee, Some((damage, hitstun))) = (source, target.counter_ready()) {
                target.hit_set.push(attacker_id);
                events.push(Event::Countered {
                    counterer: target_id,
                    attacker: attacker_id,
                    damage,
                });
                self.fighters[ai].take_hit(damage, hitstun, 0, events);
                return;
            }
            if let Some(perfect) = target.guard_active() {
                events.push(Event::Blocked {
                    attacker: attacker_id,
                    target: target_id,
                    action,
                    perfect,
                });
                target.mach.gain(spec.damage / 20);
                // Mỗi hit bị chặn tiêu 8 + 0,5 × áp lực đòn; perfect guard không tiêu.
                let cost = (16 + spec.guard_pressure) * SUB_PER_POINT / 2;
                if !perfect && target.stamina.drain_sub(cost) {
                    target.state = State::GuardBreak {
                        remaining: crate::fighter::GUARD_BREAK,
                    };
                    events.push(Event::GuardBroken { fighter: target_id });
                }
                return;
            }
        }

        events.push(Event::Hit {
            attacker: attacker_id,
            target: target_id,
            action,
            damage: spec.damage,
        });
        target.take_hit(spec.damage, spec.hitstun, push_dir * spec.knockback, events);
        let attacker = &mut self.fighters[ai];
        attacker.mach.gain(spec.damage / 10);
        // Chỉ hit cận chiến mở cửa nối thuật.
        if let (
            Source::Melee,
            State::Attack {
                elapsed,
                confirmed_at,
                ..
            },
        ) = (source, &mut attacker.state)
        {
            confirmed_at.get_or_insert(*elapsed);
        }
    }
}

/// FNV-1a 64-bit ghi số nguyên little-endian với độ rộng cố định, để cùng trạng thái cho cùng
/// hash trên native 64-bit và wasm32.
#[derive(Debug)]
struct StableHasher(u64);

impl Default for StableHasher {
    fn default() -> Self {
        Self(0xCBF2_9CE4_8422_2325)
    }
}

impl Hasher for StableHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0100_0000_01B3);
        }
    }

    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_le_bytes());
    }

    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_le_bytes());
    }

    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_le_bytes());
    }

    fn write_usize(&mut self, i: usize) {
        self.write_u64(i as u64);
    }

    fn write_isize(&mut self, i: isize) {
        self.write_u64(i as i64 as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fighter::{GUARD_STARTUP, MAX_HP};
    use crate::input::Buttons;
    use crate::kit::{LONG_LUU, PX};
    use crate::meter::Meter;

    const A: FighterId = FighterId(0);
    const B: FighterId = FighterId(1);

    /// A ở bên trái quay phải, B cách `distance` px; `b_facing` là hướng của B.
    fn duel(distance: i32, b_facing: i8) -> World {
        let mut world = World::new();
        world.spawn(&LONG_LUU, 400 * PX, 1);
        world.spawn(&LONG_LUU, (400 + distance) * PX, b_facing);
        world
    }

    fn press(seq: u32, pressed: Buttons) -> InputFrame {
        InputFrame {
            seq,
            pressed,
            ..InputFrame::default()
        }
    }

    fn hold(seq: u32, held: Buttons) -> InputFrame {
        InputFrame {
            seq,
            held,
            ..InputFrame::default()
        }
    }

    fn run_idle(world: &mut World, ticks: u32) -> Vec<Event> {
        (0..ticks).flat_map(|_| world.step(&[])).collect()
    }

    #[test]
    fn light_hits_on_first_active_tick_and_only_once() {
        let mut world = duel(60, -1);
        let mut events = world.step(&[(A, press(1, Buttons::LIGHT))]);
        let mut hit_tick = None;
        for _ in 0..40 {
            let tick = world.tick();
            let step = world.step(&[]);
            if hit_tick.is_none() && step.iter().any(|e| matches!(e, Event::Hit { .. })) {
                hit_tick = Some(tick);
            }
            events.extend(step);
        }
        assert_eq!(hit_tick, Some(11));
        let hits = events
            .iter()
            .filter(|e| matches!(e, Event::Hit { .. }))
            .count();
        assert_eq!(hits, 1);
        assert_eq!(world.fighter(B).hp, MAX_HP - 45);
    }

    #[test]
    fn frontal_guard_blocks_and_back_attack_lands() {
        for (b_facing, expect_block) in [(-1, true), (1, false)] {
            let mut world = duel(60, b_facing);
            let mut seq = 0;
            let mut b_guard = || {
                seq += 1;
                (B, hold(seq, Buttons::GUARD))
            };
            for _ in 0..20 {
                world.step(&[b_guard()]);
            }
            let mut events = world.step(&[(A, press(1, Buttons::LIGHT)), b_guard()]);
            for _ in 0..20 {
                events.extend(world.step(&[b_guard()]));
            }
            let blocked = events
                .iter()
                .any(|e| matches!(e, Event::Blocked { perfect: false, .. }));
            let hit = events.iter().any(|e| matches!(e, Event::Hit { .. }));
            assert_eq!(
                (blocked, hit),
                (expect_block, !expect_block),
                "B quay {b_facing}"
            );
        }
    }

    #[test]
    fn guard_startup_is_not_instant() {
        let mut world = duel(60, -1);
        // B bắt đầu đỡ cùng lúc A ra đòn: đòn trúng ở tick 11, sau startup đỡ 6 tick,
        // nên rơi vào perfect guard.
        const { assert!(GUARD_STARTUP < 11) };
        let mut events = world.step(&[(A, press(1, Buttons::LIGHT)), (B, hold(1, Buttons::GUARD))]);
        for seq in 2..20 {
            events.extend(world.step(&[(B, hold(seq, Buttons::GUARD))]));
        }
        assert!(events.contains(&Event::Blocked {
            attacker: A,
            target: B,
            action: ActionKind::Light(0),
            perfect: true,
        }));
        assert_eq!(world.fighter(B).hp, MAX_HP);
    }

    #[test]
    fn empty_stamina_breaks_guard() {
        let mut world = duel(60, -1);
        for seq in 1..=20 {
            world.step(&[(B, hold(seq, Buttons::GUARD))]);
        }
        world.fighters[1].stamina = Meter::new(100, 5, 20, 600);
        let mut events =
            world.step(&[(A, press(1, Buttons::LIGHT)), (B, hold(21, Buttons::GUARD))]);
        for seq in 22..40 {
            events.extend(world.step(&[(B, hold(seq, Buttons::GUARD))]));
        }
        assert!(events.contains(&Event::GuardBroken { fighter: B }));
        assert_eq!(world.fighter(B).hp, MAX_HP);
    }

    #[test]
    fn duplicate_or_old_seq_never_acts_twice() {
        let mut world = duel(300, -1);
        let mut events = world.step(&[(A, press(5, Buttons::SKILL1))]);
        events.extend(world.step(&[(A, press(5, Buttons::SKILL1))]));
        events.extend(world.step(&[(A, press(4, Buttons::SKILL1))]));
        events.extend(world.step(&[(A, press(6, Buttons::NONE)), (A, press(7, Buttons::LIGHT))]));
        let started = events
            .iter()
            .filter(|e| matches!(e, Event::ActionStarted { .. }))
            .count();
        let rejected: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::InputRejected { seq, .. } => Some(*seq),
                _ => None,
            })
            .collect();
        assert_eq!(started, 1);
        assert_eq!(rejected, [5, 4, 7]);
        assert_eq!(world.fighter(A).energy.points(), 80);
    }

    #[test]
    fn mashing_light_chains_three_steps_then_restarts() {
        let mut world = duel(600, -1);
        let mut started = Vec::new();
        for seq in 1..=120 {
            for event in world.step(&[(A, press(seq, Buttons::LIGHT))]) {
                if let Event::ActionStarted { action, .. } = event {
                    started.push(action);
                }
            }
        }
        assert_eq!(
            started[..4],
            [
                ActionKind::Light(0),
                ActionKind::Light(1),
                ActionKind::Light(2),
                ActionKind::Light(0)
            ]
        );
    }

    #[test]
    fn skill_cancel_needs_a_confirmed_hit() {
        for (distance, expect_cancel) in [(60, true), (600, false)] {
            let mut world = duel(distance, -1);
            let mut started = Vec::new();
            let mut collect = |events: Vec<Event>| {
                for event in events {
                    if let Event::ActionStarted { action, .. } = event {
                        started.push(action);
                    }
                }
            };
            collect(world.step(&[(A, press(1, Buttons::LIGHT))]));
            collect(run_idle(&mut world, 11));
            // Ý định nối thuật ngay sau tick trúng.
            collect(world.step(&[(A, press(2, Buttons::SKILL1))]));
            collect(run_idle(&mut world, 19));
            let cancelled = started.contains(&ActionKind::Skill(0));
            assert_eq!(cancelled, expect_cancel, "khoảng cách {distance}");
        }
    }

    #[test]
    fn skill_costs_are_atomic_and_cooldown_blocks_reuse() {
        let mut world = duel(600, -1);
        world.fighters[0].energy = Meter::new(100, 30, 0, 0);
        let events = world.step(&[(A, press(1, Buttons::SKILL3))]);
        assert!(events.is_empty(), "thiếu năng lượng thì không ra thuật");
        assert_eq!(world.fighter(A).energy.points(), 30);

        world.step(&[(A, press(2, Buttons::SKILL1))]);
        assert_eq!(world.fighter(A).energy.points(), 10);
        run_idle(&mut world, 60);
        world.fighters[0].energy = Meter::energy();
        let events = world.step(&[(A, press(3, Buttons::SKILL1))]);
        assert!(events.is_empty(), "Lưu Tiễn còn hồi chiêu 3 giây");
    }

    #[test]
    fn counter_stance_punishes_frontal_attack() {
        let mut world = duel(60, -1);
        let mut events = world.step(&[
            (A, press(1, Buttons::LIGHT)),
            (B, press(1, Buttons::SKILL2)),
        ]);
        events.extend(run_idle(&mut world, 30));
        assert!(events.contains(&Event::Countered {
            counterer: B,
            attacker: A,
            damage: 65,
        }));
        assert_eq!(world.fighter(A).hp, MAX_HP - 65);
        assert_eq!(world.fighter(B).hp, MAX_HP);
    }

    #[test]
    fn dash_iframes_avoid_hit() {
        let mut world = duel(60, -1);
        // Đòn nhẹ của A active ở tick 11–16. B lướt từ tick 8: startup 3 tick, rồi bất tử
        // 6 tick đầu đoạn di chuyển, phủ đúng tick 11–16.
        world.step(&[(A, press(1, Buttons::LIGHT))]);
        run_idle(&mut world, 7);
        let mut events = world.step(&[(
            B,
            InputFrame {
                seq: 1,
                move_x: -1,
                pressed: Buttons::DASH,
                ..InputFrame::default()
            },
        )]);
        events.extend(run_idle(&mut world, 10));
        assert!(!events.iter().any(|e| matches!(e, Event::Hit { .. })));
        assert_eq!(world.fighter(B).stamina.points(), 75);
    }

    fn hits(events: &[Event]) -> Vec<(FighterId, u32)> {
        events
            .iter()
            .filter_map(|e| match *e {
                Event::Hit { target, damage, .. } => Some((target, damage)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn projectile_flies_and_hits_once() {
        let mut world = duel(400, -1);
        let mut events = world.step(&[(A, press(1, Buttons::SKILL1))]);
        let mut first_hit = None;
        for _ in 0..60 {
            let tick = world.tick();
            let step = world.step(&[]);
            if first_hit.is_none() && !hits(&step).is_empty() {
                first_hit = Some(tick);
            }
            if tick == 18 {
                assert_eq!(
                    world.projectiles().len(),
                    1,
                    "đạn sinh ở tick active đầu tiên"
                );
            }
            events.extend(step);
        }
        assert_eq!(hits(&events), [(B, 80)]);
        // Sinh ở tick 18 với mép trước cách 44 px; va chạm cần mép trước vượt hẳn 380 px,
        // tức 22 tick bay với 16 px/tick.
        assert_eq!(first_hit, Some(18 + 22));
        assert!(world.projectiles().is_empty());
    }

    #[test]
    fn projectile_vanishes_at_max_range() {
        let mut world = duel(700, -1);
        world.step(&[(A, press(1, Buttons::SKILL1))]);
        let events = run_idle(&mut world, 90);
        assert!(hits(&events).is_empty());
        assert!(world.projectiles().is_empty());
    }

    #[test]
    fn projectile_is_blocked_only_from_the_front() {
        for (b_facing, expect_block) in [(-1, true), (1, false)] {
            let mut world = duel(200, b_facing);
            let mut events = Vec::new();
            for seq in 1..=60 {
                let a_input = (
                    A,
                    press(
                        seq,
                        if seq == 20 {
                            Buttons::SKILL1
                        } else {
                            Buttons::NONE
                        },
                    ),
                );
                events.extend(world.step(&[a_input, (B, hold(seq, Buttons::GUARD))]));
            }
            let blocked = events.iter().any(|e| matches!(e, Event::Blocked { .. }));
            assert_eq!(blocked, expect_block, "B quay {b_facing}");
            assert_eq!(hits(&events).is_empty(), expect_block);
        }
    }

    #[test]
    fn projectile_does_not_trigger_counter_stance() {
        let mut world = duel(200, -1);
        // Lưu Tiễn chạm B ở tick 27; B bấm Hồi Thế ở tick 13 nên tư thế active ở tick 23–40.
        let mut events = Vec::new();
        for seq in 1..=60 {
            let mut inputs = Vec::new();
            if seq == 1 {
                inputs.push((A, press(seq, Buttons::SKILL1)));
            }
            if seq == 14 {
                inputs.push((B, press(seq, Buttons::SKILL2)));
            }
            events.extend(world.step(&inputs));
        }
        assert!(!events.iter().any(|e| matches!(e, Event::Countered { .. })));
        assert_eq!(hits(&events), [(B, 80)]);
        assert_eq!(world.fighter(A).hp, MAX_HP);
    }
}
