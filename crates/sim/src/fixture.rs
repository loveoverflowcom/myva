//! Kịch bản dùng chung cho test lõi, adapter ECS, bộ chạy headless và bản WASM, để cùng một
//! fixture chạy được ở mọi nơi rồi so hash từng tick.
//!
//! Phòng thử: một người chơi Long Lưu, hai quái bùn và NPC hướng dẫn. Người chơi do
//! [`Brawler`] điều khiển: nó gửi [`CommandEnvelope`] qua cổng nhận lệnh như client thật và chỉ
//! đọc trạng thái mà người chơi nhìn thấy.

use crate::fighter::{FighterId, State, Team};
use crate::kit::{LONG_LUU, PX};
use crate::monster::QUAI_BUN;
use crate::npc::{GUIDE, NpcId};
use crate::protocol::{Action, Attack, CommandEnvelope, CommandFrame, SessionEpoch, TickReport};
use crate::rng::Rng;
use crate::session::{Session, SessionConfig};
use crate::world::World;

pub const PLAYERS: Team = Team(1);
pub const HOSTILE: Team = Team(2);
/// Phiên của người chơi trong fixture.
pub const PLAYER_EPOCH: SessionEpoch = SessionEpoch(1);
/// Instance mặc định của fixture.
pub const INSTANCE: u64 = 0x4D59_5641;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrainingGround {
    pub player: FighterId,
    pub monsters: [FighterId; 2],
    pub guide: NpcId,
}

pub fn training_ground(seed: u64) -> (Session, TrainingGround) {
    let mut session = Session::new(SessionConfig::new(INSTANCE, seed));
    let player = session.spawn_player(&LONG_LUU, 500 * PX, 1, Some(PLAYERS), PLAYER_EPOCH);
    let monsters = [
        session.spawn_monster(&QUAI_BUN, 850 * PX, -1, Some(HOSTILE)),
        session.spawn_monster(&QUAI_BUN, 1_250 * PX, -1, Some(HOSTILE)),
    ];
    let guide = session.spawn_npc(&GUIDE, 200 * PX);
    (
        session,
        TrainingGround {
            player,
            monsters,
            guide,
        },
    )
}

/// Người chơi giả lập có seed: áp sát quái gần nhất, đánh nhẹ, thỉnh thoảng đỡ hoặc dùng Lưu
/// Tiễn; hết quái thì về gặp NPC hướng dẫn, tương tác một lần rồi đứng yên.
#[derive(Clone, Debug)]
pub struct Brawler {
    actor: FighterId,
    session: SessionEpoch,
    rng: Rng,
    seq: u32,
    /// Số tick còn giữ đỡ; đỡ cần giữ qua startup mới chặn được.
    guarding: u32,
    /// Đã nói chuyện với NPC; không bấm lặp mỗi tick.
    greeted: bool,
}

/// Khoảng cách tâm–tâm để bắt đầu đòn nhẹ.
const STRIKE_RANGE: i32 = 60 * PX;
const ARROW_RANGE: i32 = 400 * PX;

impl Brawler {
    pub fn new(seed: u64, actor: FighterId, session: SessionEpoch) -> Self {
        Self {
            actor,
            session,
            // Tách dòng số với bộ não quái dùng cùng seed phiên.
            rng: Rng::new(seed ^ 0xB4A3_17E5_0000_0001),
            seq: 0,
            guarding: 0,
            greeted: false,
        }
    }

    /// Lệnh cho tick hiện tại của `world`.
    pub fn command(&mut self, world: &World) -> CommandEnvelope {
        self.seq += 1;
        let frame = self.decide(world);
        CommandEnvelope::new(self.session, self.actor, self.seq, world.tick(), frame)
    }

    fn decide(&mut self, world: &World) -> CommandFrame {
        let me = world.fighter(self.actor);
        let mut frame = CommandFrame::IDLE;
        if me.state == State::Downed {
            return frame;
        }
        let target = world
            .fighters()
            .iter()
            .filter(|f| f.id != me.id && !me.is_ally(f) && f.state != State::Downed)
            .min_by_key(|f| ((f.x - me.x).abs(), f.id));
        let Some(target) = target else {
            let Some(npc) = world.npcs().first() else {
                return frame;
            };
            let dx = npc.x - me.x;
            if dx.abs() > npc.spec.reach / 2 {
                frame.move_x = dx.signum() as i8;
            } else if !self.greeted && me.can_interact() {
                self.greeted = true;
                frame.action = Some(Action::Interact);
            }
            return frame;
        };

        let dx = target.x - me.x;
        let toward = dx.signum() as i8;
        let threatened =
            matches!(target.state, State::Attack { .. }) && dx.abs() <= STRIKE_RANGE * 2;
        if threatened && self.guarding == 0 && self.rng.chance(1, 12) {
            self.guarding = 40;
        }
        if self.guarding > 0 {
            self.guarding -= 1;
            frame.guard = true;
            return frame;
        }
        frame.move_x = toward;
        if dx.abs() <= STRIKE_RANGE {
            if self.rng.chance(1, 2) {
                frame.action = Some(Action::Attack(Attack::Light));
            }
        } else if dx.abs() <= ARROW_RANGE && self.rng.chance(1, 45) {
            frame.action = Some(Action::Skill(0));
        } else if self.rng.chance(1, 90) {
            frame.action = Some(Action::Dash);
        }
        frame
    }
}

/// Chạy fixture `ticks` tick trên lõi thuần (đường server-only), trả báo cáo từng tick.
pub fn run_core(seed: u64, ticks: u32) -> Vec<TickReport> {
    let (mut session, ground) = training_ground(seed);
    let mut brawler = Brawler::new(seed, ground.player, PLAYER_EPOCH);
    (0..ticks)
        .map(|_| {
            let command = brawler.command(session.world());
            session.submit(command).expect("lệnh fixture phải hợp lệ");
            session.step()
        })
        .collect()
}

/// Hash sau mỗi tick của [`run_core`]; dùng để so native/WASM.
pub fn fingerprint(seed: u64, ticks: u32) -> Vec<u64> {
    run_core(seed, ticks).iter().map(|r| r.hash).collect()
}
