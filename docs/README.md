# Mục lục tài liệu MyVa — Thần Mạch

Phiên bản nền tảng: **v0.1 / 2026-10-09**. Ngôn ngữ thiết kế chính: tiếng Việt.

## Cách hiểu trạng thái

| Nhãn | Ý nghĩa |
| --- | --- |
| Yêu cầu đã chốt | Người dùng đã xác định; thay đổi phải được nêu rõ trong quyết định mới |
| Đề xuất v0.1 | Phương án để prototype, có thể thay đổi theo kết quả chơi thử |
| Gate | Điều kiện phải đạt trước khi chuyển sang bước phụ thuộc |
| Chưa kiểm chứng | Không được mô tả như tính năng đã triển khai |

Yêu cầu đã chốt: MyVa / Thần Mạch; hậu duệ hoặc người kế thừa; MMORPG hành động 2D HD; web, Android, iOS; Rust + Bevy, Leptos, CMP; mobile hướng native; solo và cày chay có đường tiến triển; phát hành tài nguyên bị giới hạn; server authoritative; nhánh đầu tiên và mặc định `develop`.

## Thiết kế

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Review](design/review.md) | Draft mạnh ở đâu, mâu thuẫn nào cần giải và ưu tiên nào giảm rủi ro? |
| [GDD](design/gdd.md) | Người chơi làm gì, vui ở đâu và vòng chơi được giới hạn thế nào? |
| [World Bible](design/world-bible.md) | Thần Mạch là gì, nhân vật muốn gì và linh vực có bản sắc ra sao? |
| [Combat](design/combat.md) | Di chuyển, combo, phòng thủ và đọc boss hoạt động thế nào? |
| [Progression & Social](design/progression-social.md) | Solo, bang hội, build và chiến dịch liên vùng cùng tồn tại thế nào? |
| [Economy](design/economy.md) | Tài nguyên hồi phục mà tồn kho, ngân sách phát hành và tiền tệ vẫn được kiểm soát thế nào? |

### Đề xuất v0.2 — thế giới, thần thoại và hệ nhân vật

Chờ duyệt qua [quyết định 0002](decisions/0002-lineages-world-structure.md); chưa sửa các tài liệu v0.1 ở trên.

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Báo cáo thiết kế](design/worldbuilding-report.md) | Vision hiện tại thiếu gì, mâu thuẫn ở đâu, và cần chủ dự án quyết định điều gì? (mục A–J) |
| [Glossary](design/glossary.md) | Mỗi thuật ngữ nghĩa là gì, và tồn tại để phục vụ việc chơi nào? |
| [Lineages](design/lineages.md) | "Hệ nhân vật" được phân tầng thế nào; năm truyền thừa ra mắt khác nhau ra sao? |
| [Combat, progression & balance](design/combat-progression-balance.md) | Truyền thừa đổi combat thế nào, học kỹ năng liên vùng ra sao, và kiểm chứng cân bằng bằng gì? |
| [World atlas](design/world-atlas.md) | Các linh vực nối nhau thế nào; từng bản đồ có gì? |
| [Narrative](design/narrative.md) | Thần Mạch là gì, ai là phản diện, câu chuyện và chiến dịch tiến triển ra sao? |
| [Monetization](design/monetization.md) | Kinh tế bản ra mắt mở rộng thế nào và kiếm tiền mà không bán sức mạnh ra sao? |

## Nghiên cứu

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Mythology survey](research/mythology-survey.md) | Thần thoại và văn hóa nào có thể dùng, ở mức nào, với nguồn nào? |
| [Risk register](research/risk-register.md) | Rủi ro thiết kế, văn hóa, bản quyền, kỹ thuật, pháp lý là gì; duyệt văn hóa theo quy trình nào? |

## Kỹ thuật và sản xuất

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Architecture](technical/architecture.md) | Ai sở hữu state, native integration cần chứng minh gì và giao dịch được phục hồi ra sao? |
| [Gameplay foundation](technical/gameplay-foundation.md) | Adapter ECS gọi luật lõi thế nào, lệnh/sự kiện/ID ra sao, và headless/replay/WASM đã chứng minh gì? (D04) |
| [Combat graybox](reports/combat-graybox.md) | Trận Bevy chơi được tới đâu; keyboard/gamepad/touch, replay và số đo nào đã có, playtest nào còn thiếu? |
| [Web feasibility](reports/web-feasibility.md) | Spike Bevy + Leptos chạy/kiểm thử gì, còn thiếu browser/device nào? |
| [Native feasibility](reports/native-feasibility.md) | Standalone build, CMP embedding và Android/iOS thật có bằng chứng hoặc blocker gì? |
| [Assets & Performance](technical/assets-performance.md) | Tải gì, giữ gì trong RAM/VRAM và đo hiệu năng thế nào? |
| [Vertical slice](production/vertical-slice.md) | Bản thử đầu tiên phải chơi được gì và khi nào đủ điều kiện tiến tiếp? |
| [Roadmap](production/roadmap.md) | Thứ tự từ thiết kế đến MMORPG nhiều linh vực là gì? |
| [Content roadmap](production/content-roadmap.md) | Truyền thừa, linh vực, boss và chương truyện được thêm vào ở giai đoạn nào? (đề xuất v0.2) |
| [QA](production/qa.md) | Kiểm tra combat, networking, kinh tế, đồ họa và mobile bằng chứng gì? |
| [Work plan](work-plan/README.md) | Công việc tiếp theo có thể giao và review riêng là gì? |
| [Planning & Kanban](work-plan/planning.md) | Issue, project và điều kiện chuyển trạng thái được theo dõi ở đâu? |
| [Bevy adoption — ADR 0003](decisions/0003-bevy-engine-adoption.md) | Engine hiện hành, phiên bản, ownership và gate Web/Android/iOS là gì? |
| [Foundation decision](decisions/0001-project-foundation.md) | Quy tắc tên, nhánh, scope và nguồn sự thật là gì? |
| [Decision 0002 (đề xuất)](decisions/0002-lineages-world-structure.md) | Truyền thừa, linh vực thứ hai, vũ trụ quan và chính sách văn hóa được đề xuất chốt thế nào? |

## Quy tắc cập nhật

- Luật gameplay được mô tả trong GDD và tài liệu chuyên môn; work-plan chỉ giữ mục tiêu, phụ thuộc và ranh giới review.
- Khi thay đổi một tham số chung như số skill hoặc level cap, sửa các tài liệu tham chiếu trong cùng thay đổi.
- Phân biệt tên hư cấu do dự án tạo với nhân vật/truyền thuyết có nguồn văn hóa thực tế. Không gắn một sáng tác mới thành “truyền thuyết cổ”.
- Nguồn kỹ thuật và ngày kiểm tra được đặt gần phần liên quan. Thông số thử nghiệm phải kèm điều kiện đo.
- Một gate không đạt phải tạo kết luận hoặc điều chỉnh thiết kế trước khi chuyển sang bước phụ thuộc.
