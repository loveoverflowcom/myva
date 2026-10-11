//! Trận client (`GameplayPlugin` + `MatchPlugin`, đúng hệ của bản web/native nhưng không cửa sổ,
//! GPU hay thiết bị) và trận chạy thẳng trên lõi (`Battle`) phải cho cùng hash từng tick, cùng
//! sự kiện, cùng kết quả và cùng replay. Input đi qua đường thiết bị thật: `Intent` →
//! `LocalInput` → cổng lệnh.

use bevy::prelude::*;
use bevy::time::{TimePlugin, TimeUpdateStrategy};
use myva_gameplay::headless::TickHistory;
use myva_gameplay::{Authority, GameplayPlugin, SimEvent, SimSession};
use myva_graybox::{ArenaView, GraySet, HostCommand, Intent, Match, MatchPlugin, insert_match};
use myva_sim::battle::{Battle, Bout, Mode, Outcome};
use myva_sim::bot::RandomBot;
use myva_sim::protocol::{CommandFrame, EventRecord};
use myva_sim::tick::Tick;

/// Lệnh người chơi theo tick, đọc như một thiết bị trong `GraySet::Devices`.
#[derive(Resource, Default)]
struct Script(Vec<CommandFrame>);

#[derive(Resource, Default)]
struct Collected(Vec<EventRecord>);

fn play_script(
    script: Res<Script>,
    sim: Res<SimSession>,
    mut intent: ResMut<Intent>,
    mut last_press: Local<Option<Tick>>,
) {
    let tick = sim.get().world().tick();
    let Some(frame) = script.0.get(tick as usize) else {
        return;
    };
    let input = frame.to_input(0);
    // Khung đầu không có tick nào chạy; chỉ bấm một lần cho mỗi tick.
    let pressed = if *last_press == Some(tick) {
        myva_sim::Buttons::NONE
    } else {
        *last_press = Some(tick);
        input.pressed
    };
    intent.add(input.move_x, input.held, pressed);
}

fn collect(mut reader: MessageReader<SimEvent>, mut collected: ResMut<Collected>) {
    collected
        .0
        .extend(reader.read().map(|SimEvent(record)| *record));
}

fn client(mode: Mode, autopilot: bool, script: Vec<CommandFrame>) -> App {
    let (mut bout, session) = Bout::start(mode, 1);
    bout.set_autopilot(autopilot);
    let mut app = App::new();
    app.add_plugins((
        TimePlugin,
        GameplayPlugin::new(Authority::Client),
        MatchPlugin,
    ))
    .insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
    .init_resource::<TickHistory>()
    .init_resource::<Collected>()
    .insert_resource(Script(script))
    .add_systems(RunFixedMainLoop, play_script.in_set(GraySet::Devices))
    .add_systems(Update, collect);
    insert_match(&mut app, bout, session);
    app.finish();
    app.cleanup();
    app
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimSession>().get().world().tick()
}

/// Chạy tới khi trận lắng (phiên dừng) hoặc hết `ticks`.
fn run(app: &mut App, ticks: Tick) {
    for _ in 0..ticks * 2 + 10 {
        if !app.world().resource::<SimSession>().is_running() || tick(app) >= ticks {
            return;
        }
        app.update();
    }
    panic!("vòng fixed không tiến: tick {}", tick(app));
}

/// Cùng kịch bản chạy thẳng trên lõi; trả hash sau từng tick.
fn core(mode: Mode, autopilot: bool, script: &[CommandFrame], ticks: Tick) -> (Battle, Vec<u64>) {
    let mut battle = Battle::new(mode, 1);
    battle.bout_mut().set_autopilot(autopilot);
    let mut hashes = Vec::new();
    while battle.world().tick() < ticks {
        let frame = script
            .get(battle.world().tick() as usize)
            .copied()
            .unwrap_or(CommandFrame::IDLE);
        let Some(report) = battle.step(frame) else {
            break;
        };
        hashes.push(report.hash);
    }
    (battle, hashes)
}

fn assert_same(app: &App, battle: &Battle, hashes: &[u64]) {
    let history: Vec<u64> = app
        .world()
        .resource::<TickHistory>()
        .0
        .iter()
        .map(|&(_, hash)| hash)
        .collect();
    assert_eq!(history.len(), hashes.len(), "số tick");
    if let Some(at) = history.iter().zip(hashes).position(|(a, b)| a != b) {
        panic!("client lệch lõi ở tick {at}");
    }
    let sim = app.world().resource::<SimSession>();
    assert_eq!(sim.get().replay().to_text(), battle.replay().to_text());
    let game = app.world().resource::<Match>();
    assert_eq!(game.bout.outcome(), battle.bout().outcome());
    for id in [battle.bout().player(), battle.bout().rival()] {
        assert_eq!(game.bout.tally(id), battle.bout().tally(id));
    }
}

