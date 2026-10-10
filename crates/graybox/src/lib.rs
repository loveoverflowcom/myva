//! Client graybox Bevy của MyVa — Thần Mạch (work-plan 020, ADR 0003): một arena phẳng, Long Lưu
//! do người chơi điều khiển, đánh boss Kẻ Giữ Đập hoặc đấu tập với bot. Chỉ vẽ hình khối; mọi
//! luật nằm trong `myva-sim`, client gom input và thể hiện trạng thái.
//!
//! Ranh giới: [`myva_sim::battle::Battle`] chạy trong `FixedUpdate` 60 Hz, không phụ thuộc FPS;
//! Bevy giữ input thiết bị, entity thể hiện, UI và renderer. `FighterId` của lõi là ID miền,
//! `Entity` chỉ dùng nội bộ để vẽ. Replay ghi input của mọi bên và được chạy lại để kiểm chứng
//! khi trận lắng.
//!
//! Host (bin native hoặc `myva-web-game`) tạo `App` với `DefaultPlugins` và cửa sổ của mình, rồi
//! thêm [`GrayboxPlugin`]. Lệnh từ host đi qua message [`HostCommand`] và
//! [`touch::ShowTouchControls`]; host đọc trạng thái qua [`Session`] hoặc [`telemetry::json`].

mod controls;
mod hud;
pub mod scene;
mod session;
pub mod telemetry;
pub mod text;
pub mod touch;

use bevy::prelude::*;
use myva_sim::tick::TICK_HZ;

pub use myva_sim::battle::Mode;
pub use session::{CombatLog, GraySet, HostCommand, Session};

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
        app.insert_resource(Time::<Fixed>::from_hz(f64::from(TICK_HZ)))
            .insert_resource(ClearColor(scene::BACKGROUND))
            .insert_resource(Session::new(self.mode, self.autopilot))
            .add_message::<HostCommand>()
            .add_plugins((
                session::plugin,
                controls::plugin,
                touch::plugin,
                scene::plugin,
                hud::plugin,
            ));
    }
}
