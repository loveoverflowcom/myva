# Quyết định 0004 — năm thế lực và phân tầng trực hệ

- **Quyết định sản phẩm:** 2026-10-11; ghi nhận trong repo: 2026-10-12.
- **Trạng thái:** **Đã chốt về năm nền tảng và trục phân chia**, theo [issue #18](https://github.com/loveoverflowcom/myva/issues/18). Tên phe trong game, nguồn/roster cụ thể, nhánh, kỹ năng và balance vẫn là **đề xuất chưa duyệt**.
- **Phạm vi:** worldbuilding và tài liệu; không triển khai năm thế lực trong gameplay.

## Bối cảnh

[ADR 0002](0002-lineages-world-structure.md) đề xuất năm dòng mạch xuyên văn hóa và chỉ dùng Cổ Linh hư cấu. Chủ dự án sau đó chọn năm nền tảng tôn giáo và yêu cầu phân chia theo nguồn thần linh/pháp hệ. Cần thay đúng phần xung đột, giữ lịch sử và các quyết định còn mở.

## Quyết định có hiệu lực

1. Năm thế lực lấy nền tảng từ **Kitô giáo, Hồi giáo, Ấn Độ giáo, Phật giáo, Đạo giáo**. Đây là nhãn nguồn thiết kế; tên tổ chức trong game chưa chốt. Không thay chúng bằng năm nguyên tố, châu lục hoặc vai trò chiến đấu.
2. Dùng phân tầng **Thế lực → Nguồn bảo trợ/truyền thừa cụ thể → Trực hệ → Nhánh chiến đấu → Kit**. Một thế lực có nhiều trực hệ, mỗi trực hệ có thể có nhiều nhánh và kit.
3. Trực hệ phải trả lời **nhận từ ai, bằng cách nào, qua quan hệ gì**. Quan hệ có thể là ban phước, được chọn, truyền pháp, thầy–trò hoặc hậu duệ nếu hồ sơ nguồn phù hợp; không mặc định huyết thống cho mọi nguồn.
4. Linh vực/bản đồ là chiều độc lập: xuất thân, địa hình, nhiệm vụ, di chuyển và tải asset. Nhiều thế lực cùng hiện diện trong một vùng; một thế lực hoạt động ở nhiều vùng. Nơi sinh, ngoại hình, quốc tịch không khóa trực hệ/kit hoặc cộng chỉ số.
5. Phân biệt dữ kiện nguồn với sáng tác MyVa. Ba Ngôi không là ba thần độc lập; Allah không có con; Muhammad là ngôn sứ/sứ giả, không phải thần; Phật không phải thần sáng thế. Phật/Bồ-tát, thần, thiên thần, thánh, ngôn sứ và tiên không hoán đổi vai trò. Chi tiết và nguồn tại [hồ sơ nghiên cứu](../research/five-factions-sources.md).
6. Được nghiên cứu tên/thực thể tôn giáo có thật làm nguồn trực hệ. Không áp lệnh cấm bao trùm khiến mọi nguồn bị thay bằng Cổ Linh vô danh. Mức xuất hiện, tên hiển thị và hình tượng cần quyết định riêng; người chơi vẫn là người kế thừa, không mặc định điều khiển chính thực thể ấy.
7. Không tôn giáo nào mặc định là phe ác, mạnh hơn vì nhiều tín đồ, hoặc chỉ làm hỗ trợ bắt buộc. Xung đột có hành động cụ thể, hợp tác và khác biệt nội bộ; khắc chế qua cơ chế, không qua hệ số sát thương giữa tôn giáo.

## Đối chiếu ADR 0002 theo từng mục

| Mục cũ | Trạng thái sau #18 | Cách áp dụng |
| --- | --- | --- |
| D1 | **Thay định nghĩa** | Giữ ích lợi tách địa lý/nhân vật; dùng năm tầng ở trên. Vai trò và Mạch tính là thuộc tính, tổ chức chính trị là chiều riêng |
| D2 | **Bị thay thế về trục phân chia và roster mặc định** | Long Lưu, Sơn Cốt, Phong Vũ, Lâm Trảo, Tơ Vọng là kit/mẫu cơ chế cũ; không phải năm thế lực hoặc roster ra mắt đã duyệt. Chưa ánh xạ sang tôn giáo |
| D3 | **Chưa duyệt** | Hai linh vực, cấp 30, số kit và mùa ra mắt vẫn là giả thuyết phạm vi |
| D4 | **Chưa duyệt** | Song Nguyên là ứng viên thay Sa Đăng; giữ địa danh/bản đồ hữu ích, không đổi tên ngầm |
| D5 | **Chưa duyệt** | Ký ức chảy, Đại Mạch/Phân Dòng/Hợp Dòng, Người Đo Mạch là lore ST; không giải thích tôn giáo có thật là mảnh vỡ của một chân lý game |
| D6 | **Thay phần cấm toàn bộ nguồn đang được thờ và phần chỉ dùng Cổ Linh** | Dùng hồ sơ từng nguồn, tách DK/ST, review tên/hình tượng. Đề xuất cũ về tri thức hạn chế, biểu tượng quốc gia và loot không được coi là đã duyệt cùng #18 |
| D7 | **Chưa duyệt** | Một thuật khách là đề xuất; học qua nguồn/trực hệ tương thích, không yêu cầu xuất thân hoặc đến một vùng tôn giáo độc quyền |
| D8 | **Chưa duyệt bộ gate cụ thể** | Không tuyên bố cân bằng khi chưa playtest; ngưỡng/bộ T1–T6 vẫn là GT, không được #18 duyệt ngầm |
| D9 | **Chưa duyệt phần bổ sung** | Nguyên tắc kinh tế/thương mại đã chốt của ADR 0001 giữ nguyên; phần bổ sung ở ADR 0002 không được duyệt bởi #18 |

## Bàn giao và ranh giới

[Lineages](../design/lineages.md) là danh mục ứng viên và hợp đồng bàn giao cho [#20](https://github.com/loveoverflowcom/myva/issues/20); tên nguồn ở đó là tên tham chiếu, mọi trực hệ/nhánh/skill mới đều chưa duyệt. #20 sở hữu kit/trang bị/ruleset; [combat.md](../design/combat.md) và #19 sở hữu luật combat chung. #18 không thêm item, PvP, nút hoặc timing mới.

Giữ [ADR 0001](0001-project-foundation.md), [Bevy — ADR 0003](0003-bevy-engine-adoption.md), Leptos, CMP, server authority, kinh tế hữu hạn, solo/cày chay. Giữ #2 graybox một kit Long Lưu; #4 slice Vân Thủy, ba bản đồ, ba kit, một boss theo gate hiện hành. Giữ ID kit/replay/asset, cơ chế combat và tải asset theo vùng. Thêm thế lực không nhân ngân sách tài nguyên hoặc số kit phải sản xuất.

## Những quyết định còn mở

- Tên riêng, mức hiển thị nguồn thật, hình tượng và người phản biện từng truyền thống.
- Danh sách trực hệ được chọn; quan hệ truyền thừa cụ thể, đổi trực hệ/thế lực, học chéo.
- Ánh xạ lore cho kit hiện tại; số kit phát hành và lịch mở roster.
- Các đề xuất D3–D9 chưa duyệt của ADR 0002; thay đổi phải có record riêng.

Review ứng viên không tự nâng trạng thái thành đã duyệt. Khi duyệt một nguồn hoặc kit, ghi người quyết định, ngày, phiên bản, phần được chọn và liên kết bằng chứng; kiểm chứng balance là bước riêng.
