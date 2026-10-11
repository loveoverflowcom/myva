//! Kinh tế tài nguyên của MyVa — Thần Mạch (docs/design/economy.md, work-plan 030).
//!
//! - [`ledger`]: sổ cái sáu trạng thái `M_r = R + N + I + X + Q + Z`, mỗi chuyển có mã sự kiện
//!   duy nhất; retry cùng mã trả kết quả cũ.
//! - [`budget`]: cấp độ hiệu dụng và ngân sách tái sinh theo giờ.
//! - [`sim`]: bộ chạy headless tìm lỗi bảo toàn và vượt trần, không dự báo hành vi người thật.
//!
//! Đây là mô hình để mô phỏng. Server online dùng transaction và unique constraint của
//! PostgreSQL cho cùng các quy tắc (architecture.md §7). Mọi hệ số là giả thuyết (GT).

pub mod budget;
pub mod ledger;
pub mod sim;

pub use budget::{Budget, BudgetInputs, BudgetParams};
pub use ledger::{Applied, Bucket, Cause, EventId, Ledger, LedgerError, Transfer};
