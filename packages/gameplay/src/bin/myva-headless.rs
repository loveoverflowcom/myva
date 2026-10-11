//! Chạy fixture phòng thử qua app Bevy ECS headless (không cửa sổ/GPU/audio) và đối chiếu hash
//! từng tick với đường server-only của lõi.
//!
//! ```text
//! myva-headless [--seed N] [--ticks N] [--authority server|client] [--replay FILE] [--hashes FILE]
//! ```
//!
//! `--hashes` ghi hash ECS sau mỗi tick (hex, mỗi dòng một tick) để so với bản WASM. Mã thoát
//! khác 0 khi hai đường lệch nhau.

use std::collections::BTreeMap;
use std::process::ExitCode;

use bevy_ecs::prelude::*;
use myva_gameplay::components::{Monster, Npc, Player, Projectile, SimId};
use myva_gameplay::headless::{TickHistory, headless_app, run_until};
use myva_gameplay::{Authority, LastTick, RewardOutbox, ScriptedPlayer, SimEvent, SimSession};
use myva_sim::fixture::{self, Brawler, PLAYER_EPOCH};
use myva_sim::{Event, State};

const USAGE: &str = "dùng: myva-headless [--seed N] [--ticks N] [--authority server|client] \
                     [--replay FILE] [--hashes FILE]";

struct Options {
    seed: u64,
    ticks: u32,
    authority: Authority,
    replay: Option<String>,
    hashes: Option<String>,
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        seed: 7,
        ticks: 60 * 60,
        authority: Authority::Server,
        replay: None,
        hashes: None,
    };
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| format!("{flag} thiếu giá trị\n{USAGE}"))
        };
        match flag.as_str() {
            "--seed" => options.seed = value()?.parse().map_err(|e| format!("--seed: {e}"))?,
            "--ticks" => options.ticks = value()?.parse().map_err(|e| format!("--ticks: {e}"))?,
            "--authority" => {
                options.authority = match value()?.as_str() {
                    "server" => Authority::Server,
                    "client" => Authority::Client,
                    other => return Err(format!("--authority lạ: {other}\n{USAGE}")),
                }
            }
            "--replay" => options.replay = Some(value()?.clone()),
            "--hashes" => options.hashes = Some(value()?.clone()),
            _ => return Err(USAGE.to_owned()),
        }
    }
    Ok(options)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse(&args).and_then(|options| run(&options)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(options: &Options) -> Result<(), String> {
    let (session, ground) = fixture::training_ground(options.seed);
    let mut app = headless_app(session, options.authority);
    app.insert_resource(ScriptedPlayer(Brawler::new(
        options.seed,
        ground.player,
        PLAYER_EPOCH,
    )));
    // Đọc sự kiện mỗi frame như presentation sẽ làm.
    app.add_systems(bevy_app::Update, count_events)
        .init_resource::<EventCounts>();
    let frames = run_until(&mut app, options.ticks);

    let ecs = app.world().resource::<TickHistory>().0.clone();
    let core: Vec<u64> = fixture::fingerprint(options.seed, options.ticks);
    let mismatch = ecs
        .iter()
        .zip(&core)
        .find(|((_, ecs), core)| ecs != *core)
        .map(|((tick, ecs), core)| (*tick, *ecs, *core));

    let session = app.world().resource::<SimSession>().get();
    println!(
        "seed {} · {:?} · {} tick qua {} frame · hash cuối {:016x}",
        options.seed,
        options.authority,
        session.world().tick(),
        frames,
        session.world().state_hash()
    );
    for fighter in session.world().fighters() {
        println!(
            "  #{} {:?} {}: HP {}/{}{}",
            fighter.id.0,
            session.role(fighter.id),
            fighter.kit.lineage,
            fighter.hp,
            fighter.kit.body.max_hp,
            if fighter.state == State::Downed {
                ", bị hạ"
            } else {
                ""
            }
        );
    }
    if let Some(replay) = &options.replay {
        std::fs::write(replay, session.replay().to_text()).map_err(|e| format!("{replay}: {e}"))?;
        println!("  đã ghi replay vào {replay}");
    }
    if let Some(path) = &options.hashes {
        let text: String = ecs
            .iter()
            .map(|(_, hash)| format!("{hash:016x}\n"))
            .collect();
        std::fs::write(path, text).map_err(|e| format!("{path}: {e}"))?;
        println!("  đã ghi {} hash vào {path}", ecs.len());
    }
    let world = app.world_mut();
    let counts = &world.resource::<EventCounts>().0;
    println!("  sự kiện: {counts:?}");
    let entities = [
        ("người chơi", count::<Player>(world)),
        ("quái", count::<Monster>(world)),
        ("NPC", count::<Npc>(world)),
        ("đạn đang bay", count::<Projectile>(world)),
    ];
    println!("  entity ECS: {entities:?}");
    match world.get_resource::<RewardOutbox>() {
        Some(outbox) => println!("  claim phần thưởng (server): {}", outbox.issued()),
        None => println!("  claim phần thưởng: không có (client không cấp)"),
    }
    if world.resource::<LastTick>().report().is_none() {
        return Err("chưa chạy tick nào".to_owned());
    }
    match mismatch {
        None if ecs.len() == core.len() => {
            println!(
                "  ECS và lõi server-only khớp {} hash từng tick",
                core.len()
            );
            Ok(())
        }
        None => Err(format!(
            "số tick lệch: ECS {} ≠ lõi {}",
            ecs.len(),
            core.len()
        )),
        Some((tick, ecs, core)) => Err(format!(
            "lệch ở tick {tick}: ECS {ecs:016x}, lõi {core:016x}"
        )),
    }
}

fn count<T: Component>(world: &mut World) -> usize {
    world
        .query_filtered::<(), (With<T>, With<SimId>)>()
        .iter(world)
        .count()
}

#[derive(Resource, Default)]
struct EventCounts(BTreeMap<&'static str, u32>);

fn count_events(mut reader: MessageReader<SimEvent>, mut counts: ResMut<EventCounts>) {
    for SimEvent(record) in reader.read() {
        let kind = match record.event {
            Event::InputRejected { .. } => "từ chối input",
            Event::ActionStarted { .. } => "ra đòn",
            Event::Hit { .. } => "trúng",
            Event::Blocked { .. } => "bị đỡ",
            Event::Countered { .. } => "phản công",
            Event::GuardBroken { .. } => "vỡ thế",
            Event::Downed { .. } => "bị hạ",
            Event::StatusApplied { .. } => "chịu hiệu ứng",
            Event::StatusEnded { .. } => "hết hiệu ứng",
            Event::Interacted { .. } => "tương tác",
        };
        *counts.0.entry(kind).or_default() += 1;
    }
}
