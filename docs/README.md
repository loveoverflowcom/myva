# Mục lục tài liệu MyVa — Thần Mạch

Phiên bản nền tảng: **v0.1 / 2026-10-09**; đồng bộ #18: **2026-10-12**. Ngôn ngữ thiết kế chính: tiếng Việt.

## Cách hiểu trạng thái

| Nhãn | Ý nghĩa |
| --- | --- |
| Yêu cầu đã chốt | Người dùng đã xác định; thay đổi phải được nêu rõ trong quyết định mới |
| Đề xuất v0.1 | Phương án để prototype, có thể thay đổi theo kết quả chơi thử |
| Gate | Điều kiện phải đạt trước khi chuyển sang bước phụ thuộc |
| Chưa kiểm chứng | Không được mô tả như tính năng đã triển khai |

Yêu cầu đã chốt: MyVa / Thần Mạch; hậu duệ hoặc người kế thừa; MMORPG hành động 2D HD; web, Android, iOS; Rust + Bevy, Leptos, CMP; mobile hướng native; solo và cày chay có đường tiến triển; phát hành tài nguyên bị giới hạn; server authoritative; nhánh đầu tiên và mặc định `develop`. [ADR 0004](decisions/0004-five-factions-lineage-hierarchy.md) ghi năm thế lực lấy nền tảng Kitô giáo, Hồi giáo, Ấn Độ giáo, Phật giáo, Đạo giáo và trục **Thế lực → Nguồn bảo trợ → Trực hệ → Nhánh chiến đấu → Kit**; địa lý độc lập. Tên/roster/skill mới chưa duyệt và chưa triển khai.

## Thiết kế

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Review](design/review.md) | Draft mạnh ở đâu, mâu thuẫn nào cần giải và ưu tiên nào giảm rủi ro? |
| [GDD](design/gdd.md) | Người chơi làm gì, vui ở đâu và vòng chơi được giới hạn thế nào? |
| [World Bible](design/world-bible.md) | Thần Mạch là gì, nhân vật muốn gì và linh vực có bản sắc ra sao? |
| [Mạch công/thủ — #19](reports/combat-mach.md) | Đại thuật, Thoát Mạch, CC budget, feedback và giới hạn bằng chứng tự động |
| [Combat](design/combat.md) | Di chuyển, combo, phòng thủ và đọc boss hoạt động thế nào? |
| [Progression & Social](design/progression-social.md) | Solo, bang hội, build và chiến dịch liên vùng cùng tồn tại thế nào? |
| [Economy](design/economy.md) | Tài nguyên hồi phục mà tồn kho, ngân sách phát hành và tiền tệ vẫn được kiểm soát thế nào? |

### Thế giới và hệ nhân vật — quyết định hiện hành và đề xuất

#18 đã thay trục phân chia của [quyết định 0002](decisions/0002-lineages-world-structure.md). Các đề xuất còn lại của ADR 0002 giữ trạng thái riêng; không coi toàn bộ v0.2 đã được duyệt.

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Báo cáo thiết kế](design/worldbuilding-report.md) | Vision hiện tại thiếu gì, mâu thuẫn ở đâu, và cần chủ dự án quyết định điều gì? (mục A–J) |
| [Glossary](design/glossary.md) | Mỗi thuật ngữ nghĩa là gì, và tồn tại để phục vụ việc chơi nào? |
| [Lineages](design/lineages.md) | Năm tầng nhân vật, 10 trực hệ ứng viên, điểm mạnh/yếu/khắc chế và hợp đồng bàn giao #20 |
| [Combat, progression & balance](design/combat-progression-balance.md) | Truyền thừa đổi combat thế nào, học chéo trực hệ ra sao, và kiểm chứng cân bằng bằng gì? |
| [World atlas](design/world-atlas.md) | Các linh vực nối nhau thế nào; từng bản đồ có gì? |
| [Narrative](design/narrative.md) | Thần Mạch là gì, ai là phản diện, câu chuyện và chiến dịch tiến triển ra sao? |
| [Monetization](design/monetization.md) | Kinh tế bản ra mắt mở rộng thế nào và kiếm tiền mà không bán sức mạnh ra sao? |

## Nghiên cứu

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Nguồn năm thế lực](research/five-factions-sources.md) | Phân biệt thần, thiên thần, ngôn sứ, Phật/Bồ-tát và tiên; nguồn nào hỗ trợ từng ứng viên #18? |
| [Mythology survey](research/mythology-survey.md) | Thần thoại và văn hóa nào có thể dùng, ở mức nào, với nguồn nào? |
| [Risk register](research/risk-register.md) | Rủi ro thiết kế, văn hóa, bản quyền, kỹ thuật, pháp lý là gì; duyệt văn hóa theo quy trình nào? |

## Kỹ thuật và sản xuất

