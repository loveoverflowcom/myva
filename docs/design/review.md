# Review draft — từ ý tưởng đến thiết kế có thể kiểm chứng

**Đề xuất v0.1, 2026-10-09.** Review này bảo toàn hướng MMORPG, chiến đấu kỹ năng và thần thoại đa văn hóa; làm rõ những chỗ chưa đủ để triển khai.

## Điểm mạnh

1. Hậu duệ/người kế thừa giúp người chơi có hành trình trưởng thành và cho phép viết nhân vật mới, thay vì phải cân bằng trực tiếp quyền năng của các vị thần.
2. Khám phá và tiến triển MMO tạo động lực quay lại; boss và địa hình dựa trên kỹ năng tạo chất lượng cho từng phút chơi.
3. Linh vực tạo nguồn nội dung dài hạn; học truyền thừa khác vùng mở hướng build theo chiều ngang.
4. Solo và cày chay được đặt thành nguyên tắc từ đầu, giúp thiết kế reward tránh ép người chơi vào guild hoặc nạp tiền.
5. Draft đã quan tâm tài nguyên, tải asset và server authoritative trước khi làm thế giới lớn.

## Khoảng trống và quyết định đề xuất

| Vấn đề | Hệ quả nếu giữ nguyên | Quyết định v0.1 |
| --- | --- | --- |
| MMO, đối kháng và platformer đều có scope lớn | Làm nhiều nội dung trước khi biết chiến đấu có vui không | Bản thử Vân Thủy 20–30 phút, boss và replay combat trước online đầy đủ |
| “Kỹ năng quyết định chiến thắng” chưa phân biệt mode | Người chơi cấp thấp tưởng có thể thắng mọi chênh lệch gear | PvE cho gear tạo tiến triển có giới hạn; ranked PvP chuẩn hóa ở giai đoạn sau |
| “Tuyệt đối không lạm phát” chưa có định nghĩa | Có trần tài nguyên vẫn có thể tăng giá vì thiếu hàng hoặc tiền mới | Ghi riêng tồn lượng, lượt phát hành, tiền tệ, chỉ số giá và khả năng tiếp cận; dùng mục tiêu đo được |
| Tổng cấp độ có thể tăng mãi hoặc bị alt/bot thao túng | Tạo động lực nuôi tài khoản phụ để tăng nguồn cung | Cấp độ đóng góp bị cap, giảm dần; active cohort có trọng số; ngân sách toàn thế giới độc lập shard |
| Nguồn cung phản ứng giá trực tiếp | Đầu cơ có thể đẩy controller và gây dao động | Dữ liệu trễ, nhu cầu tiêu dùng đã hoàn tất, rate limit, deadband và chế độ ngân sách an toàn |
| Một châu lục được xem như một “hệ” | Giản lược các cộng đồng khác nhau thành một kiểu nhân vật | Linh vực là địa lý hư cấu; truyền thừa là các nhánh riêng, có cổng review văn hóa |
| Xuất thân theo nơi sống | Có thể khóa người chơi vào vị trí thực hoặc chủng tộc | Người chơi tự chọn linh vực; không dùng IP/GPS/quốc tịch để khóa sức mạnh |
| Học kỹ năng mọi vùng không có giới hạn build | Build tối ưu lấy tất cả, roster bị vô hiệu | 5 slot chiến đấu; 2 căn bản + 3 thuật; cơ chế giao thoa có chi phí cơ hội |
| Native Bevy trong CMP bị hiểu nhầm là tính năng sẵn có | Chọn nội dung rồi mới phát hiện lifecycle/render không ổn | Spike Android và iOS là gate; chưa thông qua thì chưa cam kết mở rộng mobile |
| Skeletal animation và GPU effects chưa có pipeline | Asset đẹp nhưng không chạy được trên thiết bị mục tiêu | Prototype atlas trước; skeletal và effect có gate về chất lượng và ngân sách |
| Đại chiến không có giới hạn tác nhân/mặt trận | Không kiểm soát networking, fill rate hoặc griefing | Chiến dịch nhiều mặt trận có instance giới hạn, mục tiêu chung và ngân sách chung |

## Điều chỉnh nguyên tắc kinh tế

“Không phát hành vô hạn” cần ba đại lượng rõ ràng:

- **Tồn lượng hữu hạn:** tổng vật liệu trong bể, node, kho, đồ chế tạo và hàng đợi tái chế không vượt trần; từng kỳ có ngân sách chuyển vật liệu ra điểm khai thác.
- **Tạo mới:** tăng kích thước bể bằng đơn vị vật liệu chưa từng tồn tại. Pilot đề xuất tạo mới bằng 0 sau khi khởi tạo bể.
- **Tổng lượt tái phát hành:** đếm cả vật liệu đã tiêu hao rồi tái chế và cấp ra lần nữa. Nếu đại lượng này cũng phải hữu hạn suốt đời, cần trần `J_max`; chạm trần sẽ giảm/dừng hồi phục, kể cả còn vật liệu trong bể.

Nếu tài nguyên liên tục bị tiêu hao rồi cấp lại mãi qua thời gian thì tổng lượt tái phát hành không thể hữu hạn, dù không tạo thêm đơn vị vật liệu. Đề xuất dùng bể hữu hạn, tái chế và hạn mức định kỳ; tài nguyên hiếm có phân bổ sự kiện/mùa trong cùng trần toàn thế giới, không tự reset bể. Chốt chính sách `J_max` sau simulation, giữ rõ khác biệt với tạo mới.

Trần nguồn cung cũng không bảo đảm chỉ số giá bất biến. Thay đổi nhu cầu, tốc độ lưu thông, phân phối và lượng tiền đều ảnh hưởng giá. Cam kết sản phẩm phù hợp hơn: **không mint mất kiểm soát, không nhân đôi nguồn qua shard, không để người mới thiếu vật tư thiết yếu và phát hiện mất cân bằng qua dữ liệu.**

## Phạm vi cố định cho bản thiết kế nền

| Tham số | Đề xuất dùng chung |
| --- | --- |
| Linh vực đầu | Vân Thủy, tên hư cấu; làng Bến Lau |
| Nội dung slice | 3 bản đồ nhỏ, 3 truyền thừa, 1 boss Kẻ Giữ Đập |
| Thời lượng vòng chính | 20–30 phút, không phải tổng thời gian farm |
| Loadout | 2 đòn nhẹ/nặng + 3 thuật, tổng 5 slot; thanh Mạch mở đại thuật biến thể một thuật |
| Level cap prototype | 1–10 ở slice; 1–20 ở MVP, đều cần cân bằng |
| Tài nguyên slice | 2 loại thường và 1 loại hiếm làm mẫu |
| Online | Pilot có giới hạn người chơi; chưa dùng từ MMO quy mô lớn như năng lực đã đạt |
| Origin tự chọn | Tầm nhìn dài hạn; slice/MVP bắt đầu cùng Vân Thủy |

## Trade-off cần chấp nhận

- Combo online cần tín hiệu đủ rõ và cửa sổ nhập hợp lý trên mobile. Không bê nguyên nhịp đối kháng local sang trận đông người.
- Người chơi solo vẫn có tiến triển chiến đấu hoàn chỉnh; raid và công trình guild có bản sắc riêng nhưng không độc quyền gear bắt buộc.
- Cosmetic phải giữ silhouette, telegraph và hitbox dễ đọc. Đẹp hơn không được làm đối thủ khó nhận diện chiêu hơn.
- Kinh tế thiếu nguồn cung cũng là thất bại. Hạn mức không được biến thành cơ chế bắt người chơi chờ hoặc trả tiền để chơi tiếp.
- Cache giảm tải mạng nhưng tốn dung lượng; người dùng cần biết gói tiếp theo và có thể xóa dữ liệu tải lại.

## Các quyết định còn mở

| Câu hỏi | Bằng chứng để chốt | Gate |
| --- | --- | --- |
| Hữu hạn tồn lượng hay hữu hạn tổng lượt mint cho từng loại? | Simulation tài nguyên, tái chế và trạng thái hết ngân sách | Trước online economy |
| Native renderer trong CMP có dùng được ổn định cả hai hệ? | Bản thử surface/context, input, background, phục hồi và voice/audio focus | Trước mobile slice |
| Combat chịu được latency và jitter bao nhiêu? | Hai client qua network impairment và replay authoritative | Trước co-op MVP |
| 3 truyền thừa có thực sự khác nhau? | Người mới chơi thử và hoàn thành boss bằng từng kit | Trước làm thêm roster |
| Làm solo có bị thiếu reward hoặc tài nguyên? | Theo dõi time-to-upgrade và nguồn vật tư trong cohort solo | Trước mở pilot |
| Tham chiếu thần thoại nào được dùng? | Danh mục nguồn theo từng cộng đồng và review nội dung | Trước public World Expansion |

Ưu tiên tiếp theo là **kiểm chứng native integration và combat**, cùng một simulator kinh tế nhỏ. Chưa thêm bản đồ lục địa hoặc sản xuất roster lớn khi các gate này chưa đạt.
