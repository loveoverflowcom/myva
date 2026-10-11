# MyVa xtask build contract

**Trạng thái:** development orchestration cho cấu trúc MyVa hiện có. Simulation,
gameplay, bridge v2, build profiles và native event-loop ownership giữ nguyên.

> **Single Surface Rule:** If a repository-level operation can be expressed
> through xtask, callers must use xtask.

Developer, CI và automation gọi `cargo xtask`. Cargo, Python, Bash, wasm-bindgen,
Node/Playwright và Gradle wrapper là implementation primitives. Script cũ vẫn
chạy được để giữ compatibility; không thêm logic orchestration thứ hai vào script,
npm hoặc CI. Các lệnh/tệp evidence của revision lịch sử giữ nguyên. Contributor
và agent dùng cùng contract này qua [CONTRIBUTING](../../CONTRIBUTING.md).

Tham khảo đã audit ở VOT revision `f3fea641`: [VOT xtask](https://github.com/loveoverflowcom/vot-workspace/tree/f3fea641a5fd226b883c4e32224ad8c97b8323f0/xtask),
[Cargo alias](https://github.com/loveoverflowcom/vot-workspace/blob/f3fea641a5fd226b883c4e32224ad8c97b8323f0/.cargo/config.toml),
[build contract](https://github.com/loveoverflowcom/vot-workspace/blob/f3fea641a5fd226b883c4e32224ad8c97b8323f0/docs/architecture/xtask-build-contract.md).
MyVa kế thừa typed commands, plan/executor separation, scoped process ownership
và Single Surface Rule; không nhập service, release/deploy, Trunk hoặc mobile
embedding framework của VOT.

## CLI và primitive mapping

| Surface | Actual execution |
| --- | --- |
| `build web [--release \| --profile dev\|web\|release]` | `scripts/build-web-shell.sh`, `PROFILE=web` mặc định; build hai crate WASM, validate assets, copy public/fixture, wasm-bindgen hai bundle, bundle report |
| `build sim\|gameplay\|economy\|assets\|graybox [--release]` | Cargo build đúng package bằng name và lockfile; graybox hỗ trợ `--no-default-features` |
| `build all [--release]` | Cargo build root workspace rồi web; tuần tự, không bao gồm mobile prototype |
| `run web [--port N]` | Kiểm port, incremental build web, start owned Python server, readiness, foreground lifecycle |
| `app web [--port N]` | Require bundle và matching report trên server hiện có, rồi mở browser; không build/start server |
| `run graybox [--release] [--no-default-features] -- <args>` | Cargo run graybox, forward nguyên arguments |
| `run headless\|replay\|economy\|assets -- <args>` | Reuse CLI `myva-headless`, `myva-replay`, `myva-econ-sim`, `myva-assets` |
| `verify sim` | Package tests; replay demo rồi verify bằng CLI |
| `verify gameplay` | Sim/gameplay tests; ECS/core hashes từng tick, seeds 1/7/12, fixture thường/Mạch; ghi và verify từng replay |
| `verify economy` | Package tests; simulator seed 7 trong 90 ngày, invariant failures nonzero; CSV artifact |
| `verify assets` | Validator tests, generated fixture `--check`, manifest/provenance/hash/PNG/bundle/resource budgets, Python report contract tests |
| `verify web` | WASM clippy, pure sim WASM build, hai bundle web, managed server, Playwright bridge/lifecycle/combat/Mạch/assets tests |
| `verify determinism [--ticks N --seeds 1,7,12]` | Existing native/WASM script: release fixture, Node, hash từng tick cho normal/Mạch; giữ report seed/ticks/hash |
| `verify repository` | Format, docs, root clippy/tests, replay/headless/economy/assets/report contracts; browser/determinism chọn riêng |
| `verify xtask` | Focused format, xtask clippy và CLI/plan/impact/lifecycle tests |
| `fmt [--check]` | Cargo fmt root workspace; explicit apply hoặc read-only check |
| `doctor` | Read-only prerequisites, không chạy gameplay/browser/device test |
| `preflight`, `ci` | Cùng full planner: repository checks + determinism + WASM/bridge/build/browser; không tự provision |
| `preflight --changed-only [--base REF]` | Chọn subset theo Git và dependency impact; cùng executors với full contract |
| `doctor mobile --platform android\|ios` | Rust targets, host, JDK/SDK/NDK/ADB/wrapper hoặc Xcode/SDK/simulator tooling |
| `build native-probe --platform android --mode check\|build\|apk\|cmp` | Existing Android script; Cargo và Gradle wrapper `--no-daemon`; không install/run |
| `devices [--platform android\|ios]` | Android devices trên ADB server có sẵn; `xcrun simctl list devices available` trên macOS; không boot simulator |

`verify web -- <Playwright args>` cho phép chọn spec khi debug. Full preflight/CI
không filter. Fresh JSON report phải có passing tests, không skipped/flaky/failing
tests hoặc global errors; `--list`, report cũ và zero tests không tạo PASS.

## Web artifact và lifecycle contract

- Shell: `apps/web/shell`, Leptos CSR; game: `apps/web/game`, Bevy WASM; static:
  `apps/web/public`. Không dùng Trunk. Script và wasm-bindgen 0.2.129 giữ pipeline.
- `build web` mặc định profile `web`; `--release` dùng `release`; `--profile dev`
  dùng Cargo debug. Output luôn `target/web-shell`, game assets tại
  `game/assets/fixture`, generated imports và snippets theo pipeline hiện hữu.
  `CARGO_TARGET_DIR` chỉ đổi nơi Cargo đặt intermediate binaries; public/evidence
  output vẫn ở root `target`. Cấu hình `build.target-dir` riêng không được script
  hỗ trợ; dùng biến môi trường khi cần đổi intermediate directory.
- Root tự tìm từ ancestor của cwd, fallback compile manifest path; không phụ
  thuộc việc shell đang ở root. Command cwd luôn là workspace root.
- Bind loopback `127.0.0.1`; port mặc định 8080, CLI `--port` hoặc `MYVA_WEB_PORT`.
  Tests dùng `MYVA_WEB_TEST_PORT`, mặc định 8080. Không scan/kill process theo port.
- Readiness là HTTP 200 `/bundle-report.json`, đúng nội dung bundle trên đĩa và
  token riêng của owned child. Spawn thành công chưa là readiness. Bind race,
  child exit, timeout hoặc khác report đều thất bại và dọn child của xtask.
- Sau readiness mới in Play/Readiness URL, workspace root, PID, log path và
  open/stop/restart commands. Foreground Ctrl+C/SIGTERM dừng owned process group,
  cho graceful shutdown tối đa 750 ms rồi kill phần còn lại và reap child.
- Process group thuộc từng command, kể cả build; lỗi, timeout, cancellation và
  parent exit vẫn cleanup descendants. Không dùng PID file cũ, process name,
  ADB/Gradle daemon hoặc listener discovery để kill. Unix có group ownership;
  non-Unix chỉ có direct-child fallback, các runner hiện yêu cầu Linux/macOS tools.
- Probe output dùng file-backed capture (tối đa 4 MiB) và timeout 30 giây;
  readiness bounded 20 giây. Build không có deadline giả, có cancellation.
- E2E dùng server do xtask sở hữu; Playwright không spawn server thứ hai. npm
  `test:web` forward về xtask; direct Playwright fallback cũng gọi xtask.

## Changed-only dependency impact

Default base `origin/develop`; không tự fetch, đổi base hoặc dùng `HEAD` thay cho
base thiếu. `git merge-base BASE HEAD` phải thành công trước gate execution.
Union gồm committed diff merge-base→HEAD, staged, unstaged, deleted và untracked
paths. Diff `--no-renames` giữ cả old/new paths, kể cả move khỏi một package.

Cargo metadata `--no-deps --locked --offline` cung cấp package directories và local
dependencies, gồm dev/build/target-specific dependencies. Planner tính reverse
transitive closure; không chỉ nhìn tên folder sửa trực tiếp. Dependency bên ngoài
repo, local dependency không map được, metadata/base lỗi đều trả nonzero trước
checks. Không diễn giải lỗi discovery thành diff rỗng.

| Input | Verification selection |
| --- | --- |
| Sim/gameplay | Native tests/clippy cho package và consumer, sim replay/headless, native/WASM determinism, WASM/bridge và browser E2E |
| Graybox | Graybox và web consumer tests/clippy; sim/gameplay behavior, determinism, WASM/browser |
| Web Rust/public/bridge/tests | Package checks, WASM clippy/build, determinism, hai bundle và browser E2E |
| Asset package/content/budgets | Validator/generator/provenance/hash/resource budgets và report tests; consumers graybox/web kéo theo browser verification |
| Economy | Native package checks và simulator invariant/CSV; không build web không liên quan |
| Native prototype | Root format/docs + nested format, script syntax, Android check/APK/CMP compile; iOS/runtime vẫn BLOCKED/NOT_RUN |
| Toolchain/Cargo/config/CI/xtask/scripts/docs/shared contracts hoặc unknown | Full root contract; native input trong cùng diff giữ thêm native gates |
| Empty diff | Full root contract, không báo pass từ một empty plan |

Format và docs luôn chạy. Native workspaces độc lập không là yêu cầu mặc định
của root CI; native paths chọn gate Android tường minh. Full contract không tự
claim Kotlin/iOS/device coverage. Printed selection và ordered plan là các stage
thực sự chạy; không có heavyweight build song song hoặc simulated PASS.

## Prerequisites, resources và evidence

`.cargo/config.toml` đặt alias và hai build jobs mặc định. `CARGO_BUILD_JOBS` hoặc
global `--jobs N` override; không dùng số CPU của host để tự tăng concurrency.
Android giữ Gradle worker/memory limits hiện có. Không auto-install rustup
targets/components, wasm-bindgen, npm packages, browser, JDK/SDK/NDK; không tải
asset/model, tạo online service, production server hoặc deploy. Cargo/Gradle
build vẫn resolve dependencies theo manifest/lockfile/wrapper hiện có.

Actions sở hữu checkout/cache/toolchains/system dependencies/Node/npm/browser
provisioning và artifact upload. Repository verification chỉ gọi `cargo xtask ci`.
Không cần toolchain Android/Xcode để chạy root CI. Script entrypoints cũ giữ
compatibility nhưng callers mới phải qua xtask khi có command tương ứng.

| Status | Meaning |
| --- | --- |
| PASS | Stage thực sự chạy và exit 0, với execution report khi cần |
| FAIL | Command, assertion, hash hoặc verification sai; nonzero |
| BLOCKED | Thiếu prerequisite, port, unsupported platform/build implementation; nonzero |
| NOT_RUN | Dry-run, stage sau lỗi, hoặc gate/runtime/device chưa được thực hiện |

Không có optional-skip→PASS. CI/full preflight thiếu browser/WASM prerequisites
thì thất bại. Dry-run không probe doctor, build, start server hoặc open browser;
chỉ changed-only đọc Git/metadata. Artifacts: `target/xtask-evidence` (logs, replay,
hashes, economy CSV), `target/wasm-determinism/report.json`, `target/web-evidence`.
Không tự chuyển synthetic report tests thành benchmark evidence.

## Native feasibility boundary

Android `check` chỉ type-check; `build` link/copy `.so`; `apk` package standalone;
`cmp` package shell phương án B. Mode-specific prerequisites không bắt ADB/JDK
cho một Rust-only check. Doctor kiểm ADB tooling nhưng không coi device có mặt là
runtime PASS. Device listing yêu cầu ADB server loopback có sẵn, không tự start
daemon, boot emulator, install hoặc launch. iOS simulator listing chỉ chạy trên
macOS/Xcode; Linux báo NOT_RUN hoặc BLOCKED nếu chọn iOS tường minh.

Android discovery gửi duy nhất `host:devices-l` qua ADB smart socket loopback,
theo [AOSP host service protocol](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/android15-s1-release/SERVICES.TXT).
Không gọi CLI version negotiation có thể restart shared daemon. Payload bounded
64 KiB và socket timeout hai giây; missing/refused/truncated response không tạo PASS.

iOS chưa có packaging, target CMP hoặc adapter hoàn chỉnh; build iOS luôn BLOCKED.
Stock Bevy/Winit vẫn sở hữu event loop; giới hạn re-entry Android và UIKit
ownership iOS trong [native feasibility](../reports/native-feasibility.md) giữ
nguyên. Không thêm FFI/embedding adapter hoặc suy compile PASS thành device PASS.
[Verification report](../reports/xtask-verification.md) ghi bằng chứng của PR này.
