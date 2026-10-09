//! Bộ chạy headless cho một tài nguyên thường (work-plan 030, bước 1).
//!
//! Mỗi giờ: phục hồi `Q → R` đến hạn, đo cấp độ hiệu dụng, tính `B_r`, cấp gói hỗ trợ và điểm
//! khai thác, rồi cho cohort người chơi khai thác, chế tác và tiêu hao. Sau mỗi giờ kiểm tra bảo
//! toàn, trần giờ/ngày và đối soát hàng đợi phục hồi với `Q`. Một phần chuyển được gửi lại cùng
//! mã để thử idempotency.
//!
//! Hành vi người chơi ở đây là giả định thô; kết quả chỉ dùng để bắt lỗi luật, không dự báo giá
//! hay trải nghiệm. Chưa mô hình hóa kênh, thị trường, tiền tệ, alt/bot và tài nguyên hiếm.

use std::collections::VecDeque;

use crate::budget::{self, ActivityParams, BudgetInputs, BudgetParams};
use crate::ledger::{Applied, Bucket, Cause, EventId, Ledger, Transfer};

const FORMULA_VERSION: u32 = 1;

/// Cú sốc rơi vào đầu ngày: với thông số §6, trần ngày cạn sau khoảng 13 giờ, nên sốc buổi tối
/// chỉ gặp ngân sách 0 và không kiểm tra được gì.
pub const SURGE_HOUR_OF_DAY: u32 = 6;

#[derive(Clone, Debug, PartialEq)]
pub struct SimConfig {
    pub seed: u64,
    pub hours: u32,
    /// `M_r`.
    pub cap: u64,
    pub budget: BudgetParams,
    pub activity: ActivityParams,
    /// Phần trăm `B_r` dành cho gói hỗ trợ người mới; nằm trong `B_r`, không cộng thêm.
    pub support_share_pct: u64,
    pub starter_pack: u64,
    pub recovery_delay_hours: u32,
    /// Tổng sức chứa của các điểm khai thác.
    pub node_capacity: u64,
    pub base_players: u32,
    /// Giờ số tài khoản tăng 10 lần trong một giờ (economy.md §12).
    pub surge_hour: Option<u32>,
    /// Phần nghìn số chuyển được gửi lại cùng mã.
    pub retry_permille: u32,
}

