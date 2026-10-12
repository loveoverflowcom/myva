# Báo cáo thiết kế — thế giới, thần thoại và hệ nhân vật

> Đồng bộ [issue #18](https://github.com/loveoverflowcom/myva/issues/18) ngày **2026-10-12**, từ `develop` tại `e8a09f8`. Đây là bàn giao worldbuilding/docs; không viết gameplay code hoặc sản xuất asset mới.
> Quyết định hiện hành: [ADR 0001](../decisions/0001-project-foundation.md), [Bevy — ADR 0003](../decisions/0003-bevy-engine-adoption.md), [năm thế lực — ADR 0004](../decisions/0004-five-factions-lineage-hierarchy.md). [ADR 0002](../decisions/0002-lineages-world-structure.md) giữ đề xuất lịch sử và được đối chiếu từng mục.

## Cách đọc

- **Đã chốt:** năm nền tảng tôn giáo và trục phân chia trực hệ của #18; không có nghĩa đã triển khai gameplay.
- **DK:** dữ kiện có nguồn, có phạm vi truyền thống; không phải nguồn cho cơ chế game.
- **ST:** sáng tác MyVa, kể cả quan hệ trao sức mạnh/đặt tên kỹ năng.
- **GT / đề xuất chưa duyệt:** roster, nhánh, kỹ năng, thông số, lore và phạm vi ra mắt còn cần lựa chọn và kiểm chứng.

## Tóm tắt

Năm thế lực lấy nền tảng từ **Kitô giáo, Hồi giáo, Ấn Độ giáo, Phật giáo, Đạo giáo**. Phân tầng: **Thế lực → Nguồn bảo trợ → Trực hệ → Nhánh chiến đấu → Kit**. Linh vực/bản đồ là chiều địa lý độc lập; không khóa nguồn/kit theo nơi sinh, tộc người hoặc châu lục.

[Lineages](lineages.md) đề xuất **10 trực hệ ứng viên**, hai cho mỗi thế lực, mỗi trực hệ có hai hướng chiến đấu, một cơ chế đặc trưng hữu hạn, thuật ví dụ, điểm mạnh/yếu và cách đáp trả. Tên/roster/skill/quan hệ hư cấu chưa duyệt. Không ghép năm tên Long Lưu, Sơn Cốt, Phong Vũ, Lâm Trảo, Tơ Vọng với năm tôn giáo.

Giữ combat core, Bevy/Leptos/CMP, server authority, ngân sách tài nguyên và tải asset theo vùng. Graybox vẫn là Long Lưu baseline; slice #4 giữ Vân Thủy, ba map, ba kit, một boss theo gate. #18 không sản xuất năm roster hoặc mở PvP vào slice.

## Bản đồ tài liệu

| Mục | Nội dung hiện hành | Nguồn |
| --- | --- | --- |
| A | Quyết định có hiệu lực, phần thay thế và phần chưa duyệt | [ADR 0004](../decisions/0004-five-factions-lineage-hierarchy.md) |
| B | Vai trò nguồn và giới hạn nghiên cứu của năm nền tảng | [Nguồn năm thế lực](../research/five-factions-sources.md); [khảo sát cảnh quan/mô-típ cũ](../research/mythology-survey.md) |
| C | Năm tầng và các chiều độc lập | [Lineages §1](lineages.md#1-phân-tầng-khái-niệm), [glossary](glossary.md) |
| D | Danh mục ứng viên, skill/đối trọng GT, trạng thái kit cũ | [Lineages §4–6](lineages.md#4-danh-mục-trực-hệ-ứng-viên) |
| E | Luật combat chung; GT cân bằng; bàn giao #20 | [Combat](combat.md), [balance](combat-progression-balance.md), [hợp đồng kit](lineages.md#7-hợp-đồng-bàn-giao-cho-issue-20) |
| F | Địa lý độc lập nguồn, địa danh/bản đồ giữ lại | [World Bible](world-bible.md), [world atlas](world-atlas.md) |
| G | Cốt truyện ST, tổ chức xuyên thế lực, phản diện theo hành động | [Narrative](narrative.md) |
| H | Kinh tế không đổi theo số thế lực/nguồn | [Economy](economy.md), [monetization](monetization.md) |
| I | Review từng nguồn/nội dung; bỏ lệnh cấm bao trùm mâu thuẫn #18 | [Risk register](../research/risk-register.md) |
| J | Giới hạn slice/mở rộng và công việc còn mở | [Content roadmap](../production/content-roadmap.md), [#4 work-plan](../work-plan/040-vertical-slice.md), [#6 backlog](../work-plan/backlog/world-expansion.md) |

## A. Đánh giá Game Vision hiện tại

Đã giải quyết xung đột giữa mô hình cũ “năm dòng mạch xuyên văn hóa” và quyết định mới: giữ các kit để kiểm chứng cơ chế, đổi tầng phân loại lore. Vai trò chiến đấu và Mạch tính không thay thế nguồn thần linh/pháp hệ; tổ chức như Hắc Triều, Liên Bến, Nhà Bảng không được đếm thành năm thế lực tôn giáo.

Vân Thủy, Bến Lau, Rừng Bậc Nước, Đập Cổ và các địa danh hữu ích tiếp tục là không gian chơi. Song Nguyên thay Sa Đăng, cấp 30, bản ra mắt hai linh vực và số kit phát hành vẫn chưa duyệt. Học chéo theo quan hệ nguồn/trực hệ và thử thách, không bắt buộc xuất thân hoặc chuyển tới một vùng tôn giáo độc quyền.

Đại Mạch/Phân Dòng/Hợp Dòng có thể được giữ như đề xuất ST về thế giới game. Chúng không giải thích các tôn giáo ngoài đời là mảnh ký ức của một vị thần chung. Quan hệ ban phước, truyền pháp và thầy–trò của từng ứng viên được công bố là hư cấu, không trình bày như giáo lý thật.

## B–J. Tóm tắt các phần

- **Nguồn:** phân biệt Thiên Chúa/Allah với thiên thần, thánh và ngôn sứ; phân biệt thần/hóa thân, Phật/Bồ-tát và tiên. Nguồn có thật được nghiên cứu trực tiếp, chưa duyệt mức xuất hiện/tên hiển thị/hình tượng. [Hồ sơ nguồn](../research/five-factions-sources.md) ghi giới hạn nguồn và truy cập.
- **Combat:** mỗi thế lực có nhiều lối chơi; khắc chế mềm theo tầm, nhịp, hướng/tầng, tài nguyên và recovery. Không xếp hạng tôn giáo bằng sức mạnh hoặc số tín đồ; chưa có playtest cho roster ứng viên.
- **Narrative:** tổ chức chính trị/danh vọng có thể có thành viên nhiều thế lực. Phản diện có hành động cưỡng ép/độc quyền cụ thể; một tôn giáo không mặc định là phe ác.
- **Production:** mở vùng không tự thêm trực hệ, kit hoặc ngân sách. Các ý tưởng mở rộng của ADR 0002 là phương án để review, không trở thành yêu cầu slice.

## Quyết định cần chủ dự án duyệt

| Mã ADR 0002 | Trạng thái hiện tại | Việc còn lại |
| --- | --- | --- |
| D1 | Định nghĩa đã được thay bởi #18 | Áp năm tầng; lựa chọn UX/tạo nhân vật chưa chốt |
| D2 | Trục/roster mặc định cũ bị thay thế | Chọn ứng viên nguồn, trực hệ và ánh xạ lore cho kit; chưa chốt số kit ra mắt |
| D3 | Chưa duyệt | Định nghĩa bản ra mắt, cấp tối đa, số vùng, mùa và scope |
| D4 | Chưa duyệt | Linh vực thứ hai và đổi tên Sa Đăng/Song Nguyên |
| D5 | Chưa duyệt | Vũ trụ quan, Người Đo Mạch và lore chi tiết; tránh giải thích thay tôn giáo |
| D6 | Thay phần cấm toàn bộ nguồn đang thờ/chỉ dùng Cổ Linh | Review từng nguồn, mức xuất hiện, hình tượng và điều kiện xuất bản; không duyệt ngầm phần còn lại |
| D7 | Chưa duyệt | Một thuật khách, tương thích nguồn/trực hệ, thời điểm học và đổi |
| D8 | Bộ gate cụ thể chưa duyệt | Phương pháp/ngưỡng T1–T6 vẫn là GT; balance cần người thật |
| D9 | Phần bổ sung chưa duyệt | Nguyên tắc ADR 0001 giữ; phần bổ sung phải có quyết định riêng |

## Bàn giao issue #20

[Hợp đồng kit](lineages.md#7-hợp-đồng-bàn-giao-cho-issue-20) có trường nguồn/trực hệ/nhánh/kit, DK/ST, một cơ chế đặc trưng, 2 đòn + 3 thuật, đại thuật cùng ô, hit/block/whiff, chống trên không, solo boss, item có đánh đổi và kế hoạch test.

#20 phát triển vòng nội tại–đòn thường–thuật, Dấu Săn của Lâm Trảo, catalog item và ba ruleset; PR #18 chỉ tạo nền nguồn/lore. Luật chung theo #19/combat.md. Tiếp tục Long Lưu rồi hai kit slice theo gate; Lâm Trảo/Tơ Vọng và ứng viên tôn giáo chưa vào prototype đầu. PvP thật chờ gate authoritative/reconnect của #5.

## Đối chiếu tiêu chí issue #18

| Tiêu chí | Bằng chứng tài liệu |
| --- | --- |
| Đúng năm thế lực, không chia phe theo địa lý | README, GDD, World Bible, ADR 0004 |
| Phân biệt năm tầng và bản đồ | Lineages §1, glossary, world atlas |
| Ứng viên đủ năm nguồn, DK/ST và quan hệ rõ | Lineages §4, research/five-factions-sources |
| Đối chiếu D1–D9, không duyệt ngầm | Hai ADR 0002/0004 và bảng trên |
| Đa dạng lối chơi, đối trọng dự kiến | Hai trực hệ/hai nhánh mỗi thế lực; lineages §5; chưa tuyên bố cân bằng |
| Giữ địa danh/prototype/scope | Lineages §6, slice #4, backlog #6; diff chỉ Markdown |

## Kiểm tra local — 2026-10-12

- `cargo xtask preflight --changed-only`: format, links/anchors, native clippy/tests, replay/headless seed 1/7/12, economy và asset/report contracts đều PASS. Full preflight **BLOCKED** ở prerequisite web vì thiếu `wasm-bindgen` pin 0.2.129; các bước determinism native/WASM, WASM build và browser chưa chạy. `cargo xtask doctor` cũng báo thiếu Playwright. Không tự cài prerequisite hoặc ghi các bước chưa chạy là PASS.
- Rà consistency: đủ năm nền tảng trong các tài liệu đầu vào; 10 hồ sơ ứng viên, hai mỗi thế lực; đủ nguồn, quan hệ, nhánh, kỹ năng, mạnh/yếu và đáp trả. D1–D9 được đối chiếu riêng; không sửa source, test, config hoặc asset runtime.
- Link checker chỉ xác nhận links/anchors nội bộ. Nguồn ngoài được đối chiếu theo [hồ sơ nguồn](../research/five-factions-sources.md); giới hạn truy cập Tobit/84000 được ghi tại đó, không tuyên bố mọi URL ngoài đều HTTP PASS.

## Những gì báo cáo này không làm

Không triển khai roster/skill mới, không đổi combat/engine/economy hoặc tải asset. Không công bố balance, giấy phép asset, duyệt văn hóa hay pháp lý đã đạt. Lịch sử đề xuất 2026-10-09 giữ tại ADR 0002 và Git; trạng thái hiện hành theo ADR 0004.
