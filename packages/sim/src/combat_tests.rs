//! Regression fixtures #19; setup riêng cho từng cạnh, va chạm/input vẫn đi qua World::step.
use crate::control::{ControlBudget, DR_IMMUNITY, LOCK_LIMIT, RESPONSE_WINDOW};
use crate::fighter::{Contact, Denial, ESCAPE_COOLDOWN, ESCAPE_PROTECTION, MAX_X, MIN_X};
use crate::kit::{ActionSpec, Kit};
use crate::meter::Meter;
use crate::{ActionKind, Buttons, Event, FighterId, InputFrame, LONG_LUU, PX, State, Team, World};

const A: FighterId = FighterId(0);
const B: FighterId = FighterId(1);

fn duel(distance: i32) -> World {
    let mut world = World::new();
    world.spawn(&LONG_LUU, 400 * PX, 1);
    world.spawn(&LONG_LUU, (400 + distance) * PX, -1);
    world
}

fn input(seq: u32, pressed: Buttons, held: Buttons) -> InputFrame {
    InputFrame {
        seq,
        pressed,
        held,
        ..InputFrame::default()
    }
}

fn press(world: &mut World, actor: FighterId, seq: u32, buttons: Buttons) -> Vec<Event> {
    world.step(&[(actor, input(seq, buttons, Buttons::NONE))])
}

fn idle(world: &mut World, ticks: u32) -> Vec<Event> {
    (0..ticks).flat_map(|_| world.step(&[])).collect()
}

#[test]
fn ultimate_spends_atomically_once_and_shares_the_normal_cooldown() {
    let mut world = duel(400);
    world.fighters_mut()[0].mach.gain(100);
    world.fighters_mut()[0].energy = Meter::new(100, 34, 0, 0);
    let ultimate = Buttons::SKILL3 | Buttons::ULTIMATE;
    let denied = press(&mut world, A, 1, ultimate);
    assert!(denied.contains(&Event::ActionDenied {
        fighter: A,
        reason: Denial::Energy
    }));
    assert_eq!(world.fighter(A).mach.points(), 100);
    assert_eq!(world.fighter(A).energy.points(), 34);
    assert_eq!(world.fighter(A).cooldowns[2], 0);
    world.fighters_mut()[0].energy = Meter::new(100, 35, 0, 0);
    assert!(
        press(&mut world, A, 2, ultimate).contains(&Event::ActionStarted {
            fighter: A,
            action: ActionKind::Ultimate(2)
        })
    );
    assert_eq!(world.fighter(A).mach.points(), 0);
    assert_eq!(world.fighter(A).energy.points(), 0);
    assert_eq!(world.fighter(A).cooldowns[2], 599);
    let before = world.fighter(A).energy.sub();
    assert!(
        press(&mut world, A, 2, ultimate).contains(&Event::InputRejected { fighter: A, seq: 2 })
    );
    assert_eq!(world.fighter(A).energy.sub(), before);
    idle(&mut world, 90);
    world.fighters_mut()[0].energy = Meter::energy();
    assert!(
        press(&mut world, A, 3, Buttons::SKILL3).contains(&Event::ActionDenied {
            fighter: A,
            reason: Denial::Cooldown
        })
    );
}

#[test]
fn ui_cancel_discards_a_buffered_ultimate_without_canceling_the_attack() {
    let mut world = duel(600);
    press(&mut world, A, 1, Buttons::LIGHT);
    idle(&mut world, 23);
    world.fighters_mut()[0].mach.gain(100);
    press(&mut world, A, 2, Buttons::SKILL3 | Buttons::ULTIMATE);
    assert!(world.fighter(A).buffered.is_some());
    let events = press(&mut world, A, 3, Buttons::CANCEL_PENDING);
    assert!(events.is_empty());
    assert!(world.fighter(A).buffered.is_none());
    assert!(matches!(
        world.fighter(A).state,
        State::Attack {
            action: ActionKind::Light(0),
            ..
        }
    ));
    assert!(
        !idle(&mut world, 20)
            .iter()
            .any(|e| matches!(e, Event::ActionStarted { .. }))
    );
    assert_eq!(world.fighter(A).mach.points(), 100);
    assert_eq!(world.fighter(A).energy.points(), 100);
}

