//! Cấp độ hiệu dụng và ngân sách tái sinh theo giờ (economy.md §5–§6).

/// Tham số cấp độ hiệu dụng: `L_cap`, `T_active` (phút) và `E_pilot_cap`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivityParams {
    pub level_cap: f64,
    pub active_minutes: f64,
    pub pilot_cap: f64,
}

impl Default for ActivityParams {
    fn default() -> Self {
        Self {
            level_cap: 30.0,
            active_minutes: 60.0,
            pilot_cap: 6_000.0,
        }
    }
}

/// `e_i = l_i × a_i × q_i` với `l_i = L_cap × sqrt(min(L_i, L_cap) / L_cap)`.
pub fn contribution(level: u32, active_minutes: u32, eligibility: f64, p: &ActivityParams) -> f64 {
    let level = p.level_cap * (f64::from(level).min(p.level_cap) / p.level_cap).sqrt();
    let activity = (f64::from(active_minutes) / p.active_minutes).min(1.0);
    level * activity * eligibility.clamp(0.0, 1.0)
}

/// `E = min(Σ e_i, E_pilot_cap)`.
pub fn effective_level(contributions: impl IntoIterator<Item = f64>, p: &ActivityParams) -> f64 {
    contributions.into_iter().sum::<f64>().min(p.pilot_cap)
}

/// `d_r = clamp(1 + 0,2 × (u − 1) − 0,2 × (w − 1), 0,8, 1,2)`; `u` là tiêu hao 24 giờ / tham
/// chiếu, `w` là tồn kho / mục tiêu.
pub fn demand_factor(usage_ratio: f64, stock_ratio: f64) -> f64 {
    (1.0 + 0.2 * (usage_ratio - 1.0) - 0.2 * (stock_ratio - 1.0)).clamp(0.8, 1.2)
}

/// Tham số bộ điều khiển cho một tài nguyên.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BudgetParams {
    /// `r_base` (đơn vị/giờ).
    pub base_rate: f64,
    pub beta: f64,
    pub e_ref: f64,
    /// `H`: trần mỗi giờ.
    pub hourly_cap: u64,
    /// `D`: trần mỗi ngày.
    pub daily_cap: u64,
    /// `R_target`: dưới mức này bể bắt đầu làm chậm phục hồi.
    pub pool_target: u64,
}

impl BudgetParams {
    /// Linh thảo (`fiber`), bảng §6; `R_target` lấy từ ví dụ §11.
    pub const FIBER: Self = Self {
        base_rate: 600.0,
        beta: 0.4,
        e_ref: 3_000.0,
        hourly_cap: 800,
        daily_cap: 10_000,
        pool_target: 800,
    };

    /// Mạch khoáng (`ore`); `R_target` chưa có trong tài liệu, tạm theo tỷ lệ của Linh thảo.
    pub const ORE: Self = Self {
        base_rate: 300.0,
        beta: 0.4,
        e_ref: 3_000.0,
        hourly_cap: 450,
        daily_cap: 5_000,
        pool_target: 400,
    };
}

/// Số liệu đầu vào của một giờ.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BudgetInputs {
    pub effective_level: f64,
    pub demand: f64,
    /// `R` hiện có.
    pub pool: u64,
    pub daily_remaining: u64,
    /// `J_remaining`; `None` khi chưa bật trần suốt đời.
    pub lifetime_remaining: Option<u64>,
    /// Chỗ trống của điểm khai thác và gói hỗ trợ trong giờ.
    pub node_space: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Budget {
    pub target: f64,
    /// Tốc độ sau khi giới hạn ±5%; dùng làm `previous` cho giờ sau.
    pub candidate: f64,
    /// `B_r`: lượng được phép chuyển ra trong giờ.
    pub amount: u64,
}

