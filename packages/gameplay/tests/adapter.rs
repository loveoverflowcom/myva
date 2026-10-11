//! Adapter ECS headless: cùng fixture chạy qua lõi thuần, ECS server và ECS client phải cho cùng
//! hash từng tick; mirror khớp trạng thái lõi; tốc độ luật không phụ thuộc FPS.

use std::collections::BTreeSet;
use std::time::Duration;

use bevy_app::{App, Update};
use bevy_ecs::message::Messages;
use bevy_ecs::prelude::*;
use bevy_time::TimeUpdateStrategy;
use myva_gameplay::components::*;
use myva_gameplay::headless::{TickHistory, headless_app, run_until, tick};
use myva_gameplay::view::{self, FighterMirror, ProjectileMirror};
use myva_gameplay::{
    Authority, CommandRejected, EntityIndex, LoadSession, LocalInput, RewardIssued, RewardOutbox,
    ScriptedPlayer, SimEvent, SimSession,
};
use myva_sim::fixture::{self, Brawler, PLAYER_EPOCH, PLAYERS};
use myva_sim::kit::{ActionKind, LONG_LUU, PX};
use myva_sim::protocol::{Action, Attack, CommandFrame, EventRecord, Rejection};
use myva_sim::session::{Role, Session, SessionConfig};
use myva_sim::snapshot::{FighterView, ProjectileView};
use myva_sim::{EntityRef, Event, FighterId, SessionEpoch, State};

#[derive(Resource, Default)]
struct Collected(Vec<EventRecord>);

fn collect(mut reader: MessageReader<SimEvent>, mut collected: ResMut<Collected>) {
    collected
        .0
        .extend(reader.read().map(|SimEvent(record)| *record));
}

fn fixture_app(seed: u64, authority: Authority) -> App {
    let (session, ground) = fixture::training_ground(seed);
    let mut app = headless_app(session, authority);
    app.insert_resource(ScriptedPlayer(Brawler::new(
        seed,
        ground.player,
        PLAYER_EPOCH,
    )))
    .init_resource::<Collected>()
    .add_systems(Update, collect);
    app
}

fn hashes(app: &App) -> Vec<u64> {
    app.world()
        .resource::<TickHistory>()
        .0
        .iter()
        .map(|&(_, hash)| hash)
        .collect()
}

fn count<T: Component>(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<(), With<T>>().iter(world).count()
}

#[test]
fn runs_n_ticks_headless_with_one_tick_per_update() {
    let mut app = fixture_app(1, Authority::Server);
    let frames = run_until(&mut app, 600);
    assert_eq!(tick(&app), 600);
    // Frame đầu của `Time<Real>` có delta 0, sau đó mỗi frame đúng một tick.
    assert_eq!(frames, 601);
    assert_eq!(count::<Player>(&mut app), 1);
    assert_eq!(count::<Monster>(&mut app), 2);
    assert_eq!(count::<Npc>(&mut app), 1);
}

#[test]
fn core_ecs_server_and_ecs_client_agree_tick_by_tick() {
    const TICKS: u32 = 1_800;
    for seed in 0..6 {
        let core = fixture::fingerprint(seed, TICKS);
        let mut server = fixture_app(seed, Authority::Server);
        let mut client = fixture_app(seed, Authority::Client);
        run_until(&mut server, TICKS);
        run_until(&mut client, TICKS);
        assert_eq!(hashes(&server), core, "seed {seed}: ECS server lệch lõi");
        assert_eq!(hashes(&client), core, "seed {seed}: ECS client lệch lõi");
        let ticks: Vec<u32> = server
            .world()
            .resource::<TickHistory>()
            .0
            .iter()
            .map(|&(t, _)| t)
            .collect();
        assert_eq!(ticks, (0..TICKS).collect::<Vec<_>>());
    }
}

