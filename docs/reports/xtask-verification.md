# Unified xtask verification

**Ngày:** 2026-10-11. **Scope:** tooling/developer workflow; không thay domain,
bridge hoặc native embedding. Base implementation: `9291070` trên `develop`.

Kết quả actual execution của branch `chore/unified-xtask` dưới đây là bằng chứng
theo run, không là khẳng định platform đã đạt production/mobile gate. Reference
VOT đã đọc tại revision `f3fea641`; quyết định/mapping nằm trong
[build contract](../architecture/xtask-build-contract.md).

## Host và nguyên tắc kiểm chứng

Linux x86_64; Rust 1.97.1, wasm-bindgen 0.2.129, Python 3.14.4, Node 24.21.0,
npm 11.19, Chrome qua `/usr/bin/google-chrome`.
Các tools đã có trên host; không install dependencies. Loopback lifecycle/browser
tests cần thực thi ngoài sandbox cấm socket. Build heavyweight chạy tuần tự, tối
đa hai Cargo jobs mặc định.

Host không có system `libudev-dev`; run dùng development files đã có từ trước tại
`/tmp/myva-issue19-deps/root`, liên kết thư viện libudev thật của host. CI provision
`libudev-dev` bằng Actions. Full run local dùng:

```bash
PKG_CONFIG_PATH=/tmp/myva-issue19-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
LIBRARY_PATH=/tmp/myva-issue19-deps/root/usr/lib/x86_64-linux-gnu \
CHROME_PATH=/usr/bin/google-chrome MYVA_WEB_TEST_PORT=8187 \
CARGO_NET_OFFLINE=true cargo xtask ci
```

Android dùng Rust target ARM64, JDK 17.0.20.1, SDK 37.0, build-tools 36.0.0,
NDK 27.0.12077973, Gradle wrapper 9.7.0 và dependency cache hiện có. Gradle chạy
`--no-daemon`; không cài SDK hoặc tự mở ADB/device.

## Kết quả

| Hạng mục / command | Status | Actual evidence |
| --- | --- | --- |
| `cargo xtask ci` local, code `e0204c0` | PASS | Exit 0 trên clean tree; format/docs/native clippy/tests, gameplay/economy/assets/reports, determinism, WASM/bridge/build và browser đều chạy sau test synchronization fix |
| Root Rust tests | PASS | 201 tests, gồm 21 xtask tests; native clippy `-D warnings` |
| `cargo xtask verify xtask` | PASS | 21/21 tests qua ba lần liên tiếp; fmt/clippy cùng PASS |
| Python report tests | PASS | 34 tests; synthetic report contracts, không là benchmark |
| Gameplay / replay | PASS | Normal và Mạch, seeds 1/7/12, 3.600 ticks mỗi run, ECS/core per-tick hashes; cả sáu replay verify |
| Economy | PASS | Seed 7, 90 ngày / 2.160 giờ; invariant checks và CSV |
| Assets | PASS | Generator check 5 PNG, pixel max diff 0; 11 assets / 4 bundles qua manifest/provenance/hash/budgets; texture estimate 18,94 MiB / 32 MiB |
| Native/WASM determinism | PASS | Sáu run × 3.600 ticks, không có tick mismatch; hash cuối ở bảng dưới |
| WASM / web | PASS | WASM clippy và pure sim; Leptos + Bevy bằng profile `web`, wasm-bindgen và output cũ |
| Playwright | PASS | 11/11; skipped/flaky/unexpected đều 0; 30 document lifecycles, keyboard/gamepad/touch emulation, Mạch, stale bridge, asset failures, context loss và native replay |
| Mạch sau test synchronization fix | PASS | `verify web --port 8187 -- tests/web/mach.spec.mjs --repeat-each 3`: 6/6, không skipped/flaky/failing; WASM clippy/build và managed server cùng chạy |
| `cargo xtask build all` | PASS | Native root build rồi hai web bundles, tuần tự |
| `cargo xtask run web --port 8188` | PASS lifecycle | Incremental build, dashboard sau HTTP 200 + token + matching report; Ctrl+C exit 0 và port đóng |
| `preflight --changed-only --dry-run` | PASS discovery / NOT_RUN gates | Git + Cargo metadata thật chọn full root và native gates cho diff này; 39 planned stages đều ghi NOT_RUN |
| Missing changed-only base | FAIL expected | Nonzero trước mọi gate; không fallback sang empty diff |
| Doctor root / Android | PASS prerequisites | Browser, WASM, libudev và Android tooling có sẵn; không suy ra runtime PASS |
| Android `check`, `build`, `apk`, `cmp` | PASS compile/package | Cả bốn lệnh xtask exit 0; Gradle assemble chạy trên cache, hai APK chứa `lib/arm64-v8a/libmyva_native_probe.so` |
| Android devices | BLOCKED | ADB server loopback:5037 không có; không tự start/restart daemon |
| iOS doctor / build | BLOCKED | Linux thiếu macOS/Xcode/targets; packaging implementation chưa có; build kể cả dry-run trả nonzero |
| Mobile runtime/device matrix | NOT_RUN | Không install APK, launch/boot device, hoặc đo FPS/memory trên mobile |
| Desktop graybox GUI / OS browser opener | NOT_RUN | Build được kiểm tra; không claim native GUI interaction hoặc `app web` mở desktop browser |
| Web profiles `dev` / `release` | NOT_RUN execution | CLI/plan được unit test và dry-run; actual web run dùng profile `web` |

