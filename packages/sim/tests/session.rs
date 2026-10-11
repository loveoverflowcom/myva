//! Phiên authoritative: tái lập theo seed, replay, thứ tự lệnh và invariant qua nhiều seed.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use myva_sim::fighter::ARENA_WIDTH;
use myva_sim::fixture::{self, Brawler, PLAYER_EPOCH};
use myva_sim::protocol::{Action, Attack, CommandEnvelope, CommandFrame, Rejection, SessionEpoch};
use myva_sim::replay::Replay;
use myva_sim::status::SLOW_CAP_PERCENT;
use myva_sim::{Event, FighterId, State, World};

const TICKS: u32 = 60 * 60;

#[test]
fn same_seed_reproduces_every_tick() {
    assert_eq!(
        fixture::fingerprint(7, TICKS),
        fixture::fingerprint(7, TICKS)
    );
    assert_ne!(
        fixture::fingerprint(7, TICKS),
        fixture::fingerprint(8, TICKS)
    );
}

#[test]
fn fixture_exercises_ai_skill_status_and_defeat() {
    let mut seen = BTreeSet::new();
    for seed in 0..8 {
        for report in fixture::run_core(seed, TICKS) {
            for record in report.events {
                seen.insert(match record.event {
                    Event::ActionStarted { .. } => "action",
                    Event::Hit { .. } => "hit",
                    Event::Blocked { .. } => "blocked",
                    Event::StatusApplied { .. } => "status",
                    Event::StatusEnded { .. } => "status-ended",
                    Event::Downed { .. } => "downed",
                    Event::Interacted { .. } => "interacted",
                    _ => "other",
                });
            }
        }
    }
    for kind in [
        "action",
        "hit",
        "blocked",
        "status",
        "status-ended",
        "downed",
        "interacted",
    ] {
        assert!(
            seen.contains(kind),
            "fixture chưa chạm nhánh {kind}: {seen:?}"
        );
    }
}

#[test]
fn session_replay_plays_back_without_brains() {
    let (mut session, ground) = fixture::training_ground(3);
    let mut brawler = Brawler::new(3, ground.player, PLAYER_EPOCH);
    for _ in 0..2_000 {
        let command = brawler.command(session.world());
        session.submit(command).unwrap();
        session.step();
    }
    let text = session.replay().to_text();
    let replay = Replay::parse(&text).unwrap();
    assert_eq!(replay.seed, Some(3));
    let playback = replay.play().unwrap();
    assert_eq!(playback.world.state_hash(), session.world().state_hash());
}

#[test]
fn restoring_a_saved_world_replays_the_same_future() {
    let (mut session, ground) = fixture::training_ground(5);
    let mut brawler = Brawler::new(5, ground.player, PLAYER_EPOCH);
    for _ in 0..600 {
        let command = brawler.command(session.world());
        session.submit(command).unwrap();
        session.step();
    }
    // Lưu trong process bằng `Clone`, rồi chạy cùng lệnh đã ghi từ điểm lưu.
    let saved: World = session.world().clone();
    let replay = session.replay();
    for _ in 0..600 {
        let command = brawler.command(session.world());
        session.submit(command).unwrap();
        session.step();
    }
    let full = session.replay();
    let mut restored = saved;
    let mut inputs = full.inputs.iter().skip(replay.inputs.len()).peekable();
    while restored.tick() < session.world().tick() {
        let tick = restored.tick();
        let frame: Vec<_> = std::iter::from_fn(|| {
            inputs
                .next_if(|(t, ..)| *t == tick)
                .map(|&(_, id, input)| (id, input))
        })
        .collect();
        restored.step(&frame);
    }
    assert_eq!(restored.state_hash(), session.world().state_hash());
}

#[test]
fn clients_cannot_drive_monsters_or_pose_as_ai() {
    let (mut session, ground) = fixture::training_ground(1);
    let monster = ground.monsters[0];
    let claw = CommandFrame {
        action: Some(Action::Attack(Attack::Light)),
        ..CommandFrame::IDLE
    };
    assert_eq!(
        session.submit(CommandEnvelope::new(PLAYER_EPOCH, monster, 1, 0, claw)),
        Err(Rejection::NotController {
            actor: monster,
            session: PLAYER_EPOCH
        })
    );
    assert_eq!(
        session.submit(CommandEnvelope::new(SessionEpoch::AI, monster, 1, 0, claw)),
        Err(Rejection::NotController {
            actor: monster,
            session: SessionEpoch::AI
        })
    );
}

#[test]
fn reconnect_continues_seq_and_drops_old_epoch() {
    let (mut session, ground) = fixture::training_ground(1);
    let player = ground.player;
    let walk = CommandFrame {
        move_x: 1,
        ..CommandFrame::IDLE
    };
    session
        .submit(CommandEnvelope::new(PLAYER_EPOCH, player, 1, 0, walk))
        .unwrap();
    session.step();
    let fresh = SessionEpoch(2);
    session.rebind(player, fresh);
    let tick = session.world().tick();
    assert!(matches!(
        session.submit(CommandEnvelope::new(PLAYER_EPOCH, player, 2, tick, walk)),
        Err(Rejection::NotController { .. })
    ));
    let last = session.snapshot().fighters[usize::from(player.0)].last_seq;
    assert_eq!(last, Some(1));
    session
        .submit(CommandEnvelope::new(fresh, player, 2, tick, walk))
        .unwrap();
    let report = session.step();
    assert!(
        !report
            .events
            .iter()
            .any(|r| matches!(r.event, Event::InputRejected { .. }))
    );
}

