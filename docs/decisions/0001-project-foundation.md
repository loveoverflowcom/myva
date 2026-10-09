# Quyết định 0001 — nền tảng MyVa

- **Ngày:** 2026-10-09.
- **Trạng thái:** yêu cầu sản phẩm đã chốt; chi tiết triển khai vẫn là draft.

## Bối cảnh

MyVa là dự án riêng cho MMORPG hành động 2D HD, với trải nghiệm chiến đấu chủ động, platforming và truyền thừa thần thoại đa văn hóa. Repository bắt đầu từ tài liệu để tránh xây công cụ hoặc nội dung lớn trước khi chứng minh gameplay.

## Quyết định

1. Tên dự án: **MyVa**. Tên tiếng Việt: **Thần Mạch**.
2. Nhánh khởi tạo là `develop`; đặt nhánh mặc định cùng tên. Không tạo `main` trong bước khởi tạo. PR công việc sau này lấy `develop` làm base.
3. **Lịch sử — lựa chọn engine đã bị [ADR 0003](0003-bevy-engine-adoption.md) thay thế:** Rust + Macroquad gameplay. Quyết định hiện hành dùng **Rust + Bevy**; Leptos web shell và Kotlin Compose Multiplatform mobile shell giữ nguyên. Native embedding phải được kiểm chứng; WebView không phải phương án hoàn thành yêu cầu mobile này.
4. Server sở hữu gameplay online, item, trade, reward và ngân sách tài nguyên. Client prediction không được mint item hoặc xác nhận giao dịch.
5. Hậu duệ/người kế thừa là nhân vật chính; không gán văn hóa hoặc vị trí thật của người chơi vào sức mạnh nhân vật.
6. Solo/cày chay có con đường tiến triển chiến đấu hoàn chỉnh. Doanh thu đề xuất từ cosmetic; không bán sức mạnh hoặc quota tài nguyên.
7. Tài liệu v0.1 và thông số thử là thiết kế, không phải trạng thái triển khai. Không tự chọn license phát hành hoặc giấy phép asset cho chủ dự án.

## Hệ quả

- Có thể phát triển nhiều linh vực sau khi một slice đạt gate.
- Phần kỹ thuật có rủi ro native embedding và networking phải xử lý sớm.
- Kinh tế cần simulator và ledger có invariant trước trade.
- World Bible phải tách sáng tác hư cấu khỏi truyền thuyết có nguồn.

## Khi nào xem xét lại

Thay đổi stack, platform, thương mại hóa sức mạnh hoặc quy tắc kinh tế cốt lõi cần một quyết định mới, nêu kết quả thử nghiệm và ảnh hưởng đến các tài liệu phụ thuộc. Thông số cân bằng có thể thay đổi trong cùng PR với bằng chứng đo.
