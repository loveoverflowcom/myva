//! Selected loadouts use the same session/FixedUpdate path and pins as the core battle.
use myva_gameplay::headless::{headless_app, run_until};
use myva_gameplay::{Authority, LastTick, SimSession};
use myva_sim::battle::{Battle, Bout, Mode, PLAYER_EPOCH};
use myva_sim::registry::legacy_fixture_loadout;
use myva_sim::replay::Replay;
use myva_sim::{CommandEnvelope, CommandFrame};

#[test]
fn pinned_loadout_has_per_tick_ecs_core_and_replay_parity() {
    for mode in [Mode::Boss, Mode::Duel] {
        for seed in [1, 7, 12] {
            let (mut bout, session) =
                Bout::start_with_loadout(mode, seed, legacy_fixture_loadout()).unwrap();
            bout.set_autopilot(true);
            let mut app = headless_app(session, Authority::Client);
            let mut battle = Battle::with_loadout(mode, seed, legacy_fixture_loadout()).unwrap();
            battle.bout_mut().set_autopilot(true);
            for tick in 0..600 {
                let world = app.world().resource::<SimSession>().get().world();
                let frame = bout.autopilot_command(world).unwrap();
                app.world_mut()
                    .resource_mut::<SimSession>()
                    .submit(CommandEnvelope::new(
                        PLAYER_EPOCH,
                        bout.player(),
                        tick + 1,
                        tick,
                        frame,
                    ))
                    .unwrap();
                run_until(&mut app, tick + 1);
                let report = battle.step(CommandFrame::IDLE).unwrap();
                assert_eq!(app.world().resource::<LastTick>().report(), Some(&report));
            }
            let replay = app.world().resource::<SimSession>().get().replay();
            assert_eq!(replay.to_text(), battle.replay().to_text());
            assert_eq!(
                Replay::parse(&replay.to_text())
                    .unwrap()
                    .play()
                    .unwrap()
                    .world
                    .state_hash(),
                battle.world().state_hash()
            );
        }
    }
}
