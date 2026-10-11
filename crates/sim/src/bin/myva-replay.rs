//! Kiểm tra và giải thích replay của lõi mô phỏng.
//!
//! ```text
//! myva-replay verify <file>   chạy lại và so mọi mốc hash
//! myva-replay events <file>   in sự kiện theo tick: ra đòn, trúng, đỡ, phản công, hiệu ứng, bị hạ
//! myva-replay demo <file>     ghi trận bot B1 (phản ứng 250 ms) đánh Kẻ Giữ Đập
//! ```
//!
//! Hash chỉ so được với replay ghi từ cùng build và cùng target.

use std::process::ExitCode;

use myva_sim::boss::{BossBrain, KE_GIU_DAP};
use myva_sim::bot::PatternReader;
use myva_sim::replay::{Recorder, Replay};
use myva_sim::{Event, FighterId, LONG_LUU, PX, State, World};

const USAGE: &str = "dùng: myva-replay <verify|events|demo> <file>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command, path] if command == "verify" => verify(path),
        [command, path] if command == "events" => events(path),
        [command, path] if command == "demo" => demo(path),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn load(path: &str) -> Result<Replay, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    Replay::parse(&text).map_err(|e| format!("{path}: {e}"))
}

fn verify(path: &str) -> Result<(), String> {
    let replay = load(path)?;
    let playback = replay.play().map_err(|e| format!("{path}: {e}"))?;
    println!(
        "khớp {} mốc hash qua {} tick; hash cuối {:016x}",
        replay.checkpoints.len(),
        replay.ticks,
        playback.world.state_hash()
    );
    for fighter in playback.world.fighters() {
        println!(
            "#{} {}: HP {}/{}, {:?}",
            fighter.id.0, fighter.kit.lineage, fighter.hp, fighter.kit.body.max_hp, fighter.state
        );
    }
    Ok(())
}

fn events(path: &str) -> Result<(), String> {
    let playback = load(path)?.play().map_err(|e| format!("{path}: {e}"))?;
    for (tick, event) in &playback.events {
        println!("t{tick:>6}  {}", describe(&playback.world, event));
    }
    Ok(())
}

fn describe(world: &World, event: &Event) -> String {
    let who = |id: FighterId| format!("#{} {}", id.0, world.fighter(id).kit.lineage);
    let action = |id: FighterId, action| world.fighter(id).kit.spec(action).name;
    match *event {
        Event::InputRejected { fighter, seq } => {
            format!("{} input seq {seq} bị từ chối", who(fighter))
        }
        Event::ActionStarted { fighter, action: a } => {
            format!("{} bắt đầu {}", who(fighter), action(fighter, a))
        }
        Event::Hit {
            attacker,
            target,
            action: a,
            damage,
        } => format!(
            "{} trúng {} bằng {}: -{damage}",
            who(attacker),
            who(target),
            action(attacker, a)
        ),
        Event::Blocked {
            attacker,
            target,
            action: a,
            perfect,
        } => format!(
            "{} đỡ {} của {}{}",
            who(target),
            action(attacker, a),
            who(attacker),
            if perfect { " (hoàn hảo)" } else { "" }
        ),
        Event::Countered {
            counterer,
            attacker,
            damage,
        } => format!("{} phản công {}: -{damage}", who(counterer), who(attacker)),
        Event::GuardBroken { fighter } => format!("{} vỡ thế đỡ", who(fighter)),
        Event::Downed { fighter } => format!("{} bị hạ", who(fighter)),
        Event::StatusApplied {
            target,
            source,
            kind,
            percent,
            ticks,
        } => format!(
            "{} chịu {kind:?} {percent}% trong {ticks} tick từ {}",
            who(target),
            who(source)
        ),
        Event::StatusEnded { fighter, kind } => format!("{} hết {kind:?}", who(fighter)),
        Event::Interacted { fighter, npc } => format!(
            "{} tương tác {}",
            who(fighter),
            world.npcs()[usize::from(npc.0)].spec.name
        ),
    }
}

fn demo(path: &str) -> Result<(), String> {
    let mut world = World::new();
    let player = world.spawn(&LONG_LUU, 400 * PX, 1);
    let boss = world.spawn(&KE_GIU_DAP, 1_100 * PX, -1);
    let mut recorder = Recorder::new(&world, 60);
    let (mut brain, mut reader) = (BossBrain::new(), PatternReader::with_reaction_ms(250));
    let limit = 5 * 60 * 60;
    while world.tick() < limit
        && [player, boss]
            .iter()
            .all(|&id| world.fighter(id).state != State::Downed)
    {
        let inputs = [
            (player, reader.next_frame(&world, player, boss)),
            (boss, brain.next_frame(&world, boss, player)),
        ];
        recorder.step(&mut world, &inputs);
    }
    let replay = recorder.snapshot(&world);
    std::fs::write(path, replay.to_text()).map_err(|e| format!("{path}: {e}"))?;
    println!(
        "đã ghi {} tick vào {path}; người chơi {} HP, boss {} HP",
        replay.ticks,
        world.fighter(player).hp,
        world.fighter(boss).hp
    );
    Ok(())
}
