//! Phiên mô phỏng authoritative: `World` + cổng nhận lệnh + bộ não AI + replay.
//!
//! Đây là đường chạy duy nhất mỗi tick, dùng chung cho server-only, adapter ECS và bộ chạy
//! headless:
//!
//! 1. Bộ não AI đọc trạng thái đầu tick theo thứ tự `FighterId`, gửi lệnh qua cùng
//!    [`CommandQueue`] như người chơi (epoch [`SessionEpoch::AI`]).
//! 2. Lấy lệnh của tick theo thứ tự `FighterId`.
//! 3. [`World::step`] chạy `input → movement → collision → combat → status → events`.
//! 4. Ghi replay và trả [`TickReport`].
//!
//! Phiên không cấp phần thưởng: sự kiện `Downed` là dữ kiện để server nghiệp vụ quyết định, có
//! khóa chống trùng riêng.

use std::any::Any;

use crate::fighter::{FighterId, Team};
use crate::input::InputFrame;
use crate::kit::Kit;
use crate::monster::{AiState, MonsterBrain};
use crate::npc::{NpcId, NpcSpec};
use crate::protocol::{
    CommandAck, CommandEnvelope, CommandFrame, CommandQueue, Rejection, SessionEpoch, TickReport,
};
use crate::replay::{Recorder, Replay};
use crate::rng::Rng;
use crate::snapshot::Snapshot;
use crate::world::World;

/// Vai trò gameplay của nhân vật; `World` không cần biết, presentation và server thì cần.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Player,
    Monster,
}

/// Bộ não sinh input cho một nhân vật. Thêm loại quái là thêm một `Controller`, không sửa
/// vòng lặp tick. `seq` của khung trả về bị bỏ qua: phiên tự đánh số lệnh AI.
pub trait Controller: Any + Send + Sync {
    fn next_frame(&mut self, world: &World, me: FighterId) -> InputFrame;

    fn ai_state(&self) -> Option<AiState> {
        None
    }
}

impl Controller for MonsterBrain {
    fn next_frame(&mut self, world: &World, me: FighterId) -> InputFrame {
        MonsterBrain::next_frame(self, world, me)
    }

    fn ai_state(&self) -> Option<AiState> {
        Some(self.state())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SessionConfig {
    /// ID instance/phòng do server cấp; thuộc khóa chống trùng phần thưởng.
    pub instance: u64,
    /// Seed gốc; mỗi bộ não nhận seed dẫn xuất theo `FighterId`.
    pub seed: u64,
    /// Số tick giữa hai mốc hash trong replay.
    pub checkpoint_every: u32,
}

impl SessionConfig {
    pub const fn new(instance: u64, seed: u64) -> Self {
        Self {
            instance,
            seed,
            checkpoint_every: 60,
        }
    }
}

struct Member {
    role: Role,
    controller: Option<Box<dyn Controller>>,
    /// `seq` lệnh AI cuối cùng phiên đã gửi cho nhân vật này.
    seq: u32,
}

pub struct Session {
    config: SessionConfig,
    world: World,
    queue: CommandQueue,
    members: Vec<Member>,
    recorder: Option<Recorder>,
}

impl Session {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            world: World::new(),
            queue: CommandQueue::new(),
            members: Vec::new(),
            recorder: None,
        }
    }

