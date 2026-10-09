# Mục lục tài liệu MyVa — Thần Mạch

Phiên bản nền tảng: **v0.1 / 2026-10-09**. Ngôn ngữ thiết kế chính: tiếng Việt.

## Cách hiểu trạng thái

| Nhãn | Ý nghĩa |
| --- | --- |
| Yêu cầu đã chốt | Người dùng đã xác định; thay đổi phải được nêu rõ trong quyết định mới |
| Đề xuất v0.1 | Phương án để prototype, có thể thay đổi theo kết quả chơi thử |
| Gate | Điều kiện phải đạt trước khi chuyển sang bước phụ thuộc |
| Chưa kiểm chứng | Không được mô tả như tính năng đã triển khai |

Yêu cầu đã chốt: MyVa / Thần Mạch; hậu duệ hoặc người kế thừa; MMORPG hành động 2D HD; web, Android, iOS; Rust + Macroquad, Leptos, CMP; mobile hướng native; solo và cày chay có đường tiến triển; phát hành tài nguyên bị giới hạn; server authoritative; nhánh đầu tiên và mặc định `develop`.

## Thiết kế

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Review](design/review.md) | Draft mạnh ở đâu, mâu thuẫn nào cần giải và ưu tiên nào giảm rủi ro? |
| [GDD](design/gdd.md) | Người chơi làm gì, vui ở đâu và vòng chơi được giới hạn thế nào? |
| [World Bible](design/world-bible.md) | Thần Mạch là gì, nhân vật muốn gì và linh vực có bản sắc ra sao? |
| [Combat](design/combat.md) | Di chuyển, combo, phòng thủ và đọc boss hoạt động thế nào? |
| [Progression & Social](design/progression-social.md) | Solo, bang hội, build và chiến dịch liên vùng cùng tồn tại thế nào? |
| [Economy](design/economy.md) | Tài nguyên hồi phục mà tồn kho, ngân sách phát hành và tiền tệ vẫn được kiểm soát thế nào? |

## Kỹ thuật và sản xuất

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Architecture](technical/architecture.md) | Ai sở hữu state, native integration cần chứng minh gì và giao dịch được phục hồi ra sao? |
| [Assets & Performance](technical/assets-performance.md) | Tải gì, giữ gì trong RAM/VRAM và đo hiệu năng thế nào? |
| [Vertical slice](production/vertical-slice.md) | Bản thử đầu tiên phải chơi được gì và khi nào đủ điều kiện tiến tiếp? |
| [Roadmap](production/roadmap.md) | Thứ tự từ thiết kế đến MMORPG nhiều linh vực là gì? |
| [QA](production/qa.md) | Kiểm tra combat, networking, kinh tế, đồ họa và mobile bằng chứng gì? |
| [Work plan](work-plan/README.md) | Công việc tiếp theo có thể giao và review riêng là gì? |
| [Foundation decision](decisions/0001-project-foundation.md) | Quy tắc tên, nhánh, scope và nguồn sự thật là gì? |

## Quy tắc cập nhật

- Luật gameplay được mô tả trong GDD và tài liệu chuyên môn; work-plan chỉ giữ mục tiêu, phụ thuộc và ranh giới review.
- Khi thay đổi một tham số chung như số skill hoặc level cap, sửa các tài liệu tham chiếu trong cùng thay đổi.
- Phân biệt tên hư cấu do dự án tạo với nhân vật/truyền thuyết có nguồn văn hóa thực tế. Không gắn một sáng tác mới thành “truyền thuyết cổ”.
- Nguồn kỹ thuật và ngày kiểm tra được đặt gần phần liên quan. Thông số thử nghiệm phải kèm điều kiện đo.
- Một gate không đạt phải tạo kết luận hoặc điều chỉnh thiết kế trước khi chuyển sang bước phụ thuộc.
