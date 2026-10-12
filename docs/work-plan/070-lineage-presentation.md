# PR3 — Presentation metadata và asset mapping

**Trạng thái:** TODO, phiên3; phụ thuộc [PR2](060-kit-registry-loadout.md). Chưa triển khai UI hoặc asset mapping trong PR1.

## Kết quả và lý do

Client hiển thị tên/provenance/trạng thái và ánh xạ asset bằng identity ổn định, giúp presentation phát triển mà không sửa combat/replay bytes.

## Phạm vi và acceptance

- Đọc metadata ngoài `Kit`/`World`; giữ legacy labels hashed và replay IDs.
- Hiển thị rõ fixture chưa ánh xạ và ứng viên chưa duyệt; không giả là roster chính thức, không khóa theo quê hương/bản đồ.
- Mapping asset có fallback/lỗi rõ, giữ luật tải theo vùng và provenance/license; không đổi damage/economy/skill rules.
- Kiểm tra browser/native phù hợp và parity; report phần chưa chạy, review độc lập rồi normal push/draft PR. Không merge hoặc tự triển khai phiên này từ PR1.

## Rủi ro còn mở

Tên/hình tượng/roster còn chờ quyết định. UI layout và asset contracts phải lấy bằng chứng từ consumer thực tế ở head PR2.