#[test]
fn ultimate_requires_selected_slot_full_mach_ground_and_a_live_actor() {
    for (state, y, slot, mach, expected) in [
        (State::Neutral, 0, Buttons::SKILL1, 100, Denial::Unsupported),
        (State::Neutral, 0, Buttons::SKILL3, 99, Denial::Mach),
        (State::Neutral, PX, Buttons::SKILL3, 100, Denial::State),
        (
            State::Hitstun { remaining: 10 },
            0,
            Buttons::SKILL3,
            100,
            Denial::State,
        ),
    ] {
        let mut world = duel(400);
        let fighter = &mut world.fighters_mut()[0];
        fighter.state = state;
        fighter.y = y;
        fighter.mach.gain(mach);
        let events = press(&mut world, A, 1, slot | Buttons::ULTIMATE);
        assert!(events.contains(&Event::ActionDenied {
            fighter: A,
            reason: expected
        }));
        assert_eq!(world.fighter(A).mach.points(), mach);
        assert_eq!(world.fighter(A).energy.points(), 100);
    }
    let mut world = duel(400);
    world.fighters_mut()[0].state = State::Downed;
    world.fighters_mut()[0].hp = 0;
    world.fighters_mut()[0].mach.gain(100);
    assert!(press(&mut world, A, 1, Buttons::SKILL3 | Buttons::ULTIMATE).is_empty());
    assert_eq!(world.fighter(A).mach.points(), 100);
}

#[test]
fn normal_skill_also_prevents_ultimate_on_shared_cooldown() {
    let mut world = duel(400);
    world.fighters_mut()[0].mach.gain(100);
    press(&mut world, A, 1, Buttons::SKILL3);
    idle(&mut world, 70);
    let energy = world.fighter(A).energy.sub();
    assert!(
        press(&mut world, A, 2, Buttons::SKILL3 | Buttons::ULTIMATE).contains(
            &Event::ActionDenied {
                fighter: A,
                reason: Denial::Cooldown
            }
        )
    );
    assert_eq!(world.fighter(A).mach.points(), 100);
    // Meter tự hồi; từ chối không reset độ trễ hay trừ chi phí.
    assert!(world.fighter(A).energy.sub() >= energy);
}

#[test]
fn hit_block_whiff_and_wrong_facing_have_distinct_cancel_results() {
    for (distance, guard, facing, expected) in [
        (60, false, 1, Contact::Hit),
        (60, true, 1, Contact::Blocked),
        (600, false, 1, Contact::Whiff),
        (60, false, -1, Contact::Whiff),
    ] {
        let mut world = duel(distance);
        world.fighters_mut()[0].facing = facing;
        press(&mut world, A, 1, Buttons::LIGHT);
        for seq in 1..=if expected == Contact::Whiff { 16 } else { 11 } {
            world.step(&[(
                B,
                input(
                    seq,
                    Buttons::NONE,
                    if guard { Buttons::GUARD } else { Buttons::NONE },
                ),
            )]);
        }
        assert_eq!(world.fighter(A).feedback.contact, expected);
        let events = press(&mut world, A, 2, Buttons::SKILL1);
        assert_eq!(
            events.iter().any(|e| matches!(
                e,
                Event::ActionStarted {
                    action: ActionKind::Skill(0),
                    ..
                }
            )),
            expected == Contact::Hit
        );
        if expected != Contact::Hit {
            assert_eq!(world.fighter(A).energy.points(), 100);
        }
    }
}