    pub fn config(&self) -> SessionConfig {
        self.config
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// Người chơi do phiên `session` điều khiển qua [`Session::submit`].
    pub fn spawn_player(
        &mut self,
        kit: &'static Kit,
        x: i32,
        facing: i8,
        team: Option<Team>,
        session: SessionEpoch,
    ) -> FighterId {
        assert_ne!(session, SessionEpoch::AI, "epoch 0 dành cho AI server");
        let id = self.spawn(kit, x, facing, team, Role::Player, None);
        self.queue.bind(id, session);
        id
    }

    /// Quái thường với [`MonsterBrain`] nhận seed dẫn xuất từ seed phiên.
    pub fn spawn_monster(
        &mut self,
        kit: &'static Kit,
        x: i32,
        facing: i8,
        team: Option<Team>,
    ) -> FighterId {
        let id = FighterId(u16::try_from(self.members.len()).expect("quá nhiều nhân vật"));
        let brain = MonsterBrain::new(self.derive_seed(id));
        self.spawn_controlled(kit, x, facing, team, Role::Monster, Box::new(brain))
    }

    pub fn spawn_controlled(
        &mut self,
        kit: &'static Kit,
        x: i32,
        facing: i8,
        team: Option<Team>,
        role: Role,
        controller: Box<dyn Controller>,
    ) -> FighterId {
        let id = self.spawn(kit, x, facing, team, role, Some(controller));
        self.queue.bind(id, SessionEpoch::AI);
        id
    }

    pub fn spawn_npc(&mut self, spec: &'static NpcSpec, x: i32) -> NpcId {
        self.assert_not_started();
        self.world.spawn_npc(spec, x)
    }

    /// Reconnect: phiên mới thay phiên cũ; lệnh chưa chạy của phiên cũ bị hủy.
    pub fn rebind(&mut self, actor: FighterId, session: SessionEpoch) {
        assert_eq!(self.role(actor), Role::Player, "chỉ người chơi đổi phiên");
        assert_ne!(session, SessionEpoch::AI, "epoch 0 dành cho AI server");
        self.queue.bind(actor, session);
    }

    /// Nhận lệnh của người chơi; tick áp dụng do phiên gán.
    pub fn submit(&mut self, envelope: CommandEnvelope) -> Result<CommandAck, Rejection> {
        if envelope.session == SessionEpoch::AI {
            return Err(Rejection::NotController {
                actor: envelope.actor,
                session: envelope.session,
            });
        }
        self.queue.submit(envelope, self.world.tick())
    }

    /// Chạy một tick authoritative.
    pub fn step(&mut self) -> TickReport {
        let now = self.world.tick();
        let every = self.config.checkpoint_every;
        let recorder = self
            .recorder
            .get_or_insert_with(|| Recorder::new(&self.world, every));
        for (index, member) in self.members.iter_mut().enumerate() {
            let Some(controller) = member.controller.as_mut() else {
                continue;
            };
            let id = FighterId(index as u16);
            let input = controller.next_frame(&self.world, id);
            member.seq += 1;
            let envelope = CommandEnvelope::new(
                SessionEpoch::AI,
                id,
                member.seq,
                now,
                CommandFrame::from_input(&input),
            );
            let accepted = self.queue.submit(envelope, now);
            debug_assert!(accepted.is_ok(), "lệnh AI bị từ chối: {accepted:?}");
        }
        let inputs = self.queue.drain(now);
        let events = recorder.step(&mut self.world, &inputs);
        TickReport::new(now, events, self.world.state_hash())
    }

    pub fn role(&self, id: FighterId) -> Role {
        self.members[usize::from(id.0)].role
    }

    /// Bộ não của nhân vật `id` nếu đúng kiểu `T`, để host đọc trạng thái riêng (ví dụ pha boss).
    pub fn controller<T: Controller>(&self, id: FighterId) -> Option<&T> {
        let controller: &dyn Controller =
            self.members.get(usize::from(id.0))?.controller.as_deref()?;
        (controller as &dyn Any).downcast_ref()
    }

    /// Chỉnh bộ não của nhân vật `id` (ví dụ tắt bot đấu tập). Không chạm `World`: lệnh AI vẫn
    /// đi qua cổng và được ghi vào replay, nên đổi giữa trận không làm replay lệch.
    pub fn controller_mut<T: Controller>(&mut self, id: FighterId) -> Option<&mut T> {
        let controller: &mut dyn Controller = self
            .members
            .get_mut(usize::from(id.0))?
            .controller
            .as_deref_mut()?;
        (controller as &mut dyn Any).downcast_mut()
    }

    pub fn ai_state(&self, id: FighterId) -> Option<AiState> {
        self.members[usize::from(id.0)]
            .controller
            .as_ref()
            .and_then(|c| c.ai_state())
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot::capture(&self.world, |id| (self.role(id), self.ai_state(id)))
    }

    /// Replay từ tick 0 tới hiện tại, gồm cả lệnh AI đã áp dụng nên chạy lại không cần bộ não.
    pub fn replay(&self) -> Replay {
        let mut replay = match &self.recorder {
            Some(recorder) => recorder.snapshot(&self.world),
            None => Recorder::new(&self.world, self.config.checkpoint_every).snapshot(&self.world),
        };
        replay.seed = Some(self.config.seed);
        replay
    }

    /// Seed cho bộ não của nhân vật `id`: cùng seed phiên và cùng thứ tự spawn cho cùng seed.
    pub fn derive_seed(&self, id: FighterId) -> u64 {
        Rng::new(self.config.seed ^ (u64::from(id.0) << 48)).next_u64()
    }

    fn spawn(
        &mut self,
        kit: &'static Kit,
        x: i32,
        facing: i8,
        team: Option<Team>,
        role: Role,
        controller: Option<Box<dyn Controller>>,
    ) -> FighterId {
        self.assert_not_started();
        let id = self.world.spawn_in_team(kit, x, facing, team);
        self.members.push(Member {
            role,
            controller,
            seq: 0,
        });
        id
    }

    /// Replay hiện chỉ ghi spawn ở tick 0; spawn giữa trận cần dòng replay riêng.
    fn assert_not_started(&self) {
        assert_eq!(self.world.tick(), 0, "chỉ spawn trước tick đầu tiên");
    }
}
