# MyVa — Thần Mạch

**MMORPG hành động 2D HD về những người kế thừa sức mạnh thần thoại.**

Người chơi trưởng thành từ một linh vực, học cách chiến đấu, khám phá các truyền thừa khác và tham gia những chiến dịch nối nhiều lục địa. Thế giới có chỗ cho người chơi solo, người cày chay và bang hội; sức mạnh nhân vật phải đi cùng kỹ năng điều khiển.

> Dễ bắt đầu, khó thành thạo; cày chay vẫn mạnh, kỹ năng tạo khác biệt, thế giới mở rộng trong một nền kinh tế có giới hạn và được kiểm chứng.

## Trạng thái dự án

- **Giai đoạn:** thiết kế tiền sản xuất; tài liệu draft v0.1, ngày 2026-10-09.
- **Hiện có:** review ý tưởng, GDD, World Bible, luật chiến đấu, kinh tế, kiến trúc đề xuất và kế hoạch kiểm chứng.
- **Chưa có:** runtime gameplay, server, asset thành phẩm hoặc tích hợp Macroquad native vào CMP đã được xác nhận.
- **Nhánh đầu tiên và mặc định:** `develop`. Các nhánh công việc và PR sau này lấy `develop` làm base.
- **Tên tiếng Anh:** MyVa. **Tên tiếng Việt:** Thần Mạch.

Các con số trong tài liệu là giả thuyết thiết kế cần đo bằng prototype. Không coi chúng là kết quả benchmark hoặc lời hứa vận hành.

## Nền tảng và stack

| Phần | Định hướng |
| --- | --- |
| Gameplay và render | Rust + Macroquad |
| Web | Gameplay WebAssembly; Leptos quản lý shell, tài khoản và nội dung ngoài trận |
| Android, iOS | Kotlin Compose Multiplatform quản lý shell; kiểm chứng Macroquad native qua cầu nối và bề mặt render |
| Online | Server authoritative; simulation tách khỏi renderer |
| Persistence | PostgreSQL là đề xuất ban đầu; xác nhận ở prototype persistence |
| Assets | Atlas, tải theo nhu cầu, cache bền vững và ngân sách bộ nhớ; skeletal runtime cần thử nghiệm riêng |

## Đọc tài liệu

1. [Review draft và quyết định cần kiểm chứng](docs/design/review.md).
2. [GDD](docs/design/gdd.md), [World Bible](docs/design/world-bible.md), [chiến đấu](docs/design/combat.md).
3. [Tiến triển, solo và bang hội](docs/design/progression-social.md), [kinh tế](docs/design/economy.md).
4. [Kiến trúc](docs/technical/architecture.md), [assets và hiệu năng](docs/technical/assets-performance.md).
5. [Vertical slice](docs/production/vertical-slice.md), [roadmap](docs/production/roadmap.md), [QA](docs/production/qa.md).
6. [Hàng đợi công việc](docs/work-plan/README.md) và [quyết định nền tảng](docs/decisions/0001-project-foundation.md).

[Mục lục đầy đủ](docs/README.md) phân biệt yêu cầu đã chốt, phương án đề xuất và câu hỏi còn mở.

Theo dõi công việc tại [Thần Mạch — Planning](https://github.com/users/loveoverflowcom/projects/3) và [Thần Mạch — Kanban](https://github.com/users/loveoverflowcom/projects/4/views/1). [Hướng dẫn planning](docs/work-plan/planning.md) liên kết sáu issue, phụ thuộc và điều kiện chuyển cột.

## Điểm bắt đầu

Tầm nhìn dài hạn gồm nhiều linh vực và lựa chọn xuất thân. Bản thử đầu tiên chỉ có **Vân Thủy**, một linh vực hư cấu lấy cảm hứng từ cảnh quan Đông Nam Á: ba bản đồ, ba truyền thừa, một boss và một vòng chơi 20–30 phút. Kiểm chứng cảm giác chiến đấu, khả năng tích hợp native và luật tài nguyên trước khi mở rộng nội dung online.

Không khởi tạo một workspace với hàng chục crate hoặc service khi chưa có prototype chứng minh ranh giới cần thiết. Các sơ đồ kỹ thuật là thiết kế dự kiến, không mô tả mã nguồn đang tồn tại.