/// `B_r = floor(max(0, min(candidate × ΔT, H, D_remaining, R, J_remaining, NodeSpace)))` với
/// `ΔT = 1 giờ`. Giới hạn ±5% chỉ làm mượt mục tiêu; trần cứng vẫn đưa ngân sách về 0 ngay.
pub fn hourly_budget(p: &BudgetParams, previous_rate: f64, input: &BudgetInputs) -> Budget {
    assert!(previous_rate >= 0.0, "tốc độ trước phải không âm");
    let scarcity = if p.pool_target == 0 {
        1.0
    } else {
        (input.pool as f64 / p.pool_target as f64).clamp(0.0, 1.0)
    };
    let target = p.base_rate
        * (1.0 + p.beta * (1.0 + input.effective_level / p.e_ref).ln())
        * input.demand.clamp(0.8, 1.2)
        * scarcity;
    let candidate = target.clamp(previous_rate * 0.95, previous_rate * 1.05);
    let amount = [
        p.hourly_cap,
        input.daily_remaining,
        input.pool,
        input.lifetime_remaining.unwrap_or(u64::MAX),
        input.node_space,
    ]
    .into_iter()
    .fold(candidate.max(0.0).floor() as u64, u64::min);
    Budget {
        target,
        candidate,
        amount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_inputs() -> BudgetInputs {
        // economy.md §11: E = E_ref, d = s = 1, R = 900, trần ngày còn 700.
        BudgetInputs {
            effective_level: 3_000.0,
            demand: 1.0,
            pool: 900,
            daily_remaining: 700,
            lifetime_remaining: None,
            node_space: u64::MAX,
        }
    }

    #[test]
    fn reproduces_worked_example() {
        let budget = hourly_budget(&BudgetParams::FIBER, 700.0, &example_inputs());
        assert!((budget.target - 766.36).abs() < 0.01, "{}", budget.target);
        assert!((budget.candidate - 735.0).abs() < 1e-9);
        assert_eq!(budget.amount, 700);
        assert_eq!(budget.amount * 20 / 100, 140, "phần hỗ trợ người mới");
    }

    #[test]
    fn hard_limits_cut_immediately_despite_smoothing() {
        // Giờ sau ví dụ: bể còn 200, trần ngày đã hết.
        let input = BudgetInputs {
            pool: 200,
            daily_remaining: 0,
            ..example_inputs()
        };
        let budget = hourly_budget(&BudgetParams::FIBER, 735.0, &input);
        assert!(budget.target < 200.0, "s = 200/800 làm chậm mục tiêu");
        assert!((budget.candidate - 735.0 * 0.95).abs() < 1e-9);
        assert_eq!(budget.amount, 0);

        let refilled_day = BudgetInputs {
            daily_remaining: 10_000,
            ..input
        };
        let budget = hourly_budget(&BudgetParams::FIBER, 735.0, &refilled_day);
        assert_eq!(budget.amount, 200, "không cấp quá lượng còn trong bể");
    }

    #[test]
    fn more_players_never_lifts_hard_caps() {
        let crowd = BudgetInputs {
            effective_level: 1e9,
            pool: u64::MAX,
            daily_remaining: u64::MAX,
            ..example_inputs()
        };
        let budget = hourly_budget(&BudgetParams::FIBER, 10_000.0, &crowd);
        assert_eq!(budget.amount, BudgetParams::FIBER.hourly_cap);
    }

    #[test]
    fn effective_level_has_diminishing_returns_and_cap() {
        let p = ActivityParams::default();
        assert_eq!(contribution(30, 60, 1.0, &p), 30.0);
        assert_eq!(contribution(99, 600, 1.0, &p), 30.0);
        assert_eq!(contribution(30, 30, 1.0, &p), 15.0);
        assert_eq!(contribution(30, 60, 0.0, &p), 0.0);
        let gain_low = contribution(10, 60, 1.0, &p) - contribution(5, 60, 1.0, &p);
        let gain_high = contribution(30, 60, 1.0, &p) - contribution(25, 60, 1.0, &p);
        assert!(gain_low > gain_high);
        assert_eq!(effective_level(std::iter::repeat_n(30.0, 300), &p), 6_000.0);
    }

    #[test]
    fn demand_factor_is_bounded() {
        assert_eq!(demand_factor(1.0, 1.0), 1.0);
        assert_eq!(demand_factor(5.0, 1.0), 1.2);
        assert_eq!(demand_factor(1.0, 5.0), 0.8);
    }
}