#[test]
fn mirror_matches_core_state_every_tick() {
    let mut app = fixture_app(7, Authority::Server);
    let mut projectiles_seen = BTreeSet::new();
    for _ in 0..2_400 {
        app.update();
        let world = app.world_mut();
        let snapshot = world.resource::<SimSession>().get().snapshot();
        let index_len = world.resource::<EntityIndex>().len();
        assert_eq!(
            index_len,
            snapshot.fighters.len() + snapshot.npcs.len() + snapshot.projectiles.len()
        );
        for view in &snapshot.fighters {
            let entity = world
                .resource::<EntityIndex>()
                .get(EntityRef::Fighter(view.id))
                .unwrap();
            let e = world.entity(entity);
            assert_eq!(e.get::<SimId>(), Some(&SimId(EntityRef::Fighter(view.id))));
            assert_eq!(
                e.get::<Position>(),
                Some(&Position {
                    x: view.x,
                    y: view.y
                })
            );
            assert_eq!(
                e.get::<Health>(),
                Some(&Health {
                    hp: view.hp,
                    max: view.max_hp
                })
            );
            assert_eq!(e.get::<Stance>(), Some(&Stance(view.state)));
            assert_eq!(e.get::<CurrentAction>(), Some(&CurrentAction(view.action)));
            assert_eq!(e.get::<Cooldowns>(), Some(&Cooldowns(view.cooldowns)));
            assert_eq!(
                e.get::<StatusEffects>(),
                Some(&StatusEffects(view.statuses.clone()))
            );
            assert_eq!(e.get::<AttackBox>(), Some(&AttackBox(view.attack_box)));
            assert_eq!(e.get::<Ai>().map(|a| a.0), view.ai);
            assert_eq!(e.contains::<Monster>(), view.role == Role::Monster);
        }
        for view in &snapshot.projectiles {
            projectiles_seen.insert(view.id);
            let entity = world
                .resource::<EntityIndex>()
                .get(EntityRef::Projectile(view.id))
                .expect("đạn đang bay phải có entity");
            let e = world.entity(entity);
            assert_eq!(e.get::<Volume>(), Some(&Volume(view.rect)));
            assert_eq!(e.get::<Owner>(), Some(&Owner(view.owner)));
        }
        let live = world
            .query_filtered::<(), With<Projectile>>()
            .iter(world)
            .count();
        assert_eq!(live, snapshot.projectiles.len(), "đạn đã mất phải bị xóa");
        let (fighters, projectiles) = mirrored_views(world);
        assert_eq!(
            fighters, snapshot.fighters,
            "khung nhìn từ mirror = snapshot"
        );
        assert_eq!(projectiles, snapshot.projectiles);
    }
    assert!(
        !projectiles_seen.is_empty(),
        "fixture phải sinh đạn để kiểm tra spawn/despawn"
    );
}

fn mirrored_views(world: &mut World) -> (Vec<FighterView>, Vec<ProjectileView>) {
    world
        .run_system_cached(
            |fighters: Query<FighterMirror>,
             projectiles: Query<ProjectileMirror, With<Projectile>>| {
                (view::fighters(&fighters), view::projectiles(&projectiles))
            },
        )
        .unwrap()
}

#[test]
fn domain_ids_survive_entity_reuse() {
    let mut app = fixture_app(7, Authority::Client);
    let mut seen = BTreeSet::new();
    let mut retired = BTreeSet::new();
    for _ in 0..2_400 {
        app.update();
        let world = app.world_mut();
        let live: BTreeSet<_> = world
            .query_filtered::<&SimId, With<Projectile>>()
            .iter(world)
            .map(|id| id.0)
            .collect();
        for id in &live {
            assert!(!retired.contains(id), "ID miền {id:?} bị tái sử dụng");
        }
        retired.extend(seen.difference(&live).copied());
        seen.extend(live);
        for (id, entity) in world.resource::<EntityIndex>().iter() {
            assert_eq!(world.entity(entity).get::<SimId>(), Some(&SimId(id)));
        }
    }
    assert!(
        retired.len() >= 2,
        "cần nhiều đạn sinh rồi mất: {retired:?}"
    );
}

/// Frame dài/ngắn khác nhau chỉ đổi số tick mỗi frame, không đổi luật hay tốc độ mô phỏng.
#[test]
fn rule_speed_is_independent_of_frame_rate() {
    const TICKS: u32 = 900;
    let core = fixture::fingerprint(11, TICKS);
    let profiles: [&[u64]; 4] = [
        &[33_333_333],
        &[16_666_667],
        &[6_944_444],
        // Jitter: frame nhanh, chậm, rất chậm.
        &[5_000_000, 30_000_000, 11_000_000, 48_000_000],
    ];
    for frames in profiles {
        let mut app = fixture_app(11, Authority::Server);
        let mut elapsed = Duration::ZERO;
        let mut frame = 0;
        while tick(&app) < TICKS {
            let delta = Duration::from_nanos(frames[frame % frames.len()]);
            app.insert_resource(TimeUpdateStrategy::ManualDuration(delta));
            app.update();
            // Frame đầu tiên của `Time<Real>` không tính thời gian.
            if frame > 0 {
                elapsed += delta;
            }
            frame += 1;
            let expected = (elapsed.as_secs_f64() * 60.0).floor() as i64;
            assert!(
                (i64::from(tick(&app)) - expected).abs() <= 1,
                "{frames:?}: tick {} sau {:?}",
                tick(&app),
                elapsed
            );
        }
        assert_eq!(hashes(&app)[..TICKS as usize], core[..], "{frames:?}");
    }
}

