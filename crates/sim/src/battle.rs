//! Phiên đấu graybox (work-plan 020): ghép thế giới, nguồn input của từng bên, bản ghi replay và
//! kết quả trận. Thuần Rust như phần còn lại của lõi, nên client Bevy native, bản web và test
//! headless chạy cùng một vòng đấu.
//!
//! Client chỉ đưa ý định của người chơi; bộ não boss, bot và số thứ tự input do phiên quyết định.
//! Kết quả được kiểm chứng bằng cách chạy lại replay của chính trận đó ([`Battle::verify`]).

use crate::boss::{BossBrain, BossPhase, KE_GIU_DAP};
use crate::bot::{PatternReader, RandomBot};
use crate::fighter::{FighterId, State};
use crate::input::InputFrame;
use crate::kit::{ActionKind, LONG_LUU, PX};
use crate::replay::{Recorder, Replay, ReplayError};
use crate::tick::{TICK_HZ, Tick};
use crate::world::{Event, World};

/// Mỗi giây ghi một mốc hash.
pub const CHECKPOINT_EVERY: u32 = TICK_HZ;
/// Sau khi có người bị hạ, thế giới chạy thêm chừng này tick cho trạng thái lắng rồi dừng, nên
/// replay của một trận luôn có điểm kết thúc.
pub const SETTLE_TICKS: u32 = TICK_HZ;
/// Bot B1 lái hộ người chơi phản ứng theo ngân sách combat.md §10.
pub const AUTOPILOT_REACTION_MS: u32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Đánh boss Kẻ Giữ Đập.
    Boss,
    /// Đấu tập Long Lưu với bot B0.
    Duel,
}

impl Mode {
    pub const fn other(self) -> Self {
        match self {
            Mode::Boss => Mode::Duel,
            Mode::Duel => Mode::Boss,
        }
    }

    /// Mã ASCII ổn định cho cấu hình và bridge.
    pub const fn id(self) -> &'static str {
        match self {
            Mode::Boss => "boss",
            Mode::Duel => "duel",
        }
    }
}

/// Kết quả nhìn từ phía người chơi.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Outcome {
    Victory,
    Defeat,
    /// Hai bên bị hạ cùng tick.
    Draw,
}

impl Outcome {
    /// `None` khi chưa ai bị hạ. Chỉ đọc trạng thái thế giới, nên replay chạy lại suy ra được.
    pub fn of(world: &World, player: FighterId, rival: FighterId) -> Option<Self> {
        let downed = |id| world.fighter(id).state == State::Downed;
        match (downed(player), downed(rival)) {
            (false, false) => None,
            (false, true) => Some(Outcome::Victory),
            (true, false) => Some(Outcome::Defeat),
            (true, true) => Some(Outcome::Draw),
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Outcome::Victory => "victory",
            Outcome::Defeat => "defeat",
            Outcome::Draw => "draw",
        }
    }
}

/// Đòn cuối cùng gây damage cho một bên.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    Action(FighterId, ActionKind),
    /// Bị tư thế phản công của `FighterId` bắt.
    Counter(FighterId),
}

/// Số liệu playtest của một bên (combat.md §13): đòn đã dùng, damage, đỡ và nguyên nhân bị hạ.
/// Damage là giá trị danh nghĩa của đòn, kể cả phần vượt quá sinh lực còn lại.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub actions: u32,
    pub hits: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub blocks: u32,
    pub perfect_blocks: u32,
    pub counters: u32,
    pub guard_breaks: u32,
    pub last_hit_by: Option<Cause>,
}

#[derive(Clone, Debug)]
enum Pilot {
    Manual,
    Reader(PatternReader),
    Random(RandomBot),
}

#[derive(Clone, Debug)]
enum RivalControl {
    Boss(BossBrain),
    Random(RandomBot),
    Idle,
}

