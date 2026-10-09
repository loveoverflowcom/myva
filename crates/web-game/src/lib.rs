//! A rendering/lifecycle spike, deliberately independent of production combat rules.
//!
//! The shell owns DOM events and sends a direction through the small WASM bridge.
//! Unmounting its iframe destroys the whole Bevy event loop and GPU context.

#[cfg(any(target_arch = "wasm32", test))]
mod state;

#[cfg(target_arch = "wasm32")]
mod browser;

#[cfg(target_arch = "wasm32")]
pub use browser::{reset_game, set_input, set_paused, start_game, telemetry};
