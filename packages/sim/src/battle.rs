//! Trận graybox (work-plan 020) trên đường tick duy nhất của D04: [`Session`] giữ `World`, cổng
//! nhận lệnh, bộ não AI và replay; module này thêm đội hình, bot lái hộ người chơi, trọng tài
//! (kết quả, số liệu playtest, nhịp lắng) và tự kiểm chứng replay.
//!
//! [`Bout`] không giữ phiên, để adapter ECS (`myva-gameplay`) giữ `Session` trong resource còn
//! client giữ `Bout`. [`Battle`] ghép hai thứ cho test, công cụ và runner headless. Cả hai gửi
//! lệnh người chơi qua cùng cổng với cùng `seq`, nên cùng input cho cùng hash từng tick.
//!
//! Boss Kẻ Giữ Đập và bot đấu tập B0 là [`Controller`] của phiên như quái thường. Bot lái hộ
//! người chơi ([`Pilot`]) chạy phía client và gửi lệnh như một người chơi; kết quả được kiểm
//! chứng bằng cách chạy lại replay của chính trận đó ([`Bout::verify`]).

use crate::boss::{BossBrain, BossPhase, KE_GIU_DAP};
use crate::bot::{PatternReader, RandomBot};
use crate::fighter::{FighterId, State};
use crate::input::InputFrame;
use crate::kit::{ActionKind, LONG_LUU, PX};
use crate::protocol::{CommandEnvelope, CommandFrame, SessionEpoch, TickReport};
use crate::replay::{Replay, ReplayError};
use crate::session::{Controller, Role, Session, SessionConfig};
use crate::tick::{TICK_HZ, Tick};
use crate::world::{Event, World};

/// Mỗi giây ghi một mốc hash.
pub const CHECKPOINT_EVERY: u32 = TICK_HZ;
/// Sau khi có người bị hạ, thế giới chạy thêm chừng này tick cho trạng thái lắng rồi dừng, nên
/// replay của một trận luôn có điểm kết thúc.
pub const SETTLE_TICKS: u32 = TICK_HZ;
/// Bot B1 lái hộ người chơi phản ứng theo ngân sách combat.md §10.
pub const AUTOPILOT_REACTION_MS: u32 = 250;
/// Phiên điều khiển người chơi trong trận cục bộ.
pub const PLAYER_EPOCH: SessionEpoch = SessionEpoch(1);

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

/// Boss Kẻ Giữ Đập làm bộ não AI của phiên, nhắm vào người chơi.
#[derive(Clone, Debug)]
pub struct BossController {
    brain: BossBrain,
    target: FighterId,
}

impl BossController {
    pub fn new(target: FighterId) -> Self {
        Self {
            brain: BossBrain::new(),
            target,
        }
    }

    pub fn phase(&self) -> BossPhase {
        self.brain.phase()
    }
}

impl Controller for BossController {
    fn next_frame(&mut self, world: &World, me: FighterId) -> InputFrame {
        self.brain.next_frame(world, me, self.target)
    }
}

/// Bot B0 của chế độ đấu tập; tắt thì đứng yên nhưng vẫn gửi khung rỗng qua cổng.
#[derive(Clone, Debug)]
pub struct SparringBot {
    bot: RandomBot,
    enabled: bool,
}

