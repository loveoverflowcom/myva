//! RNG có seed cho bot và kịch bản headless. Luật combat hiện không dùng ngẫu nhiên.

/// SplitMix64: nhỏ, nhanh, cho cùng chuỗi số trên mọi target với cùng seed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Số nguyên đều trong `0..n`; `n` phải dương.
    pub fn below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "khoảng rỗng");
        (((self.next_u64() >> 32) * u64::from(n)) >> 32) as u32
    }

    /// `true` với xác suất `num / den`.
    pub fn chance(&mut self, num: u32, den: u32) -> bool {
        self.below(den) < num
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let (mut a, mut b) = (Rng::new(42), Rng::new(42));
        for _ in 0..1_000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::new(7);
        for n in 1..200 {
            assert!(rng.below(n) < n);
        }
    }
}
