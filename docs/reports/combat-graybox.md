# #2 — Graybox chiến đấu Bevy: trận đầu tiên chơi được

Ngày thực hiện: **2026-10-10**. Issue [#2](https://github.com/loveoverflowcom/myva/issues/2), work-plan [020](../work-plan/020-combat-graybox.md), quyết định [ADR 0003](../decisions/0003-bevy-engine-adoption.md). Mã ứng dụng và harness ở revision `ead6d36884d32589d9590ab66bcb264b09e55762`; bản web build profile `web`, `dirty=false`.

Báo cáo này xác nhận một trận Long Lưu đấu Kẻ Giữ Đập **chơi được và kiểm chứng được bằng replay** trên trình duyệt và native desktop. Nó **không** thay playtest người thật, không đo thiết bị mobile thật và không cân bằng lại thông số: mọi luật dùng nguyên lõi `myva-sim` hiện có.

## Cách tái lập

```bash
cargo test --workspace
cargo run -p myva-graybox -- --screenshot combat-native.png   # Linux cần libudev-dev
./scripts/build-web-shell.sh && python3 scripts/serve-web-shell.py
CHROME_PATH=/usr/bin/google-chrome npm run test:web
CHROME_PATH=/usr/bin/google-chrome node scripts/measure-web.mjs   # khi server đang chạy
```

Harness ghi JSON, ảnh và replay vào `target/web-evidence/`; CI upload thư mục này. Bản tóm tắt của run trong báo cáo nằm ở [evidence JSON](evidence/combat-2026-10-10.json).

## Ranh giới và luồng dữ liệu

- **Lõi** (`myva_sim::battle::Battle`): ghép `World`, bộ não boss, bot B0/B1, `Recorder` và kết quả trận. Phiên tự đánh số `seq` cho mọi bên, nên đổi người lái giữa trận không làm input bị từ chối. Khi có người bị hạ, thế giới chạy thêm 60 tick rồi dừng; replay luôn có điểm kết thúc. Thuần Rust, build được cho wasm32.
- **Client Bevy** (`myva-graybox`): input thiết bị được gom trong `RunFixedMainLoop` trước vòng fixed; `Battle::step` chạy trong `FixedUpdate` 60 Hz; cảnh, HUD và nút cảm ứng đọc trạng thái trong `Update`. Nút nhấn được dồn tới tick kế tiếp, nên FPS khác tick rate không làm mất hay nhân đôi một lần nhấn. `FighterId` là ID miền, `Entity` chỉ để vẽ. Adapter này **tạm thời** nằm trong graybox vì foundation [#12](https://github.com/loveoverflowcom/myva/issues/12) chưa có.
- **Web**: `myva-web-game` dựng cửa sổ canvas và bridge; shell Leptos sở hữu DOM, chữ tiếng Việt và nút điều hướng. Envelope lên **v2**: `event` mang tóm tắt trận (HP, Mạch, pha, kết quả, trạng thái kiểm chứng), thêm lệnh `rematch/mode/hitboxes/autopilot/touch/replay` và message `replay`. Hai phía vẫn kiểm origin, source window, version và session; shell kiểm kiểu từng trường trước khi hiển thị.
- **Không authoritative**: client, bridge và replay local không mint reward hay quyết định kết quả online. Kiểm chứng ở đây là chạy lại trên cùng build.

## Điều khiển

Mapping mặc định theo [combat.md §12](../design/combat.md#12-điều-khiển-và-khả-năng-tiếp-cận); bảng hằng nằm ở `crates/graybox/src/controls.rs`. Khác biệt và lựa chọn cần playtest:

- Đấu lại bằng **Enter** vì F5 là phím tải lại trang; Start của gamepad chỉ đấu lại khi trận đã có kết quả, tránh bấm nhầm. M đổi chế độ, B bật/tắt bot đấu tập, H hitbox. Native thêm F9 lưu replay, Esc thoát.
- **Chord gamepad**: giữ RB, hoặc nhấn RB cùng khung với X/Y/B, thì ra thuật 1/2/3 và không kèm đòn cơ bản; thả RB khi còn giữ X không phát thêm đòn nhẹ. Không có cửa sổ trễ để chờ chord, nên X nhấn trước RB một khung vẫn là đòn nhẹ.
- **Cảm ứng**: joystick nổi ở nửa trái; 8 nút bên phải, đường kính tối thiểu 48 px logic. Màn hình ngang dùng hai hàng (thuật ở trên; nhảy/lướt/đỡ cạnh nhẹ/nặng), màn hình hẹp xếp ba hàng và hiện gợi ý xoay ngang. Nút phát ở lúc chạm xuống, đỡ giữ theo ngón; mỗi ngón gắn với một nút hoặc joystick tới khi nhấc lên. Nút thuật tự tối khi hồi chiêu hoặc thiếu năng lượng và hiện số giây còn lại.
- Chưa có màn hình đổi phím, chế độ đỡ toggle, bố cục gọn 7 nút hoặc tùy chỉnh vị trí/độ mờ nút.
- Chữ trong canvas **không dấu**: font mặc định của Bevy chỉ có 95 glyph ASCII và dự án chưa chọn font có giấy phép. Chữ tiếng Việt đầy đủ ở shell.

## Replay và kiểm chứng kết quả

Replay ghi input của mọi bên theo định dạng `myva-replay 1` sẵn có. Khi trận lắng, client tự ghi replay ra văn bản, đọc lại và chạy lại không cần bộ não boss, rồi so hash cuối và kết quả với trận đang chơi; banner và shell hiện “replay đã kiểm chứng” kèm số tick và hash. Bản web tải file qua nút “Tải replay”, bản native lưu bằng F9.

Run E2E: bot B1 (phản ứng 250 ms) hạ boss sau **3.772 tick**, không mất máu, đi qua cả ba pha; trình duyệt báo khớp; file `myva-boss-r1-t3772.myva-replay` ghi trên **wasm32** được `myva-replay verify` bản **native** chạy lại khớp **63 mốc hash**, hash cuối `45df0d28a1c763f9`. Replay của ảnh native (`graybox-r1-t952`) cũng khớp 16 mốc hash. Đây là bằng chứng hash ổn định giữa native 64-bit và wasm32 ở revision này, không phải lời hứa cho mọi build hoặc phần luật dùng float sau này.

## Trạng thái nghiệm thu

| Hạng mục | Kết quả | Phạm vi bằng chứng |
| --- | --- | --- |
| Chuyển graybox Macroquad sang Bevy | PASS | `crates/graybox` là Bevy 0.20; Macroquad và bản web cũ bị gỡ, lịch sử còn trong git |
| Kit Long Lưu, boss Kẻ Giữ Đập | PASS | Dùng nguyên dữ liệu và luật lõi; không đổi thông số |
| Nhân vật, đòn, đạn, vùng cảnh báo boss | PASS | Hàm `shapes()` có test: vùng báo trùng hộp đòn thật khi active, đạn báo cả làn bay; [ảnh native](evidence/combat-native.png) |
| Di chuyển, nhảy, lướt, đỡ, đánh, thuật | PASS | Qua cùng đường input của lõi; unit test lõi và E2E bàn phím/gamepad/cảm ứng |
| HUD HP, sức bền, năng lượng, hồi chiêu, Mạch | PASS | Kèm pha boss, vạch 70%/35%, nguyên nhân bị hạ. Mạch tích lũy nhưng **chưa có đại thuật** để tiêu |
| Bàn phím | PASS | Chrome: phím thật qua CDP; trúng boss, tạm dừng, ô chat giữ phím, nhả phím khi mất focus |
| Gamepad | PASS, giả lập | `navigator.getGamepads()` giả theo Standard mapping; **chưa thử tay cầm vật lý** |
| Cảm ứng | PASS, giả lập | Chromium touch emulation qua CDP, DPR 2, dọc và [ngang toàn màn hình](evidence/combat-touch-landscape.png); **chưa thử điện thoại thật** |
| Replay và kiểm chứng kết quả | PASS | Tự kiểm chứng trong client; replay WASM được native chạy lại khớp |
| Trận trong shell Leptos | PASS | [Ảnh chiến thắng](evidence/combat-web-victory.png); 30 vòng vào/rời không lỗi |
| Playtest người thật, sai thao tác, cửa cancel/guard | **NOT_RUN** | Bước 2–3 của work-plan 020 còn mở |
| Mobile native (Android/iOS) | **BLOCKED** | Chờ gate #10 trên thiết bị thật |

Toàn bộ **6 E2E PASS**, 0 skipped, 0 flaky, 254 giây; Chrome `153.0.8010.36`, Linux `7.0.0-38-generic`, Intel Core Ultra 7 155H, harness ép WebGL2 qua SwiftShader. Rust: 75 test PASS (sim 46, graybox 17, economy 12); Clippy host và wasm32, fmt và kiểm tra link docs PASS.

## Số đo

| Chỉ số | Kết quả |
| --- | --- |
| Game WASM sau wasm-bindgen | 83.952.708 byte; gzip ước tính 11.064.025 byte (spike D03: 63.294.408 / 8.251.795) |
| Tổng file web sinh ra | 84.563.824 byte; gzip ước tính 11.172.327 byte |
| Runtime startup, 30 lần vào | Lần đầu 1.643 ms; 29 lần sau median 1.584 ms, p95 mẫu 1.677 ms (nearest rank); localhost |
| DOM sau GC, vòng 1 → 30 | Documents 2 → 2; nodes 222 → 222; listeners 42 → 42; JS heap 2.691.972 → 2.951.600 byte |
| Desktop GPU (Chrome có cửa sổ, ANGLE Intel MTL OpenGL 4.6) | RAF/Bevy Update 60,15 Hz; tick luật 59,95/giây; keydown → tick nhận nút 20,6–37,3 ms (6 mẫu) |
| Headless SwiftShader | RAF/Update 4,2–4,7 Hz; tick luật vẫn 61,8–62,6/giây nhờ fixed timestep; keydown → tick 236–478 ms |
| Trận bot B1 trong E2E | 3.772 tick (62,9 giây luật) trong 65,7 giây thực dù khung hình chỉ ~4 Hz |

Trễ đo từ `keydown` DOM tới tick mô phỏng tiêu thụ khung có nút nhấn, quan sát trên requestAnimationFrame. Nó **không** là input-to-photon và không gồm trễ phần cứng; phím đến từ CDP chứ không từ bàn phím vật lý. Trên SwiftShader trễ bị chặn bởi thời gian khung hình, nên chỉ dùng để phát hiện hồi quy, không đánh giá cảm giác điều khiển. Cửa sổ Chrome có thể nhận input thật trong lúc đo; xem `observedInputEvents` trong JSON.

WASM tăng khoảng 20,7 MB vì `bevy_ui`, `bevy_text`, font mặc định và gilrs. Cold start qua mạng mobile chắc chắn vượt ngân sách trong [assets-performance](../technical/assets-performance.md); giảm kích thước (wasm-opt, cắt feature, nén khi phục vụ) thuộc [#13](https://github.com/loveoverflowcom/myva/issues/13), không được bỏ qua khi lên slice.

## Chưa làm và giới hạn

- **Đại thuật** (Triều Dâng tăng cường, 100 Mạch) chưa có trong lõi; thanh Mạch chỉ hiển thị. Thêm vào cần quyết định cách xác nhận (giữ/toggle) và test lõi riêng.
- Chưa có hai bệ cao, van điều tiết, co-op hay gợi ý sau ba lần thua của boss (combat.md §9); coyote time/jump buffer, DR khống chế và đòn không trung riêng vẫn chưa có.
- Graybox chưa dùng foundation #12; khi #12 xong, adapter fixed tick chuyển sang boundary chung.
- Native desktop chỉ chạy bản debug trên Linux/X11 với Vulkan Intel; chưa thử Windows/macOS hay Wayland thuần.
- Font tiếng Việt cho canvas, đổi phím, toggle đỡ, bố cục gọn 7 nút, giảm flash/rung và âm thanh chưa có.
- Không có server; số đo latency mạng của combat.md §10 chưa chạy.
