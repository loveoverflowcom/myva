# QA — luật, trải nghiệm và độ tin cậy

**Đề xuất v0.1.** Mục tiêu kiểm tra quyết định có ảnh hưởng người chơi, không viết test chỉ lặp lại implementation.

## Các lớp kiểm chứng

| Lớp | Kiểm tra | Bằng chứng |
| --- | --- | --- |
| Simulation core | Input → state hợp lệ, combat windows, cooldown, boss phase | Scenario và replay với seed/config/build cụ thể |
| Kinh tế | Stock caps, budget, mint/sink, ledger, shards, controller | Simulation dài hạn và báo cáo invariant/distribution |
| Online | Authoritative hit, sequence, lag, reconnect, duplicate command | Hai client độc lập, network impairment và server trace |
| Persistence | Atomic reward/trade/craft; crash/restore | Fault injection tại các ranh giới commit |
| Bevy web + Leptos | Đồ họa, input/focus/IME, loading/error, resize/DPR, visibility, bridge/version/session, navigation/cleanup | Browser run có version/build, screenshot đã xem, counters và memory profile qua vòng vào/rời |
| Bevy + CMP native | Surface/context/thread, touch cancel, IME ngoài game, audio focus, safe area, navigation/lifecycle | Android/iOS thật riêng biệt: 30 vòng vào/rời, 20 background/resume, log và screenshot |
| Playtest | Hiểu luật, solo progression, boss fairness, mobile layout | Quan sát người chơi và các lỗi thao tác thực tế |

Kiểm tra rules và render độc lập vẫn cần một vòng tích hợp CMP thật. Desktop/web chạy tốt không chứng minh Android/iOS native embedding đã đúng.

