# #2 — Graybox chiến đấu Bevy: trận đầu tiên chơi được

Ngày thực hiện: **2026-10-10**, cập nhật **2026-10-11** khi graybox chuyển sang nền ECS dùng chung D04 ([#12](https://github.com/loveoverflowcom/myva/issues/12)). Issue [#2](https://github.com/loveoverflowcom/myva/issues/2), work-plan [020](../work-plan/020-combat-graybox.md), quyết định [ADR 0003](../decisions/0003-bevy-engine-adoption.md), nền [gameplay D04](../technical/gameplay-foundation.md). Số liệu dưới đây là run 2026-10-11, mã ứng dụng và harness ở revision `82524ccfeeaa8f647cc6b34facd8a81ec4635945` của nhánh `feat/bevy-combat-graybox`; bản web build profile `web`, `dirty=false`.

Báo cáo này xác nhận một trận Long Lưu đấu Kẻ Giữ Đập **chơi được và kiểm chứng được bằng replay** trên trình duyệt và native desktop. Nó **không** thay playtest người thật, không đo thiết bị mobile thật và không cân bằng lại thông số: mọi luật dùng nguyên lõi `myva-sim` hiện có.

## Cách tái lập

```bash
cargo test --workspace
cargo test -p myva-graybox --test parity    # client ECS không cửa sổ = lõi, từng tick
cargo run -p myva-graybox -- --screenshot combat-native.png   # Linux cần libudev-dev
./scripts/build-web-shell.sh && python3 scripts/serve-web-shell.py
CHROME_PATH=/usr/bin/google-chrome npm run test:web
CHROME_PATH=/usr/bin/google-chrome node scripts/measure-web.mjs   # khi server đang chạy
```

Harness ghi JSON, ảnh và replay vào `target/web-evidence/`; CI upload thư mục này. Bản tóm tắt của run trong báo cáo nằm ở [evidence JSON](evidence/combat-2026-10-11.json); run trước khi chuyển sang D04 giữ ở [evidence 2026-10-10](evidence/combat-2026-10-10.json).

## Ranh giới và luồng dữ liệu

- **Lõi** (`myva_sim::battle`): trận dựng trên `Session` của D04, đường tick duy nhất cho server, runner headless và client. Boss Kẻ Giữ Đập (`BossController`) và bot đấu tập B0 (`SparringBot`) là `Controller` của phiên, gửi lệnh qua cùng cổng như người chơi; phiên tự đánh số `seq` lệnh AI, nên tắt bot giữa trận không làm lệnh bị từ chối. `Bout` (đội hình, bot lái hộ, trọng tài, kiểm chứng replay) không giữ phiên; `Battle` ghép hai thứ cho test và `myva-replay`. Khi có người bị hạ, thế giới chạy thêm 60 tick rồi dừng; replay luôn có điểm kết thúc. Thuần Rust, build được cho wasm32.
- **Client Bevy** (`myva-graybox`): `GameplayPlugin::new(Authority::Client)` của `myva-gameplay` chạy `Session::step` trong `FixedUpdate` 60 Hz và mirror kết quả sang component; `MatchPlugin` của graybox thêm phần client. Thiết bị → `Intent` → `LocalInput` trong `RunFixedMainLoop` trước vòng fixed; bot lái hộ thay ý định ngay trước `GameplaySet::Input`; trọng tài đọc `TickReport` trong `GameplaySet::Events`, khi trận lắng thì kiểm chứng replay rồi dừng phiên. Cảnh, HUD, nút cảm ứng và telemetry đọc `ArenaView` dựng lại từ component mirror; popup đọc `SimEvent`. Đấu lại và đổi chế độ nạp phiên mới bằng `LoadSession`. Nút nhấn được chốt tới tick kế tiếp, nên FPS khác tick rate không làm mất hay nhân đôi một lần nhấn. `FighterId` là ID miền, `Entity` chỉ sống trong process. Adapter tạm của bản 2026-10-10 đã gỡ.
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

Replay ghi lệnh của mọi bên theo định dạng `myva-replay 2` của D04 (thêm seed; bản 1 vẫn đọc được). Khi trận lắng, client tự ghi replay ra văn bản, đọc lại và chạy lại không cần bộ não boss, rồi so hash cuối và kết quả với trận đang chơi; banner và shell hiện “replay đã kiểm chứng” kèm số tick và hash. Bản web tải file qua nút “Tải replay”, bản native lưu bằng F9.

Run E2E: bot B1 (phản ứng 250 ms) hạ boss sau **3.773 tick**, không mất máu, đi qua cả ba pha; trình duyệt báo khớp; file `myva-boss-r1-t3773.myva-replay` ghi trên **wasm32** được `myva-replay verify` bản **native** chạy lại khớp **63 mốc hash**, hash cuối `fabc0e52bccb31a2`. Replay của ảnh native (`graybox-r1-t950`) cũng khớp 16 mốc hash.

Đối chứng client với lõi (`crates/graybox/tests/parity.rs`): app chỉ gồm `TimePlugin` + `GameplayPlugin` + `MatchPlugin`, input đi qua `Intent` → `LocalInput` như thiết bị thật, so với `Battle` chạy thẳng trên lõi. Cả bốn kịch bản cho cùng hash **từng tick**, cùng số liệu từng bên và replay **giống từng byte**: bot B1 đánh boss tới thắng và tự kiểm chứng; 2.400 tick input ngẫu nhiên có đủ đi, đỡ, nhảy, lướt, đòn và thuật (sự kiện không lặp, mỗi cú bấm vào đúng một tick); đổi chế độ rồi tắt bot đấu tập giữa trận; tạm dừng bỏ cú bấm. Đây là bằng chứng hash ổn định giữa native 64-bit và wasm32 ở revision này, không phải lời hứa cho mọi build hoặc phần luật dùng float sau này.

## Trạng thái nghiệm thu

| Hạng mục | Kết quả | Phạm vi bằng chứng |
| --- | --- | --- |
| Chuyển graybox Macroquad sang Bevy | PASS | `crates/graybox` là Bevy 0.20; Macroquad và bản web cũ bị gỡ, lịch sử còn trong git |
| Chạy trên nền ECS D04 (#12) | PASS | `GameplayPlugin` + `Session`; boss là `Controller`; presentation đọc mirror; đối chứng từng tick với lõi ở trên |
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

Toàn bộ **6 E2E PASS**, 0 skipped, 0 flaky, 258 giây; Chrome `153.0.8010.36`, Linux `7.0.0-38-generic`, Intel Core Ultra 7 155H, harness ép WebGL2 qua SwiftShader. Rust: 126 test PASS (sim 79, gameplay 14, graybox 21, economy 12); Clippy host và wasm32, fmt và kiểm tra link docs PASS.

Trong lúc chạy lại, bước “nhả phím khi mất focus” của test vòng đời fail 2/4 lần với cùng mã game: test chờ cố định 600 ms trong khi khung SwiftShader (~4–5 Hz) không đều. Đường bàn phím → ý định không đổi so với bản 2026-10-10; test giờ đợi vị trí đứng yên qua ba khung Bevy rồi mới kiểm tra nó tiếp tục đứng yên. Bản đã sửa pass 3/3 lần chạy riêng và trong lần chạy cả bộ cuối.

## Số đo

| Chỉ số | Kết quả |
| --- | --- |
| Game WASM sau wasm-bindgen | 84.842.609 byte; gzip ước tính 11.182.853 byte (run 2026-10-10: 83.952.708 / 11.064.025; spike D03: 63.294.408 / 8.251.795) |
| Tổng file web sinh ra | 85.453.825 byte; gzip ước tính 11.291.227 byte |
| Runtime startup, 30 lần vào | Lần đầu 1.658 ms; 29 lần sau median 1.570 ms, p95 mẫu 1.643 ms (nearest rank); localhost |
| DOM sau GC, vòng 1 → 30 | Documents 2 → 2; nodes 222 → 222; listeners 42 → 42; JS heap lúc nghỉ trước/sau 3.077.664 → 3.258.384 byte |
| Desktop GPU (Chrome có cửa sổ, ANGLE Intel MTL OpenGL 4.6) | Chưa đo lại ở run 2026-10-11. Run 2026-10-10: RAF/Bevy Update 60,15 Hz; tick luật 59,95/giây; keydown → tick nhận nút 20,6–37,3 ms (6 mẫu) |
| Headless SwiftShader | RAF/Update 4,68 Hz; tick luật vẫn 62,5/giây nhờ fixed timestep; keydown → tick 254–469 ms (6 mẫu; run trước 4,2–4,7 Hz, 236–478 ms) |
| Trận bot B1 trong E2E | 3.773 tick (62,9 giây luật) trong 65,3 giây thực dù khung hình chỉ ~4 Hz |

Trễ đo từ `keydown` DOM tới tick mô phỏng tiêu thụ khung có nút nhấn, quan sát trên requestAnimationFrame. Nó **không** là input-to-photon và không gồm trễ phần cứng; phím đến từ CDP chứ không từ bàn phím vật lý. Trên SwiftShader trễ bị chặn bởi thời gian khung hình, nên chỉ dùng để phát hiện hồi quy, không đánh giá cảm giác điều khiển. Cửa sổ Chrome có thể nhận input thật trong lúc đo; xem `observedInputEvents` trong JSON.

WASM tăng khoảng 21,5 MB so với spike D03 vì `bevy_ui`, `bevy_text`, font mặc định, gilrs và (khoảng 0,9 MB) lớp ECS dùng chung. Cold start qua mạng mobile chắc chắn vượt ngân sách trong [assets-performance](../technical/assets-performance.md); giảm kích thước (wasm-opt, cắt feature, nén khi phục vụ) thuộc [#13](https://github.com/loveoverflowcom/myva/issues/13), không được bỏ qua khi lên slice.

## Chưa làm và giới hạn

- **Đại thuật** (Triều Dâng tăng cường, 100 Mạch) chưa có trong lõi; thanh Mạch chỉ hiển thị. Thêm vào cần quyết định cách xác nhận (giữ/toggle) và test lõi riêng.
- Chưa có hai bệ cao, van điều tiết, co-op hay gợi ý sau ba lần thua của boss (combat.md §9); coyote time/jump buffer, DR khống chế và đòn không trung riêng vẫn chưa có.
- Pha boss chỉ đọc được qua `Session::controller::<BossController>()`, chưa nằm trong snapshot/mirror; client mạng sẽ cần trường riêng.
- Native desktop chỉ chạy bản debug trên Linux/X11 với Vulkan Intel; chưa thử Windows/macOS hay Wayland thuần.
- Font tiếng Việt cho canvas, đổi phím, toggle đỡ, bố cục gọn 7 nút, giảm flash/rung và âm thanh chưa có.
- Không có server; số đo latency mạng của combat.md §10 chưa chạy.