impl SimConfig {
    /// Linh thảo trong pilot: `M_r` 100.000, phục hồi 100% sau 24 giờ, mô phỏng 90 ngày.
    pub fn fiber(seed: u64) -> Self {
        Self {
            seed,
            hours: 90 * 24,
            cap: 100_000,
            budget: BudgetParams::FIBER,
            activity: ActivityParams::default(),
            support_share_pct: 20,
            starter_pack: 20,
            recovery_delay_hours: 24,
            node_capacity: 6_000,
            base_players: 200,
            surge_hour: Some(30 * 24 + SURGE_HOUR_OF_DAY),
            retry_permille: 50,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HourRow {
    pub hour: u32,
    pub players: u32,
    pub effective_level: f64,
    pub target: f64,
    pub candidate: f64,
    pub budget: u64,
    pub issued: u64,
    /// Số dư theo thứ tự `R, N, I, X, Q, Z`.
    pub balances: [u64; 6],
}

impl HourRow {
    pub const CSV_HEADER: &'static str =
        "hour,players,E,target,candidate,budget,issued,R,N,I,X,Q,Z";

    pub fn to_csv(&self) -> String {
        let [r, n, i, x, q, z] = self.balances;
        format!(
            "{},{},{:.1},{:.2},{:.2},{},{},{r},{n},{i},{x},{q},{z}",
            self.hour,
            self.players,
            self.effective_level,
            self.target,
            self.candidate,
            self.budget,
            self.issued
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimReport {
    pub rows: Vec<HourRow>,
    pub violations: Vec<String>,
    /// Số lần gửi lại được sổ cái nhận diện là đã ghi.
    pub replays: u64,
}

/// SplitMix64 riêng của crate để kinh tế không phụ thuộc lõi combat.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Số nguyên đều trong `lo..=hi`.
    fn range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + (((self.next_u64() >> 32) * u64::from(hi - lo + 1)) >> 32) as u32
    }
}

struct Runner {
    ledger: Ledger,
    next_id: u64,
    last: Option<Transfer>,
    report: SimReport,
}

impl Runner {
    fn transfer(&mut self, hour: u32, from: Bucket, to: Bucket, amount: u64, cause: Cause) {
        if amount == 0 {
            return;
        }
        self.next_id += 1;
        let transfer = Transfer {
            id: EventId(self.next_id),
            from,
            to,
            amount,
            cause,
            formula_version: FORMULA_VERSION,
        };
        if let Err(error) = self.ledger.apply(transfer) {
            self.report
                .violations
                .push(format!("giờ {hour}: {cause:?} bị từ chối: {error}"));
        }
        self.last = Some(transfer);
    }

    fn retry_last(&mut self, hour: u32) {
        let Some(transfer) = self.last else { return };
        match self.ledger.apply(transfer) {
            Ok(Applied::Replayed) => self.report.replays += 1,
            outcome => self.report.violations.push(format!(
                "giờ {hour}: gửi lại {:?} cho {outcome:?}",
                transfer.id
            )),
        }
    }
}

pub fn run(config: &SimConfig) -> SimReport {
    let mut rng = Rng(config.seed);
    let mut runner = Runner {
        ledger: Ledger::new(config.cap),
        next_id: 0,
        last: None,
        report: SimReport::default(),
    };
    let p = &config.budget;
    let mut previous_rate = p.base_rate;
    let mut issued_today = 0;
    let mut recovering: VecDeque<(u32, u64)> = VecDeque::new();
    let mut usage_24h: VecDeque<u64> = VecDeque::new();
    let usage_reference = (p.base_rate * 24.0).max(1.0);
    let inventory_target = (config.cap * 2 / 5).max(1) as f64;

    for hour in 0..config.hours {
        if hour % 24 == 0 {
            // Ngân sách ngày chưa dùng hết hạn, không cộng dồn.
            issued_today = 0;
        }
        while let Some(&(ready, amount)) = recovering.front() {
            if ready > hour {
                break;
            }
            recovering.pop_front();
            runner.transfer(
                hour,
                Bucket::Recovering,
                Bucket::Pool,
                amount,
                Cause::Recovery,
            );
        }

        let mut players = config.base_players * rng.range(80, 120) / 100;
        if config.surge_hour == Some(hour) {
            players *= 10;
        }
        let effective_level = budget::effective_level(
            (0..players).map(|_| {
                budget::contribution(rng.range(1, 30), rng.range(5, 90), 1.0, &config.activity)
            }),
            &config.activity,
        );

        let ledger = &runner.ledger;
        let usage_ratio = usage_24h.iter().sum::<u64>() as f64 / usage_reference;
        let stock_ratio = ledger.balance(Bucket::Inventory) as f64 / inventory_target;
        let inputs = BudgetInputs {
            effective_level,
            demand: budget::demand_factor(usage_ratio, stock_ratio),
            pool: ledger.balance(Bucket::Pool),
            daily_remaining: p.daily_cap.saturating_sub(issued_today),
            lifetime_remaining: None,
            node_space: config
                .node_capacity
                .saturating_sub(ledger.balance(Bucket::Nodes)),
        };
        let budget = budget::hourly_budget(p, previous_rate, &inputs);
        previous_rate = budget.candidate;

        // Phần hỗ trợ chưa dùng ở lại trong bể, không đổ sang điểm công cộng trong cùng giờ.
        let support_budget = budget.amount * config.support_share_pct / 100;
        let newcomers = u64::from(players / 20);
        let packs = (support_budget / config.starter_pack.max(1)).min(newcomers);
        for _ in 0..packs {
            runner.transfer(
                hour,
                Bucket::Pool,
                Bucket::Inventory,
                config.starter_pack,
                Cause::StarterPack,
            );
        }
        let public = budget.amount - support_budget;
        runner.transfer(hour, Bucket::Pool, Bucket::Nodes, public, Cause::NodeGrant);
        let issued = packs * config.starter_pack + public;
        issued_today += issued;

        let ledger = &runner.ledger;
        let harvest = ledger
            .balance(Bucket::Nodes)
            .min(u64::from(players) * u64::from(rng.range(1, 4)));
        runner.transfer(
            hour,
            Bucket::Nodes,
            Bucket::Inventory,
            harvest,
            Cause::Harvest,
        );
        let inventory = runner.ledger.balance(Bucket::Inventory);
        let craft = inventory / 50;
        let consume = inventory / 100;
        runner.transfer(
            hour,
            Bucket::Inventory,
            Bucket::Embedded,
            craft,
            Cause::Craft,
        );
        runner.transfer(
            hour,
            Bucket::Inventory,
            Bucket::Recovering,
            consume,
            Cause::Consume,
        );
        let wear = runner.ledger.balance(Bucket::Embedded) / 200;
        runner.transfer(
            hour,
            Bucket::Embedded,
            Bucket::Recovering,
            wear,
            Cause::Wear,
        );
        let used = consume + wear;
        if used > 0 {
            recovering.push_back((hour + config.recovery_delay_hours, used));
        }
        usage_24h.push_back(used);
        if usage_24h.len() > 24 {
            usage_24h.pop_front();
        }
        if rng.range(1, 1_000) <= config.retry_permille {
            runner.retry_last(hour);
        }

        let ledger = &runner.ledger;
        let mut check = |ok: bool, message: String| {
            if !ok {
                runner
                    .report
                    .violations
                    .push(format!("giờ {hour}: {message}"));
            }
        };
        check(
            ledger.is_conserved(),
            format!("tổng {} ≠ trần {}", ledger.total(), ledger.cap()),
        );
        check(
            issued <= p.hourly_cap,
            format!("cấp {issued} > trần giờ {}", p.hourly_cap),
        );
        check(
            issued_today <= p.daily_cap,
            format!("cấp {issued_today} > trần ngày {}", p.daily_cap),
        );
        let queued: u64 = recovering.iter().map(|&(_, amount)| amount).sum();
        check(
            queued == ledger.balance(Bucket::Recovering),
            format!(
                "hàng đợi phục hồi {queued} ≠ Q {}",
                ledger.balance(Bucket::Recovering)
            ),
        );

        runner.report.rows.push(HourRow {
            hour,
            players,
            effective_level,
            target: budget.target,
            candidate: budget.candidate,
            budget: budget.amount,
            issued,
            balances: runner.ledger.balances(),
        });
    }
    runner.report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ninety_days_hold_every_invariant() {
        for seed in 0..4 {
            let config = SimConfig::fiber(seed);
            let report = run(&config);
            assert!(report.violations.is_empty(), "{:#?}", report.violations);
            assert_eq!(report.rows.len(), 90 * 24);
            assert!(report.replays > 0, "idempotency phải được thử");
            for day in report.rows.chunks(24) {
                let issued: u64 = day.iter().map(|row| row.issued).sum();
                assert!(issued <= config.budget.daily_cap);
            }
        }
    }

    #[test]
    fn tenfold_surge_does_not_raise_supply_above_caps() {
        let config = SimConfig::fiber(9);
        let surge = config.surge_hour.unwrap();
        let report = run(&config);
        let row = &report.rows[surge as usize];
        assert!(row.players >= config.base_players * 8);
        assert!(row.budget > 0, "sốc phải rơi vào giờ còn ngân sách");
        assert!(row.issued <= config.budget.hourly_cap);
        let before = report.rows[surge as usize - 1].candidate;
        assert!(
            row.candidate <= before * 1.05 + 1e-9,
            "tốc độ chỉ tăng tối đa 5%/giờ"
        );
    }

    #[test]
    fn same_seed_same_rows() {
        let config = SimConfig {
            hours: 24 * 7,
            ..SimConfig::fiber(3)
        };
        assert_eq!(run(&config), run(&config));
    }
}
