#!/usr/bin/env bash
# So hash từng tick của cùng fixture: lõi chạy WASM (Node) với adapter ECS native.
# Dùng: scripts/check-wasm-determinism.sh [ticks] [seed...]; kết quả ở target/wasm-determinism.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
ticks="${1:-3600}"
shift || true
(( $# > 0 )) || set -- 1 7 12
seeds=("$@")
if [[ "$(wasm-bindgen --version 2>/dev/null || true)" != "wasm-bindgen 0.2.129" ]]; then
  echo 'Install: cargo install wasm-bindgen-cli --version 0.2.129 --locked' >&2
  exit 1
fi
out="$root/target/wasm-determinism"
mkdir -p "$out"
cargo build --locked --manifest-path "$root/Cargo.toml" -p myva-sim --example wasm-fingerprint \
  --target wasm32-unknown-unknown --release
wasm-bindgen --target nodejs --out-dir "$out/pkg" \
  "$root/target/wasm32-unknown-unknown/release/examples/wasm_fingerprint.wasm"
cargo build --locked --manifest-path "$root/Cargo.toml" -p myva-gameplay --bin myva-headless --release
for seed in "${seeds[@]}"; do
  "$root/target/release/myva-headless" --seed "$seed" --ticks "$ticks" --hashes "$out/native-$seed.txt" >/dev/null
done
node "$root/scripts/check-wasm-determinism.mjs" "$out" "$ticks" "${seeds[@]}"
