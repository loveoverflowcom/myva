//! App headless: không cửa sổ, không GPU, không audio; chạy đúng [`GameplayPlugin`] của client.
//!
//! `TimeUpdateStrategy::FixedTimesteps(1)` cho mỗi `App::update` đúng một tick cố định sau frame
//! đầu, nên runner/test không phụ thuộc đồng hồ thật. Test FPS dùng `ManualDuration` để mô phỏng
//! frame dài ngắn khác nhau.

use bevy_app::App;
use bevy_ecs::prelude::*;
use bevy_time::{TimePlugin, TimeUpdateStrategy};
use myva_sim::session::Session;

use crate::{Authority, GameplayPlugin, SimSession};

/// Lịch sử `(tick, hash)` sau mỗi tick; chỉ ghi khi resource này tồn tại.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct TickHistory(pub Vec<(u32, u64)>);

pub fn headless_app(session: Session, authority: Authority) -> App {
    let mut app = App::new();
    app.add_plugins((TimePlugin, GameplayPlugin::new(authority)))
        .insert_resource(SimSession::new(session))
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
        .init_resource::<TickHistory>();
    app.finish();
    app.cleanup();
    app
}

pub fn tick(app: &App) -> u32 {
    app.world().resource::<SimSession>().get().world().tick()
}

/// Chạy frame cho tới khi lõi đã chạy `ticks` tick; trả số frame đã dùng.
pub fn run_until(app: &mut App, ticks: u32) -> u32 {
    let mut frames = 0;
    while tick(app) < ticks {
        app.update();
        frames += 1;
        assert!(
            frames <= ticks.saturating_mul(16) + 16,
            "fixed loop không tiến: tick {} sau {frames} frame",
            tick(app)
        );
    }
    frames
}
