# Roadmap theo điều kiện hoàn thành

**Đề xuất v0.1.** Chưa đặt ngày phát hành hoặc ước lượng nhân sự khi chưa có dữ liệu prototype.

Mỗi giai đoạn chỉ mở rộng khi đạt gate; nội dung trễ không được bù bằng cách bỏ qua integrity, native lifecycle hoặc combat playtest.

| Giai đoạn | Kết quả có thể review | Gate chuyển tiếp |
| --- | --- | --- |
| 0. GDD & World Bible | Bộ docs hiện tại, baseline scope và danh sách câu hỏi | Mâu thuẫn giữa combat, tiến triển, kinh tế và platform được xử lý |
| 1. Feasibility | Combat graybox, native spike Android/iOS, simulator kinh tế độc lập | Có bằng chứng điều khiển, native và invariant; tham số được cập nhật |
| 2. Vertical slice | Vân Thủy chơi 20–30 phút, 3 kit, boss, craft/tái sinh | Các gate trong vertical-slice đạt; asset pipeline có mẫu chuẩn |
| 3. MVP Online | Pilot một linh vực: account, quest/progression, inventory, co-op, giao dịch giới hạn, guild cơ bản | Persistence/restart/reconnect đúng; solo sống được; load có giới hạn được đo |
| 4. World Expansion | Thêm linh vực, chọn origin, học truyền thừa thứ hai, vận chuyển và trade liên vùng | Nội dung mới qua review văn hóa; không tăng cấp vô hạn hoặc nhân đôi ngân sách |
| 5. Intercontinental War | Mặt trận có giới hạn, liên minh, tuyến tiếp tế, chiến dịch theo mùa | Chống snowball, phân ghép, recovery và tải mạng đã thử |

## MVP Online có gì

- Một linh vực đầu Vân Thủy; tài khoản có một nhân vật hoạt động trong pilot, quy tắc alt vẫn được simulation trước.
- Level 1–20 là đề xuất; quest, inventory, equipment, craft và skill progression được lưu server.
- Co-op 2–4 người; party invite, leave, reconnect; reward không phụ thuộc last-hit đơn thuần.
- Giao dịch vật tư thường qua một flow đơn giản và nguyên tử; ledger, lock và idempotency có kiểm tra.
- Guild cơ bản: tạo/tham gia, phân quyền tối thiểu, mục tiêu hợp tác không độc quyền sức mạnh.
- UI báo disconnect/sync rõ, cache có thể quản lý, mobile lifecycle đã đạt gate.
- Công cụ vận hành tối thiểu: telemetry, backup/restore thử được, safe economy config, báo cáo lỗi và xử lý lạm dụng.
- Pilot chốt mức concurrent/channel và concurrent toàn deployment từ load test, thay vì tuyên bố hỗ trợ một MMO lớn.

Guild war và mở nhiều origin không được đưa ngầm vào MVP khi chưa có tài liệu và bằng chứng tải tương ứng.

## Thứ tự online

1. Simulation authoritative và input sequencing cho 2 client.
2. Progression/inventory bền vững, reward commit đúng khi process lỗi.
3. Co-op encounter và reconnect với session ownership.
4. Craft/harvest có ledger và budget chung; giao dịch nguyên tử.
5. Quest, party/guild flow; quan sát pilot và kiểm tra solo.
6. Tăng giới hạn người chơi từng bước khi latency, tick time và database nằm trong ngân sách.

Giữ gameplay core tách renderer từ consumer đầu tiên. Chưa tách microservice hoặc xây orchestration nhiều vùng nếu một server module có thể phục vụ pilot đúng và dễ phục hồi.

## Mở rộng thế giới

Mỗi linh vực mới cần một xung đột riêng, một kỹ năng làm thay đổi lối chơi, một đặc trưng địa hình và ít nhất một boss đọc pattern. Không coi thay màu quái hoặc tăng level là mở rộng hoàn chỉnh.

Khi mở origin tự chọn, mọi xuất thân phải có tutorial, vật tư đầu và đường thoát khỏi hub tương đương. Việc sinh ra ở vùng khác không biến thành lựa chọn bắt buộc để có build mạnh nhất.

## Chiến dịch liên lục địa

- Trước tiên thử hai linh vực kết nối bằng một tuyến tiếp tế.
- Chia mục tiêu thành mặt trận giới hạn tác nhân; victory points được ghi nhận qua server, không cộng lại reward khi replay.
- Có mục tiêu solo/nhóm nhỏ song song raid; người chơi mới góp được mà không cần đồ tối đa.
- Kết quả mùa tạo danh hiệu, cosmetic và câu chuyện tiếp theo; hạn chế lợi thế kinh tế vĩnh viễn của bang thắng.
- Chống độc quyền tuyến vận chuyển: đường thay thế, thời hạn quyền kiểm soát và giới hạn thuế/chi phí đã cấu hình.

## Cổng quyết định thương mại

Monetization chỉ triển khai sau khi solo, progression và kinh tế được kiểm chứng. Đề xuất cosmetic và tùy biến thể hiện; không bán damage, stamina, hồi tài nguyên, quyền trade ưu tiên hoặc rút ngắn thời gian bắt buộc để mạnh hơn. Mọi cosmetic cần kiểm tra readability PvP.

Các giả thuyết về audience, retention và khả năng trả phí chưa có nghiên cứu thị trường trong bộ docs này. Pilot cần cung cấp bằng chứng trước quyết định vận hành lâu dài.
