//! Lõi mô phỏng combat của MyVa — Thần Mạch.
//!
//! Đây là vai trò "mô phỏng Rust thuần" trong `docs/technical/architecture.md` §2: nhận trạng
//! thái, input và tick rồi trả trạng thái cùng sự kiện. Crate không có window, texture, audio
//! hay I/O, nên client graybox, bộ chạy headless và server sau này dùng chung được.
//!
//! Quy ước (combat.md §4, architecture.md §5):
//! - Thời gian luật tính bằng tick 60 Hz; timing thiết kế ghi bằng ms và được làm tròn lên.
//! - Vị trí, tài nguyên và damage là số nguyên để replay tái lập được giữa các target.
//! - Mọi thông số trong [`kit`] là giả thuyết (GT) cho graybox, chưa được cân bằng.
//!
//! Chưa làm: entity đạn, đại thuật, Thoát Mạch, DR khống chế, coyote time/jump buffer,
//! đòn không trung riêng và boss. Mỗi phần được thêm khi work-plan 020 cần kiểm chứng nó.

pub mod bot;
pub mod fighter;
pub mod input;
pub mod kit;
pub mod meter;
pub mod rng;
pub mod tick;
pub mod world;

pub use fighter::{Fighter, FighterId, State};
pub use input::{Buttons, InputFrame, Intent};
pub use kit::{ActionKind, Kit, LONG_LUU, PX, Phase};
pub use world::{Event, World};
