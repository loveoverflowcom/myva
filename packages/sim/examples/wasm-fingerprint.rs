//! Fixture phòng thử chạy trong WASM để `scripts/check-wasm-determinism.sh` so hash từng tick với
//! bản native. Chỉ dùng cho kiểm chứng, không phải API phát hành.

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn fingerprint(seed: u64, ticks: u32) -> Vec<u64> {
    myva_sim::fixture::fingerprint(seed, ticks)
}