#[test]
fn only_the_server_issues_rewards_and_only_once() {
    for seed in [1, 7, 12] {
        let mut server = fixture_app(seed, Authority::Server);
        let mut client = fixture_app(seed, Authority::Client);
        let issued = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = issued.clone();
        server.add_systems(Update, move |mut reader: MessageReader<RewardIssued>| {
            sink.lock().unwrap().extend(reader.read().map(|m| m.0));
        });
        run_until(&mut server, 3_600);
        run_until(&mut client, 3_600);

        let session = server.world().resource::<SimSession>().get();
        let defeated: Vec<FighterId> = server
            .world()
            .resource::<Collected>()
            .0
            .iter()
            .filter_map(|r| match r.event {
                Event::Downed { fighter } if session.role(fighter) == Role::Monster => {
                    Some(fighter)
                }
                _ => None,
            })
            .collect();
        assert!(!defeated.is_empty(), "seed {seed}: fixture phải hạ quái");
        let claims = issued.lock().unwrap().clone();
        let keys: BTreeSet<_> = claims.iter().map(|c| c.key()).collect();
        assert_eq!(claims.len(), defeated.len(), "seed {seed}");
        assert_eq!(keys.len(), claims.len(), "seed {seed}: claim trùng khóa");
        assert!(keys.iter().all(|k| k.instance == fixture::INSTANCE));
        assert_eq!(
            server.world().resource::<RewardOutbox>().issued(),
            defeated.len()
        );

        // Client chạy cùng luật, thấy cùng quái bị hạ, nhưng không có đường cấp phần thưởng.
        assert_eq!(
            client.world().resource::<Collected>().0,
            server.world().resource::<Collected>().0
        );
        assert!(client.world().get_resource::<RewardOutbox>().is_none());
        assert!(
            client
                .world()
                .get_resource::<Messages<RewardIssued>>()
                .is_none()
        );
    }
}

fn duel_app(frame_nanos: u64) -> (App, FighterId) {
    let mut session = Session::new(SessionConfig::new(1, 1));
    let player = session.spawn_player(&LONG_LUU, 400 * PX, 1, Some(PLAYERS), SessionEpoch(1));
    let mut app = headless_app(session, Authority::Client);
    app.insert_resource(LocalInput::new(player, SessionEpoch(1), None))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_nanos(
            frame_nanos,
        )))
        .init_resource::<Collected>()
        .add_systems(Update, collect);
    app.update();
    (app, player)
}

fn started(app: &App) -> Vec<ActionKind> {
    app.world()
        .resource::<Collected>()
        .0
        .iter()
        .filter_map(|r| match r.event {
            Event::ActionStarted { action, .. } => Some(action),
            _ => None,
        })
        .collect()
}

#[test]
fn one_press_is_one_action_at_any_frame_rate() {
    // 240 FPS: bốn frame mỗi tick; 30 FPS: hai tick mỗi frame.
    for nanos in [4_166_667, 33_333_333] {
        let (mut app, _) = duel_app(nanos);
        app.world_mut()
            .resource_mut::<LocalInput>()
            .press(Action::Attack(Attack::Light));
        for _ in 0..90 {
            app.update();
        }
        assert_eq!(started(&app), [ActionKind::Light(0)], "{nanos} ns/frame");
    }
}