#[test]
fn autopilot_boss_fight_matches_the_core_and_verifies() {
    let limit = 5 * 60 * 60;
    let mut app = client(Mode::Boss, true, Vec::new());
    run(&mut app, limit);
    let (battle, hashes) = core(Mode::Boss, true, &[], limit);
    assert_eq!(battle.bout().outcome(), Some(Outcome::Victory));
    assert_same(&app, &battle, &hashes);

    let game = app.world().resource::<Match>();
    let verified = game
        .verification
        .clone()
        .expect("trận lắng phải được kiểm chứng");
    assert_eq!(verified.unwrap().outcome, Some(Outcome::Victory));
    assert!(!app.world().resource::<SimSession>().is_running());

    // Phiên đã dừng: thêm khung hình không chạy thêm tick.
    let settled = tick(&app);
    app.update();
    assert_eq!(tick(&app), settled);
}

#[test]
fn device_input_reaches_the_core_unchanged() {
    // Kịch bản có đủ đi, đỡ, nhảy, lướt, đòn nhẹ/nặng và ba thuật.
    let mut bot = RandomBot::new(42);
    let script: Vec<_> = (0..2_400)
        .map(|_| CommandFrame::from_input(&bot.next_frame()))
        .collect();
    let mut app = client(Mode::Boss, false, script.clone());
    run(&mut app, 2_400);
    let (battle, hashes) = core(Mode::Boss, false, &script, 2_400);
    assert_same(&app, &battle, &hashes);

    let events = &app.world().resource::<Collected>().0;
    let player = battle.bout().player();
    let started = events
        .iter()
        .filter(|r| matches!(r.event, myva_sim::Event::ActionStarted { fighter, .. } if fighter == player))
        .count();
    assert!(started > 20, "kịch bản phải ra nhiều đòn: {started}");
    let ids: std::collections::BTreeSet<_> = events.iter().map(|r| r.id).collect();
    assert_eq!(ids.len(), events.len(), "sự kiện không lặp");
    assert_eq!(
        app.world().resource::<Match>().applied_presses as usize,
        script
            .iter()
            .take(hashes.len())
            .filter(|f| f.action.is_some())
            .count(),
        "mỗi cú bấm vào đúng một tick"
    );
}

#[test]
fn rematch_and_mode_switch_load_a_clean_instance() {
    let mut app = client(Mode::Boss, true, Vec::new());
    run(&mut app, 300);
    app.world_mut().write_message(HostCommand::SwitchMode);
    app.update();
    {
        let game = app.world().resource::<Match>();
        assert_eq!((game.bout.mode(), game.bout.round()), (Mode::Duel, 2));
        assert!(game.bout.autopilot(), "đổi chế độ giữ bot lái");
        assert!(game.verification.is_none());
    }
    // Khung này đã nạp phiên mới và chạy tick đầu của nó.
    assert_eq!(tick(&app), 1);
    let arena = app.world().resource::<ArenaView>().clone();
    assert_eq!(arena.fighters.len(), 2);
    assert!(
        arena.fighters.iter().all(|f| f.kit.id == "long-luu"),
        "đối thủ đấu tập dùng kit Long Lưu, không còn boss"
    );
    assert_eq!(arena.boss_phase, None);
    assert_eq!(arena.sparring, Some(true));

    // Tắt bot đấu tập giữa trận vẫn khớp lõi làm cùng việc ở cùng tick.
    run(&mut app, 120);
    app.world_mut()
        .write_message(HostCommand::ToggleSparringBot);
    run(&mut app, 600);
    assert_eq!(app.world().resource::<ArenaView>().sparring, Some(false));
    let mut battle = Battle::new(Mode::Duel, 2);
    battle.bout_mut().set_autopilot(true);
    let mut hashes = Vec::new();
    while battle.world().tick() < 600 {
        if battle.world().tick() == 120 {
            battle.toggle_sparring_bot();
        }
        hashes.push(battle.step(CommandFrame::IDLE).unwrap().hash);
    }
    let history = &app.world().resource::<TickHistory>().0;
    let duel: Vec<u64> = history[history.len() - 600..]
        .iter()
        .map(|&(_, h)| h)
        .collect();
    assert_eq!(duel, hashes);
    assert!(battle.verify().is_ok());
}

#[test]
fn paused_time_holds_the_tick_and_drops_presses() {
    // Tạm dừng có hiệu lực từ khung sau (thời gian ảo của khung này đã tính); cú bấm đến khi trận
    // đang dừng ở tick 31 không được phát lại khi chạy tiếp.
    let mut script = vec![CommandFrame::IDLE; 120];
    script[31] = CommandFrame {
        action: Some(myva_sim::protocol::Action::Attack(
            myva_sim::protocol::Attack::Light,
        )),
        ..CommandFrame::IDLE
    };
    let mut app = client(Mode::Duel, false, script);
    run(&mut app, 30);
    app.world_mut().write_message(HostCommand::SetPaused(true));
    app.update();
    assert_eq!(tick(&app), 31);
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(tick(&app), 31);
    app.world_mut().write_message(HostCommand::SetPaused(false));
    run(&mut app, 90);
    assert_eq!(
        app.world().resource::<Match>().applied_presses,
        0,
        "cú bấm lúc tạm dừng không được phát lại"
    );
}