/// Đổi thứ tự đến của lệnh giữa các tác nhân không đổi kết quả.
#[test]
fn arrival_order_between_actors_does_not_matter() {
    use myva_sim::fixture::{HOSTILE, PLAYERS};
    use myva_sim::kit::{LONG_LUU, PX};
    use myva_sim::session::{Session, SessionConfig};

    let run = |reverse: bool| {
        let mut session = Session::new(SessionConfig::new(1, 1));
        let a = session.spawn_player(&LONG_LUU, 500 * PX, 1, Some(PLAYERS), SessionEpoch(1));
        let b = session.spawn_player(&LONG_LUU, 560 * PX, -1, Some(HOSTILE), SessionEpoch(2));
        let (mut bot_a, mut bot_b) = (
            Brawler::new(10, a, SessionEpoch(1)),
            Brawler::new(20, b, SessionEpoch(2)),
        );
        (0..1_200)
            .map(|_| {
                let mut commands = vec![
                    bot_a.command(session.world()),
                    bot_b.command(session.world()),
                ];
                if reverse {
                    commands.reverse();
                }
                for command in commands {
                    session.submit(command).unwrap();
                }
                session.step().hash
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(run(false), run(true));
}

#[test]
fn invariants_hold_across_seeds() {
    for seed in 0..24 {
        let (mut session, ground) = fixture::training_ground(seed);
        let mut brawler = Brawler::new(seed, ground.player, PLAYER_EPOCH);
        let mut downed_at: BTreeMap<FighterId, u32> = BTreeMap::new();
        let mut ids = BTreeSet::new();
        for _ in 0..TICKS {
            let command = brawler.command(session.world());
            session.submit(command).unwrap();
            let report = session.step();
            let world = session.world();
            let mut in_tick = HashSet::new();
            for record in &report.events {
                assert!(ids.insert(record.id), "seed {seed}: trùng {:?}", record.id);
                assert!(
                    in_tick.insert(record.event),
                    "seed {seed}: sự kiện lặp trong tick {}: {:?}",
                    report.tick,
                    record.event
                );
                check_event(world, &downed_at, report.tick, record.event, seed);
                if let Event::Downed { fighter } = record.event {
                    assert!(
                        downed_at.insert(fighter, report.tick).is_none(),
                        "seed {seed}: {fighter:?} bị hạ hai lần"
                    );
                }
            }
            check_state(world, seed);
        }
        // Mỗi nhân vật đã hạ vẫn nằm yên tới cuối trận.
        for id in downed_at.keys() {
            assert_eq!(session.world().fighter(*id).state, State::Downed);
        }
    }
}

fn check_event(
    world: &World,
    downed_at: &BTreeMap<FighterId, u32>,
    tick: u32,
    event: Event,
    seed: u64,
) {
    let dead_before = |id: FighterId| downed_at.get(&id).is_some_and(|&t| t < tick);
    match event {
        Event::ActionStarted { fighter, .. } | Event::Interacted { fighter, .. } => {
            assert!(
                !dead_before(fighter),
                "seed {seed}: {fighter:?} đã hạ vẫn hành động"
            );
        }
        Event::Hit {
            attacker,
            target,
            action,
            ..
        }
        | Event::Blocked {
            attacker,
            target,
            action,
            ..
        } => {
            assert!(
                !dead_before(target),
                "seed {seed}: đánh trúng {target:?} đã hạ"
            );
            // Đạn đã bay vẫn có thể trúng sau khi người bắn bị hạ; đòn cận chiến thì không.
            let projectile = world
                .fighter(attacker)
                .kit
                .spec(action)
                .projectile
                .is_some();
            assert!(
                projectile || !dead_before(attacker),
                "seed {seed}: {attacker:?} đã hạ vẫn đánh cận chiến"
            );
        }
        Event::Countered {
            counterer,
            attacker,
            ..
        } => {
            assert!(!dead_before(counterer) && !dead_before(attacker));
        }
        Event::StatusApplied { target, .. } => {
            assert!(!dead_before(target));
        }
        _ => {}
    }
}

fn check_state(world: &World, seed: u64) {
    for f in world.fighters() {
        assert!(f.hp <= f.kit.body.max_hp);
        assert_eq!(f.hp == 0, f.state == State::Downed, "seed {seed}");
        let half = f.kit.body.half_width;
        assert!((half..=ARENA_WIDTH - half).contains(&f.x));
        assert!(f.y >= 0);
        for meter in [f.stamina, f.energy, f.mach] {
            assert!(meter.sub() <= meter.max_sub());
        }
        for (slot, &cooldown) in f.cooldowns.iter().enumerate() {
            assert!(
                cooldown <= f.kit.skills[slot].cooldown,
                "seed {seed}: hồi chiêu vượt trần"
            );
        }
        let statuses = f.statuses.as_slice();
        assert!(
            statuses.windows(2).all(|w| w[0].kind < w[1].kind),
            "tối đa một mỗi loại"
        );
        assert!(
            statuses
                .iter()
                .all(|s| s.remaining > 0 && s.percent <= SLOW_CAP_PERCENT)
        );
        if f.state == State::Downed {
            assert!(statuses.is_empty(), "seed {seed}: bị hạ phải xóa hiệu ứng");
        }
    }
    let ids: Vec<_> = world.projectiles().iter().map(|p| p.id).collect();
    assert!(
        ids.windows(2).all(|w| w[0] < w[1]),
        "seed {seed}: ID đạn {ids:?}"
    );
}
