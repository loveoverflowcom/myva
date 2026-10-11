//! Trận graybox Bevy trong game document cùng origin do shell Leptos nhúng (#11, #2).
//!
//! Shell sở hữu DOM, navigation và chữ tiếng Việt; document này chỉ có canvas. Bevy (winit) đọc
//! bàn phím, chạm và gamepad trực tiếp trên canvas; bridge WASM nhỏ nhận lệnh vòng đời và trả
//! telemetry/replay. Luật nằm trong `myva-sim`, client trong `myva-graybox`. Gỡ iframe hủy cả
//! event loop Bevy và GPU context.

#[cfg(target_arch = "wasm32")]
mod browser;

#[cfg(target_arch = "wasm32")]
pub use browser::{
    rematch, request_replay, set_autopilot, set_paused, show_touch_controls, start_game,
    switch_mode, take_replay, telemetry, toggle_hitboxes, touch_layout,
};
