//! Boss graybox: đọc pattern thì thắng được, đứng yên thì thua (work-plan 020, review boundary).

use std::collections::BTreeSet;

use myva_sim::boss::{BossBrain, BossPhase, KE_GIU_DAP};
use myva_sim::bot::PatternReader;
use myva_sim::{FighterId, InputFrame, LONG_LUU, PX, State, World};

/// Năm phút ở 60 Hz.
const LIMIT: u32 = 5 * 60 * 60;

struct Fight {
    world: World,
    player: FighterId,
    boss: FighterId,
    brain: BossBrain,
}

impl Fight {
    fn new() -> Self {
        let mut world = World::new();
        let player = world.spawn(&LONG_LUU, 400 * PX, 1);
        let boss = world.spawn(&KE_GIU_DAP, 1_100 * PX, -1);
        Self {
            world,
            player,
            boss,
            brain: BossBrain::new(),
        }
    }

    fn over(&self) -> bool {
        [self.player, self.boss]
            .iter()
            .any(|&id| self.world.fighter(id).state == State::Downed)
    }

    fn step(&mut self, player_input: InputFrame) {
        let boss_input = self.brain.next_frame(&self.world, self.boss, self.player);
        self.world
            .step(&[(self.player, player_input), (self.boss, boss_input)]);
    }
}

/// combat.md §10 dự trù 250 ms phản ứng trong 700 ms tín hiệu tối thiểu.
#[test]
fn pattern_reader_with_250ms_reaction_beats_the_boss() {
    let mut fight = Fight::new();
    let mut reader = PatternReader::with_reaction_ms(250);
    let mut phases = BTreeSet::new();
    while !fight.over() && fight.world.tick() < LIMIT {
        let input = reader.next_frame(&fight.world, fight.player, fight.boss);
        fight.step(input);
        phases.insert(fight.brain.phase());
    }
    let player = fight.world.fighter(fight.player);
    let boss = fight.world.fighter(fight.boss);
    assert_eq!(
        boss.state,
        State::Downed,
        "sau {} tick boss còn {} HP",
        fight.world.tick(),
        boss.hp
    );
    assert!(player.hp > 0);
    assert!(
        phases.contains(&BossPhase::Combine),
        "phải đi qua cả ba pha: {phases:?}"
    );
}

#[test]
fn standing_still_loses() {
    let mut fight = Fight::new();
    let mut seq = 0;
    while !fight.over() && fight.world.tick() < LIMIT {
        seq += 1;
        fight.step(InputFrame {
            seq,
            ..InputFrame::default()
        });
    }
    assert_eq!(fight.world.fighter(fight.player).state, State::Downed);
}
