//! Replay dạng văn bản: kịch bản khởi tạo, input theo tick và các mốc hash để phát hiện lệch.
//!
//! Lõi chỉ chuyển đổi chuỗi; đọc/ghi file do công cụ `myva-replay` và client lo. Input của mọi
//! bên, kể cả boss, đều được ghi, nên chạy lại không cần bộ não boss hay bot.
//!
//! ```text
//! myva-replay 1
//! fighter long-luu 640000 1
//! input 0 0 1 1 0 8        # tick, nhân vật, seq, move_x, held, pressed (bitflag)
//! check 60 9f0c...         # hash sau khi chạy xong tick 60
//! end 3600
//! ```
//!
//! Hash chỉ so được trong cùng build và cùng target (combat-progression-balance.md §11).

use std::fmt::{self, Write as _};

use crate::boss::KE_GIU_DAP;
use crate::fighter::FighterId;
use crate::input::{Buttons, InputFrame};
use crate::kit::{Kit, LONG_LUU};
use crate::tick::Tick;
use crate::world::{Event, World};

pub const HEADER: &str = "myva-replay 1";

/// Các kit có thể xuất hiện trong replay.
pub const KITS: [&Kit; 2] = [&LONG_LUU, &KE_GIU_DAP];

pub fn kit_by_id(id: &str) -> Option<&'static Kit> {
    KITS.into_iter().find(|kit| kit.id == id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spawn {
    pub kit: &'static Kit,
    pub x: i32,
    pub facing: i8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Replay {
    pub spawns: Vec<Spawn>,
    /// Theo thứ tự tick tăng dần.
    pub inputs: Vec<(Tick, FighterId, InputFrame)>,
    /// Hash của thế giới sau khi chạy xong tick tương ứng.
    pub checkpoints: Vec<(Tick, u64)>,
    pub ticks: Tick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayError {
    Parse {
        line: usize,
        message: String,
    },
    Desync {
        tick: Tick,
        expected: u64,
        actual: u64,
    },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayError::Parse { line, message } => write!(f, "dòng {line}: {message}"),
            ReplayError::Desync {
                tick,
                expected,
                actual,
            } => write!(
                f,
                "lệch ở tick {tick}: hash ghi lại {expected:016x}, chạy lại {actual:016x}"
            ),
        }
    }
}

impl std::error::Error for ReplayError {}

/// Kết quả chạy lại: thế giới cuối và mọi sự kiện kèm tick.
#[derive(Debug)]
pub struct Playback {
    pub world: World,
    pub events: Vec<(Tick, Event)>,
}

impl Replay {
    pub fn world(&self) -> World {
        let mut world = World::new();
        for spawn in &self.spawns {
            world.spawn(spawn.kit, spawn.x, spawn.facing);
        }
        world
    }

    /// Chạy lại từ đầu; dừng ở mốc hash đầu tiên bị lệch.
    pub fn play(&self) -> Result<Playback, ReplayError> {
        let mut world = self.world();
        let mut events = Vec::new();
        let mut inputs = self.inputs.iter().peekable();
        let mut checkpoints = self.checkpoints.iter().peekable();
        let mut frame = Vec::new();
        while world.tick() < self.ticks {
            let tick = world.tick();
            frame.clear();
            while let Some(&(_, id, input)) = inputs.next_if(|(t, ..)| *t == tick) {
                frame.push((id, input));
            }
            events.extend(world.step(&frame).into_iter().map(|e| (tick, e)));
            if let Some(&(_, expected)) = checkpoints.next_if(|(t, _)| *t == tick + 1) {
                let actual = world.state_hash();
                if actual != expected {
                    return Err(ReplayError::Desync {
                        tick: tick + 1,
                        expected,
                        actual,
                    });
                }
            }
        }
        Ok(Playback { world, events })
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "{HEADER}");
        for spawn in &self.spawns {
            let _ = writeln!(out, "fighter {} {} {}", spawn.kit.id, spawn.x, spawn.facing);
        }
        let mut checkpoints = self.checkpoints.iter().peekable();
        for &(tick, id, input) in &self.inputs {
            while let Some((at, hash)) = checkpoints.next_if(|(at, _)| *at <= tick) {
                let _ = writeln!(out, "check {at} {hash:016x}");
            }
            let _ = writeln!(
                out,
                "input {tick} {} {} {} {} {}",
                id.0,
                input.seq,
                input.move_x,
                input.held.bits(),
                input.pressed.bits()
            );
        }
        for (at, hash) in checkpoints {
            let _ = writeln!(out, "check {at} {hash:016x}");
        }
        let _ = writeln!(out, "end {}", self.ticks);
        out
    }

    pub fn parse(text: &str) -> Result<Self, ReplayError> {
        let mut replay = Replay::default();
        let mut lines = text
            .lines()
            .enumerate()
            .map(|(i, line)| (i + 1, line.split('#').next().unwrap_or("").trim()))
            .filter(|(_, line)| !line.is_empty());
        match lines.next() {
            Some((_, HEADER)) => {}
            other => {
                return Err(ReplayError::Parse {
                    line: other.map_or(1, |(line, _)| line),
                    message: format!("thiếu header \"{HEADER}\""),
                });
            }
        }
        let mut ended = false;
        for (line, content) in lines {
            let error = |message: String| ReplayError::Parse { line, message };
            if ended {
                return Err(error("có dữ liệu sau dòng end".to_owned()));
            }
            let fields: Vec<&str> = content.split_whitespace().collect();
            let number = |index: usize| -> Result<i64, ReplayError> {
                fields
                    .get(index)
                    .ok_or_else(|| error(format!("thiếu trường thứ {index}")))?
                    .parse::<i64>()
                    .map_err(|e| error(format!("trường thứ {index}: {e}")))
            };
            let narrow = |value: i64, what: &str| error(format!("{what} ngoài phạm vi: {value}"));
            match fields[0] {
                "fighter" => {
                    let id = fields.get(1).copied().unwrap_or("");
                    let kit = kit_by_id(id).ok_or_else(|| error(format!("kit lạ: {id}")))?;
                    let x = number(2)?;
                    let facing = number(3)?;
                    replay.spawns.push(Spawn {
                        kit,
                        x: i32::try_from(x).map_err(|_| narrow(x, "x"))?,
                        facing: i8::try_from(facing).map_err(|_| narrow(facing, "hướng"))?,
                    });
                }
                "input" => {
                    let tick = number(1)?;
                    let tick = Tick::try_from(tick).map_err(|_| narrow(tick, "tick"))?;
                    if replay.inputs.last().is_some_and(|&(last, ..)| tick < last) {
                        return Err(error("tick input phải tăng dần".to_owned()));
                    }
                    let fighter = number(2)?;
                    let seq = number(3)?;
                    let move_x = number(4)?;
                    let held = number(5)?;
                    let pressed = number(6)?;
                    replay.inputs.push((
                        tick,
                        FighterId(u16::try_from(fighter).map_err(|_| narrow(fighter, "nhân vật"))?),
                        InputFrame {
                            seq: u32::try_from(seq).map_err(|_| narrow(seq, "seq"))?,
                            move_x: i8::try_from(move_x).map_err(|_| narrow(move_x, "move_x"))?,
                            held: buttons(held).ok_or_else(|| narrow(held, "held"))?,
                            pressed: buttons(pressed).ok_or_else(|| narrow(pressed, "pressed"))?,
                        },
                    ));
                }
                "check" => {
                    let tick = number(1)?;
                    let hash = fields
                        .get(2)
                        .ok_or_else(|| error("thiếu hash".to_owned()))?;
                    replay.checkpoints.push((
                        Tick::try_from(tick).map_err(|_| narrow(tick, "tick"))?,
                        u64::from_str_radix(hash, 16).map_err(|e| error(format!("hash: {e}")))?,
                    ));
                }
                "end" => {
                    let ticks = number(1)?;
                    replay.ticks = Tick::try_from(ticks).map_err(|_| narrow(ticks, "tick"))?;
                    ended = true;
                }
                other => return Err(error(format!("loại dòng lạ: {other}"))),
            }
        }
        if !ended {
            return Err(ReplayError::Parse {
                line: text.lines().count(),
                message: "thiếu dòng end".to_owned(),
            });
        }
        Ok(replay)
    }
}

