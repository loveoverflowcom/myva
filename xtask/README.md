# MyVa xtask

`cargo xtask` là entrypoint development của MyVa — Thần Mạch. Quy tắc, mapping
primitive, dependency impact và giới hạn bằng chứng có một nguồn tại
[build contract](../docs/architecture/xtask-build-contract.md).

```bash
cargo xtask doctor
cargo xtask build web
cargo xtask run web --port 8080
# Trong terminal khác; chỉ mở bundle đã build và server đã sẵn sàng:
cargo xtask app web --port 8080
cargo xtask preflight --changed-only
cargo xtask ci
```

`run web` build incremental cả Leptos và Bevy qua pipeline hiện hữu, serve trên
loopback, kiểm tra `/bundle-report.json`, in URL/PID/log, rồi giữ foreground đến
Ctrl+C. `app web` không build hoặc start server. Port bận trả lỗi, không kill
listener. Log tại `target/xtask-evidence/web-<port>.log`.

```bash
cargo xtask build web --release
cargo xtask build web --profile dev
cargo xtask build graybox
cargo xtask build sim
cargo xtask build gameplay
cargo xtask build economy
cargo xtask build assets
cargo xtask build all
cargo xtask run graybox -- --duel
cargo xtask run graybox --no-default-features -- --no-art
cargo xtask run headless -- --seed 7 --ticks 3600
cargo xtask run replay -- verify /absolute/path/match.myva-replay
cargo xtask run economy -- --seed 7 --days 90 > econ.csv
cargo xtask run assets -- validate assets/fixture/manifest.json

cargo xtask verify sim
cargo xtask verify gameplay
cargo xtask verify economy
cargo xtask verify assets
cargo xtask verify web
cargo xtask verify web -- tests/web/protocol.spec.mjs
cargo xtask verify determinism --ticks 3600 --seeds 1,7,12
cargo xtask verify repository
cargo xtask verify xtask
cargo xtask fmt
cargo xtask fmt --check
cargo xtask preflight --changed-only --base origin/develop
```

Paths của CLI con được tính từ workspace root. Status của orchestrator đi ra
stderr; CSV của `run economy` vẫn là stdout thuần. Global `--dry-run` dùng trước
hoặc sau subcommand; nó báo `NOT_RUN`, không thực thi build/check/server/browser.
`preflight --changed-only --dry-run` chỉ đọc Git và Cargo metadata để in đúng plan.
`--jobs N` override concurrency; nếu không truyền, giữ `CARGO_BUILD_JOBS` và Cargo
config (mặc định 2). Các build heavyweight chạy tuần tự. `--no-default-features`
của graybox là lựa chọn tường minh bỏ gamepad, không là thay thế cho CI mặc định.

```bash
cargo xtask doctor mobile --platform android
cargo xtask doctor mobile --platform ios
cargo xtask build native-probe --platform android --mode check
cargo xtask build native-probe --platform android --mode build
cargo xtask build native-probe --platform android --mode apk
cargo xtask build native-probe --platform android --mode cmp
cargo xtask devices
cargo xtask devices --platform android
cargo xtask devices --platform ios
```

`native-probe` mặc định `apk`; cả bốn mode reuse script và Gradle wrapper. Kết quả
compile/package không là runtime/device PASS. Không có iOS build implementation;
`build native-probe --platform ios` trả `BLOCKED` và nonzero. `devices` yêu cầu ADB
server đã được developer mở; không tự boot emulator hoặc install/launch APK.
Không có command install/run mobile trong PR này.

Prerequisites do người dùng hoặc Actions provision: Rust 1.98.1 + rustfmt/clippy,
wasm32 target, wasm-bindgen CLI 0.2.129, Bash, Python 3, Node/npm, Playwright trong
`node_modules` và Chromium (hoặc `CHROME_PATH`). Linux native graybox cần libudev
development files. Android dùng toolchain pinned trong
[prototype README](../prototypes/native/README.md); iOS cần macOS/Xcode và các
gate còn thiếu trong [feasibility report](../docs/reports/native-feasibility.md).
xtask không gọi `rustup target add`, `cargo install`, `npm ci`, SDK installer hoặc
browser installer.

Module boundaries:

| Module | Responsibility |
| --- | --- |
| `cli` | clap parsing, typed targets/options |
| `config` | workspace root, paths, tool/target pins và loopback defaults |
| `command` | argument vectors, environment, cwd, stdout artifacts |
| `plan` | composable ordered stages; shared local/CI contract |
| `process` | cancellation, bounded probe capture, owned process groups và cleanup |
| `impact` | Git comparison và Cargo reverse dependency closure |
| `web` | port/readiness/build identity, foreground serving, browser và E2E lifecycle |
| `doctor`, `mobile` | prerequisite diagnostics và native feasibility orchestration |

Focused tests dùng `cargo xtask verify xtask`; full contract dùng
`cargo xtask ci`. Các lifecycle tests thực sự spawn/stop process và bind loopback;
sandbox cấm socket/process không đủ để chạy nhóm này. Không bỏ qua test rồi báo
PASS. [Verification report](../docs/reports/xtask-verification.md) ghi môi trường
và phần chưa chạy.

`verify repository` chạy thêm `scripts/check-skills.py` và negative suite bắt buộc
qua `--self-test`. Validator dùng Python standard library, kiểm canonical skills,
metadata/provenance, index, local links/anchors, references, Claude symlinks và
routing fixtures. Thiếu test target hoặc suite rỗng là FAIL/nonzero. Structural
PASS không chứng minh runtime discovery hoặc routing behavior; hai phần đó phải
được đánh giá riêng. Changed-only cho `.agents/`, `.claude/`, `AGENTS.md`,
`CLAUDE.md`, validator và test tương ứng chạy các gate này cùng xtask clippy/tests,
vẫn giữ workspace-wide Rust format và documentation checks. Diff trộn Rust hoặc
unknown inputs vẫn giữ dependency impact/full fallback hiện hữu.
