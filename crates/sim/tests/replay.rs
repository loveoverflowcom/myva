//! Replay tất định và invariant dưới bot ngẫu nhiên (bậc B0).

use myva_sim::bot::RandomBot;
use myva_sim::fighter::{MAX_HP, MAX_X, MIN_X};
use myva_sim::{FighterId, LONG_LUU, PX, State, World};

const TICKS: u32 = 60 * 60;

fn run(seed: u64, mut check: impl FnMut(&World)) -> Vec<u64> {
    let mut world = World::new();
    let a = world.spawn(&LONG_LUU, 600 * PX, 1);
    let b = world.spawn(&LONG_LUU, 700 * PX, -1);
    let (mut bot_a, mut bot_b) = (RandomBot::new(seed), RandomBot::new(seed ^ 0xA5A5));
    let mut hashes = Vec::new();
    for tick in 0..TICKS {
        world.step(&[(a, bot_a.next_frame()), (b, bot_b.next_frame())]);
        check(&world);
        if tick % 60 == 0 {
            hashes.push(world.state_hash());
        }
    }
    hashes
}

#[test]
fn same_seed_replays_identically() {
    assert_eq!(run(1, |_| {}), run(1, |_| {}));
    assert_ne!(run(1, |_| {}), run(2, |_| {}));
}

#[test]
fn random_bots_keep_invariants() {
    for seed in 0..20 {
        run(seed, |world| {
            for f in world.fighters() {
                assert!(f.hp <= MAX_HP);
                assert_eq!(f.hp == 0, f.state == State::Downed, "seed {seed}");
                assert!((MIN_X..=MAX_X).contains(&f.x));
                assert!(f.y >= 0);
                for meter in [f.stamina, f.energy, f.mach] {
                    assert!(meter.sub() <= meter.max_sub());
                }
            }
        });
    }
}

#[test]
fn spawned_ids_are_sequential() {
    let mut world = World::new();
    assert_eq!(world.spawn(&LONG_LUU, 0, 1), FighterId(0));
    assert_eq!(world.spawn(&LONG_LUU, 0, 1), FighterId(1));
    assert_eq!(world.fighter(FighterId(0)).x, MIN_X);
}

#[test]
fn random_bots_exercise_hit_resolution() {
    use myva_sim::Event;

    let mut world = World::new();
    let a = world.spawn(&LONG_LUU, 600 * PX, 1);
    let b = world.spawn(&LONG_LUU, 700 * PX, -1);
    let (mut bot_a, mut bot_b) = (RandomBot::new(3), RandomBot::new(4));
    let mut hits = 0;
    for _ in 0..TICKS {
        let events = world.step(&[(a, bot_a.next_frame()), (b, bot_b.next_frame())]);
        hits += events
            .iter()
            .filter(|e| matches!(e, Event::Hit { .. }))
            .count();
    }
    assert!(hits > 0, "bot B0 phải chạm tới nhánh trúng đòn");
}
