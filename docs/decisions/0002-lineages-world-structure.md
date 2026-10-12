# Quyết định 0002 — truyền thừa, linh vực và cấu trúc thế giới

- **Ngày:** 2026-10-09.
- **Trạng thái gốc (2026-10-09):** **Đề xuất — chờ chủ dự án duyệt.**
- **Đối chiếu 2026-10-12:** [ADR 0004](0004-five-factions-lineage-hierarchy.md) ghi quyết định #18: thay định nghĩa D1, thay trục/roster D2 và phần cấm bao trùm/chỉ dùng Cổ Linh của D6. D3–D5 và D7–D9 vẫn chưa duyệt; phần còn lại của D6 cũng không được duyệt ngầm. Bảng bên dưới giữ **đề xuất lịch sử**, không phải luật hiện hành. [Bản trước #18 tại develop e8a09f8](https://github.com/loveoverflowcom/myva/blob/e8a09f8/docs/decisions/0002-lineages-world-structure.md) giữ nguyên để đối chiếu.
- **Không thay đổi** bất kỳ yêu cầu đã chốt nào của [quyết định 0001](0001-project-foundation.md).

## Bối cảnh

Đợt nghiên cứu Worldbuilding, Mythology & Character Systems ([báo cáo](../design/worldbuilding-report.md)) phát hiện: chữ "hệ" đang gộp nhiều khái niệm; ba truyền thừa của slice chưa đủ phủ vai trò cho bản ra mắt; chưa có định nghĩa "bản ra mắt" giữa MVP và World Expansion; vũ trụ quan chưa trả lời vì sao nhiều thần thoại cùng tồn tại; tên "Sa Đăng" mâu thuẫn chính nguyên tắc chống khuôn mẫu của World Bible.

## Các quyết định đề xuất — bản gốc, đọc cùng ADR 0004

