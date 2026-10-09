# Vertical slice — Bến Lau và Kẻ Giữ Đập

**Đề xuất v0.1.** Mục tiêu là một vòng chơi có thể hoàn thành, lặp lại và đo được trước khi sản xuất MMO nhiều vùng.

## Kết quả người chơi nhận được

Trong 20–30 phút, một người mới đến Bến Lau, học di chuyển và phòng thủ, chọn một truyền thừa, vượt hai tình huống địa hình, thu thập vật tư, chế tạo một món đồ và đánh bại Kẻ Giữ Đập. Có thể chơi lại bằng kit khác và nhận thấy chiến thuật khác biệt.

Đây là lát cắt chất lượng, không phải bản game chỉ có màn giới thiệu hoặc animation demo. Single-player prototype được phép chạy simulation local để thử luật; trạng thái đó không chuyển thành item online thật.

## Phạm vi

| Phần | Số lượng/giới hạn đề xuất |
| --- | --- |
| Linh vực | Vân Thủy, hư cấu |
| Bản đồ | Bến Lau: hub; Rừng Bậc Nước: học di chuyển/khai thác; Đập Cổ: arena boss |
| Truyền thừa | Long Lưu, Sơn Cốt, Phong Vũ |
| Loadout | 2 đòn căn bản nhẹ/nặng + 3 thuật; thanh Mạch mở đại thuật là biến thể của một thuật |
| Level | 1–10 để thử pacing; không ép farm tới cap mới đánh boss |
| Đối thủ | 3 archetype thường: cận chiến, đạn chậm, kiểm soát địa hình; 1 boss nhiều phase |
| Tài nguyên | 2 vật tư thường, 1 vật tư hiếm; đơn vị và trần xem economy |
| Chế tạo | 1 recipe bắt buộc học, vài lựa chọn sidegrade; thu hồi một phần vật liệu |
| Nhiệm vụ | Chuỗi chính hữu hạn, 1 nhánh phụ minh họa lựa chọn; tránh lặp “giết 100 quái” |
| Platform | Bevy + Leptos web qua #11; Android/iOS Bevy + CMP qua #10 riêng từng hệ rồi mobile slice |
| Online kiểm chứng | 2 client co-op nhỏ sau khi slice local đạt cảm giác chơi |

Số lượng này chỉ là ngân sách slice. Không đồng nghĩa MMORPG phát hành chỉ có ba truyền thừa.

Trước slice, [ADR 0003](../decisions/0003-bevy-engine-adoption.md) và foundation #12 phải được áp dụng vào combat #2; economy #3 giữ độc lập engine. #13 đo pipeline trên platform đã đạt gate. Cảnh spike 2D hoặc graybox Macroquad lịch sử không thay thế các gate này.

## Nhịp trải nghiệm

| Thời điểm dự kiến | Tình huống | Điều cần học |
| --- | --- | --- |
| 0–3 phút | Sự cố dòng mạch ở Bến Lau; thử kit và chỉnh điều khiển | Nhân vật là người kế thừa; đi, nhảy, đánh |
| 3–8 phút | Rừng Bậc Nước, đạn chậm và đường rẽ | Dash, guard, đọc telegraph; đường an toàn cho người mới |
| 8–13 phút | Một điểm tài nguyên đang suy giảm | Thu hoạch có trữ lượng; lựa chọn dùng vật tư |
| 13–18 phút | Chế tạo sidegrade, mở cửa Đập Cổ | Build là lựa chọn; không chỉ cộng lực chiến |
| 18–30 phút | Boss, checkpoint gần arena, retry nhanh | Nhận diện pattern, combo có thời điểm và phản công |

Nếu chơi thử kéo dài vì người chơi lạc hoặc UI khó hiểu, sửa dẫn đường trước khi giảm máu boss. Thời gian loading và chờ mạng được đo riêng.

## Tiêu chí hoàn thành

| Gate | Bằng chứng phải có |
| --- | --- |
| G1: điều khiển | 5 người chưa được hướng dẫn riêng; ít nhất 4 người tự tìm được di chuyển, guard và dùng thuật sau tutorial; ghi lỗi thao tác |
| G2: khác biệt build | Mỗi kit có thể thắng boss với cùng ngân sách trang bị; người chơi mô tả được ít nhất một khác biệt chiến thuật |
| G3: boss công bằng | Mỗi lần trúng chiêu truy được về telegraph, input và server/replay; không có hitbox sai hoặc phase không thể né |
| G4: solo tiến triển | Hoàn thành chuỗi chính và recipe đầu bằng nguồn solo trong slice; không yêu cầu party, chợ hoặc trả tiền |
| G5: kinh tế | Invariant tồn lượng và ledger luôn đúng; tăng account/shard không tăng ngân sách ngoài cấu hình |
| G6: web | Tải được gói đầu, lỗi download có retry, shader fallback; đạt ngân sách đề xuất trên thiết bị đã chọn |
| G7: mobile native | Android và iOS có bằng chứng render/input/lifecycle/native; không dùng WebView để đánh dấu gate đạt |
| G8: online nhỏ | 2 client chơi boss; disconnect/reconnect không nhân đôi reward, không tự xác nhận thắng |

5 người là mẫu tìm vấn đề UX, không đủ để kết luận retention hoặc thị trường. Ghi nhận video/replay, thiết bị, build, điều kiện mạng và kết quả từng người.

## Asset cần cho slice

- Một bộ silhouette/portrait cho mỗi truyền thừa; cùng tỉ lệ và anchor.
- Animation tối thiểu: idle, chạy, nhảy/rơi, dash, guard, light/heavy, trúng đòn, gục, thuật.
- Tiles/background cho ba bản đồ; layers phục vụ parallax có giới hạn.
- Telegraph/hit effect/đạn rõ ở màn hình mobile; color không là tín hiệu duy nhất.
- Boss có animation phase và vùng nguy hiểm đồng bộ timeline simulation.
- HUD, icon skill và âm thanh feedback; mọi asset có source, version, trạng thái quyền sử dụng.

Trước sản xuất asset cuối, dùng placeholder đọc rõ silhouette để thử combat. Chốt pipeline export, anchor, hitbox và atlas bằng một nhân vật trước khi vẽ cả roster.

## Điều chưa làm ở slice

Auction house toàn thế giới, chat công khai có moderation đầy đủ, guild war, nhà ở, craft hàng trăm recipe, mount, pet chiến đấu, nhiều lục địa, ranked PvP và monetization không thuộc slice. Các tính năng này có gate riêng trong roadmap.

## Điều kiện dừng và sửa

- Combat không vui sau hai vòng thử có thay đổi cụ thể: ưu tiên sửa nhịp, feedback và input trước thêm nội dung.
- Native embedding chưa đạt trên một hệ: ghi blocker và tiếp tục web/economy độc lập; không gọi mobile đã hoàn tất.
- Resource controller gây đói vật tư hoặc dao động: dùng ngân sách cấu hình an toàn, sửa mô hình trước mở trade.
- Không đạt 60 FPS: xác định CPU/GPU/memory bottleneck; 30 FPS là fallback minh bạch, không thay điều kiện benchmark đã hứa.