#[test]
fn whiff_cannot_dash_cancel_or_turn_during_commitment() {
    let mut world = duel(600);
    press(&mut world, A, 1, Buttons::HEAVY);
    idle(&mut world, 34);
    let facing = world.fighter(A).facing;
    let events = world.step(&[(
        A,
        InputFrame {
            move_x: -1,
            ..input(2, Buttons::DASH, Buttons::NONE)
        },
    )]);
    assert!(events.contains(&Event::ActionDenied {
        fighter: A,
        reason: Denial::Window
    }));
    assert!(matches!(
        world.fighter(A).state,
        State::Attack {
            action: ActionKind::Heavy,
            ..
        }
    ));
    assert_eq!(world.fighter(A).facing, facing);
    assert_eq!(world.fighter(A).stamina.points(), 88);
}

#[test]
fn late_confirm_and_resource_failure_do_not_cancel_or_spend() {
    let mut world = duel(60);
    press(&mut world, A, 1, Buttons::LIGHT);
    idle(&mut world, 11);
    world.fighters_mut()[0].energy = Meter::new(100, 19, 0, 0);
    assert!(
        press(&mut world, A, 2, Buttons::SKILL1).contains(&Event::ActionDenied {
            fighter: A,
            reason: Denial::Energy
        })
    );
    assert!(matches!(
        world.fighter(A).state,
        State::Attack {
            action: ActionKind::Light(0),
            ..
        }
    ));
    assert_eq!(world.fighter(A).energy.points(), 19);
    world.fighters_mut()[0].energy = Meter::energy();
    idle(&mut world, 6);
    assert!(
        !press(&mut world, A, 3, Buttons::SKILL1)
            .iter()
            .any(|e| matches!(e, Event::ActionStarted { .. }))
    );
}

#[test]
fn escape_requires_a_fresh_press_after_hitstun_and_never_reuses_guard() {
    let mut world = duel(60);
    world.fighters_mut()[1].mach.gain(100);
    // Đỡ trước khi bị đánh sau lưng không trở thành lần nhấn thoát ở tick sau.
    world.fighters_mut()[1].facing = 1;
    press(&mut world, A, 1, Buttons::LIGHT);
    for seq in 1..=12 {
        world.step(&[(
            B,
            input(
                seq,
                if seq == 1 {
                    Buttons::GUARD
                } else {
                    Buttons::NONE
                },
                Buttons::GUARD,
            ),
        )]);
    }
    assert!(matches!(world.fighter(B).state, State::Hitstun { .. }));
    assert_eq!(world.fighter(B).mach.points(), 100);
    let stale = world.step(&[(B, input(13, Buttons::GUARD, Buttons::GUARD))]);
    assert!(stale.contains(&Event::ActionDenied {
        fighter: B,
        reason: Denial::FreshGuardRequired
    }));
    world.step(&[(B, input(14, Buttons::NONE, Buttons::NONE))]);
    let events = world.step(&[(B, input(15, Buttons::GUARD, Buttons::GUARD))]);
    assert!(events.contains(&Event::MachEscaped { fighter: B }));
    assert_eq!(world.fighter(B).mach.points(), 50);
    assert_eq!(
        world.fighter(B).feedback.escape_cooldown,
        ESCAPE_COOLDOWN - 1
    );
    assert_eq!(
        world.fighter(B).state,
        State::Escape {
            remaining: ESCAPE_PROTECTION - 1
        }
    );
    let gap = (world.fighter(A).x - world.fighter(B).x).abs();
    assert!(gap > 60 * PX);
    let hp = world.fighter(A).hp;
    idle(&mut world, 12);
    assert_eq!(world.fighter(A).hp, hp, "thoát không gây damage");
}

#[test]
fn escape_refuses_insufficient_mach_and_cooldown_and_cannot_buffer() {
    for (mach, cooldown, reason) in [(49, 0, Denial::Mach), (100, 10, Denial::Cooldown)] {
        let mut world = duel(60);
        let f = &mut world.fighters_mut()[1];
        f.mach.gain(mach);
        f.state = State::Hitstun { remaining: 10 };
        f.feedback.escape_cooldown = cooldown;
        f.buffered = Some((crate::Intent::Skill(0), 8));
        let events = world.step(&[(B, input(1, Buttons::GUARD, Buttons::GUARD))]);
        assert!(events.contains(&Event::ActionDenied { fighter: B, reason }));
        assert_eq!(world.fighter(B).mach.points(), mach);
        assert!(world.fighter(B).buffered.is_none());
    }
}