fn buttons(bits: i64) -> Option<Buttons> {
    let bits = u16::try_from(bits).ok()?;
    let all = [
        Buttons::JUMP,
        Buttons::DASH,
        Buttons::GUARD,
        Buttons::LIGHT,
        Buttons::HEAVY,
        Buttons::SKILL1,
        Buttons::SKILL2,
        Buttons::SKILL3,
    ];
    let mut out = Buttons::NONE;
    for button in all {
        if bits & button.bits() != 0 {
            out |= button;
        }
    }
    (out.bits() == bits).then_some(out)
}

/// Ghi replay trong khi chạy: thay `world.step(...)` bằng `recorder.step(&mut world, ...)`.
#[derive(Clone, Debug)]
pub struct Recorder {
    replay: Replay,
    checkpoint_every: u32,
}

impl Recorder {
    /// Bắt đầu ghi một thế giới vừa tạo (tick 0); `checkpoint_every` tick lưu một mốc hash.
    pub fn new(world: &World, checkpoint_every: u32) -> Self {
        assert_eq!(world.tick(), 0, "replay phải bắt đầu từ tick 0");
        Self {
            replay: Replay {
                spawns: world
                    .fighters()
                    .iter()
                    .map(|f| Spawn {
                        kit: f.kit,
                        x: f.x,
                        facing: f.facing,
                    })
                    .collect(),
                ..Replay::default()
            },
            checkpoint_every: checkpoint_every.max(1),
        }
    }

