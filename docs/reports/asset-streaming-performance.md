# D05-B — Asset streaming và performance baseline

**Trạng thái:** runtime và harness đã triển khai qua [PR #25](https://github.com/loveoverflowcom/myva/pull/25); qualification render/performance **BLOCKED**, ngày 2026-10-11. Issue [#13](https://github.com/loveoverflowcom/myva/issues/13) vẫn mở. Base được fetch trên dot cloud: `2c305b6` (`develop`), đã có D05-A PR21, combat/Mạch PR23 và xtask PR24. Không chạy trên Mac; giữ nguyên luật combat/protocol v2/replay v3. Review và merge code/harness không xác nhận các gate render/device bên dưới.

## Review lại PR #25 trước merge

GitHub được đọc lại ngày 2026-10-11: PR không có conversation comments, review submissions hoặc inline review threads. Hai lượt review source độc lập, ưu tiên defect có thể tái hiện, đã kiểm toàn bộ diff với merge-base `2c305b6`, cùng caller và test liên quan: **không còn finding actionable được xác nhận**. Ba regression đã sửa trong head `f60a989` là late optional VFX reset, Low→High/art-off viewport và resize/DPR acquisition epoch; lần review này không thêm thay đổi gameplay hoặc sửa lỗi suy đoán.

Kiểm chứng mới trên source `f60a989`: repository checks (221 Rust tests, 38 Python contracts, format/docs/native clippy, replay/headless/economy/assets), sáu native/WASM 3600-tick determinism fixtures, WASM clippy và optimized web build. Playwright chỉ được syntax/discovery: 22 test trong 6 file; browser bodies và rendered qualification không chạy lại do blocker đã xác nhận. Không kiểm hoặc poll CI theo yêu cầu lần này.

[Evidence review trước merge](evidence/d05b-pr25-review-2026-10-11.json) ghi exact source SHA, command/log hashes và các gate còn thiếu. Việc merge PR vào `develop` chỉ tích hợp runtime/harness; issue #13 vẫn mở và art production vẫn **NO_GO**.

## Runtime hiện hành

- `CatalogLoader` chỉ đọc và kiểm `fixture/manifest.json`, không decode texture. Bốn file `assets/runtime-*.myva` là projection tái lập của catalog, chỉ tham chiếu bản PNG/sheet gốc, không nhân đôi atlas. `export-streaming-manifests.py --check` và unit tests bảo vệ partition/hash/path/budget; web build kiểm rồi copy chúng.
- `StreamRuntime` dùng `StreamingPlanner<Handle<ArtFixture>>`: tối đa 4 bundle, 2 ticket đang chạy/drain, 3 attempt, deadline 10 giây và backoff tuyến tính 250 ms. Character/map là required; boss chỉ required trong Boss; VFX optional không chặn required-ready. Duel không tải/giữ boss bundle.
- Bevy `load_builder().with_settings(...).load()` truyền projection catalog đã kiểm cho loader. Loader kiểm manifest và projection **trước payload I/O**, size/SHA-256 trước parse/decode, PNG header dimensions trước cấp phát, rồi cùng `check_documents`. Tên/path/manifest không được phép nới budget hoặc nhập nội dung ngoài fixture.
- Lỗi required sau retry chặn art presentation; combat vẫn có hình học graybox/hitbox/telegraph/HUD. Lỗi VFX optional dùng đường hình học đã tồn tại cho đạn/feedback; không đổi hit rules. Art tách hoàn toàn khỏi sim/session/replay.
- G hoặc `set_art_enabled(false)` thoát art scene: despawn mọi `ArtEntity` (map/body/projectile/VFX/benchmark atlas clones), clear `ArtTables`, drop planner handles. G/on tải lại; `retry_art()` chủ động thử lại lỗi. Mode switch giữ immutable common bundles và bỏ boss khi vào Duel. Không có hidden global texture cache của client.
- Bevy 0.20 không có public API abort request. Cancel chỉ làm completion cũ stale; handle pending vẫn được giữ tới `Loaded`/`Failed`, slot không được acknowledge sớm. Deadline là hạn ownership, **không bảo đảm transport chết hoặc retry/fallback xong trong 10 giây**. Fetch treo vĩnh viễn có thể giữ slot; capture có wall-clock timeout và báo incomplete/nonzero. Host teardown là giới hạn hiện hành, không giả abort hoặc khẳng định GPU memory đã trả.

## Cảnh render và phép đo

`benchmark.rs` dùng D05-A atlases, animation idle/cosmetic, seed 7 và [cấu hình](../../benchmarks/asset-performance.json). Preset 50/200/500 giữ basis cũ (1 player + actors + monsters), còn rival gameplay được ghi riêng: actual body sprites là 51/201/501 khi đầy đủ; particle/background/HUD đếm riêng. Stress entity không có gameplay/controller/AI/damage/reward hoặc lệnh replay. Fixture stress dùng atlas chia sẻ, grid theo seed, actor scale 0,28, monster 0,16, particle 0,5, alpha body 0,55/particle 0,3 và z = 0,4 phía sau combat signals. Đây là workload nhỏ/translucent có batching, không là 500 nhân vật full-size/unique texture hay capacity MMORPG.

High có 100 cosmetic particle, 4 parallax layers và world scale 1. Low có 50/2 và **world render target RGBA8 thật ở 75% kích thước physical window**, presenter riêng giữ UI camera full resolution. Framing/geometry telegraph/combat feedback/HUD không thay. Đây là đặc tính code và CPU asset-target tests; readability/overdraw/motion trên ảnh thật chưa được xác nhận ở môi trường này.

Snapshot ghi actual ECS atlas-owner counts, frame intervals liên tục sau 5 giây warmup và source `bevy_update`. Rate Update/RAF không được gọi presented FPS. Raw interval array chỉ serialize khi yêu cầu capture, tránh chi phí O(n) mỗi frame của soak. Buffer bounded 120.000 intervals; truncation/incomplete không thể thành capture success.

Native output là raw acquisition, chưa là báo cáo budget qualification. Browser harness đưa raw capture qua `report-asset-performance.py`, có frame p50/p95/p99, source rate, WASM linear-memory allocation và request-to-loaded-with-dependencies latency nếu thực sự chạy. RGBA8 texture base và Low color target chỉ là estimate; buffers/internal targets/driver allocations chưa có instrumentation. RSS game-only không thay native game+CMP RSS. Click-to-art-ready không thay endpoint first-controllable + GPU upload/presentation. Startup network 15 Mbps/RTT 100 ms và observed compressed wire chưa được đo.

## Môi trường dot cloud và blocker

Rust `1.97.1`, Bevy `0.20.0`, wasm-bindgen `0.2.129`; Linux x86_64, CPU nhìn thấy Intel Xeon Platinum 8573C, 9 logical entries, khoảng 9,7 GiB RAM, Chromium `154.0.8037.57`. Toolchain và Debian libudev dependencies chính thức được đặt trong workspace riêng, không đổi máy người dùng hay system setup.

- Shell Chromium launch **BLOCKED**: AF_UNIX/socket cho process singleton trả `Operation not permitted`; retry reviewed không thay kết quả. AF_INET socket được tạo trong command, nhưng dot cloud CUA browser thử URL loopback của owned server trả `net::ERR_CONNECTION_REFUSED`. Không đổi security, không route qua Mac, không dùng file URL hoặc đường bị chặn. Probe server đã dừng.
- Native window runtime **BLOCKED**: shell không có DISPLAY/Wayland, Xvfb hoặc device `/dev/dri`/NVIDIA. Không coi headless CPU tests là native rendered/device PASS.
- Android/iOS thật **BLOCKED** theo gate riêng #10. Không có mobile-runtime/CMP integration hay hardware-GPU kết luận từ nhánh này.
- Repository không có `AGENTS.md` hoặc `.agents/skills` (CONTRIBUTING xác nhận). Review source độc lập theo yêu cầu đã chạy trên diff merge-base và source Bevy pin. Không có repo-specific skill để chạy; ba lỗi được sửa và review lại trước publication.

## Lệnh tái lập

Tất cả từ root repository, với prerequisites trong `cargo xtask doctor`. Commands dưới đây tồn tại; browser/native render cần môi trường thực sự hỗ trợ.

```sh
cargo xtask verify assets
cargo test --locked -p myva-graybox --lib
cargo xtask verify gameplay
cargo xtask verify determinism --ticks 3600 --seeds 1,7,12
cargo xtask preflight --changed-only --base origin/develop
cargo xtask build web --profile web
CHROME_PATH=/path/to/chromium cargo xtask verify web --port 8081 -- tests/web/assets.spec.mjs tests/web/runtime-performance.spec.mjs
MYVA_RUN_SOAK=1 CHROME_PATH=/path/to/chromium cargo xtask verify web --port 8081 -- tests/web/runtime-performance.spec.mjs --grep '600-second'
cargo xtask run graybox -- --demo --benchmark 50 --benchmark-tier High --benchmark-seed 7 --benchmark-seconds 60 --benchmark-output target/bench-50-High.json
# Lặp 50/200/500 × High/Low; --benchmark-output không chấp nhận --duel/--no-art/--screenshot.
```

Browser suite chuẩn có sáu 60-second presets, paused-state parity, lifecycle 30 cycle, delayed cancellation và fault tests. Soak 600 giây chỉ đăng ký khi `MYVA_RUN_SOAK=1`; không tạo skipped test để làm aggregate checker PASS giả. Suite thông thường không chứng minh soak. Chụp ảnh harness tạo chỉ được gọi visual-reviewed sau khi người kiểm đã mở và inspect thực tế.

## Gate nhánh hiện hành

[Verification ledger](evidence/d05b-verification-2026-10-11.json) tách aggregate trước review và kiểm chứng source cuối sau review. Aggregate trước đó exit 1 ở browser launch, **BLOCKED, không PASS**: 22 case discovery nhưng body chưa chạy. Sau ba sửa lỗi, `cargo xtask verify repository` PASS: 221 Rust tests (55 graybox unit + 5 parity), 38 Python contracts, native clippy, assets và replay/headless checks. WASM clippy, sáu 3600-tick native/WASM determinism fixtures, optimized web build, Node syntax/discovery và ba invalid-CLI guards đã chạy lại và PASS. Không retry browser bị chặn sau review; full preflight/render qualification vẫn BLOCKED.

Review đã sửa: optional VFX đến muộn làm mất sprite đang hiện; Low→High/Off giữ computed viewport nhỏ của Bevy; resize/DPR trộn sample của hai raster workload. Ba regression tests chạy PASS, gồm CPU `camera_system` thật và preservation entity/image/layout/visibility. Review lại không còn lỗi actionable được xác nhận. Không thay gate screenshot/GPU bằng review source.

[Headless AssetServer evidence](evidence/d05b-headless-assets-2026-10-11.json): 30 cycle, 120 accepted-load sample; min 19 / p50 100 / p95 266 / p99 433 / max 484 ms (nearest rank, mean 99.908 ms). Full `ArtPlugin` CPU systems materialize map, hai fighter, projectile và hit VFX mỗi cycle; fixed ArenaView và injected hit event chỉ dùng kiểm ownership, không là gameplay-spawn coverage. Đây là debug CPU filesystem/hash/PNG, polling 5 ms, concurrent test suite và cache không kiểm soát; không phải cold network/GPU upload/render baseline. Sau mỗi exit, fixture/image/atlas assets, planner/table/sprite owners và flights đều 0. Không suy ra driver đã trả VRAM.

[Web bundle evidence](evidence/d05b-web-bundle-2026-10-11.json): game WASM 87.238.855 byte, gzip estimate 11.650.506 byte; shell WASM 441.660 / 75.359 byte; tổng file 88.342.886 / gzip estimate 12.194.711 byte. Game SHA-256 `8ff90d6d058212e7a1f0e0832bf40748c79bb65d343e5f9686df6d54f7809c0e`. Build profile `web`, clean local source checkpoint `1152f4e` từ base `2c305b6`; ledger ghi source fingerprint để đối chiếu code được publication. GitHub Git Data API có thể tạo canonical commit khác do metadata; không giả đây là embedded binary attestation của final PR commit. Gzip là phép nén file tái lập, không phải wire/RAM/startup đã đo.

| Gate | Trạng thái hiện hành |
| --- | --- |
| Projection export/contracts | PASS: 4 unit tests và exact export check |
| xtask tooling | PASS: clippy + 21 tests, gồm owned server readiness/shutdown; không xác nhận browser chạy |
| Report contract | PASS: 34 synthetic tests; không là phép đo |
| Pinned WASM typecheck/build | PASS final library/WASM clippy + optimized web/bindgen build |
| Logic/replay | PASS: baseline sim/gameplay + sáu 3600-tick ECS/core/replay fixtures; 360-tick final stress/tier test giữ hash/replay identical |
| Real AssetServer 30-cycle/fault/scene-switch CPU tests | PASS: 55 unit + 5 parity graybox tests, gồm 30 actual file-I/O/PNG/store và body/projectile/VFX owner cycle; 221 native workspace tests tổng; không GPU/window |
| Six rendered presets FPS/frame quantiles/RAM | BLOCKED: browser/window runtime không truy cập được |
| Render-world/driver GPU lifecycle, missing assets render fallback | BLOCKED; CPU store ownership được kiểm riêng |
| WASM file sizes | MEASURED: pinned final build file sizes/SHA; gzip ESTIMATED, không observed wire |
| CPU/GPU isolated work, presented FPS, network startup, compressed P1 wire | NOT_RUN/BLOCKED; không có instrumentation/evidence đầy đủ |
| Screenshot đã mở/inspect, readability/telegraph/touch | BLOCKED ở runtime mới |
| 600-second render soak | BLOCKED; không thay bằng 60s hoặc unit tests |
| Android/iOS/CMP gate | BLOCKED; không phải scope completion |

**Art production vẫn NO_GO.** Cần rendered browser/native/device capture và visual review, budget actual và lifecycle evidence trên GPU trước khi mở rộng art. Code/tests/harness không tự chứng minh 60/30 FPS, 350 MiB game+CMP RSS hay 128 MiB driver allocation.

## Snapshot chuẩn bị trước runtime (lịch sử, đã superseded)

Phần sau giữ kết quả chuẩn bị ở base `33c77e1`; các câu “chưa có D05-A/backend/benchmark” và checkpoint Mac mô tả thời điểm cũ, không mô tả nhánh dot cloud hiện hành.

## Kết quả hiện tại

PR [#15](https://github.com/loveoverflowcom/myva/pull/15) và [#16](https://github.com/loveoverflowcom/myva/pull/16) đã merge thông thường, lần lượt SHA `50772e0f76f0f41fd2d3b61e39c40ad0ddd305d7` và `33c77e1e7e507c8a76723c14fa9731f4b5b45875`. Cả hai head có check `rust` PASS trước merge; merge không xác nhận mobile/playtest/performance. Issues 10/11/12/13 giữ mở. Không force/reset/rewrite/deploy hoặc thêm CI.

Phần độc lập D05-B gồm planner quản lý bundle/request/ownership và hợp đồng cấu hình/báo cáo capture. Chúng chưa nối `AssetServer`, chưa preload hoặc vẽ asset vào client. Planner test không là phép đo load/unload Bevy; report unit test không là benchmark. Cảnh benchmark chưa tồn tại, vì chỉ có hình khối combat và chưa có fixture/atlas/validator D05-A. Chưa tạo hay inspect screenshot D05-B.

Đã mở và inspect **ảnh lịch sử PR16**, không phải run D05-B: [native](evidence/combat-native.png) có HUD và vùng báo vàng trước boss ở tick 950; silhouette vẫn là hình khối, không có atlas/texture/parallax để đánh giá. [Touch landscape emulation](evidence/combat-touch-landscape.png) có joystick/nút ở foreground và HUD bars trên màn 844×390; ảnh không chứng minh touch/safe-area của điện thoại thật. Các tín hiệu ấy phải giữ khi giảm tier; chưa kiểm readability khi thêm 100 particle/50 body.

## Baseline lịch sử đã đối chiếu

[Evidence combat](evidence/combat-2026-10-11.json) ở PR16 ghi run SHA `82524ccfeeaa8f647cc6b34facd8a81ec4635945`, `dirty=false`, profile `web`, Rust `1.97.1`, Bevy `0.20.0`, wasm-bindgen `0.2.129`, Leptos `0.8.22`. Đọc JSON và [script size](../../scripts/web-bundle-report.py), không build lại.

| Chỉ số | Giá trị lịch sử | Cách hiểu |
| --- | --- | --- |
| Game WASM sau wasm-bindgen | 84.842.609 byte | Raw file, không phải RAM hoặc wire payload |
| Gzip ước tính game WASM | 11.182.853 byte | `gzip.compress(data, mtime=0)`; không phải transfer đo được |
| Tổng file sinh ra | 85.453.825 byte | Bao gồm file ngoài payload thực tế |
| Tổng gzip ước tính | 11.291.227 byte | Không đủ kết luận P1/cold start đạt budget |

WASM SHA256: `d41f5183ec23bf439d1e1d238d01f641ec7dbb6bfc2e8eee5de18b5db14de7d3`. Head PR16 `6687677bb27c7bc9b1084d33a049f2f885859ea6` hơn run một commit docs/evidence. Không có before/after D05-B build comparison. Không so graybox combat với spike D03 để gọi là tối ưu cùng cảnh.

## Phương pháp và điều kiện cho run tiếp theo

Cấu hình [JSON](../../benchmarks/asset-performance.json): 1 player gameplay, 19 actor render-only, 30 monster render-only, 100 particle cosmetic, 4 parallax layers và HUD. Stress 50/200/500 đếm player + actor + monster; particle/background/HUD ghi riêng. Stress 50 trùng baseline, 200 là 1+79+120, 500 là 1+199+300. Ghi seed, tier, resolution, DPR và scale. Render-only không tạo fighter/AI/damage hoặc reward mới. Low chỉ giảm cosmetic particle/parallax/scale; telegraph, vùng nguy hiểm và combat feedback giữ nguyên.

Ghi commit, dirty flag, profile, compiler/dependency versions, seed/config, OS/device/CPU/RAM, renderer string/class/backend, browser và network/cache method. Tách browser hardware GPU, SwiftShader/software, native desktop, Android thật, iOS thật. RAF/Bevy Update không được gọi là presented FPS. Input latency phải ghi DOM-to-observed-state hay physical input-to-photon; không trộn.

RAM encoded bytes, CPU decode/staging và GPU upload/texture allocation là ba lớp riêng. RGBA8 texture ước tính `width*height*4`, mipmaps ghi riêng; không suy từ PNG/gzip. Strong handles ở scene owner, sprite/material và render-world có thể giữ asset sống. Loaded-with-dependencies chưa là upload hoàn tất; despawn/drop owner không chứng minh driver trả physical GPU memory. Capture từng lớp nếu tool hỗ trợ; thiếu metric ghi NOT_RUN/BLOCKED.

Cold/warm phải định nghĩa browser cache, HTTP compression, fetch/decode/upload/first controllable frame. So build features/profile/LTO/wasm-opt trên cùng revision/cảnh và kiểm correctness/parity. `wasm-opt` cần phiên bản hỗ trợ instruction set build; không chỉ báo tool exit 0. Feature hiện tại đã tắt defaults; web có UI/text/default font/gamepad, PNG hiện chỉ native. Chưa chọn cách loại bỏ feature có tác động correctness.

### Audit tĩnh build và lazy loading

Đọc manifest, imports và build/serve scripts ở base SHA trên; chưa chạy build hoặc profiler.

| Phần | Hiện trạng kiểm tra từ code | Thí nghiệm tiếp theo và giới hạn |
| --- | --- | --- |
| Features/dependencies | Bevy defaults tắt; sprite/render/UI/text, font, keyboard/touch/gamepad được dùng. Imports shell/game có consumer cho các direct dependency; chưa thấy dependency thừa rõ ràng | `cargo tree --locked -e features --target wasm32-unknown-unknown`; kiểm closure sau D05-A. Không bỏ HUD/gamepad/telegraph để giảm size |
| Profile | `web` kế thừa release, `opt-level="s"`, thin LTO, 16 codegen units, strip debuginfo | So cùng fixture với `z`, fat LTO và 1 codegen unit từng biến một; raw/gzip, compile/load/frame time và replay. Không giả `z` nhỏ hơn `s` |
| Postprocess | `wasm-bindgen 0.2.129`; chưa gọi `wasm-opt` | Pin Binaryen hỗ trợ instruction set của Rust build, validate module rồi headless/parity/web QA trước so size; chưa chọn version |
| Transfer | Script dùng `gzip.compress(..., mtime=0)`; local server `no-cache`, chưa gửi Content-Encoding gzip | Ghi raw, gzip estimate và observed wire riêng; cold/warm có cache/network method. Không lấy gzip estimate làm P1 đạt budget |
| Lazy runtime | Shell mount iframe game sau thao tác vào trận; game WASM init trong document đó | Runtime đã tách entry shell/game. Asset bundle lazy loading vẫn thiếu; bỏ iframe không chứng minh GPU release |

Các candidate trên chưa được áp dụng. Không có số giảm size hoặc performance cải thiện. [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) mô tả tradeoff `s`/`z`, LTO và codegen units; [Binaryen](https://github.com/WebAssembly/binaryen/blob/main/README.md) là nguồn cho tool postprocess, không chứng minh compatibility với build MyVa chưa chạy.

## Budget và gate D05-B

| Hạng mục | Mục tiêu từ tài liệu | Actual D05-B |
| --- | --- | --- |
| High | 60 FPS, frame p95 ≤16,67 ms, p99 ≤25 ms | NOT_RUN |
| Low | 30 FPS, frame p95 ≤33,33 ms | NOT_RUN |
| WASM heap | ≤192 MiB | NOT_RUN |
| Native RSS | ≤350 MiB, bao gồm shell theo platform | NOT_RUN |
| GPU texture/buffer estimate | ≤128 MiB | NOT_RUN |
| P1 compressed transfer | ≤25 MiB runtime + initial region | NOT_RUN |
| Warm start / cold p95 | ≤3 s / ≤20 s, 15 Mbps RTT 100 ms | NOT_RUN |
| CPU/GPU time, live bundle/handles, load/unload | Tool/method và latency rõ | NOT_RUN |
| Headless rules, replay/gameplay parity | Hash/replay không đổi | NOT_RUN cho nhánh D05-B |
| Missing/corrupt texture, cancel/rapid scene changes | Retry/fallback hữu hạn, stale request không quay lại | NOT_RUN runtime |
| 30 load/unload, soak khoảng 10 phút | Handle/bundle/RAM/GPU capture không tăng không giải thích | NOT_RUN |
| Screenshot/visual regression | Môi trường xác định, ảnh mở và inspect | NOT_RUN |
| Android/iOS thật | Platform gate và device evidence | BLOCKED |
| Repo skill review | Engineering/Bevy/performance/game QA/review | BLOCKED: AGENTS/.agents/skills vắng mặt |

**Art production: NO_GO.** High 60 / Low 30 FPS vẫn là mục tiêu, chưa đạt. Chưa xác định bottleneck CPU, GPU, decode, upload hoặc network bằng capture D05-B; kích thước WASM lịch sử chỉ là baseline file size. Không có mobile hoặc hardware GPU conclusion từ các phép đo software lịch sử. Các giới hạn D03/D04/combat giữ nguyên trong báo cáo tương ứng.

## Kiểm chứng phần chuẩn bị độc lập

**PASS:** 11 unit test planner; 34 test report contract bằng Python standard library; standalone Clippy với `-D warnings` và rustfmt check cho module mới. Review phát hiện đường nhập FPS/percentile giả, stale field trong NOT_RUN, P1 không có wire evidence, startup thiếu cache/span và RSS thiếu process scope; đã sửa và rerun test liên quan. [Validation summary](evidence/d05b-preparation-validation-2026-10-11.json) ghi command, compiler và SHA256 file được kiểm tra.

Rust check trên Mac dùng toolchain đã có `1.96` (`rustc 1.96.1`) cho **module std-only**, không build Bevy/workspace pin `1.97.1`. 30 cycle trong unit test chỉ chứng minh owner giả được drop; không xác nhận 30 asset load/unload hoặc GPU release. Synthetic capture chỉ dùng trong unit tests, không lưu thành benchmark evidence.

Lệnh local nhẹ, từ root repo:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_asset_performance_report.py' -v
python3 scripts/check-docs.py
git diff --check
rustup run 1.96 rustfmt --edition 2024 --check crates/graybox/src/streaming.rs
rustup run 1.96 clippy-driver --edition 2024 --test crates/graybox/src/streaming.rs -o /tmp/myva-d05b-streaming-tests -D warnings
/tmp/myva-d05b-streaming-tests
```

[Report tool](../../scripts/report-asset-performance.py) nhận capture thực có metadata/config/method và xuất JSON/Markdown; không tự capture hoặc chạy benchmark:

```sh
python3 scripts/report-asset-performance.py --help
python3 scripts/report-asset-performance.py capture.json --output-json report.json --output-markdown report.md
```

Full pinned commands **NOT_RUN trên Mac**, chờ window/disk/toolchain: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `./scripts/check-wasm-determinism.sh 3600 1 7 12`, `PROFILE=web CARGO_BUILD_JOBS=1 ./scripts/build-web-shell.sh`, `npm run test:web`. CI hiện có chạy trên draft PR; xem status theo exact head SHA, không suy green từ run của base. Actual asset validator/benchmark/fault/soak commands chưa tồn tại, không đưa lệnh giả cho gate ấy.

## Resource checkpoint trên Mac

Ollama không có trong PATH, apps hoặc vị trí cache/model/launch-agent chuẩn. Không có gì để uninstall; không xóa custom data hoặc empty Trash.

Chỉ xóa compiler cache inactive/untracked của Cantos: `target/debug/{.fingerprint,build,deps,examples,incremental}` và `target/wasm32-unknown-unknown/debug`. Kiểm `ps`/`lsof` trước; không kill process. Không dùng `cargo clean` toàn target. Giữ `revision-evidence` khoảng 8,58 GiB, `read-benchmarks` khoảng 3,08 GiB, PostgreSQL run data, mọi evidence/root log và local source changes. Metadata/inode/mtime của entries giữ lại cùng SHA256 các root file nhỏ khớp trước/sau; git status Cantos không đổi. Không di chuyển evidence.

Free disk từ 1.502.511.104 lên 6.462.877.696 byte lúc cleanup (khoảng 1,40 → 6,02 GiB); sau clone khoảng 6 GiB. macOS 15.6 (24G84); RAM 24 GiB do user cung cấp, chưa xác minh hardware read. Rust 1.97.1 chưa cài; wasm-bindgen/wasm-opt không ở PATH. Task D05-B này chưa download toolchain/model hoặc chạy heavy build/benchmark/soak. Parent yêu cầu giữ MyVa ở công việc nhẹ để cấp cửa sổ cài runtime/model và kiểm chứng riêng cho Cantos. Snapshot tiến trình sau cleanup không thấy build/download liên quan; đây là checkpoint, không phải theo dõi liên tục. Cần parent cấp window mới trước heavy hoặc measured run.

## API pin và nguồn

Đã đọc manifest/lockfile và source chính thức Bevy `v0.20.0`; exact-version docs.rs không truy cập được qua tool lúc audit. [AssetServer](https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_asset/src/server/mod.rs), [tracking/retry](https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_asset/src/server/info.rs), [Handle](https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_asset/src/handle.rs), [render asset cleanup](https://github.com/bevyengine/bevy/blob/v0.20.0/crates/bevy_render/src/render_asset.rs), [feature definitions](https://github.com/bevyengine/bevy/blob/v0.20.0/Cargo.toml). Runtime compatibility của nhánh chưa build.

Resume: approve/land bounded D05-A, cấp disk-safe exclusive window, nối planner vào scene AssetServer thực, dựng benchmark đúng config, chạy validation/headless/parity/screenshot/fault/cycles/soak và inspect ảnh. Báo PASS/FAIL/NOT_RUN cho từng môi trường; push bình thường, giữ PR D05-B draft và không merge.