#[test]
fn escape_at_both_walls_keeps_bodies_inside_and_protects_against_attacks() {
    for (x, facing) in [(MIN_X, 1), (MAX_X, -1)] {
        let mut world = duel(60);
        world.fighters_mut()[1].x = x;
        world.fighters_mut()[1].state = State::Hitstun { remaining: 10 };
        world.fighters_mut()[1].mach.gain(50);
        world.fighters_mut()[0].x = x + i32::from(facing) * 30 * PX;
        world.fighters_mut()[0].facing = -facing;
        world.fighters_mut()[0].state = State::Attack {
            action: ActionKind::Heavy,
            elapsed: LONG_LUU.heavy.startup,
            instance: 1,
            confirmed_at: None,
        };
        let events = world.step(&[(B, input(1, Buttons::GUARD, Buttons::GUARD))]);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Hit { target: B, .. }))
        );
        assert_eq!(world.fighter(B).hp, 1000);
        assert!(world.fighter(B).is_invulnerable());
        for f in world.fighters() {
            assert!((MIN_X..=MAX_X).contains(&f.x));
        }
    }
}

const BURST: Kit = Kit {
    heavy: ActionSpec {
        damage: 160,
        knockback: 0,
        ..LONG_LUU.heavy
    },
    ..LONG_LUU
};

#[test]
fn many_attackers_and_a_projectile_share_damage_and_control_budget() {
    let mut world = World::new();
    for _ in 0..3 {
        world.spawn_in_team(&BURST, 400 * PX, 1, Some(Team(1)));
    }
    let target = world.spawn_in_team(&LONG_LUU, 460 * PX, -1, Some(Team(2)));
    for (index, f) in world.fighters_mut()[..3].iter_mut().enumerate() {
        let action = if index == 2 {
            ActionKind::Skill(0)
        } else {
            ActionKind::Heavy
        };
        f.state = State::Attack {
            action,
            elapsed: BURST.spec(action).startup,
            instance: index as u32,
            confirmed_at: None,
        };
    }
    let events = world.step(&[]);
    let damage: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Hit {
                target: t, damage, ..
            } if *t == target => Some(*damage),
            _ => None,
        })
        .collect();
    assert_eq!(damage, [160, 90, 0]);
    assert_eq!(world.fighter(target).hp, 750);
    assert_eq!(world.fighter(target).state, State::Neutral);
    assert!(world.fighter(target).feedback.control.immune > 0);
}

#[test]
fn target_dr_is_shared_by_melee_projectile_and_guard_break() {
    let mut world = duel(60);
    let mut events = Vec::new();
    let f = &mut world.fighters_mut()[1];
    f.take_hit(0, 18, 0, &mut events);
    assert_eq!(f.state, State::Hitstun { remaining: 18 });
    f.take_hit(0, 18, 0, &mut events);
    // DR không rút ngắn một stun đang có.
    assert_eq!(f.state, State::Hitstun { remaining: 18 });
    f.state = State::Neutral;
    f.break_guard(&mut events);
    assert_eq!(f.state, State::GuardBreak { remaining: 12 });
    f.state = State::Neutral;
    f.take_hit(0, 18, 60 * PX, &mut events);
    assert_eq!(f.state, State::Neutral);
    assert_eq!(f.x, 460 * PX, "displacement cũng chịu DR");
    assert_eq!(f.feedback.control.immune, DR_IMMUNITY);
}

