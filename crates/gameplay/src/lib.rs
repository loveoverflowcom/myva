//! Adapter Bevy ECS cho lõi mô phỏng MyVa — Thần Mạch (D04 / #12).
//!
//! Crate này **không có luật chiến đấu**. Luật nằm trong `myva-sim`; mỗi tick cố định 60 Hz,
//! [`GameplayPlugin`] gửi lệnh vào [`myva_sim::Session`], gọi `Session::step` đúng một lần, rồi
//! mirror snapshot sang entity/component và phát sự kiện. Thứ tự trong `FixedUpdate`:
//!
//! | Set | Việc | Luật lõi tương ứng |
//! | --- | --- | --- |
//! | [`GameplaySet::Input`] | Lệnh client/người chơi giả lập vào cổng nhận lệnh | — |
//! | [`GameplaySet::Simulate`] | `Session::step`: AI → `input → movement → collision → combat → status` | Toàn bộ |
//! | [`GameplaySet::Mirror`] | Snapshot → entity/component bất biến | — |
//! | [`GameplaySet::Events`] | `SimEvent`, `TickCompleted`, phần thưởng (chỉ server) | `events` |
//!
//! Tốc độ luật không phụ thuộc FPS: `Time<Fixed>` chạy `FixedUpdate` 0..n lần mỗi frame theo thời
//! gian đã trôi, input gom theo frame được chốt vào tick kế tiếp. Không dùng renderer, window,
//! asset hay audio của Bevy; host tự thêm `TimePlugin` (client qua `DefaultPlugins`, headless qua
//! [`headless::headless_app`]).

pub mod authority;
pub mod components;
pub mod headless;
pub mod input;
pub mod mirror;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_time::{Fixed, Time, TimePlugin};
use myva_sim::protocol::{EventRecord, TickReport};
use myva_sim::session::Session;
use myva_sim::tick::{TICK_HZ, Tick};

pub use authority::{Authority, RewardClaim, RewardIssued, RewardKey, RewardOutbox};
pub use input::{CommandRejected, LocalInput, ScriptedPlayer};
pub use mirror::EntityIndex;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Input,
    Simulate,
    Mirror,
    Events,
}

/// Phiên mô phỏng do adapter giữ. Không expose `&mut World`: mọi thay đổi trạng thái đi qua
/// lệnh và `Session::step`. Một app giữ một phiên suốt đời; vào instance khác thì dựng app mới,
/// như web shell tháo cả runtime khi rời game (ADR 0003).
#[derive(Resource)]
pub struct SimSession(Session);

impl SimSession {
    pub fn new(session: Session) -> Self {
        Self(session)
    }

    pub fn get(&self) -> &Session {
        &self.0
    }

    pub(crate) fn session_mut(&mut self) -> &mut Session {
        &mut self.0
    }

    /// Đổi phiên điều khiển người chơi khi reconnect.
    pub fn rebind(&mut self, actor: myva_sim::FighterId, session: myva_sim::SessionEpoch) {
        self.0.rebind(actor, session);
    }
}

/// Báo cáo của tick vừa chạy trong lần `FixedUpdate` hiện tại.
#[derive(Resource, Debug, Default)]
pub struct LastTick(Option<TickReport>);

impl LastTick {
    pub fn report(&self) -> Option<&TickReport> {
        self.0.as_ref()
    }
}

/// Một sự kiện lõi, kèm ID duy nhất `(tick, index)` để presentation bỏ bản lặp.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimEvent(pub EventRecord);

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickCompleted {
    pub tick: Tick,
    pub hash: u64,
}

pub struct GameplayPlugin {
    authority: Authority,
}

impl GameplayPlugin {
    pub fn new(authority: Authority) -> Self {
        Self { authority }
    }
}

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(f64::from(TICK_HZ)))
            .insert_resource(self.authority)
            .init_resource::<EntityIndex>()
            .init_resource::<mirror::MirrorCache>()
            .init_resource::<LastTick>()
            .add_message::<SimEvent>()
            .add_message::<TickCompleted>()
            .add_message::<CommandRejected>()
            .configure_sets(
                FixedUpdate,
                (
                    GameplaySet::Input,
                    GameplaySet::Simulate,
                    GameplaySet::Mirror,
                    GameplaySet::Events,
                )
                    .chain()
                    // Client có thể chỉ tạo phiên khi vào arena.
                    .run_if(resource_exists::<SimSession>),
            )
            // Phiên có sẵn khi khởi động thì entity có ngay, trước tick đầu tiên.
            .add_systems(
                Startup,
                mirror::mirror_world.run_if(resource_exists::<SimSession>),
            )
            .add_systems(
                FixedUpdate,
                (
                    (input::submit_local_input, input::submit_scripted_player)
                        .chain()
                        .in_set(GameplaySet::Input),
                    step_simulation.in_set(GameplaySet::Simulate),
                    mirror::mirror_world.in_set(GameplaySet::Mirror),
                    publish_events.in_set(GameplaySet::Events),
                ),
            );
        if self.authority == Authority::Server {
            app.insert_resource(RewardOutbox::new())
                .add_message::<RewardIssued>()
                .add_systems(
                    FixedUpdate,
                    authority::issue_rewards
                        .in_set(GameplaySet::Events)
                        .after(publish_events),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        assert!(
            app.is_plugin_added::<TimePlugin>(),
            "GameplayPlugin cần TimePlugin để chạy FixedUpdate"
        );
    }
}

fn step_simulation(mut session: ResMut<SimSession>, mut last: ResMut<LastTick>) {
    last.0 = Some(session.session_mut().step());
}

fn publish_events(
    last: Res<LastTick>,
    history: Option<ResMut<headless::TickHistory>>,
    mut events: MessageWriter<SimEvent>,
    mut completed: MessageWriter<TickCompleted>,
) {
    let Some(report) = last.report() else {
        return;
    };
    events.write_batch(report.events.iter().copied().map(SimEvent));
    completed.write(TickCompleted {
        tick: report.tick,
        hash: report.hash,
    });
    if let Some(mut history) = history {
        history.0.push((report.tick, report.hash));
    }
}
