# D03 — Bevy trong Leptos web shell

Ngày thực hiện: **2026-10-09**. Issue [#11](https://github.com/loveoverflowcom/myva/issues/11), quyết định [ADR 0003](../decisions/0003-bevy-engine-adoption.md). Đây là prototype tích hợp; cảnh thu thập linh lực chỉ kiểm tra render/input, không thay luật combat hoặc kinh tế. **Từ 2026-10-10**, [#2](https://github.com/loveoverflowcom/myva/issues/2) thay cảnh này bằng trận graybox và nâng bridge lên v2; số đo dưới đây vẫn thuộc revision của D03. Bằng chứng mới nằm trong [báo cáo combat](combat-graybox.md).

## Cách tái lập

```bash
cargo install wasm-bindgen-cli --version 0.2.129 --locked
./scripts/build-web-shell.sh
python3 scripts/serve-web-shell.py
# Mở http://127.0.0.1:8080/
```

Build mặc định dùng profile `web` (`opt-level = "s"`, thin LTO); `PROFILE=dev` phục vụ sửa lỗi nhanh. Rust `1.97.1`, Bevy `=0.20.0`, Leptos `=0.8.22` CSR, wasm-bindgen `=0.2.129` cùng CLI; dependency đầy đủ được khóa ở `Cargo.lock`. `target/web-shell/bundle-report.json` ghi revision, dirty flag, profile, compiler, bytes/gzip ước tính và SHA256 từng file. Đây không phải số byte truyền thực tế qua mạng.

```bash
npm ci
# Dùng Chrome đã có trên máy:
CHROME_PATH=/usr/bin/google-chrome npm run test:web
# Hoặc cài Chromium của Playwright rồi bỏ CHROME_PATH:
npx playwright install chromium
npm run test:web
```

Harness tự start server và chờ HTTP `bundle-report.json` nếu chưa có server. Báo cáo JSON, trace khi fail và screenshot nằm ở `target/web-evidence/`; CI upload thư mục này ngay cả khi test fail. Server bind localhost mặc định; dùng HTTPS hoặc port forwarding tới localhost khi thử điện thoại. Không dùng HTTP LAN để suy ra secure-context API hoạt động.

## Ownership và giới hạn runtime

- Leptos sở hữu tên người chơi local, navigation, loading/error, appbar/HUD và ô nhập thử tiếng Việt; chưa có authentication/server. Build CSR không chạy SSR/hydration.
- Game là iframe **cùng origin** `/game/`; mỗi lần vào tạo document/session mới, mỗi lần ra xóa document và listener trên shell. Đây là boundary hủy Winit/WASM/GPU, không gọi lại `App::run()` trong cùng document. Đổi lại có chi phí compile/khởi tạo mỗi lần vào và giới hạn focus/fullscreen của iframe. Không có WebView mobile.
- Envelope `{v: 1, session, type}` hỗ trợ `init/ready/command/event/pause/exit/error`; cả hai phía kiểm origin, source window, version và session. `command` hiện chỉ có `reset`; mỗi 500 ms gửi score/pause, không serialize ECS world. Session phân biệt lần mount, không phải credential hoặc chứng minh quyền game/server.
- JS adapter chỉ nhận phím khi canvas có focus; Tab và shortcut vẫn dành cho browser. Pointer có capture, active pointer ID và cancel; blur/hidden/pause xóa input cũ. Leptos nhận text/IME trong DOM. `set_input` clamp/normalize/loại NaN; WASM giữ state scene và Bevy cập nhật sprite.
- Renderer duy nhất của spike là **WebGL2** với feature tối thiểu trong `crates/web-game/Cargo.toml`; không tự fallback sang WebGPU. Kiểm khả năng trước tải game, hiện lỗi khi không hỗ trợ. WebGPU/support matrix hardware chưa được đo.
- `fit_canvas_to_parent` theo kích thước/DPR; context loss giữ lỗi và yêu cầu rời/vào lại. Pause dừng scene input/movement, không hứa GPU render loop đã ngủ. Thoát document mới là teardown toàn runtime.
- Server phục vụ WASM bằng `application/wasm`, JS đúng MIME; CSP cho cùng origin và `wasm-unsafe-eval`, không bật JS `unsafe-eval`. `style-src 'unsafe-inline'` cần cho style động của window/canvas. Filename ổn định dùng `no-cache` để revalidate, không cache bất biến khi chưa có content hash deployment. Không có service worker hoặc network game background.

## Đọc số đo đúng phạm vi

`startupMs` đo từ lúc module runtime bắt đầu tới lần telemetry đầu có `ready`; nó gồm fetch/init game nhưng không phải navigation-to-first-presented-frame. `ready` là một Bevy Update hoàn tất; screenshot mới xác nhận sprite thực sự xuất hiện. `updateHz` là tốc độ Bevy Update, không phải số frame GPU trình chiếu. Input sample là thời gian DOM input tới lần quan sát vị trí đổi trên RAF; không đo hardware input-to-photon. CDP heap/DOM/listener chỉ thấy phần browser tương ứng, không chứng minh hết leak WASM/driver/VRAM.

## Trạng thái gate

**Chức năng desktop đã kiểm chứng; gate #11 chưa hoàn thành.** Run cuối trên nguồn `72a755b97d2634da5bb793a2aa6af52fa37757f3`: **3 E2E tests PASS, 0 skipped, 0 flaky**, thời gian 133,5 giây. Chrome `153.0.8010.36`, Linux `7.0.0-38-generic`, Intel Core Ultra 7 155H (22 logical CPUs), RAM 22 GiB; harness ép WebGL2 qua **SwiftShader software renderer**. Không suy ra kết quả hardware/mobile từ run này.

| Kiểm tra | Kết quả / phạm vi |
| --- | --- |
| Cargo build profile `web`; Clippy host + WASM; fmt | PASS |
| Toàn workspace Rust | 59 tests PASS; core/replay và economy giữ nguyên invariant |
| Bevy sprite + Leptos loading/HUD/input/pause/reset | PASS trong Chrome; ảnh đã kiểm tra trực quan |
| 30 lần vào/rời game document | PASS; 30/30 mount có ready và tháo iframe; không có page/console error trong run bình thường |
| Focus, giữ phím rồi blur, Tab/Shift+Tab, text tiếng Việt | PASS; text được điền tự động, chưa xác minh IME composition từ bàn phím OS |
| Pointer touch/cancel, resize và DPR 2 | PASS trên Chromium touch emulation; không phải điện thoại thật |
| Fullscreen | PASS vào/ra fullscreen trong Chrome |
| Visibility | PASS wiring bằng `document.hidden`/event tổng hợp và ưu tiên manual pause; background tab/OS thật **NOT_RUN** |
| Context loss, WebGL2 unavailable, WASM fetch failure | PASS fault injection, shell hiện lỗi; context loss cần rời/vào lại |
| Presentation FPS, frame pacing/VRAM, audio gesture | **NOT_RUN**; số đo Update bên dưới không đạt cơ sở để xác nhận budget render |
| Browser mobile thật | **BLOCKED**: không có thiết bị kết nối; Android/iOS native không được suy ra từ web |

Chi tiết có trong [evidence JSON](evidence/web-2026-10-09.json), [ảnh desktop](evidence/web-desktop.png) và [ảnh touch emulation](evidence/web-touch-emulation.png). Evidence gắn hash WASM, compiler/profile và revision. Cờ `dirty=true` khi build do báo cáo web này chưa commit; mã ứng dụng và harness đã ở đúng revision nêu trên.

## Số đo của run cuối

| Chỉ số | Kết quả |
| --- | --- |
| Bevy game WASM sau wasm-bindgen | 63.294.408 byte; gzip ước tính 8.251.795 byte |
| Leptos shell WASM | 384.356 byte; gzip ước tính 71.041 byte |
| Tổng file sinh ra (gồm cả `.d.ts`) | 63.839.976 byte; gzip ước tính 8.352.997 byte |
| Resource transfer lần vào đầu | 63.827.896 byte theo Resource Timing; navigation HTML ghi riêng trong JSON; server local không gzip |
| Runtime startup lần đầu | 1.841 ms, từ runtime module tới telemetry ready |
| Runtime startup 29 lần tiếp | Median 1.123 ms, p95 mẫu 1.199,7 ms (nearest rank); localhost, không phải phép đo mạng 15 Mbps/RTT 100 ms |
| Sau GC ở vòng 1 → 30 | Documents 2 → 2; nodes 177 → 177; listeners 37 → 37 |
| JS heap sau GC ở vòng 1 → 30 | 2.665.300 → 2.920.572 byte; không gồm chứng nhận WASM/VRAM đã giải phóng hết |
| TaskDuration sau khi thoát | Tăng 7,7 ms trong 1,52 giây quan sát; shell status không bị runtime cũ cập nhật |
| Bevy Update, mẫu unpaused sau sample đầu mỗi vòng | Median **4,38 Hz**, 50 mẫu; đây là kết quả thấp trên môi trường đo, không được gọi là 60 FPS |
| DOM keyboard input → quan sát vị trí đổi | **412,9 ms**, chỉ một mẫu; không đủ tính p95 hoặc input-to-photon |

Run này chưa chứng minh budget render/input. Cần profile trên renderer/device mục tiêu và đo frame presentation trước khi mở rộng gameplay. Không nâng budget hoặc dùng số gzip ước tính để báo cold start mobile đã đạt. Ngân sách production trong [assets-performance](../technical/assets-performance.md) vẫn là gate riêng.

Để phân biệt kết quả headless với desktop GPU, một phép đối chiếu riêng trên cùng build đo 5 giây sau 2 giây warm-up:

| Cấu hình Chrome | Renderer thực tế | RAF / Bevy Update | Gọi `set_input` → thấy state đổi |
| --- | --- | --- | --- |
| Headless ép software | ANGLE SwiftShader | 4,57 / 4,57 Hz | 252,9 ms |
| Headless default | ANGLE SwiftShader | 4,40 / 4,40 Hz | 243,9 ms |
| Cửa sổ desktop, default | ANGLE Intel Mesa Graphics MTL, OpenGL 4.6 | 60,20 / 60,00 Hz | 16,9 ms |

[Dữ liệu đối chiếu renderer](evidence/web-renderer-diagnostics.json) ghi renderer string, canvas, thời gian và counter. Cửa sổ desktop có thể nhận input thực trong lúc đo; đây là mẫu scene đang hoạt động, không phải controlled idle benchmark. Số latency dùng API `set_input`, không phải phím vật lý và không so trực tiếp với mẫu keyboard 412,9 ms ở E2E. RAF/Update không là presentation FPS, 5 giây không thay phép soak/thermal 10 phút. Sự khác biệt cho thấy cần tách môi trường headless/software và hardware khi đọc số đo; chưa xác nhận đầy đủ budget production.

Tái lập phép đối chiếu khi server đang chạy: `CHROME_PATH=/usr/bin/google-chrome node scripts/measure-web.mjs`; chạy nơi có display để đo cấu hình cửa sổ desktop. Bỏ qua cấu hình headed khi host không có display, ghi rõ trong output.

DPR cần phân biệt emulation với device: chỉ đặt context `deviceScaleFactor=2` trong Chrome này làm `devicePixelRatio=2` nhưng `ResizeObserver.devicePixelContentBoxSize` vẫn ở scale 1, kể cả với một div độc lập. Harness touch dùng thêm `--force-device-scale-factor=2`; canvas CSS `357×403` có backing `714×806`, div CSS `100×100` có device box `200×200`. [Bốn cấu hình đối chiếu](evidence/web-dpr-diagnostics.json) được đo riêng; không sửa engine để né phép kiểm.

**Không đóng #11** trước khi có browser mobile thật, presentation/frame pacing/GPU và các ca còn thiếu. Chưa có audio trong scene: audio gesture/interruption là `NOT_RUN`, không có audio pipeline để xác nhận. SSR/hydration, WebGPU, network throttling và cache production chưa được kiểm chứng.

## Nguồn API

- [Bevy 0.20 Window](https://docs.rs/bevy/0.20.0/bevy/window/struct.Window.html) — canvas, resize, browser default input.
- [Bevy 0.20 feature flags](https://docs.rs/crate/bevy/0.20.0/features) — renderer/platform được chọn trong manifest.
- [Leptos lifecycle](https://book.leptos.dev/ssr/22_life_cycle.html) — lý do cần phân biệt CSR với SSR/hydration.
