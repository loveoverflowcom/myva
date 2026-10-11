//! Sổ cái vật liệu cho một loại tài nguyên (economy.md §4).

use std::collections::BTreeMap;
use std::fmt;

/// Trạng thái giữ vật liệu, đo bằng đơn vị của chính loại tài nguyên.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bucket {
    /// `R`: bể chưa phân bổ.
    Pool,
    /// `N`: trên điểm khai thác, kể cả đã cấp cho kênh nhưng chưa thu.
    Nodes,
    /// `I`: kho người chơi, bang hội, thư và ký quỹ thị trường.
    Inventory,
    /// `X`: nằm trong vật phẩm hoặc công trình, theo thành phần.
    Embedded,
    /// `Q`: đã tiêu hao, chờ quay về bể.
    Recovering,
    /// `Z`: loại bỏ vĩnh viễn.
    Removed,
}

impl Bucket {
    pub const ALL: [Bucket; 6] = [
        Bucket::Pool,
        Bucket::Nodes,
        Bucket::Inventory,
        Bucket::Embedded,
        Bucket::Recovering,
        Bucket::Removed,
    ];

    pub const fn symbol(self) -> char {
        match self {
            Bucket::Pool => 'R',
            Bucket::Nodes => 'N',
            Bucket::Inventory => 'I',
            Bucket::Embedded => 'X',
            Bucket::Recovering => 'Q',
            Bucket::Removed => 'Z',
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// Các chuyển trạng thái hợp lệ. Không có đường nào tạo vật liệu ngoài bể hoặc đưa `Z` trở lại.
pub const fn is_allowed(from: Bucket, to: Bucket) -> bool {
    use Bucket::*;
    matches!(
        (from, to),
        // Cấp cho điểm khai thác; gói hỗ trợ người mới nằm trong cùng ngân sách.
        (Pool, Nodes | Inventory)
            // Khai thác; đóng điểm hoặc kênh trả phần còn lại về bể.
            | (Nodes, Inventory | Pool)
            // Chế tác và tháo dỡ.
            | (Inventory, Embedded)
            | (Embedded, Inventory)
            // Tiêu hao, sửa chữa, hao hụt khi tháo dỡ.
            | (Inventory | Embedded, Recovering | Removed)
            // Phục hồi sau thời gian chờ.
            | (Recovering, Pool)
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cause {
    NodeGrant,
    StarterPack,
    Harvest,
    NodeClose,
    Craft,
    Salvage,
    Consume,
    Wear,
    Recovery,
}

/// Một chuyển trạng thái. `id` duy nhất để retry an toàn; `formula_version` ghi phiên bản công
/// thức đã sinh ra nó.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub id: EventId,
    pub from: Bucket,
    pub to: Bucket,
    pub amount: u64,
    pub cause: Cause,
    pub formula_version: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    New,
    /// Mã đã được ghi với cùng nội dung; không chuyển lần hai.
    Replayed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedgerError {
    IllegalTransition {
        from: Bucket,
        to: Bucket,
    },
    Insufficient {
        bucket: Bucket,
        available: u64,
        requested: u64,
    },
    ZeroAmount,
    /// Cùng mã sự kiện nhưng nội dung khác; không ghi đè.
    Conflict(EventId),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::IllegalTransition { from, to } => {
                write!(f, "chuyển {} → {} không hợp lệ", from.symbol(), to.symbol())
            }
            LedgerError::Insufficient {
                bucket,
                available,
                requested,
            } => write!(
                f,
                "{} chỉ còn {available}, cần {requested}",
                bucket.symbol()
            ),
            LedgerError::ZeroAmount => write!(f, "lượng chuyển bằng 0"),
            LedgerError::Conflict(id) => write!(f, "mã sự kiện {} đã dùng cho nội dung khác", id.0),
        }
    }
}

impl std::error::Error for LedgerError {}

#[derive(Clone, Debug)]
pub struct Ledger {
    cap: u64,
    balances: [u64; 6],
    applied: BTreeMap<EventId, Transfer>,
}

impl Ledger {
    /// Sổ mới với toàn bộ trần `M_r` nằm trong bể.
    pub fn new(cap: u64) -> Self {
        let mut balances = [0; 6];
        balances[Bucket::Pool.index()] = cap;
        Self::with_balances(balances)
    }

    /// Sổ với số dư cho sẵn theo thứ tự `R, N, I, X, Q, Z`; trần bằng tổng.
    pub fn with_balances(balances: [u64; 6]) -> Self {
        Self {
            cap: balances.iter().sum(),
            balances,
            applied: BTreeMap::new(),
        }
    }

    pub fn cap(&self) -> u64 {
        self.cap
    }

    pub fn balance(&self, bucket: Bucket) -> u64 {
        self.balances[bucket.index()]
    }

    pub fn balances(&self) -> [u64; 6] {
        self.balances
    }

    pub fn total(&self) -> u64 {
        self.balances.iter().sum()
    }

    pub fn is_conserved(&self) -> bool {
        self.total() == self.cap
    }

    /// Ghi một chuyển nguyên tử: đủ điều kiện thì chuyển trọn, không thì không đổi gì.
    pub fn apply(&mut self, transfer: Transfer) -> Result<Applied, LedgerError> {
        if let Some(previous) = self.applied.get(&transfer.id) {
            return if *previous == transfer {
                Ok(Applied::Replayed)
            } else {
                Err(LedgerError::Conflict(transfer.id))
            };
        }
        let Transfer {
            from, to, amount, ..
        } = transfer;
        if amount == 0 {
            return Err(LedgerError::ZeroAmount);
        }
        if !is_allowed(from, to) {
            return Err(LedgerError::IllegalTransition { from, to });
        }
        let available = self.balance(from);
        if available < amount {
            return Err(LedgerError::Insufficient {
                bucket: from,
                available,
                requested: amount,
            });
        }
        self.balances[from.index()] -= amount;
        self.balances[to.index()] += amount;
        self.applied.insert(transfer.id, transfer);
        debug_assert!(self.is_conserved());
        Ok(Applied::New)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transfer(id: u64, from: Bucket, to: Bucket, amount: u64) -> Transfer {
        Transfer {
            id: EventId(id),
            from,
            to,
            amount,
            cause: Cause::NodeGrant,
            formula_version: 1,
        }
    }

    #[test]
    fn worked_example_conserves_total() {
        // economy.md §11: bảy gói 20 đơn vị và bảy điểm 80 đơn vị trong một giờ.
        let mut ledger = Ledger::with_balances([900, 4_000, 60_000, 25_000, 10_100, 0]);
        assert_eq!(ledger.cap(), 100_000);
        for i in 0..7 {
            ledger
                .apply(transfer(i, Bucket::Pool, Bucket::Inventory, 20))
                .unwrap();
            ledger
                .apply(transfer(100 + i, Bucket::Pool, Bucket::Nodes, 80))
                .unwrap();
        }
        assert_eq!(ledger.balances(), [200, 4_560, 60_140, 25_000, 10_100, 0]);
        assert!(ledger.is_conserved());
    }

    #[test]
    fn retry_with_same_id_does_not_move_twice() {
        let mut ledger = Ledger::new(1_000);
        let grant = transfer(1, Bucket::Pool, Bucket::Nodes, 100);
        assert_eq!(ledger.apply(grant), Ok(Applied::New));
        assert_eq!(ledger.apply(grant), Ok(Applied::Replayed));
        assert_eq!(ledger.balance(Bucket::Nodes), 100);

        let tampered = Transfer {
            amount: 500,
            ..grant
        };
        assert_eq!(
            ledger.apply(tampered),
            Err(LedgerError::Conflict(EventId(1)))
        );
        assert_eq!(ledger.balance(Bucket::Nodes), 100);
    }

    #[test]
    fn rejects_illegal_or_partial_moves() {
        let mut ledger = Ledger::with_balances([10, 0, 5, 0, 0, 7]);
        for (from, to) in [
            (Bucket::Removed, Bucket::Pool),
            (Bucket::Inventory, Bucket::Pool),
            (Bucket::Pool, Bucket::Embedded),
            (Bucket::Pool, Bucket::Recovering),
        ] {
            assert_eq!(
                ledger.apply(transfer(1, from, to, 1)),
                Err(LedgerError::IllegalTransition { from, to })
            );
        }
        assert_eq!(
            ledger.apply(transfer(2, Bucket::Inventory, Bucket::Embedded, 6)),
            Err(LedgerError::Insufficient {
                bucket: Bucket::Inventory,
                available: 5,
                requested: 6,
            })
        );
        assert_eq!(
            ledger.apply(transfer(3, Bucket::Pool, Bucket::Nodes, 0)),
            Err(LedgerError::ZeroAmount)
        );
        assert_eq!(ledger.balances(), [10, 0, 5, 0, 0, 7]);
    }

    #[test]
    fn nothing_leaves_removed() {
        for to in Bucket::ALL {
            assert!(!is_allowed(Bucket::Removed, to));
        }
    }
}
