//! Thế giới phòng thử: bước tick tất định nhận input và trả sự kiện (architecture.md §2).
//!
//! Mỗi tick chạy đúng thứ tự `input → movement → collision → combat → status → events`
//! (ADR 0003). Adapter ECS và server gọi [`World::step`] chứ không tự chạy từng pha, nên chỉ có
//! một bộ luật:
//!
//! 1. **input**: nhận tối đa một khung mỗi nhân vật, xử lý tương tác NPC, bắt đầu hành động.
//! 2. **movement**: di chuyển nhân vật; đạn cũ bay rồi đạn mới sinh tại chỗ.
//! 3. **collision**: gom mọi va chạm đòn/đạn từ cùng một trạng thái.
//! 4. **combat**: áp dụng va chạm: đỡ, phản công, damage, hiệu ứng khi trúng.
//! 5. **status**: tiến bộ đếm đòn, đỡ, lướt, hitstun, tài nguyên, hồi chiêu và hiệu ứng.
//! 6. **events**: trả sự kiện theo thứ tự phát sinh; tăng tick.

use std::hash::{Hash, Hasher};

use crate::fighter::{Fighter, FighterId, State, Team};
use crate::input::{InputFrame, Intent};
use crate::kit::{ActionKind, Kit};
use crate::meter::SUB_PER_POINT;
use crate::npc::{Npc, NpcId, NpcSpec};
use crate::projectile::{Projectile, ProjectileId};
use crate::status::StatusKind;
use crate::tick::Tick;

/// Sự kiện do mô phỏng quyết định; client chỉ thể hiện, không tự tạo. `damage` là damage danh
/// nghĩa của đòn, không bị cắt theo HP còn lại.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// Hiệu ứng mới hoặc được làm mạnh/làm mới; `ticks` là thời lượng còn lại.
    StatusApplied {
        target: FighterId,
        source: FighterId,
        kind: StatusKind,
        percent: u8,
        ticks: u32,
    },
    /// Hiệu ứng hết hạn tự nhiên. Bị hạ xóa mọi hiệu ứng mà không phát sự kiện này.
    StatusEnded {
        fighter: FighterId,
        kind: StatusKind,
    },
    /// Mô phỏng xác nhận tương tác; nội dung hội thoại/nhiệm vụ do server nghiệp vụ quyết định.
    Interacted {
        fighter: FighterId,
        npc: NpcId,
    },
}

#[derive(Clone, Debug, Default, Hash)]
pub struct World {
    tick: Tick,
    fighters: Vec<Fighter>,
    projectiles: Vec<Projectile>,
    npcs: Vec<Npc>,
    next_instance: u32,
    next_projectile: u32,
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

    /// Thêm nhân vật không phe đứng trên nền tại `x` (mili-pixel).
    pub fn spawn(&mut self, kit: &'static Kit, x: i32, facing: i8) -> FighterId {
        self.spawn_in_team(kit, x, facing, None)
    }

    pub fn spawn_in_team(
        &mut self,
        kit: &'static Kit,
        x: i32,
        facing: i8,
        team: Option<Team>,
    ) -> FighterId {
        let id = FighterId(u16::try_from(self.fighters.len()).expect("quá nhiều nhân vật"));
        self.fighters.push(Fighter::new(id, kit, x, facing, team));
        id
    }