| Tài liệu | Câu hỏi được giải quyết |
| --- | --- |
| [Architecture](technical/architecture.md) | Ai sở hữu state, native integration cần chứng minh gì và giao dịch được phục hồi ra sao? |
| [Repository layout](technical/repository-layout.md) | Mã nguồn hiện nằm ở đâu, apps/packages/prototypes sở hữu gì và dependency đi theo hướng nào? |
| [xtask build contract](architecture/xtask-build-contract.md) | Entrypoint development, CI, process ownership và changed-only dependency impact |
| [xtask verification](reports/xtask-verification.md) | Những command đã thực thi, PASS/FAIL/BLOCKED/NOT_RUN và giới hạn native feasibility |
| [Repository refactor report](reports/repository-layout.md) | Mapping trước/sau, consumer compatibility và kiểm chứng nào đã chạy hoặc còn bị chặn? |
| [Gameplay foundation](technical/gameplay-foundation.md) | Adapter ECS gọi luật lõi thế nào, lệnh/sự kiện/ID ra sao, và headless/replay/WASM đã chứng minh gì? (D04) |
| [Faction/lineage catalog](technical/faction-lineage-catalog.md) | Identity, provenance, validator và Team độc lập; boundary không đổi combat/replay |
| [Kit registry/loadout](technical/kit-registry-loadout.md) | Implementation/equip permission, exact revision/hash, khóa round và replay v3/v4 compatibility |
| [Equipment registry/build rules](technical/equipment-registry.md) | Catalog ba item GT, budget/stacking, LockedLoadout và pure headless prototypes; combat effects còn TODO |
| [Equipment evidence](reports/equipment-registry.md) | Checks thực sự chạy, Rust NOT_RUN và review/publication boundary |
| [PR2 registry/loadout evidence](reports/kit-registry-loadout.md) | Local tests/parity, legacy byte comparison, review và bàn giao PR3 |
| [PR1 domain evidence](reports/faction-lineage-domain.md) | Regression replay legacy, kiểm thử local và gate còn bị chặn |
| [Combat graybox](reports/combat-graybox.md) | Trận Bevy chơi được tới đâu; keyboard/gamepad/touch, replay và số đo nào đã có, playtest nào còn thiếu? |
| [Web feasibility](reports/web-feasibility.md) | Spike Bevy + Leptos chạy/kiểm thử gì, còn thiếu browser/device nào? |
| [Native feasibility](reports/native-feasibility.md) | Standalone build, CMP embedding và Android/iOS thật có bằng chứng hoặc blocker gì? |
| [Asset pipeline](technical/asset-pipeline.md) | Atlas, pivot/padding/scale, clip, manifest và validator asset 2D được quy ước thế nào? (D05-A) |
| [D05-A report](reports/asset-pipeline.md) | Fixture, validator và loader Bevy đã chạy gì trên native/web; còn gì BLOCKED/NOT_RUN? |
| [D05-B preparation](reports/asset-streaming-performance.md) | Planner/report contract độc lập; runtime và benchmark còn bị chặn ở đâu? |
| [Assets & Performance](technical/assets-performance.md) | Tải gì, giữ gì trong RAM/VRAM và đo hiệu năng thế nào? |
| [Vertical slice](production/vertical-slice.md) | Bản thử đầu tiên phải chơi được gì và khi nào đủ điều kiện tiến tiếp? |
| [Roadmap](production/roadmap.md) | Thứ tự từ thiết kế đến MMORPG nhiều linh vực là gì? |
| [Content roadmap](production/content-roadmap.md) | Truyền thừa, linh vực, boss và chương truyện được thêm vào ở giai đoạn nào? (đề xuất v0.2) |
| [QA](production/qa.md) | Kiểm tra combat, networking, kinh tế, đồ họa và mobile bằng chứng gì? |
| [Work plan](work-plan/README.md) | Công việc tiếp theo có thể giao và review riêng là gì? |
| [Planning & Kanban](work-plan/planning.md) | Issue, project và điều kiện chuyển trạng thái được theo dõi ở đâu? |
| [Bevy adoption — ADR 0003](decisions/0003-bevy-engine-adoption.md) | Engine hiện hành, phiên bản, ownership và gate Web/Android/iOS là gì? |
| [Foundation decision](decisions/0001-project-foundation.md) | Quy tắc tên, nhánh, scope và nguồn sự thật là gì? |
| [Five factions — ADR 0004](decisions/0004-five-factions-lineage-hierarchy.md) | Quyết định #18 có hiệu lực ở đâu, thay D1/D2/D6 thế nào, phần nào chưa duyệt? |
| [Decision 0002 (lịch sử, duyệt từng phần)](decisions/0002-lineages-world-structure.md) | Truyền thừa, linh vực thứ hai, vũ trụ quan và chính sách văn hóa được đề xuất chốt thế nào? |

[CharacterCatalog và selected-kit asset mapping](technical/character-catalog-assets.md) · [Kiểm chứng PR3](reports/character-catalog-assets.md): metadata chung Leptos/Bevy, placeholder/unresolved status và lifecycle theo KitId.

## Quy tắc cập nhật

- Luật gameplay được mô tả trong GDD và tài liệu chuyên môn; work-plan chỉ giữ mục tiêu, phụ thuộc và ranh giới review.
- Khi thay đổi một tham số chung như số skill hoặc level cap, sửa các tài liệu tham chiếu trong cùng thay đổi.
- Phân biệt tên hư cấu do dự án tạo với nhân vật/truyền thuyết có nguồn văn hóa thực tế. Không gắn một sáng tác mới thành “truyền thuyết cổ”.
- Nguồn kỹ thuật và ngày kiểm tra được đặt gần phần liên quan. Thông số thử nghiệm phải kèm điều kiện đo.
- Một gate không đạt phải tạo kết luận hoặc điều chỉnh thiết kế trước khi chuyển sang bước phụ thuộc.
