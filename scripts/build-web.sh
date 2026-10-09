#!/usr/bin/env bash
# Build graybox cho trình duyệt vào target/web/.
#
# Loader mq_js_bundle.js được copy từ crate macroquad đã tải về (MIT/Apache-2.0) lúc build,
# không lưu trong repo. Chạy thử: python3 -m http.server -d target/web 8080
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/target/web"
profile="${PROFILE:-release}"

cargo build --manifest-path "$root/Cargo.toml" -p myva-graybox \
  --target wasm32-unknown-unknown --profile "$profile"

manifest="$(cargo metadata --manifest-path "$root/Cargo.toml" --format-version 1 \
  | grep -o '"manifest_path":"[^"]*/macroquad-[0-9][^"/]*/Cargo.toml"' \
  | head -n 1 | cut -d '"' -f 4)"
if [[ -z "$manifest" ]]; then
  echo "không tìm thấy crate macroquad trong cargo metadata" >&2
  exit 1
fi

target_dir="$root/target/wasm32-unknown-unknown/$profile"
[[ "$profile" == "dev" ]] && target_dir="$root/target/wasm32-unknown-unknown/debug"

mkdir -p "$out"
cp "$target_dir/myva-graybox.wasm" "$out/"
cp "$(dirname "$manifest")/js/mq_js_bundle.js" "$out/"
cp "$root/crates/graybox/web/index.html" "$out/"
echo "đã build vào $out ($(du -h "$out/myva-graybox.wasm" | cut -f1) wasm)"
