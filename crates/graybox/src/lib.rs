//! Client graybox Bevy của MyVa — Thần Mạch (work-plan 020, ADR 0003): một arena phẳng, Long Lưu
//! do người chơi điều khiển, đánh boss Kẻ Giữ Đập hoặc đấu tập với bot. Chỉ vẽ hình khối; mọi
//! luật nằm trong `myva-sim`, client gom input và thể hiện trạng thái.
//!
//! Ranh giới (D04, `docs/technical/gameplay-foundation.md`): `GameplayPlugin` của `myva-gameplay`
//! chạy `Session::step` trong `FixedUpdate` 60 Hz và mirror kết quả sang component; boss và bot
//! đấu tập là `Controller` của phiên. Client ghi ý định vào `LocalInput`, đọc component mirror
//! và `SimEvent` để vẽ. `FighterId` của lõi là ID miền, `Entity` chỉ sống trong process. Replay
//! ghi lệnh của mọi bên và được chạy lại để kiểm chứng khi trận lắng.
//!
//! Host (bin native hoặc `myva-web-game`) tạo `App` với `DefaultPlugins` và cửa sổ của mình, rồi
//! thêm [`GrayboxPlugin`]. Lệnh từ host đi qua message [`HostCommand`] và
//! [`touch::ShowTouchControls`]; host đọc trạng thái qua [`ArenaView`], [`Match`] hoặc
//! [`telemetry::json`].

mod controls;
mod fight;
mod hud;
pub mod scene;
pub mod telemetry;
pub mod text;
pub mod touch;

use bevy::prelude::*;
use myva_gameplay::{Authority, GameplayPlugin};
use myva_sim::battle::Bout;

pub use fight::{ArenaView, CombatLog, GraySet, HostCommand, Intent, Match};
pub use myva_sim::battle::Mode;

/// Thêm trận graybox vào một `App` đã có `DefaultPlugins`.
pub struct GrayboxPlugin {
    pub mode: Mode,
    /// Bot lái người chơi từ đầu (B1 khi đánh boss, B0 khi đấu tập).
    pub autopilot: bool,
}

impl Default for GrayboxPlugin {
    fn default() -> Self {
        Self {
            mode: Mode::Boss,
            autopilot: false,
        }
    }
}

impl Plugin for GrayboxPlugin {
    fn build(&self, app: &mut App) {
        let (mut bout, session) = Bout::start(self.mode, 1);
        bout.set_autopilot(self.autopilot);
        app.insert_resource(ClearColor(scene::BACKGROUND))
            // Client chơi offline/dự đoán: cùng luật với server nhưng không cấp phần thưởng.
            .add_plugins((GameplayPlugin::new(Authority::Client), MatchPlugin))
            .add_plugins((controls::plugin, touch::plugin, scene::plugin, hud::plugin));
        fight::insert(app, bout, session);
    }
}

/// Luật client của trận không cần cửa sổ, thiết bị hay renderer: lệnh host, bot lái hộ, trọng
/// tài và [`ArenaView`]. Dùng cùng `GameplayPlugin`; test headless thêm thẳng vào app headless.
pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        fight::plugin(app);
    }
}

/// Đưa trận `bout` cùng phiên của nó vào app (phiên, `LocalInput`, [`Match`]).
pub fn insert_match(app: &mut App, bout: Bout, session: myva_sim::Session) {
    fight::insert(app, bout, session);
}
