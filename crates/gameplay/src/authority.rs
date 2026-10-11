//! Ai được quyết định kết quả chính thức của tick.
//!
//! Client và server chạy cùng plugin, cùng luật. Khác biệt duy nhất: chỉ [`Authority::Server`]
//! đăng ký hệ cấp phần thưởng và có [`RewardOutbox`]. Client không có đường nào tạo
//! [`RewardClaim`]: constructor là nội bộ crate, schema lệnh không có trường damage/phần thưởng,
//! và kết quả client mô phỏng chỉ là dự đoán chờ server xác nhận (architecture.md §5, §7).

use std::collections::BTreeSet;

use bevy_ecs::prelude::*;
use myva_sim::session::Role;
use myva_sim::tick::Tick;
use myva_sim::{Event, FighterId};

use crate::{LastTick, SimSession};

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Authority {
    /// Server hoặc runner headless authoritative.
    Server,
    /// Client chạy cùng luật để dự đoán hoặc chơi offline; không cấp phần thưởng.
    Client,
}

/// Khóa chống trùng: một instance chỉ cấp một lần cho mỗi nhân vật bị hạ. Server nghiệp vụ dùng
/// nó làm unique key khi ghi ledger, nên retry hoặc tick phát lại không nhân đôi.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RewardKey {
    pub instance: u64,
    pub defeated: FighterId,
}

/// Yêu cầu cấp phần thưởng gửi server nghiệp vụ; nội dung phần thưởng và người nhận do bên đó
/// quyết định trong transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RewardClaim {
    key: RewardKey,
    tick: Tick,
    kit: &'static str,
}

impl RewardClaim {
    pub fn key(&self) -> RewardKey {
        self.key
    }

    /// Tick lõi xác nhận bị hạ.
    pub fn tick(&self) -> Tick {
        self.tick
    }

    pub fn kit(&self) -> &'static str {
        self.kit
    }
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewardIssued(pub RewardClaim);

/// Hàng chờ phần thưởng của server; chỉ plugin ở chế độ server tạo được.
#[derive(Resource, Debug)]
pub struct RewardOutbox {
    issued: BTreeSet<RewardKey>,
    pending: Vec<RewardClaim>,
}

impl RewardOutbox {
    pub(crate) fn new() -> Self {
        Self {
            issued: BTreeSet::new(),
            pending: Vec::new(),
        }
    }

    /// Ghi nhận một lần hạ; trả `None` nếu khóa đã cấp.
    pub(crate) fn record(
        &mut self,
        key: RewardKey,
        tick: Tick,
        kit: &'static str,
    ) -> Option<RewardClaim> {
        if !self.issued.insert(key) {
            return None;
        }
        let claim = RewardClaim { key, tick, kit };
        self.pending.push(claim);
        Some(claim)
    }

    /// Server nghiệp vụ lấy claim để commit; khóa vẫn được giữ để chặn cấp lại.
    pub fn drain(&mut self) -> Vec<RewardClaim> {
        std::mem::take(&mut self.pending)
    }

    pub fn issued(&self) -> usize {
        self.issued.len()
    }
}

pub(crate) fn issue_rewards(
    session: Res<SimSession>,
    last: Res<LastTick>,
    mut outbox: ResMut<RewardOutbox>,
    mut issued: MessageWriter<RewardIssued>,
) {
    let Some(report) = last.report() else {
        return;
    };
    let session = session.get();
    for record in &report.events {
        let Event::Downed { fighter } = record.event else {
            continue;
        };
        if session.role(fighter) != Role::Monster {
            continue;
        }
        let key = RewardKey {
            instance: session.config().instance,
            defeated: fighter,
        };
        let kit = session.world().fighter(fighter).kit.id;
        if let Some(claim) = outbox.record(key, report.tick, kit) {
            issued.write(RewardIssued(claim));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_defeat_is_claimed_once() {
        let mut outbox = RewardOutbox::new();
        let key = |instance, id| RewardKey {
            instance,
            defeated: FighterId(id),
        };
        assert!(outbox.record(key(7, 1), 100, "quai-bun").is_some());
        assert!(
            outbox.record(key(7, 1), 100, "quai-bun").is_none(),
            "sự kiện lặp"
        );
        assert!(
            outbox.record(key(7, 1), 160, "quai-bun").is_none(),
            "tick khác"
        );
        assert!(outbox.record(key(7, 2), 160, "quai-bun").is_some());
        assert!(
            outbox.record(key(8, 1), 160, "quai-bun").is_some(),
            "instance khác là lần hạ khác"
        );
        assert_eq!(outbox.drain().len(), 3);
        assert!(outbox.drain().is_empty());
        assert!(
            outbox.record(key(7, 1), 200, "quai-bun").is_none(),
            "đã drain vẫn không cấp lại"
        );
        assert_eq!(outbox.issued(), 3);
    }
}