impl SparringBot {
    pub fn new(seed: u64) -> Self {
        Self {
            bot: RandomBot::new(seed),
            enabled: true,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Controller for SparringBot {
    fn next_frame(&mut self, _: &World, _: FighterId) -> InputFrame {
        if self.enabled {
            self.bot.next_frame()
        } else {
            InputFrame::default()
        }
    }
}

/// Bot lái hộ người chơi phía client: B1 đọc tín hiệu khi đánh boss, B0 ngẫu nhiên khi đấu tập.
/// Nó chỉ sinh lệnh; lệnh vẫn đi qua cổng như bàn phím.
#[derive(Clone, Debug)]
pub enum Pilot {
    Reader(PatternReader),
    Random(RandomBot),
}

impl Pilot {
    pub fn for_mode(mode: Mode, round: u64) -> Self {
        match mode {
            Mode::Boss => Pilot::Reader(PatternReader::with_reaction_ms(AUTOPILOT_REACTION_MS)),
            Mode::Duel => Pilot::Random(RandomBot::new(round ^ 0x5EED)),
        }
    }

    pub fn command(&mut self, world: &World, me: FighterId, rival: FighterId) -> CommandFrame {
        let frame = match self {
            Pilot::Reader(reader) => reader.next_frame(world, me, rival),
            Pilot::Random(bot) => bot.next_frame(),
        };
        CommandFrame::from_input(&frame)
    }
}

/// Đội hình của một trận; `FighterId` là ID miền, không phải `Entity`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Roster {
    pub mode: Mode,
    pub round: u64,
    pub player: FighterId,
    pub rival: FighterId,
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

/// Trạng thái trận ngoài `Session`: đội hình, bot lái hộ, kết quả và số liệu từng bên.
#[derive(Clone, Debug)]
pub struct Bout {
    roster: Roster,
    pilot: Option<Pilot>,
    outcome: Option<(Outcome, Tick)>,
    tallies: [Tally; 2],
}

impl Bout {
    /// Dựng phiên cho trận `mode` ở vòng `round`: người chơi do [`PLAYER_EPOCH`] điều khiển, đối
    /// thủ là bộ não AI của phiên.
    pub fn start(mode: Mode, round: u64) -> (Self, Session) {
        let mut session = Session::new(SessionConfig {
            checkpoint_every: CHECKPOINT_EVERY,
            ..SessionConfig::new(round, round)
        });
        let (player_x, rival_x) = match mode {
            Mode::Boss => (400 * PX, 1_100 * PX),
            Mode::Duel => (640 * PX, 960 * PX),
        };
        let player = session.spawn_player(&LONG_LUU, player_x, 1, None, PLAYER_EPOCH);
        let rival = match mode {
            Mode::Boss => session.spawn_controlled(
                &KE_GIU_DAP,
                rival_x,
                -1,
                None,
                Role::Monster,
                Box::new(BossController::new(player)),
            ),
            Mode::Duel => {
                let seed = session.derive_seed(FighterId(player.0 + 1));
                session.spawn_controlled(
                    &LONG_LUU,
                    rival_x,
                    -1,
                    None,
                    Role::Monster,
                    Box::new(SparringBot::new(seed)),
                )
            }
        };
        let bout = Self {
            roster: Roster {
                mode,
                round,
                player,
                rival,
            },
            pilot: None,
            outcome: None,
            tallies: [Tally::default(); 2],
        };
        (bout, session)
    }

    /// Trận mới ở vòng kế tiếp, giữ chế độ lái của người chơi.
    pub fn rematch(&self, mode: Mode) -> (Self, Session) {
        let (mut next, session) = Self::start(mode, self.roster.round + 1);
        next.set_autopilot(self.autopilot());
        (next, session)
    }

    pub fn roster(&self) -> Roster {
        self.roster
    }

    pub fn mode(&self) -> Mode {
        self.roster.mode
    }

    pub fn round(&self) -> u64 {
        self.roster.round
    }

    pub fn player(&self) -> FighterId {
        self.roster.player
    }

    pub fn rival(&self) -> FighterId {
        self.roster.rival
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome.map(|(outcome, _)| outcome)
    }

    pub fn tally(&self, id: FighterId) -> &Tally {
        &self.tallies[usize::from(id.0)]
    }

    pub fn boss_phase(&self, session: &Session) -> Option<BossPhase> {
        session
            .controller::<BossController>(self.rival())
            .map(BossController::phase)
    }

    /// `Some(bật)` ở chế độ đấu tập, `None` khi đánh boss.
    pub fn sparring_bot(&self, session: &Session) -> Option<bool> {
        session
            .controller::<SparringBot>(self.rival())
            .map(SparringBot::enabled)
    }

    pub fn autopilot(&self) -> bool {
        self.pilot.is_some()
    }

    pub fn set_autopilot(&mut self, on: bool) {
        if on != self.autopilot() {
            self.pilot = on.then(|| Pilot::for_mode(self.roster.mode, self.roster.round));
        }
    }

    /// Lệnh bot lái hộ cho tick hiện tại của `world`; `None` khi người chơi tự lái.
    pub fn autopilot_command(&mut self, world: &World) -> Option<CommandFrame> {
        let Roster { player, rival, .. } = self.roster;
        self.pilot
            .as_mut()
            .map(|pilot| pilot.command(world, player, rival))
    }

    /// Ghi nhận một tick vừa chạy: số liệu từ sự kiện và kết quả từ trạng thái sau tick.
    pub fn observe(&mut self, world: &World, report: &TickReport) {
        for record in &report.events {
            self.count(&record.event);
        }
        if self.outcome.is_none()
            && let Some(outcome) = Outcome::of(world, self.player(), self.rival())
        {
            self.outcome = Some((outcome, world.tick()));
        }
    }

    /// Trận đã có kết quả và đã chạy hết nhịp lắng ở tick `now`; không chạy thêm tick nữa.
    pub fn is_settled(&self, now: Tick) -> bool {
        self.outcome.is_some_and(|(_, at)| now >= at + SETTLE_TICKS)
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

    /// Ghi replay của `session` ra văn bản, đọc lại và chạy lại không cần bộ não boss hay bot, rồi
    /// so trạng thái cuối và kết quả với trận đang chơi. Đây là kiểm chứng cục bộ trên cùng build;
    /// kết quả online vẫn chỉ do server quyết định.
    pub fn verify(&self, session: &Session) -> Result<Verification, VerifyError> {
        let replay = Replay::parse(&session.replay().to_text()).map_err(VerifyError::Replay)?;
        self.check(&replay, session.world().state_hash())
    }

    fn check(&self, replay: &Replay, live: u64) -> Result<Verification, VerifyError> {
        let playback = replay.play().map_err(VerifyError::Replay)?;
        let replayed = playback.world.state_hash();
        let outcome = Outcome::of(&playback.world, self.player(), self.rival());
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

/// Trận chạy trực tiếp trên lõi, không ECS: test, công cụ `myva-replay` và đối chứng với client.
pub struct Battle {
    session: Session,
    bout: Bout,
    seq: u32,
}

impl Battle {
    pub fn new(mode: Mode, round: u64) -> Self {
        let (bout, session) = Bout::start(mode, round);
        Self {
            session,
            bout,
            seq: 0,
        }
    }

    /// Trận mới ở vòng kế tiếp, giữ chế độ lái của người chơi.
    pub fn rematch(&self, mode: Mode) -> Self {
        let (bout, session) = self.bout.rematch(mode);
        Self {
            session,
            bout,
            seq: 0,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn world(&self) -> &World {
        self.session.world()
    }

    pub fn bout(&self) -> &Bout {
        &self.bout
    }

    pub fn bout_mut(&mut self) -> &mut Bout {
        &mut self.bout
    }

    pub fn toggle_sparring_bot(&mut self) {
        if let Some(bot) = self
            .session
            .controller_mut::<SparringBot>(self.bout.rival())
        {
            bot.toggle();
        }
    }

    pub fn is_settled(&self) -> bool {
        self.bout.is_settled(self.world().tick())
    }

    /// Chạy một tick với ý định `manual` của người chơi, bị bỏ qua khi bot đang lái. `None` khi
    /// trận đã lắng.
    pub fn step(&mut self, manual: CommandFrame) -> Option<TickReport> {
        if self.is_settled() {
            return None;
        }
        let world = self.session.world();
        let frame = self.bout.autopilot_command(world).unwrap_or(manual);
        self.seq += 1;
        let command = CommandEnvelope::new(
            PLAYER_EPOCH,
            self.bout.player(),
            self.seq,
            world.tick(),
            frame,
        );
        if let Err(rejection) = self.session.submit(command) {
            panic!("lệnh người chơi bị từ chối: {rejection:?}");
        }
        let report = self.session.step();
        self.bout.observe(self.session.world(), &report);
        Some(report)
    }

    /// Replay của trận tới tick hiện tại, có mốc hash cuối.
    pub fn replay(&self) -> Replay {
        self.session.replay()
    }

    pub fn verify(&self) -> Result<Verification, VerifyError> {
        self.bout.verify(&self.session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Buttons;
    use crate::protocol::{Action, Attack};

    const LIGHT: Action = Action::Attack(Attack::Light);

    fn press(action: Action) -> CommandFrame {
        CommandFrame {
            action: Some(action),
            ..CommandFrame::IDLE
        }
    }

    #[test]
    fn autopilot_beats_the_boss_and_the_replay_verifies() {
        let mut battle = Battle::new(Mode::Boss, 1);
        battle.bout_mut().set_autopilot(true);
        while !battle.is_settled() && battle.world().tick() < 5 * 60 * TICK_HZ {
            battle.step(CommandFrame::IDLE);
        }
        let bout = battle.bout();
        assert_eq!(bout.outcome(), Some(Outcome::Victory));
        assert!(matches!(
            bout.tally(bout.rival()).last_hit_by,
            Some(Cause::Action(id, _)) if id == bout.player()
        ));
        assert_eq!(
            bout.tally(bout.rival()).damage_taken,
            bout.tally(bout.player()).damage_dealt
        );
        let verified = battle.verify().unwrap();
        assert_eq!(verified.outcome, Some(Outcome::Victory));
        assert_eq!(verified.hash, battle.world().state_hash());
    }

    #[test]
    fn boss_phases_advance_as_it_loses_hp() {
        let mut battle = Battle::new(Mode::Boss, 1);
        battle.bout_mut().set_autopilot(true);
        let mut seen = vec![battle.bout().boss_phase(battle.session()).unwrap()];
        while !battle.is_settled() {
            battle.step(CommandFrame::IDLE);
            let phase = battle.bout().boss_phase(battle.session()).unwrap();
            if seen.last() != Some(&phase) {
                seen.push(phase);
            }
        }
        assert_eq!(
            seen,
            [BossPhase::Recognize, BossPhase::Terrain, BossPhase::Combine]
        );
    }

    #[test]
    fn settled_battle_stops_and_keeps_a_finite_replay() {
        let mut battle = Battle::new(Mode::Boss, 1);
        while battle.bout().outcome().is_none() {
            battle.step(CommandFrame::IDLE);
        }
        assert_eq!(battle.bout().outcome(), Some(Outcome::Defeat));
        let downed_at = battle.world().tick();
        for _ in 0..3 * SETTLE_TICKS {
            battle.step(CommandFrame::IDLE);
        }
        assert!(battle.is_settled());
        assert!(battle.step(CommandFrame::IDLE).is_none());
        assert_eq!(battle.world().tick(), downed_at + SETTLE_TICKS);
        assert_eq!(battle.replay().ticks, downed_at + SETTLE_TICKS);
        assert!(battle.verify().is_ok());
    }

    #[test]
    fn switching_pilots_and_sparring_bot_keeps_every_command_valid() {
        let mut battle = Battle::new(Mode::Duel, 3);
        battle.toggle_sparring_bot();
        assert_eq!(battle.bout().sparring_bot(battle.session()), Some(false));
        let mut rejected = 0;
        let mut started = 0;
        for tick in 0..240 {
            let frame = if tick % 40 == 0 {
                press(LIGHT)
            } else {
                CommandFrame::IDLE
            };
            if tick == 100 {
                battle.bout_mut().set_autopilot(true);
            }
            if tick == 160 {
                battle.bout_mut().set_autopilot(false);
                battle.toggle_sparring_bot();
            }
            let player = battle.bout().player();
            for record in battle.step(frame).unwrap().events {
                match record.event {
                    Event::InputRejected { .. } => rejected += 1,
                    Event::ActionStarted { fighter, .. } if fighter == player => started += 1,
                    _ => {}
                }
            }
        }
        assert_eq!(rejected, 0);
        assert!(started >= 3, "đòn tay trước và sau khi bot lái vẫn ra");
        assert_eq!(battle.bout().sparring_bot(battle.session()), Some(true));
        assert!(battle.verify().is_ok());
    }

    #[test]
    fn rematch_advances_round_and_keeps_the_pilot() {
        let mut battle = Battle::new(Mode::Boss, 1);
        battle.bout_mut().set_autopilot(true);
        battle.step(CommandFrame::IDLE);
        let next = battle.rematch(Mode::Duel);
        assert_eq!((next.bout().mode(), next.bout().round()), (Mode::Duel, 2));
        assert!(next.bout().autopilot());
        assert_eq!(next.world().tick(), 0);
        assert_eq!(next.bout().boss_phase(next.session()), None);
        assert_eq!(next.bout().sparring_bot(next.session()), Some(true));
    }

    #[test]
    fn tampered_replay_is_reported() {
        let mut battle = Battle::new(Mode::Boss, 1);
        for _ in 0..90 {
            battle.step(press(LIGHT));
        }
        assert!(battle.verify().is_ok());
        // Bỏ cú bấm đầu tiên của người chơi: chạy lại lệch ở mốc hash đầu tiên.
        let mut replay = battle.replay();
        let player = battle.bout().player();
        let first = replay
            .inputs
            .iter()
            .position(|(_, id, frame)| *id == player && !frame.pressed.is_empty())
            .unwrap();
        replay.inputs[first].2.pressed = Buttons::NONE;
        assert!(matches!(
            battle.bout().check(&replay, battle.world().state_hash()),
            Err(VerifyError::Replay(ReplayError::Desync { tick: 60, .. }))
        ));
    }
}
