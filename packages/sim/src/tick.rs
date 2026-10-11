//! Tick mô phỏng và lượng tử hóa thời gian thiết kế (combat.md §4).

/// Tần số tick ứng viên; chỉ chốt sau benchmark CPU, băng thông và latency (architecture.md §1).
pub const TICK_HZ: u32 = 60;

/// Số tick đã chạy kể từ khi tạo thế giới.
pub type Tick = u32;

/// Đổi mili giây thiết kế sang tick, làm tròn lên để hành động không xảy ra sớm hơn mô tả.
pub const fn ms_to_ticks(ms: u32) -> u32 {
    (ms * TICK_HZ).div_ceil(1000)
}

/// Thời lượng thực sau lượng tử hóa, dùng để công bố timing runtime.
pub const fn ticks_to_ms(ticks: u32) -> u32 {
    ticks * 1000 / TICK_HZ
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_combat_doc_examples() {
        // combat.md §4: startup 180 ms → 11 tick, active 100 ms → 6, recovery 240 ms → 15.
        assert_eq!(ms_to_ticks(180), 11);
        assert_eq!(ms_to_ticks(100), 6);
        assert_eq!(ms_to_ticks(240), 15);
    }

    #[test]
    fn never_runs_earlier_and_adds_less_than_one_tick() {
        for ms in 0..10_000 {
            let quantized = ticks_to_ms(ms_to_ticks(ms));
            assert!(quantized >= ms, "{ms} ms bị rút ngắn còn {quantized} ms");
            assert!(
                quantized < ms + 1000 / TICK_HZ + 1,
                "{ms} ms bị kéo dài quá một tick"
            );
        }
    }
}