Kết quả dùng `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`, `NOT_IMPLEMENTED`, `NOT_APPLICABLE`, ghi command/config/revision và phạm vi. Compile, browser interaction, screenshot captured/inspected, standalone executable và device-tested là các lớp bằng chứng khác nhau. Thiếu GPU/device/Xcode hoặc suite không chạy không được trả PASS. [ADR matrix](../decisions/0003-bevy-engine-adoption.md#compatibility-matrix-và-gate) dùng `KNOWN/UNKNOWN/BLOCKED` cho trạng thái kiến thức, không thay kết quả test.

Graybox Macroquad lịch sử không chứng minh Bevy đã được kiểm chứng. #12 giữ unit/replay/invariant headless không GPU; #13 đo frame time/memory/build size trên platform đã qua #10/#11.

## Combat scenarios

- Light/heavy không được phát hai hit vì một thao tác touch; release/hold/cancel nhất quán.
- Dash chỉ có invulnerability theo luật đã định; không xuyên collision hoặc teleport do client.
- Guard đúng hướng; stamina hết gây guard break có thời gian thoát; không block vô hạn.
- Cancel chỉ ở cửa sổ hợp lệ; chuỗi hitstun có giới hạn; không infinite combo.
- Một attack ID chỉ hit mỗi mục tiêu theo số lần đã cấu hình; frame render thấp không nhân hit.
- Boss phase đổi trong lúc projectile đang bay vẫn xử lý đúng owner/hit/reward.
- Rơi hố, chết, respawn hoặc chuyển map không mang buff/collision state ngoài luật.
- Damage telegraph và vùng hiển thị khớp với hit volume; camera shake/reduced motion không che phản ứng cần thiết.
- Ranked PvP tương lai chuẩn hóa thuộc tính; test chênh lệch gear không lọt qua normalization.

## Kinh tế scenarios

| Tình huống | Kỳ vọng |
| --- | --- |
| Thêm 100 account phụ ít hoạt động | Tổng ngân sách không tăng tuyến tính theo account |
| Thêm channel/shard | Không tạo bể tài nguyên/mint budget riêng ngoài cấu hình world |
| Gọi harvest lặp cùng command ID | Cùng kết quả trước, không mint thêm |
| Crash sau ghi ledger trước response | Retry nhận kết quả đã commit |
| Hết budget kỳ hoặc trần tồn lượng | Dừng mint đúng; tài nguyên đầu người mới dùng đường riêng có giới hạn |
| Trade qua tài khoản phụ | Không được tính như nhu cầu tiêu dùng thật để tăng nguồn cung |
| Nhiều tài khoản tích trữ | Theo dõi phân phối và availability; controller không chỉ nhìn tổng tồn kho |
| Dữ liệu nhu cầu mất hoặc giá bị wash-trade | Safe budget, không tự tăng issuance từ giá bất thường |
| Tài nguyên giảm mạnh, người mới không craft được | Detect starvation; điều chỉnh nguồn thiết yếu và tham số, không bán vật tư cứu hộ |
| Hai worker cấp budget đồng thời | Tổng allocated + spent không vượt budget đã commit |

Simulation tối thiểu gồm dân số ổn định, tăng nhanh, giảm nhanh, bot/alt, tích trữ, crash/retry và giá bị thao túng. Dùng nhiều seed; báo cáo khoảng dao động, không chỉ một đường giá đẹp.

## Network và persistence

Đề xuất ma trận impairment: RTT 30/80/150/250 ms, jitter 0/30/80 ms, loss 0/1/5%, reorder/duplicate packet và mất mạng 5–30 giây. Đây là điểm thử, không là cam kết hỗ trợ mọi tổ hợp.

Kiểm tra cụ thể:

- Server từ chối input có timestamp tương lai, sequence quá cũ, tốc độ di chuyển hoặc skill loadout sai.
- Prediction correction không đẩy người chơi qua địa hình; giữ camera dễ chịu khi correction lớn.
- Rớt mạng giữa loot, craft, trade, map-transfer: không mất/nhân đôi vật phẩm.
- Hai session cùng nhân vật: có một owner rõ; session cũ bị revoke không thể ghi state tiếp.
- Reconnect sau server restart nạp state đã commit và inventory đúng; không dùng cache client làm nguồn sự thật.
- Trade đủ số dư tại commit; không âm inventory, money, resource quota dù requests đồng thời.
- Backup restore làm được trên môi trường thử; replay jobs không trả reward hai lần.

Rollback hoặc lag compensation phải có cửa sổ giới hạn, không cho packet trễ tùy ý quyết định hit trong quá khứ. Chốt policy từ prototype combat, không ngầm suy ra từ client prediction.

## Device matrix cần chốt trước benchmark

| Nền tảng | Nhóm thiết bị cần chọn | Chưa biết phải ghi rõ |
| --- | --- | --- |
| Web | Chromium + Firefox desktop; Safari nếu trong phạm vi hỗ trợ; browser mobile khi dùng web | OS/browser/version, RAM, GPU, độ phân giải |
| Android | Ít nhất một thiết bị thấp và một thiết bị trung bình thật | API level, GPU, RAM, thermal trạng thái, touch sampling |
| iOS | Ít nhất một iPhone mục tiêu tối thiểu và một mẫu phổ biến | OS version, safe area, RAM pressure, rendering backend |

Không chốt danh sách model cụ thể nếu chưa có thiết bị hoặc benchmark. Đo cold/warm start, boss đông hiệu ứng, tải map mới, background/resume và session dài nóng máy.

## Art và accessibility

- Asset giữ anchor, world scale, facing và collision convention theo mẫu chuẩn.
- Atlas không bleeding ở các mức scale, mipmap hoặc texture filtering mục tiêu.
- Telegraph nhận biết bằng hình/chuyển động và âm thanh tùy chọn, không chỉ màu.
- Text đọc được trên màn hình nhỏ; HUD tránh safe area, ngón tay và notch.
- Có remap control, giảm rung/chớp/camera shake; glyph theo input device.
- Người chơi tắt tiếng vẫn hiểu boss; người dùng không thể giữ nút lâu có phương án input khác.
- Cosmetic không che vũ khí, telegraph hoặc đổi vùng nhìn so với đối thủ.

## Điều kiện đủ để mở pilot

Không còn lỗi mất/nhân đôi item hoặc sai ngân sách đã biết. Native gate đạt cho hệ công bố hỗ trợ. Load test ở giới hạn pilot ghi nhận frame/tick/network/database, và có cách giảm admissions khi vượt ngân sách. Solo quest/reward có người chơi thử hoàn tất. Các vấn đề còn mở có owner và phạm vi ảnh hưởng trong work-plan.