#[test]
fn a_one_tick_gap_does_not_reset_the_chain_but_a_real_response_window_does() {
    let mut budget = ControlBudget::default();
    assert_eq!(budget.grant(100, 18, 1000).damage, 100);
    for _ in 0..18 {
        budget.tick(true);
    }
    budget.tick(false);
    assert_eq!(budget.grant(200, 18, 1000).damage, 150);
    assert_eq!(budget.damage, 250);
    for _ in 0..RESPONSE_WINDOW {
        budget.tick(false);
    }
    assert_eq!((budget.damage, budget.locked_ticks), (0, 0));
    assert_eq!(budget.grant(100, 18, 1000).damage, 100);
}

#[test]
fn control_time_limit_forces_a_response_window_across_guard_breaks() {
    let mut world = duel(60);
    let f = &mut world.fighters_mut()[1];
    let mut events = Vec::new();
    f.break_guard(&mut events);
    for _ in 0..30 {
        f.advance(&mut events);
    }
    assert_eq!(f.state, State::Neutral);
    f.advance(&mut events); // Khoảng 1 tick không reset 30 tick đã bị khóa.
    f.break_guard(&mut events);
    for _ in 0..17 {
        f.advance(&mut events);
    }
    assert_eq!(f.feedback.control.locked_ticks, LOCK_LIMIT);
    assert_eq!(f.state, State::Neutral);
    assert!(f.feedback.control.immune >= RESPONSE_WINDOW);
    for _ in 0..RESPONSE_WINDOW {
        f.advance(&mut events);
    }
    assert_eq!(f.feedback.control.locked_ticks, 0);
    assert_eq!(f.state, State::Neutral);
}

#[test]
fn bounded_adversarial_control_search_respects_the_existing_rules() {
    // Mọi nhánh của bảy lựa chọn ở độ sâu 5: khoảng ngắn/dài, stun, guard break, damage không stun.
    // Đây là tìm kiếm hữu hạn của luật hiện có, không là chứng minh cho roster tương lai.
    fn explore(budget: ControlBudget, remaining: u8, lock: u32) {
        if remaining == 0 {
            return;
        }
        for (damage, stun, gap) in [
            (45, 18, 0),
            (90, 30, 0),
            (160, 18, 1),
            (80, 0, 0),
            (0, 18, 6),
            (100, 18, 11),
            (45, 18, 12),
        ] {
            let mut next = budget;
            let mut previous_lock = lock;
            for _ in 0..gap {
                let released = next.tick(previous_lock > 0);
                previous_lock = if released {
                    0
                } else {
                    previous_lock.saturating_sub(1)
                };
            }
            let granted = next.grant(damage, stun, 1000);
            assert!(next.damage <= 250);
            let locked = if granted.release {
                0
            } else {
                previous_lock.max(granted.ticks)
            };
            let released = next.tick(locked > 0);
            assert!(next.locked_ticks <= LOCK_LIMIT);
            explore(
                next,
                remaining - 1,
                if released {
                    0
                } else {
                    locked.saturating_sub(1)
                },
            );
        }
    }
    explore(ControlBudget::default(), 5, 0);
}

#[test]
fn the_mach_fixture_records_real_spending_and_replays_every_checkpoint() {
    let mut session = crate::fixture::mach_ground(7);
    let mut ultimate = 0;
    let mut escape = 0;
    for tick in 0..2_000 {
        for command in crate::fixture::mach_commands(tick) {
            session.submit(command).unwrap();
        }
        for record in session.step().events {
            match record.event {
                Event::ActionStarted {
                    action: ActionKind::Ultimate(2),
                    ..
                } => ultimate += 1,
                Event::MachEscaped { .. } => escape += 1,
                _ => {}
            }
        }
    }
    assert_eq!((ultimate, escape), (1, 1));
    assert_eq!(session.world().fighter(A).mach.points(), 18);
    let parsed = crate::replay::Replay::parse(&session.replay().to_text()).unwrap();
    let playback = parsed.play().unwrap();
    assert_eq!(playback.world.state_hash(), session.world().state_hash());
    assert_eq!(
        playback.world.fighter(A).feedback,
        session.world().fighter(A).feedback
    );
}
