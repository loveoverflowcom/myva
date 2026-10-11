#!/usr/bin/env bash
# Build both CSR shell and isolated Bevy game; run from any directory.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
profile="${PROFILE:-web}"
case "$profile" in web|release|dev) ;; *) echo "PROFILE must be web, release or dev" >&2; exit 1;; esac
if [[ "$(wasm-bindgen --version 2>/dev/null || true)" != "wasm-bindgen 0.2.129" ]]; then
  echo 'Install: cargo install wasm-bindgen-cli --version 0.2.129 --locked' >&2
  exit 1
fi
cargo build --locked --manifest-path "$root/Cargo.toml" -p myva-web-shell -p myva-web-game \
  --target wasm32-unknown-unknown --profile "$profile" -j "${CARGO_BUILD_JOBS:-2}"
target="$root/target/wasm32-unknown-unknown/$profile"
[[ "$profile" == dev ]] && target="$root/target/wasm32-unknown-unknown/debug"
out="$root/target/web-shell"
mkdir -p "$out/shell" "$out/game/pkg"
cp "$root/apps/web/public/index.html" "$root/apps/web/public/style.css" "$root/apps/web/public/bootstrap.js" "$out/"
cp "$root/apps/web/public/game/"* "$out/game/"
# Fixture art D05-A: AssetServer trên web đọc `assets/` cạnh game document. Validator chạy trước để
# không phát hành bộ asset lỗi hash/metadata.
cargo run --locked --quiet --manifest-path "$root/Cargo.toml" -p myva-assets -- validate "$root/assets/fixture/manifest.json" >/dev/null
rm -rf "$out/game/assets"
mkdir -p "$out/game/assets"
cp -R "$root/assets/fixture" "$out/game/assets/fixture"
wasm-bindgen --target web --out-dir "$out/shell" "$target/myva_web_shell.wasm"
wasm-bindgen --target web --out-dir "$out/game/pkg" "$target/myva_web_game.wasm"
python3 "$root/scripts/web-bundle-report.py" "$out" "$profile"