/// Replay của trận chạy lại khớp trạng thái đang chơi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verification {
    pub ticks: Tick,
    pub checkpoints: usize,
    pub hash: u64,
    pub outcome: Option<Outcome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerifyError {
    Replay(ReplayError),
    /// Chạy lại không lỗi nhưng thế giới hoặc kết quả khác trận đang chơi.
    Diverged {
        live: u64,
        replayed: u64,
    },
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::Replay(error) => error.fmt(f),
            VerifyError::Diverged { live, replayed } => write!(
                f,
                "replay lệch trận đang chơi: {live:016x} khác {replayed:016x}"
            ),
        }
    }
}

impl std::error::Error for VerifyError {}

#[derive(Clone, Debug)]
pub struct Battle {
    mode: Mode,
    round: u64,
    world: World,
    player: FighterId,
    rival: FighterId,
    pilot: Pilot,
    rival_control: RivalControl,
    recorder: Recorder,
    /// Phiên tự đánh số input của cả hai bên, nên đổi người lái giữa trận (tay ↔ bot) không làm
    /// khung mới bị từ chối vì `seq` cũ.
    seqs: [u32; 2],
    outcome: Option<(Outcome, Tick)>,
    tallies: [Tally; 2],
}

impl Battle {
    pub fn new(mode: Mode, round: u64) -> Self {
        let mut world = World::new();
        let (player, rival, rival_control) = match mode {
            Mode::Boss => (
                world.spawn(&LONG_LUU, 400 * PX, 1),
                world.spawn(&KE_GIU_DAP, 1_100 * PX, -1),
                RivalControl::Boss(BossBrain::new()),
            ),
            Mode::Duel => (
                world.spawn(&LONG_LUU, 640 * PX, 1),
                world.spawn(&LONG_LUU, 960 * PX, -1),
                RivalControl::Random(RandomBot::new(round)),
            ),
        };
        Self {
            mode,
            round,
            recorder: Recorder::new(&world, CHECKPOINT_EVERY),
            world,
            player,
            rival,
            pilot: Pilot::Manual,
            rival_control,
            seqs: [0; 2],
            outcome: None,
            tallies: [Tally::default(); 2],
        }
    }