CLI, planning, shell-safe argument vectors, root discovery, Git committed/staged/
unstaged/untracked/deleted/renamed paths, reverse dependency closure, missing base,
timeout/cancellation/foreign process ownership, HTTP readiness/token/port conflict,
fresh browser report và read-only ADB socket đều có unit tests. Socket/process
tests thực sự chạy, không skip để tạo PASS.

Các run đầu phát hiện assertion fixture cancellation nhầm zombie với process còn
chạy và race khi test nhả ephemeral port. Fixture đã sửa; production vẫn fail khi
port bận hoặc readiness sai. Full CI ở bảng là run sau cùng đã PASS toàn bộ.
Một manual run đã bị dừng lúc wasm-bindgen còn packaging: xtask trả FAIL cho
build và NOT_RUN cho serve. Run lifecycle ở bảng đợi dashboard ready rồi mới
probe/shutdown, đã PASS.

GitHub run [38132426266](https://github.com/loveoverflowcom/myva/actions/runs/38132426266)
FAIL ở Mạch keyboard, sau khi các gate trước và 10 web tests khác PASS. Trace ghi
`waitForTimeout(100)` thực tế kéo dài khoảng 1,9 giây trên software renderer;
gesture trước đã tạo denial `Some(Mach)` và `presses=1`. Poll của gesture tiếp
theo đọc denial cũ sau khoảng 57 ms, nhả R quá sớm và gây normal cast. Test nay
đòi `presses` tăng so với snapshot trước key-down cùng denial đúng trong cùng
snapshot; không bỏ assertion action/energy/cooldown, không thêm retry và không
đổi gameplay/bridge. Sau fix đã chạy lại full local contract trên `e0204c0`:
201 Rust tests, 34 Python tests và 11/11 Playwright PASS, không skipped/flaky.
Run này là kết luận verification cuối của report; không lấy trạng thái GitHub CI
làm bằng chứng thay cho execution local.

## Determinism và artifacts

Report native/WASM ghi lúc `2026-10-11T10:53:49.054Z`; tất cả `identical=true`,
`firstMismatch=null`, hashes so sánh từng tick, không chỉ hash cuối.

| Seed | Normal final hash | Mạch final hash |
| --- | --- | --- |
| 1 | `eb7937e10ba78ed0` | `f579852957e6041c` |
| 7 | `a3f74b34b5cfdef0` | `f579852957e6041c` |
| 12 | `b1192671baeda01b` | `f579852957e6041c` |

Evidence được tái tạo tại `target/xtask-evidence`,
`target/wasm-determinism/report.json`, `target/web-evidence/test-results.json` và
các capture JSON/PNG của E2E. CI upload ba nhóm artifacts. Full local log ở
`/tmp/myva-xtask-ci-final-head.log`; log Android ở `/tmp/myva-xtask-native-{mode}.log`.
Không commit generated binaries, APK hoặc các log lớn.

Web bundle: 87.392.796 bytes raw, gzip estimate 12.048.369 bytes; đây là estimate
của script, không là network/device benchmark. Final local bundle report ghi
revision `e0204c05cbe98eb9af3d18346315692a002a9330`, `dirty=false`. Commit tiếp theo
chỉ ghi kết quả verification vào tài liệu, không đổi implementation đã chạy.
Browser dùng Chromium SwiftShader/software và desktop touch emulation; không thay
bằng chứng trên mobile hardware.

APK debug được kiểm tra sau Gradle assemble; cached outputs hợp lệ theo Gradle,
không claim clean rebuild hoặc bit-identical reproducibility:

| Artifact | Bytes | SHA-256 |
| --- | --- | --- |
| Standalone `app-debug.apk` | 87.788.760 | `100cc4637612e91a78dd280d0c795724bdbdb5c0bb6388a78fff5d58db06f191` |
| CMP `cmp-shell-debug.apk` | 218.310.441 | `499e14bac3d0dc3704987e1746639e016c1f4cba2a053c9fb4d9cae8d83c4b72` |

## Native feasibility

iOS build/packaging chưa tồn tại, Linux không có macOS/Xcode; gate iOS BLOCKED.
Android compile/package và device/runtime phải ghi riêng. Không tự install APK,
launch device, thay stock Winit ownership hoặc thêm CMP adapter. Feasibility gốc
và ma trận thiết bị còn thiếu ở [native report](native-feasibility.md).
