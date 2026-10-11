# #19 — Mạch công/thủ và luật giao tranh Long Lưu

**Trạng thái:** lát cắt triển khai để playtest, ngày **2026-10-11**. Issue [#19](https://github.com/loveoverflowcom/myva/issues/19), kế thừa [#2](https://github.com/loveoverflowcom/myva/issues/2) và [D04](../technical/gameplay-foundation.md). Mọi thông số là **GT**, chưa cân bằng qua người chơi. Đây là một bước của issue; không đóng toàn bộ acceptance của #19, #2, #5 hoặc #10.

## Kết quả và ranh giới

Long Lưu dùng Mạch để chọn công hoặc thủ: giữ thuật 3 yêu cầu **Triều Dâng tăng cường**, còn nhấn đỡ mới sau khi vào hitstun yêu cầu **Thoát Mạch**. Vẫn **2 đòn + 3 thuật, 8 nút + joystick**. Chi phí/cooldown/timing lấy từ [combat.md](../design/combat.md); bảng baseline, phần đã làm và phần còn thiếu ở [§14](../design/combat.md#14-lát-cắt-19-và-trạng-thái-triển-khai). Kit chọn ô đại thuật trước trận; UI chọn ô/toggle chưa có.

- `myva-sim` giữ luật duy nhất. Đại thuật kế thừa chi phí năng lượng và cooldown Triều Dâng, xác thực đủ năng lượng/Mạch/cooldown/state rồi trừ một lần; yêu cầu thiếu Mạch không tự rơi về bản thường. `ActionKind::Ultimate` giữ danh tính biến thể trong hit/event/replay.
- Thoát chỉ nhận cạnh nhấn mới sau hitstun; giữ đỡ từ trước, press lặp khi còn giữ, buffer trước hit, chết và guard break không kích hoạt. Nó không gây damage, không cho tấn công trong cửa bảo vệ, tách hai bên trong giới hạn arena và giữ poise boss. Cooldown, chi phí và lý do từ chối do core xác nhận. Hazard/đòn nắm boss chưa có trong runtime nên không được coi là đã kiểm thử.
- `ControlBudget` thuộc mục tiêu: lịch sử DR trượt, damage và thời gian chuỗi dùng chung cho melee, phản công, đạn và guard break. Đổi nguồn/hướng/độ cao không reset. Khoảng trống ngắn vẫn tính vào thời gian chuỗi; chỉ một cửa phản ứng liên tục đủ dài reset budget. Khi chạm giới hạn, core cắt khóa và bảo đảm cửa phản ứng, không cần có Mạch. Immunity control không tự miễn damage; damage bị cắt chỉ khi còn thuộc chuỗi chưa có cửa phản ứng.
- Đòn hụt giữ recovery và hướng cam kết; blocked không mở hit-confirm như hit. HUD/popup/telemetry phân biệt trúng, bị đỡ, hụt, pha đòn và lý do không nối. Ý định buffered hết hạn có feedback. Counter-hit/punish được định nghĩa trong luật nhưng HUD chưa có nhãn riêng; không cấp bonus damage/stun.
- `myva-gameplay` mirror `CombatFeedback` bất biến; không có luật combat trong ECS/HUD. Graybox phân giải chạm/giữ trước khi gửi ý định. Pause, mất focus, host transition và touch cancel bỏ cử chỉ đang chờ; frame chậm trước touch-down không được tính thành thời gian giữ. Pause/mất focus gửi `CancelPending` qua core và replay để bỏ cả buffer đã nhận.

Phòng thử hiện áp cùng luật chuẩn hóa cho mọi cơ thể không giáp, bao gồm quái thường; boss có giáp nhận damage nhưng không stun/đẩy. Đây chưa là cấu hình PvE/PvP production riêng biệt. Không thêm root, launcher, air follow-up, bệ, wall combo, đổi bên hoặc roster mới. Root về thiết kế không cấm đỡ/đánh như stun; mở launcher còn cần gate và fixture riêng.

## Protocol, replay và fixture

Domain protocol lên **v2**, thêm ý định `Ultimate(slot)`, `GuardPress` và `CancelPending` (bỏ buffer khi mở UI, không hủy đòn đã cam kết); v1 bị từ chối. Bridge web vẫn là v2 của shell/runtime, không phải domain protocol. Cổng vẫn kiểm phiên, actor, sequence, cửa tick và ô thuật; client không gửi damage, Mạch hay kết quả hit. Các sự kiện bổ sung: `Whiff`, `ActionDenied`, `MachEscaped`, `ControlLimited`. `Hit.damage`/`Countered.damage` là lượng đã được cấp sau budget, còn tally và nhận Mạch đọc đúng lượng đó.

Replay lên **`myva-replay 3`**. File v1/v2 bị từ chối với lý do luật/hash không tương thích; cần revision gốc để phát file cũ. Không nâng header rồi áp luật mới lên evidence lịch sử. Báo cáo [graybox cũ](combat-graybox.md) giữ nguyên revision/số đo gốc và có ghi chú phân biệt.

Fixture `mach_ground`/`mach_commands` dùng `Session::submit`, không chèn tài nguyên hoặc sửa state: 25 đòn nhẹ tích đủ Mạch, một đại thuật, 13 đòn nhẹ tích lại, boss quét rồi nhấn đỡ mới để thoát. Sau 2.000 tick: **một đại thuật, một Thoát Mạch**, Long Lưu còn **880 HP / 18 Mạch**, boss **530/2.400 HP**. Replay phát lại mọi checkpoint. Đây là fixture wiring/determinism, không phải bot chơi tốt hay kết quả cân bằng.

## Cách tái lập và bằng chứng

```bash
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo clippy --locked -p myva-web-shell -p myva-web-game -p myva-sim -p myva-gameplay -p myva-assets --target wasm32-unknown-unknown -- -D warnings
cargo run --locked -p myva-gameplay --bin myva-headless -- --mach --seed 7 --ticks 2000 --replay mach.myva-replay
cargo run --locked -p myva-sim --bin myva-replay -- verify mach.myva-replay
./scripts/check-wasm-determinism.sh 3600 1 7 12
./scripts/build-web-shell.sh
MYVA_WEB_TEST_PORT=8091 CHROME_PATH=/usr/bin/google-chrome npm run test:web
python3 scripts/check-docs.py
```

Linux native cần `libudev-dev`; môi trường run dùng development package giải nén dưới `/tmp` và liên kết thư viện hệ thống, không sửa package hệ thống. Rust `1.97.1`, wasm-bindgen `0.2.129`, Bevy `0.20.0`, web profile `web`. Port test tùy chọn vì 8080 đã có dịch vụ khác; mặc định repository vẫn 8080.

Chi tiết build/config, digest source và kết quả thực tế nằm trong [evidence JSON](evidence/combat-mach-2026-10-11.json). Harness web ghi trạng thái, ảnh và replay tại `target/web-evidence`; harness determinism ghi từng hash và `report.json` tại `target/wasm-determinism`.

Run ngày 2026-10-11: **180 test Rust và 11 test browser PASS**, không skip/retry/flaky; rustfmt, Clippy native/WASM, asset fixture/validator và docs checker PASS. Hai fixture native/WASM khớp toàn bộ 21.600 cặp hash. Replay tải từ browser khớp 63 checkpoint trên native qua 3.774 tick; đây là bằng chứng wiring/replay, không phải playtest người thật.

| Hạng mục | Bằng chứng / phạm vi |
| --- | --- |
| Cancel / resource / direction | Regression hit/block/whiff, sai hướng, sai cửa, thiếu năng lượng, cooldown, hụt không dash-cancel và không xoay hướng |
| Mạch | Atomic spend, cooldown thường/tăng cường chung, slot sai, chết/không trung/hitstun, guard held cũ, buffer, cooldown thoát, tách bên tại hai tường |
| CC hiện hữu | Nhiều attacker + projectile cùng tick (160 + 90 + 0), lịch DR chung guard break/hitstun/displacement, khoảng 1 tick, giới hạn thời gian và cửa phản ứng |
| Tìm vòng lặp | Liệt kê hữu hạn 7 lựa chọn mỗi bước, độ sâu 5: damage, stun, guard break và các khoảng ngắn/dài; không chứng minh mọi roster tương lai |
| Core / ECS | Fixture mới đối chiếu core, server ECS, client ECS và feedback mirror; giữ suite parity graybox hiện hữu |
| Native / WASM | Fixture phòng thử cũ và fixture Mạch mới, 3.600 tick × seed 1/7/12; so từng tick, không chỉ hash cuối |
| Browser | Bàn phím, gamepad giả lập, touch giả lập: giữ chỉ yêu cầu tăng cường, thiếu Mạch không cast thường/double-spend; chạm ngắn chỉ một cast; touch cancel không cast |

Ảnh HUD luyện tập trên [desktop](evidence/combat-mach-keyboard.png) và [touch giả lập ngang](evidence/combat-mach-touch.png). Chúng ghi pha đòn/kết quả và bộ 8 nút; không phải bằng chứng thiết bị mobile thật.

## Gate còn mở

| Gate | Trạng thái và điều kiện tiếp theo |
| --- | --- |
| Người mới + người có kinh nghiệm | **NOT_RUN**: cần người thật đo hiểu hit/whiff, tự dừng combo, lỗi touch, thời gian mất quyền điều khiển, tỷ lệ dùng hành động; bot/hash không thay được |
| Confirm ngắn/dài | **NOT_RUN**: 80 ms cancel và 120 ms buffer vẫn là GT kỹ thuật; cần đo ý định bị từ chối và khả năng nhìn trúng rồi chọn nối |
| RTT 50/100/150/250 ms, jitter/mất gói | **BLOCKED** bởi transport hai client #5; không có số đo mạng từ local/headless |
| Android/iOS và tay cầm thật | **NOT_RUN**; desktop Chromium touch/Gamepad API giả lập không đóng #10 |
| Root / launch / tiếp đất / đổi bên có vách | **NOT_RUN** và chưa bật: cần effect semantics, hình học, fixture và bounded search mở rộng; wall escape hiện có không phải cross-up test |
| PvE/PvP production, co-op boss/poise | **NOT_RUN**: mới có normalized room và boss có giáp hiện hữu |

Không tuyên bố cân bằng roster, có server multiplayer, hết infinite combo của mọi kit tương lai hoặc hoàn tất playtest chỉ từ các kiểm tra này.
