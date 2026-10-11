# D05-B — Asset streaming và performance baseline

**Trạng thái:** chuẩn bị độc lập; runtime/benchmark **BLOCKED**, ngày 2026-10-11. Issue [#13](https://github.com/loveoverflowcom/myva/issues/13). Base `develop` đã xác minh: `33c77e1e7e507c8a76723c14fa9731f4b5b45875`. Không đóng #13. D05-A vẫn chưa tồn tại; [work item](../work-plan/015-bevy-asset-baseline.md) giữ ranh giới predecessor. **Cập nhật 2026-10-11:** D05-A đã có trên nhánh `feat/bevy-asset-pipeline` ([báo cáo](asset-pipeline.md)); nội dung dưới đây giữ nguyên ở thời điểm chuẩn bị D05-B.

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
