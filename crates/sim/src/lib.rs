//! Lõi mô phỏng combat của MyVa — Thần Mạch.
//!
//! Đây là vai trò "mô phỏng Rust thuần" trong `docs/technical/architecture.md` §2: nhận trạng
//! thái, lệnh và tick rồi trả trạng thái cùng sự kiện. Crate không có window, texture, audio,
//! I/O hay Bevy, nên client, bộ chạy headless và server sau này dùng chung một bộ luật. Adapter
//! ECS nằm ở `myva-gameplay` và chỉ gọi [`session::Session::step`].
//!
//! Quy ước (combat.md §4, architecture.md §5):
//! - Thời gian luật tính bằng tick 60 Hz; timing thiết kế ghi bằng ms và được làm tròn lên.
//! - Vị trí, tài nguyên và damage là số nguyên để replay tái lập được giữa các target.
//! - Mọi thông số trong [`kit`], [`monster`] và [`status`] là giả thuyết (GT) cho graybox, chưa
//!   được cân bằng.
//!
//! Chưa làm: đại thuật, Thoát Mạch, DR khống chế, coyote time/jump buffer, đòn không trung riêng,
//! spawn giữa trận và phần thưởng (thuộc server nghiệp vụ).

pub mod battle;
pub mod boss;
pub mod bot;
pub mod fighter;
pub mod fixture;
pub mod input;
pub mod kit;
pub mod meter;
pub mod monster;
pub mod npc;
pub mod projectile;
pub mod protocol;
pub mod replay;
pub mod rng;
pub mod session;
pub mod snapshot;
pub mod status;
pub mod tick;
pub mod world;

pub use fighter::{Fighter, FighterId, State, Team};
pub use input::{Buttons, InputFrame, Intent};
pub use kit::{ActionKind, Kit, LONG_LUU, PX, Phase};
pub use protocol::{CommandEnvelope, CommandFrame, EntityRef, SessionEpoch, TickReport};
pub use session::{Role, Session, SessionConfig};
pub use world::{Event, World};