    pub fn step(&mut self, world: &mut World, inputs: &[(FighterId, InputFrame)]) -> Vec<Event> {
        let tick = world.tick();
        self.replay
            .inputs
            .extend(inputs.iter().map(|&(id, input)| (tick, id, input)));
        let events = world.step(inputs);
        self.replay.ticks = world.tick();
        if world.tick().is_multiple_of(self.checkpoint_every) {
            self.replay
                .checkpoints
                .push((world.tick(), world.state_hash()));
        }
        events
    }

    /// Replay đến hiện tại, có thêm mốc hash cuối để chạy lại kiểm tra được trạng thái cuối.
    pub fn snapshot(&self, world: &World) -> Replay {
        let mut replay = self.replay.clone();
        if replay.checkpoints.last().map(|&(t, _)| t) != Some(world.tick()) {
            replay.checkpoints.push((world.tick(), world.state_hash()));
        }
        replay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boss::BossBrain;
    use crate::bot::{PatternReader, RandomBot};
    use crate::kit::PX;

    fn recorded_duel(ticks: u32) -> (Replay, World) {
        let mut world = World::new();
        let a = world.spawn(&LONG_LUU, 600 * PX, 1);
        let b = world.spawn(&LONG_LUU, 700 * PX, -1);
        let mut recorder = Recorder::new(&world, 60);
        let (mut bot_a, mut bot_b) = (RandomBot::new(5), RandomBot::new(6));
        for _ in 0..ticks {
            recorder.step(
                &mut world,
                &[(a, bot_a.next_frame()), (b, bot_b.next_frame())],
            );
        }
        (recorder.snapshot(&world), world)
    }

    #[test]
    fn text_roundtrip_replays_to_the_same_state() {
        let (replay, world) = recorded_duel(1_000);
        let text = replay.to_text();
        let parsed = Replay::parse(&text).unwrap();
        assert_eq!(parsed, replay);
        let playback = parsed.play().unwrap();
        assert_eq!(playback.world.state_hash(), world.state_hash());
        assert_eq!(
            parsed.checkpoints.last(),
            Some(&(1_000, world.state_hash()))
        );
    }

    #[test]
    fn tampered_input_is_reported_as_desync() {
        let (mut replay, _) = recorded_duel(600);
        // Ở tick 0 nhân vật đang đứng tự do, nên đổi hướng đi chắc chắn đổi vị trí. Đổi nút đòn
        // thì chưa chắc: nhân vật bận sẽ từ chối cả hai lựa chọn và trạng thái không đổi.
        let (tick, _, input) = &mut replay.inputs[0];
        assert_eq!(*tick, 0);
        input.move_x = if input.move_x > 0 { -1 } else { 1 };
        assert!(matches!(
            replay.play(),
            Err(ReplayError::Desync { tick: 60, .. })
        ));
    }

    #[test]
    fn boss_fight_replays_without_the_brain() {
        let mut world = World::new();
        let player = world.spawn(&LONG_LUU, 400 * PX, 1);
        let boss = world.spawn(&KE_GIU_DAP, 1_100 * PX, -1);
        let mut recorder = Recorder::new(&world, 120);
        let (mut brain, mut reader) = (BossBrain::new(), PatternReader::with_reaction_ms(250));
        for _ in 0..1_500 {
            let inputs = [
                (player, reader.next_frame(&world, player, boss)),
                (boss, brain.next_frame(&world, boss, player)),
            ];
            recorder.step(&mut world, &inputs);
        }
        let replay = Replay::parse(&recorder.snapshot(&world).to_text()).unwrap();
        assert_eq!(
            replay.play().unwrap().world.state_hash(),
            world.state_hash()
        );
    }

    #[test]
    fn parse_errors_name_the_line() {
        let bad_kit = format!("{HEADER}\nfighter ai-do 0 1\nend 0\n");
        assert!(matches!(
            Replay::parse(&bad_kit),
            Err(ReplayError::Parse { line: 2, .. })
        ));
        let bad_buttons = format!("{HEADER}\ninput 0 0 1 0 0 65535\nend 1\n");
        assert!(matches!(
            Replay::parse(&bad_buttons),
            Err(ReplayError::Parse { line: 2, .. })
        ));
        assert!(Replay::parse("myva-replay 9\nend 0\n").is_err());
        assert!(Replay::parse(&format!("{HEADER}\n")).is_err());
    }
}