    pub fn spawn_npc(&mut self, spec: &'static NpcSpec, x: i32) -> NpcId {
        let id = NpcId(u16::try_from(self.npcs.len()).expect("quá nhiều NPC"));
        self.npcs.push(Npc { id, spec, x });
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

    pub fn npcs(&self) -> &[Npc] {
        &self.npcs
    }

    #[cfg(test)]
    pub(crate) fn fighters_mut(&mut self) -> &mut [Fighter] {
        &mut self.fighters
    }

    /// Chạy một tick. Mỗi nhân vật nhận tối đa một khung input mỗi tick; khung thừa, trùng `seq`
    /// hoặc `seq` cũ bị từ chối.
    pub fn step(&mut self, inputs: &[(FighterId, InputFrame)]) -> Vec<Event> {
        let mut events = Vec::new();
        self.input_phase(inputs, &mut events);
        self.movement_phase();
        let strikes = self.collision_phase();
        self.combat_phase(strikes, &mut events);
        self.status_phase(&mut events);
        self.tick += 1;
        events
    }

    /// Hash trạng thái để so replay trong cùng build; không dùng làm định dạng lưu trữ.
    pub fn state_hash(&self) -> u64 {
        let mut hasher = StableHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn input_phase(&mut self, inputs: &[(FighterId, InputFrame)], events: &mut Vec<Event>) {
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

        for (fighter, frame) in self.fighters.iter().zip(&frames) {
            let wants = frame.and_then(|f| f.intent()) == Some(Intent::Interact);
            if wants
                && fighter.can_interact()
                && let Some(npc) = nearest_npc(&self.npcs, fighter)
            {
                events.push(Event::Interacted {
                    fighter: fighter.id,
                    npc,
                });
            }
        }
        for (fighter, frame) in self.fighters.iter_mut().zip(frames) {
            fighter.apply_input(frame, &mut self.next_instance, events);
        }
    }

    fn movement_phase(&mut self) {
        for fighter in &mut self.fighters {
            fighter.integrate();
        }
        // Đạn cũ bay trước; đạn mới đứng tại chỗ sinh trong tick đầu tiên.
        self.projectiles.retain_mut(Projectile::fly);
        for fighter in &self.fighters {
            if let Some(projectile) =
                Projectile::launch(fighter, ProjectileId(self.next_projectile))
            {
                self.next_projectile += 1;
                self.projectiles.push(projectile);
            }
        }
    }

    /// Gom va chạm từ cùng một trạng thái rồi mới áp dụng, để hai đòn trúng nhau cùng tick đều
    /// được tính.
    fn collision_phase(&self) -> Vec<Strike> {
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
        pending
    }

    fn combat_phase(&mut self, strikes: Vec<Strike>, events: &mut Vec<Event>) {
        for strike in strikes {
            self.apply_hit(strike, events);
        }
        self.projectiles.retain(Projectile::is_alive);
    }

    fn status_phase(&mut self, events: &mut Vec<Event>) {
        for fighter in &mut self.fighters {
            fighter.advance(events);
        }
    }

    /// Những nhân vật có thể bị trúng bởi đòn của `owner` chưa trúng ai trong `hit_set`.
    fn targets<'a>(
        &'a self,
        owner: FighterId,
        hit_set: &'a [FighterId],
    ) -> impl Iterator<Item = &'a Fighter> {
        let owner = self.fighter(owner);
        self.fighters.iter().filter(move |target| {
            target.id != owner.id
                && !owner.is_ally(target)
                && !hit_set.contains(&target.id)
                && !target.is_invulnerable()
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
        // Mục tiêu bị hạ bởi một va chạm trước đó trong cùng tick không nhận thêm đòn. Người đánh
        // bị hạ cùng tick vẫn ra đòn: hai bên trúng nhau cùng lúc đều được tính.
        if self.fighters[ti].state == State::Downed {
            return;
        }
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
        if let Some(status) = spec.on_hit
            && target.state != State::Downed
            && let Some(effect) = target.statuses.apply(status, attacker_id)
        {
            events.push(Event::StatusApplied {
                target: target_id,
                source: attacker_id,
                kind: effect.kind,
                percent: effect.percent,
                ticks: effect.remaining,
            });
        }
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

/// NPC gần nhất trong tầm của nhân vật; hòa khoảng cách thì chọn ID nhỏ hơn.
fn nearest_npc(npcs: &[Npc], fighter: &Fighter) -> Option<NpcId> {
    npcs.iter()
        .map(|npc| ((npc.x - fighter.x).abs(), npc))
        .filter(|(distance, npc)| *distance <= npc.spec.reach)
        .min_by_key(|(distance, npc)| (*distance, npc.id))
        .map(|(_, npc)| npc.id)
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
    use crate::fighter::{GUARD_STARTUP, MAX_HP, Team};
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
    fn simultaneous_lethal_hits_down_the_target_once() {
        let mut world = World::new();
        let a = world.spawn(&LONG_LUU, 400 * PX, 1);
        let target = world.spawn(&LONG_LUU, 460 * PX, -1);
        let c = world.spawn(&LONG_LUU, 520 * PX, -1);
        world.fighters[1].hp = 10;
        let mut events =
            world.step(&[(a, press(1, Buttons::LIGHT)), (c, press(1, Buttons::LIGHT))]);
        events.extend(run_idle(&mut world, 20));
        let downed = events
            .iter()
            .filter(|e| **e == Event::Downed { fighter: target })
            .count();
        assert_eq!(downed, 1, "{events:?}");
        let hits_on_target = events
            .iter()
            .filter(|e| matches!(e, Event::Hit { target: t, .. } if *t == target))
            .count();
        assert_eq!(hits_on_target, 1, "mục tiêu đã bị hạ không nhận thêm đòn");
    }

    #[test]
    fn simultaneous_trades_still_land_both_hits() {
        let mut world = duel(60, -1);
        world.fighters[0].hp = 10;
        world.fighters[1].hp = 10;
        let mut events =
            world.step(&[(A, press(1, Buttons::LIGHT)), (B, press(1, Buttons::LIGHT))]);
        events.extend(run_idle(&mut world, 20));
        assert!(events.contains(&Event::Downed { fighter: A }));
        assert!(events.contains(&Event::Downed { fighter: B }));
    }

    #[test]
    fn allies_do_not_hit_each_other() {
        let team = Some(Team(1));
        let mut world = World::new();
        let a = world.spawn_in_team(&LONG_LUU, 400 * PX, 1, team);
        world.spawn_in_team(&LONG_LUU, 460 * PX, -1, team);
        let mut events = world.step(&[(a, press(1, Buttons::LIGHT))]);
        events.extend(run_idle(&mut world, 30));
        assert!(!events.iter().any(|e| matches!(e, Event::Hit { .. })));

        let mut world = World::new();
        let a = world.spawn_in_team(&LONG_LUU, 400 * PX, 1, team);
        world.spawn_in_team(&LONG_LUU, 460 * PX, -1, Some(Team(2)));
        let mut events = world.step(&[(a, press(1, Buttons::LIGHT))]);
        events.extend(run_idle(&mut world, 30));
        assert!(events.iter().any(|e| matches!(e, Event::Hit { .. })));
    }

    #[test]
    fn interact_needs_reach_and_a_free_hand() {
        use crate::npc::{GUIDE, NpcId};

        let mut world = World::new();
        let a = world.spawn(&LONG_LUU, 400 * PX, 1);
        let npc = world.spawn_npc(&GUIDE, 400 * PX + GUIDE.reach);
        world.spawn_npc(&GUIDE, 1_500 * PX);
        let events = world.step(&[(a, press(1, Buttons::INTERACT))]);
        assert_eq!(events, [Event::Interacted { fighter: a, npc }]);
        assert_eq!(npc, NpcId(0));

        // Đang ra đòn thì không tương tác, và tương tác không hủy đòn.
        world.step(&[(a, press(2, Buttons::LIGHT))]);
        let events = world.step(&[(a, press(3, Buttons::INTERACT))]);
        assert!(events.is_empty());
        assert!(matches!(world.fighter(a).state, State::Attack { .. }));

        let mut far = World::new();
        let b = far.spawn(&LONG_LUU, 400 * PX, 1);
        far.spawn_npc(&GUIDE, 400 * PX + GUIDE.reach + 1);
        assert!(far.step(&[(b, press(1, Buttons::INTERACT))]).is_empty());
    }

    #[test]
    fn projectile_ids_are_never_reused() {
        let mut world = duel(700, -1);
        let mut ids = Vec::new();
        for seq in 1..=600 {
            world.fighters[0].energy = Meter::energy();
            world.step(&[(A, press(seq, Buttons::SKILL1))]);
            ids.extend(world.projectiles().iter().map(|p| p.id));
        }
        ids.dedup();
        assert!(ids.len() >= 3, "phải bắn nhiều lượt: {ids:?}");
        assert!(ids.windows(2).all(|w| w[0] < w[1]), "{ids:?}");
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