    /// Trận mới ở vòng kế tiếp, giữ chế độ lái của người chơi.
    pub fn rematch(&self, mode: Mode) -> Self {
        let mut next = Self::new(mode, self.round + 1);
        next.set_autopilot(self.autopilot());
        next
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn player(&self) -> FighterId {
        self.player
    }

    pub fn rival(&self) -> FighterId {
        self.rival
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome.map(|(outcome, _)| outcome)
    }

    pub fn tally(&self, id: FighterId) -> &Tally {
        &self.tallies[usize::from(id.0)]
    }

    pub fn boss_phase(&self) -> Option<BossPhase> {
        match &self.rival_control {
            RivalControl::Boss(brain) => Some(brain.phase()),
            _ => None,
        }
    }

    /// `Some(bật)` ở chế độ đấu tập, `None` khi đánh boss.
    pub fn sparring_bot(&self) -> Option<bool> {
        match self.rival_control {
            RivalControl::Random(_) => Some(true),
            RivalControl::Idle => Some(false),
            RivalControl::Boss(_) => None,
        }
    }

    pub fn toggle_sparring_bot(&mut self) {
        self.rival_control = match self.rival_control {
            RivalControl::Random(_) => RivalControl::Idle,
            RivalControl::Idle => RivalControl::Random(RandomBot::new(self.round)),
            RivalControl::Boss(_) => return,
        };
    }

    /// Bot lái người chơi: B1 đọc tín hiệu khi đánh boss, B0 ngẫu nhiên khi đấu tập.
    pub fn autopilot(&self) -> bool {
        !matches!(self.pilot, Pilot::Manual)
    }

    pub fn set_autopilot(&mut self, on: bool) {
        if on == self.autopilot() {
            return;
        }
        self.pilot = match (on, self.mode) {
            (false, _) => Pilot::Manual,
            (true, Mode::Boss) => {
                Pilot::Reader(PatternReader::with_reaction_ms(AUTOPILOT_REACTION_MS))
            }
            (true, Mode::Duel) => Pilot::Random(RandomBot::new(self.round ^ 0x5EED)),
        };
    }

    /// Trận đã có kết quả và đã chạy hết nhịp lắng; `step` không làm gì nữa.
    pub fn is_settled(&self) -> bool {
        self.outcome
            .is_some_and(|(_, at)| self.world.tick() >= at + SETTLE_TICKS)
    }

    /// Chạy một tick. `manual` là ý định của người chơi; `seq` của nó bị bỏ qua và được đánh số
    /// lại. Khi bot đang lái, `manual` bị bỏ qua.
    pub fn step(&mut self, manual: InputFrame) -> Vec<Event> {
        if self.is_settled() {
            return Vec::new();
        }
        let world = &self.world;
        let player_frame = match &mut self.pilot {
            Pilot::Manual => manual,
            Pilot::Reader(reader) => reader.next_frame(world, self.player, self.rival),
            Pilot::Random(bot) => bot.next_frame(),
        };
        let rival_frame = match &mut self.rival_control {
            RivalControl::Boss(brain) => Some(brain.next_frame(world, self.rival, self.player)),
            RivalControl::Random(bot) => Some(bot.next_frame()),
            RivalControl::Idle => None,
        };
        let mut inputs = vec![(self.player, self.stamp(self.player, player_frame))];
        if let Some(frame) = rival_frame {
            inputs.push((self.rival, self.stamp(self.rival, frame)));
        }
        let events = self.recorder.step(&mut self.world, &inputs);
        for event in &events {
            self.count(event);
        }
        if self.outcome.is_none()
            && let Some(outcome) = Outcome::of(&self.world, self.player, self.rival)
        {
            self.outcome = Some((outcome, self.world.tick()));
        }
        events
    }

    fn stamp(&mut self, id: FighterId, frame: InputFrame) -> InputFrame {
        let seq = &mut self.seqs[usize::from(id.0)];
        *seq += 1;
        InputFrame { seq: *seq, ..frame }
    }

    fn count(&mut self, event: &Event) {
        let tallies = &mut self.tallies;
        let at = |id: FighterId| usize::from(id.0);
        match *event {
            Event::ActionStarted { fighter, .. } => tallies[at(fighter)].actions += 1,
            Event::Hit {
                attacker,
                target,
                action,
                damage,
            } => {
                tallies[at(attacker)].hits += 1;
                tallies[at(attacker)].damage_dealt += damage;
                tallies[at(target)].damage_taken += damage;
                tallies[at(target)].last_hit_by = Some(Cause::Action(attacker, action));
            }
            Event::Blocked {
                target, perfect, ..
            } => {
                tallies[at(target)].blocks += 1;
                tallies[at(target)].perfect_blocks += u32::from(perfect);
            }
            Event::Countered {
                counterer,
                attacker,
                damage,
            } => {
                tallies[at(counterer)].counters += 1;
                tallies[at(counterer)].damage_dealt += damage;
                tallies[at(attacker)].damage_taken += damage;
                tallies[at(attacker)].last_hit_by = Some(Cause::Counter(counterer));
            }
            Event::GuardBroken { fighter } => tallies[at(fighter)].guard_breaks += 1,
            Event::InputRejected { .. }
            | Event::Downed { .. }
            | Event::StatusApplied { .. }
            | Event::StatusEnded { .. }
            | Event::Interacted { .. } => {}
        }
    }

    /// Replay của trận tới tick hiện tại, có mốc hash cuối.
    pub fn replay(&self) -> Replay {
        self.recorder.snapshot(&self.world)
    }

    /// Ghi replay ra văn bản, đọc lại và chạy lại không cần bộ não boss hay bot, rồi so trạng
    /// thái cuối và kết quả với trận đang chơi. Đây là kiểm chứng cục bộ trên cùng build; kết quả
    /// online vẫn chỉ do server quyết định.
    pub fn verify(&self) -> Result<Verification, VerifyError> {
        let replay = Replay::parse(&self.replay().to_text()).map_err(VerifyError::Replay)?;
        let playback = replay.play().map_err(VerifyError::Replay)?;
        let (live, replayed) = (self.world.state_hash(), playback.world.state_hash());
        let outcome = Outcome::of(&playback.world, self.player, self.rival);
        if live != replayed || outcome != self.outcome() {
            return Err(VerifyError::Diverged { live, replayed });
        }
        Ok(Verification {
            ticks: replay.ticks,
            checkpoints: replay.checkpoints.len(),
            hash: replayed,
            outcome,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Buttons;

    fn idle() -> InputFrame {
        InputFrame::default()
    }

    fn press(pressed: Buttons) -> InputFrame {
        InputFrame {
            pressed,
            ..InputFrame::default()
        }
    }

    #[test]
    fn autopilot_beats_the_boss_and_the_replay_verifies() {
        let mut battle = Battle::new(Mode::Boss, 1);
        battle.set_autopilot(true);
        while !battle.is_settled() && battle.world().tick() < 5 * 60 * TICK_HZ {
            battle.step(idle());
        }
        assert_eq!(battle.outcome(), Some(Outcome::Victory));
        let boss = battle.tally(battle.rival());
        assert!(matches!(
            boss.last_hit_by,
            Some(Cause::Action(id, _)) if id == battle.player()
        ));
        assert_eq!(
            boss.damage_taken,
            battle.tally(battle.player()).damage_dealt
        );
        let verified = battle.verify().unwrap();
        assert_eq!(verified.outcome, Some(Outcome::Victory));
        assert_eq!(verified.hash, battle.world().state_hash());
    }

    #[test]
    fn settled_battle_stops_and_keeps_a_finite_replay() {
        let mut battle = Battle::new(Mode::Boss, 1);
        while battle.outcome().is_none() {
            battle.step(idle());
        }
        assert_eq!(battle.outcome(), Some(Outcome::Defeat));
        let downed_at = battle.world().tick();
        for _ in 0..3 * SETTLE_TICKS {
            battle.step(idle());
        }
        assert!(battle.is_settled());
        assert_eq!(battle.world().tick(), downed_at + SETTLE_TICKS);
        assert_eq!(battle.replay().ticks, downed_at + SETTLE_TICKS);
        assert!(battle.verify().is_ok());
    }

    #[test]
    fn manual_seq_is_stamped_so_switching_pilots_keeps_inputs_valid() {
        let mut battle = Battle::new(Mode::Duel, 3);
        battle.toggle_sparring_bot();
        assert_eq!(battle.sparring_bot(), Some(false));
        let mut rejected = 0;
        let mut started = 0;
        for tick in 0..240 {
            // Client gửi seq luôn bằng 0; phiên vẫn chấp nhận vì tự đánh số.
            let frame = if tick % 40 == 0 {
                press(Buttons::LIGHT)
            } else {
                idle()
            };
            if tick == 100 {
                battle.set_autopilot(true);
            }
            if tick == 160 {
                battle.set_autopilot(false);
                battle.toggle_sparring_bot();
            }
            for event in battle.step(frame) {
                match event {
                    Event::InputRejected { .. } => rejected += 1,
                    Event::ActionStarted { fighter, .. } if fighter == battle.player() => {
                        started += 1;
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(rejected, 0);
        assert!(started >= 3, "đòn tay trước và sau khi bot lái vẫn ra");
        assert_eq!(battle.sparring_bot(), Some(true));
        assert!(battle.verify().is_ok());
    }

    #[test]
    fn rematch_advances_round_and_keeps_the_pilot() {
        let mut battle = Battle::new(Mode::Boss, 1);
        battle.set_autopilot(true);
        battle.step(idle());
        let next = battle.rematch(Mode::Duel);
        assert_eq!((next.mode(), next.round()), (Mode::Duel, 2));
        assert!(next.autopilot());
        assert_eq!(next.world().tick(), 0);
        assert_eq!(next.boss_phase(), None);
    }

    #[test]
    fn tampered_world_is_reported() {
        let mut battle = Battle::new(Mode::Boss, 1);
        for _ in 0..90 {
            battle.step(press(Buttons::LIGHT));
        }
        assert!(battle.verify().is_ok());
        // Mốc hash cuối lấy từ trạng thái đã bị sửa, nên chạy lại báo lệch ở tick cuối.
        battle.world.fighters_mut()[0].hp -= 1;
        assert!(matches!(
            battle.verify(),
            Err(VerifyError::Replay(ReplayError::Desync { tick: 90, .. }))
        ));
    }
}