| Mã | Đề xuất | Lý do chính | Chi tiết |
| --- | --- | --- | --- |
| D1 | Tách sáu tầng: linh vực, thế lực, truyền thừa (= hệ nhân vật), nhánh, vai trò, Mạch tính. Linh vực và thế lực không ảnh hưởng chỉ số | Tránh gộp xuất thân với lớp nhân vật; giữ quy tắc không bonus theo nơi sinh | [lineages §1](../design/lineages.md#1-phân-tầng-khái-niệm) |
| D2 | Năm truyền thừa ra mắt: Long Lưu, Sơn Cốt, Phong Vũ (giữ) + Lâm Trảo, Tơ Vọng (mới), dựa trên năm dòng mạch xuyên văn hóa | Phủ đủ vai trò, không healer bắt buộc; không gắn truyền thừa với châu lục | [kit cũ — lineages §6](../design/lineages.md#6-giữ-kit-hiện-tại-và-mở-rộng-theo-gate) |
| D3 | "Bản ra mắt 1.0" = 2 linh vực, cấp 1–30, năm truyền thừa, mùa chiến dịch đầu với một tuyến tiếp tế; là bước đầu của giai đoạn 4 trong roadmap | Lấp khoảng trống giữa MVP và World Expansion; trần 30 khớp `L_cap` của economy | [content-roadmap](../production/content-roadmap.md) |
| D4 | Linh vực thứ hai là Song Nguyên (cảm hứng Lưỡng Hà–Iran), đổi tên từ Sa Đăng; xem lại tên Thạch Phong, Hồng Nguyên khi các linh vực đó vào nghiên cứu | Nối chủ đề nước–ký ức; tránh khuôn mẫu sa mạc–đèn thần | [world-atlas §3](../design/world-atlas.md#3-danh-mục-linh-vực-và-chọn-linh-vực-thứ-hai) |
| D5 | Vũ trụ quan: Thần Mạch là ký ức chảy; Đại Mạch → Phân Dòng → Hợp Dòng; phản diện Người Đo Mạch và Mạch Bạ; Ngưỡng Mạch, Hóa Tướng, Thông Mạch | Trả lời chín câu hỏi nền; lý do trong truyện cho MMO, tài nguyên hữu hạn và tiến triển ngang | [narrative §2–3](../design/narrative.md#2-trả-lời-các-câu-hỏi-nền) |
| D6 | Không đưa thần linh đang được thờ, tri thức thiêng, biểu tượng quốc gia làm nhân vật/boss/loot; thực thể linh thiêng trong truyện là Cổ Linh hư cấu | Rủi ro văn hóa và pháp lý; tiền lệ phản đối | [risk-register §3](../research/risk-register.md#3-danh-sách-không-dùng-mức-0) |
| D7 | Truyền thừa phụ cung cấp tối đa 1 thuật khách; không nhận nội tại; dùng bộ thi triển chung | Giữ bản sắc, kiểm soát cân bằng và chi phí animation | [combat-progression-balance §5](../design/combat-progression-balance.md#5-thuật-khách--học-chéo-trực-hệ) |
| D8 | Không công bố truyền thừa nào là cân bằng khi chưa qua bộ T1–T6; harness headless là một phần của combat graybox | Biến "cân bằng" thành tuyên bố kiểm chứng được | [combat-progression-balance §11](../design/combat-progression-balance.md#11-phương-pháp-kiểm-chứng) |
| D9 | Giữ thương mại hóa của 0001; vật phẩm mua bằng tiền thật không giao dịch được; tuân thủ Nghị định 147/2024 ở thị trường Việt Nam | Chặn đường vòng tiền thật → chợ → sức mạnh; ràng buộc pháp lý thực tế | [monetization §2, §7](../design/monetization.md#2-hòa-giải-nạp-tiền-với-quyết-định-0001) |

Trạng thái từng mục hiện tại nằm trong [bảng đối chiếu ADR 0004](0004-five-factions-lineage-hierarchy.md#đối-chiếu-adr-0002-theo-từng-mục). Chủ dự án có thể duyệt từng mục còn mở; mục bị bác sẽ được ghi lại kèm lý do và phương án thay thế.

## Hệ quả khi được duyệt

Danh sách dự kiến ban đầu dưới đây chỉ áp dụng cho mục còn mở nếu được duyệt. R-08 đã được điều chỉnh theo #18; không dùng để áp cả D1–D9 trong một lần. Danh sách lịch sử:

- [GDD](../design/gdd.md) §7 (thêm dải cấp 21–30 cho bản ra mắt), §13 (mốc bản ra mắt), §15 (đóng các câu hỏi đã trả lời).
- [World Bible](../design/world-bible.md) §1 (vũ trụ quan), §3 (bảng linh vực, đổi tên), §5 (Ngàn, Nguyệt, Người Đo Mạch), §6 (truyền thừa xuyên văn hóa), §8 (gắn cung truyện với chương).
- [Progression & Social](../design/progression-social.md) §2–3 (lộ trình bản ra mắt, thuật khách).
- [Combat](../design/combat.md) §8 (đòn cơ bản của Sơn Cốt, Phong Vũ); Thoát Mạch chỉ thêm sau khi graybox xác nhận.
- [Economy](../design/economy.md) §3 (thêm Sợi Lau, Muối Mạch sau khi simulator có số liệu).
- [Roadmap](../production/roadmap.md) (liên kết content-roadmap), [README](../../README.md), [mục lục docs](../README.md).

## Không thay đổi

Tên MyVa / Thần Mạch; người chơi là hậu duệ/người kế thừa; MMORPG hành động 2D HD; web, Android, iOS; Rust + Bevy (engine cập nhật theo [ADR 0003](0003-bevy-engine-adoption.md)), Leptos, CMP; server authoritative; solo và cày chay có đường tiến triển; không bán sức mạnh hoặc quota tài nguyên; nhánh `develop`. Phạm vi vertical slice (3 bản đồ, 3 kit, 1 boss) giữ nguyên.

## Khi nào xem xét lại

- Graybox hoặc spike cho thấy Tơ Vọng hoặc Lâm Trảo không khả thi trên cảm ứng hay mạng.
- Mô phỏng T1–T6 cho thấy cấu trúc khắc chế không đạt ngưỡng sau hai vòng chỉnh.
- Cố vấn văn hóa phản đối cảm hứng của Song Nguyên hoặc một truyền thừa.
- Tra cứu nhãn hiệu phát hiện xung đột với tên đã chọn.