#[test]
fn held_guard_persists_and_release_all_clears_it() {
    let (mut app, player) = duel_app(16_666_667);
    app.world_mut().resource_mut::<LocalInput>().set_guard(true);
    for _ in 0..10 {
        app.update();
    }
    let stance = |app: &mut App| {
        let entity = app
            .world()
            .resource::<EntityIndex>()
            .get(EntityRef::Fighter(player))
            .unwrap();
        app.world().entity(entity).get::<Stance>().unwrap().0
    };
    assert!(matches!(stance(&mut app), State::Guard { .. }));
    app.world_mut().resource_mut::<LocalInput>().release_all();
    app.update();
    assert_eq!(stance(&mut app), State::Neutral);
}

#[test]
fn client_cannot_command_a_monster() {
    let (session, ground) = fixture::training_ground(2);
    let monster = ground.monsters[0];
    let mut app = headless_app(session, Authority::Client);
    app.insert_resource(LocalInput::new(monster, PLAYER_EPOCH, None));
    let rejected = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = rejected.clone();
    app.add_systems(Update, move |mut reader: MessageReader<CommandRejected>| {
        sink.lock().unwrap().extend(reader.read().copied());
    });
    run_until(&mut app, 30);
    let rejected = rejected.lock().unwrap();
    assert_eq!(rejected.len(), 30);
    assert!(rejected.iter().all(|r| r.actor == monster
        && r.reason
            == Rejection::NotController {
                actor: monster,
                session: PLAYER_EPOCH
            }));
}

#[test]
fn session_can_arrive_after_the_app_starts() {
    use bevy_time::TimePlugin;
    use myva_gameplay::GameplayPlugin;

    let mut app = App::new();
    app.add_plugins((TimePlugin, GameplayPlugin::new(Authority::Server)))
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    for _ in 0..5 {
        app.update();
    }
    assert!(app.world().resource::<EntityIndex>().is_empty());

    let (session, _) = fixture::training_ground(3);
    app.insert_resource(SimSession::new(session));
    run_until(&mut app, 10);
    assert_eq!(count::<Monster>(&mut app), 2);
    assert_eq!(tick(&app), 10);
}

#[test]
fn set_frame_replaces_the_pending_intent() {
    let (mut app, _) = duel_app(16_666_667);
    {
        let mut input = app.world_mut().resource_mut::<LocalInput>();
        input.press(Action::Attack(Attack::Heavy));
        input.set_guard(true);
        assert!(input.has_press());
        input.set_frame(CommandFrame {
            move_x: 1,
            guard: false,
            action: Some(Action::Attack(Attack::Light)),
        });
    }
    app.update();
    assert_eq!(started(&app), [ActionKind::Light(0)], "đòn nặng đã bị thay");
    assert!(!app.world().resource::<LocalInput>().has_press());
}

#[test]
fn halted_session_stops_ticking_but_keeps_its_mirror() {
    let mut app = fixture_app(4, Authority::Client);
    run_until(&mut app, 120);
    app.world_mut().resource_mut::<SimSession>().halt();
    let fighters = count::<SimId>(&mut app);
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(tick(&app), 120);
    assert!(!app.world().resource::<SimSession>().is_running());
    assert_eq!(count::<SimId>(&mut app), fighters);
}

#[test]
fn load_session_replaces_the_instance_and_its_entities() {
    let mut app = fixture_app(5, Authority::Client);
    run_until(&mut app, 300);
    let old: Vec<Entity> = app
        .world()
        .resource::<EntityIndex>()
        .iter()
        .map(|(_, entity)| entity)
        .collect();

    // Phiên mới chỉ có một người chơi, không quái, không NPC.
    let mut session = Session::new(SessionConfig::new(9, 9));
    let player = session.spawn_player(&LONG_LUU, 300 * PX, 1, None, SessionEpoch(1));
    app.world_mut().remove_resource::<ScriptedPlayer>();
    app.world_mut().commands().queue(LoadSession(session));
    app.world_mut().flush();

    assert_eq!(tick(&app), 0);
    assert!(old.iter().all(|&e| app.world().get_entity(e).is_err()));
    assert_eq!(count::<Monster>(&mut app), 0);
    assert_eq!(count::<Npc>(&mut app), 0);
    assert_eq!(count::<Player>(&mut app), 1);
    let world = app.world_mut();
    let (fighters, _) = mirrored_views(world);
    assert_eq!(
        fighters,
        world.resource::<SimSession>().get().snapshot().fighters
    );
    assert_eq!(fighters[0].id, player);

    app.insert_resource(LocalInput::new(player, SessionEpoch(1), None));
    run_until(&mut app, 60);
    assert_eq!(tick(&app), 60);
}
