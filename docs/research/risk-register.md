# Sổ rủi ro — thiết kế, văn hóa, bản quyền và kỹ thuật

> Đề xuất v0.2, 2026-10-09. Mục **I** của [báo cáo thiết kế](../design/worldbuilding-report.md). Mở rộng [World Bible §9](../design/world-bible.md#9-cổng-nghiên-cứu-và-duyệt-văn-hóa) và các rủi ro kỹ thuật ở [architecture.md](../technical/architecture.md).
> Đây là danh sách để theo dõi, không phải kết luận pháp lý. Các mục pháp lý cần luật sư có chuyên môn xác nhận.

## 1. Cách đọc

- **Khả năng / Tác động:** T (thấp), TB (trung bình), C (cao) — đánh giá định tính của nhóm thiết kế ở thời điểm viết.
- **Gate:** cột mốc phải có bằng chứng giảm rủi ro trước khi đi tiếp ([roadmap](../production/roadmap.md), [content-roadmap](../production/content-roadmap.md)).
- Khi một rủi ro xảy ra hoặc được loại bỏ, cập nhật dòng tương ứng kèm bằng chứng; không xóa lịch sử.

## 2. Sổ rủi ro

### D — Thiết kế

| ID | Rủi ro | KN | TĐ | Biện pháp | Gate |
| --- | --- | --- | --- | --- | --- |
| D-01 | Năm truyền thừa và hai linh vực vượt năng lực sản xuất | C | C | Thêm dần: slice 3 → MVP 3–4 → ra mắt 5; nhánh địa phương thay cho lớp mới; ngân sách asset từng bản đồ | Mỗi giai đoạn |
| D-02 | Truyền thừa chỉ khác nhau về ngoại hình | TB | C | Cơ chế đặc trưng, bộ pháp, ma trận khắc chế; gate G2 của slice; bộ T1, T4 | Slice, MVP |
| D-03 | Tơ Vọng (tiếng vọng, neo) quá phức tạp, khó đọc, khó điều khiển cảm ứng | C | TB | Spike sớm (R-05); phương án rút gọn: 1 neo, tiếng vọng chỉ cho 1 thuật | Trước MVP |
| D-04 | Hoang Sinh của Lâm Trảo áp đảo PvP | TB | TB | Tỷ lệ thấp hơn ở PvP, giảm dần; T5 tìm vòng lặp hồi máu | Graybox truyền thừa |
| D-05 | Thuật khách làm các build giống nhau | TB | TB | 1 ô, không nhận nội tại, danh sách cặp cấm; theo dõi tỷ lệ chọn > 40% | Ra mắt |
| D-06 | Khắc chế mềm trở thành khắc chế cứng | TB | C | Ngưỡng 45–55%; công cụ cho bên bất lợi; T1, T6 | MVP, ra mắt |
| D-07 | Thoát Mạch làm combo vô nghĩa hoặc vô dụng | TB | TB | Chi phí 50 Mạch + hồi chiêu; graybox và T5 | Graybox |
| D-08 | Lựa chọn cốt truyện làm phasing tốn chi phí | TB | TB | ≤ 3 trạng thái; không đổi hình học; danh sách cờ trạng thái | Chương 1 |
| D-09 | Đồng bộ cấp làm người cấp cao thấy vô nghĩa khi giúp bạn | T | TB | Giữ tương quan trang bị; thưởng Truyền thụ | MVP |
| D-10 | Chủ đề đập nước bị đọc như bình luận chính trị về đập thủy điện Mekong có thật | TB | C | Không dùng tên sông, nước, đập, công ty thật; Hắc Triều đa quốc tịch; xin ý kiến cố vấn khu vực | Trước công bố chương 1 |
| D-11 | Sự kiện dài vượt giới hạn thời gian chơi của người dưới 18 tuổi | C | TB | Phiên ≤ 45 phút; cảnh báo thời gian; không bắt đầu instance khi không đủ giờ | MVP (thị trường Việt Nam) |
| D-12 | Cốt truyện dài nhưng nhiệm vụ vẫn là "giết N quái" | TB | TB | Tỷ lệ nhiệm vụ ở [narrative.md §6](../design/narrative.md#6-cấu-trúc-nhiệm-vụ); review từng nhiệm vụ theo kỹ năng nó dạy | Chương 1 |

### V — Văn hóa

| ID | Rủi ro | KN | TĐ | Biện pháp | Gate |
| --- | --- | --- | --- | --- | --- |
| V-01 | Thần linh đang được thờ thành nhân vật, boss, loot | TB | C | Chính sách mức 0; thực thể trong truyện là Cổ Linh hư cấu; tiền lệ phản đối SMITE (2012) | Mọi nội dung |
| V-02 | Dùng tri thức thiêng, hạn chế của người bản địa (Dreaming, skinwalker, katsina, haka, tā moko) | TB | C | Danh sách không dùng (§3); linh vực liên quan chỉ khi có đối tác dài hạn | Trước mọi linh vực liên quan |
| V-03 | Một châu lục bị trình bày như một nền văn hóa | C | C | Linh vực dựa trên cộng đồng cụ thể; truyền thừa xuyên văn hóa; review tên | Mỗi linh vực |
| V-04 | Song Nguyên rơi vào khuôn mẫu Đông phương học (sa mạc, đèn thần, phản diện theo dân tộc) | TB | C | Đổi tên Sa Đăng; cảnh quan sông–đầm–vườn; cố vấn khu vực; checklist khuôn mẫu | Trước sản xuất Song Nguyên |
| V-05 | Biểu tượng bị nhóm cực đoan chiếm dụng (rune Othala, valknot, sonnenrad, chữ vạn) | TB | C | Danh sách biểu tượng cấm trong UI, mỹ phẩm, emblem bang hội; kiểm duyệt emblem do người chơi tạo | Trước Sương Hải; ngay khi có emblem bang hội |
| V-06 | Biểu tượng quốc gia làm kẻ thù hoặc vật phẩm (Garuda, lamassu, chim Lạc, trống đồng) | TB | C | Chỉ làm đồng minh, cơ quan, trang trí có duyệt | Mỗi linh vực |
| V-07 | Truyền thuyết quốc gia Việt Nam (Hùng Vương, Lạc Long Quân–Âu Cơ, Thánh Gióng, Sơn Tinh) bị dùng sai — nhạy cảm trong nước và với phê duyệt nội dung | TB | C | Mức 1–2; không nhại; không đưa nhân vật thờ phụng vào game | Slice trở đi |
| V-08 | Cộng đồng đang sống (người đầm lầy Iraq, Chăm, Bali, Ifugao) bị trình bày như "nguyên thủy" hoặc trang trí | TB | C | Cộng đồng hư cấu có phẩm giá, hiện đại; cố vấn; không chụp/sao chép kiến trúc thiêng cụ thể | Mỗi bản đồ liên quan |
| V-09 | Giới thiệu sáng tác mới như "truyền thuyết cổ" | TB | TB | Nhãn DK/ST trong tài liệu và trong game (nhật ký ghi "truyền thuyết của Vân Thủy", không ghi "truyền thuyết Việt Nam") | Mọi nội dung |
| V-10 | Tín ngưỡng Việt (Đạo Mẫu, lên đồng) bị sân khấu hóa | T | C | Không dùng | — |
| V-11 | Bản đồ, ranh giới có thật gây tranh cãi lãnh thổ | T | C | Thế giới hoàn toàn hư cấu; không dùng hình dạng bờ biển, đảo có thật | Mọi bản đồ thế giới, UI |
| V-12 | Thiếu ngân sách và thời gian cho cố vấn văn hóa | C | C | Đưa vào kế hoạch sản xuất như hạng mục bắt buộc (R-01) | Giai đoạn 0 |

### B — Bản quyền và nhãn hiệu

| ID | Rủi ro | KN | TĐ | Biện pháp | Gate |
| --- | --- | --- | --- | --- | --- |
| B-01 | Tên truyền thừa, linh vực trùng nhãn hiệu hoặc tên lớp nhân vật nổi tiếng | TB | TB | Tra cứu R-02 (USPTO, WIPO Global Brand Database, Cục SHTT Việt Nam, store) trước khi công bố | Trước công bố |
| B-02 | Thiết kế giống chuyển thể hiện đại (Thor của Marvel, Maui của Disney, Kratos, thần trong SMITE, Tôn Ngộ Không trong các game gần đây) | TB | C | Tham khảo nguồn gốc, không tham khảo hình chuyển thể; review thiết kế có ghi nguồn tham khảo | Concept art |
| B-03 | Trích bản dịch hiện đại còn bản quyền (Edda, Shahnameh, Popol Vuh...) | TB | TB | Tự diễn giải; nếu trích thì dùng bản dịch đã hết bảo hộ hoặc xin phép | Viết lore |
| B-04 | Giấy phép runtime animation: runtime chính thức của Spine yêu cầu mỗi người dùng có license Spine Editor; binding Rust (rusty_spine) chuyển dịch từ runtime C nên nhiều khả năng chịu cùng điều khoản | TB | TB | Đánh giá trong spike animation; so sánh với rig tự viết hoặc runtime clean-room | Spike animation |
| B-05 | Font tiếng Việt, nhạc, âm thanh không rõ giấy phép | TB | TB | Danh mục asset có license ([CONTRIBUTING](../../CONTRIBUTING.md)) | Mọi asset |
| B-06 | Marketing dùng tên Mortal Kombat, Mega Man, Ngọc Rồng Online | TB | TB | Chỉ nói về cơ chế bằng lời của MyVa; không dùng tên, logo, hình của IP khác | Marketing |

Nguồn: [Spine Runtimes License](https://esotericsoftware.com/spine-runtimes-license), [rusty_spine](https://docs.rs/rusty_spine).

### K — Kỹ thuật

| ID | Rủi ro | KN | TĐ | Biện pháp | Gate |
| --- | --- | --- | --- | --- | --- |
| K-01 | Khối lượng animation: 5 truyền thừa × 2 dáng cơ thể × mỹ phẩm | C | C | Spike pipeline (R-04): rig module với attachment, bộ thi triển chung cho thuật khách, VFX phủ thay cho biến hình | Trước MVP |
| K-02 | Tiếng vọng (đòn trễ) và sợi neo khó làm authoritative, dễ nhân hit khi reconnect | TB | C | Entity có ID do server mô phỏng; T8; replay | Graybox Tơ Vọng |
| K-03 | Móc neo, leo vách, nhảy kép gây lệch prediction và xuyên địa hình | TB | TB | Điểm neo hợp lệ do server kiểm; collision bảo thủ; reconciliation mượt | Graybox |
| K-04 | Kết quả mô phỏng cân bằng không tái lập được | TB | TB | Chạy trên một target; ghi seed, hash cấu hình, commit | Harness |
| K-05 | Hạ tầng bot B2/B3 tốn thời gian hơn lợi ích | TB | TB | Làm theo bậc; B3 tùy chọn | Slice |
| K-06 | Hiệu năng mobile với sợi, tiếng vọng, hạt hiệu ứng ở channel 50 người | TB | C | LOD hiệu ứng của người khác; tiếng vọng dùng shader rẻ; ngân sách ở [assets-performance §8](../technical/assets-performance.md#8-budget-ban-đầu-để-thử-nghiệm) | Benchmark thiết bị |
| K-07 | Ngắm neo, đổi bên trên cảm ứng khó chính xác | C | TB | Ngắm tự động theo quy tắc rõ; thử với 10 người chơi | Spike R-05 |
| K-08 | Nội dung (thuật, bản đồ, quest) viết cứng trong code | TB | TB | Dữ liệu hóa thuật và bản đồ; validator | Graybox |
| K-09 | Văn bản Việt–Anh dài khác nhau làm vỡ UI | TB | T | Ngân sách độ dài; font đủ dấu | MVP |

### P — Kinh doanh và pháp lý

| ID | Rủi ro | KN | TĐ | Biện pháp | Gate |
| --- | --- | --- | --- | --- | --- |
| P-01 | Doanh thu chỉ từ mỹ phẩm không đủ | TB | C | Đo ở pilot; phương án trong khuôn 0001 ([monetization.md §10](../design/monetization.md#10-tính-bền-vững-doanh-thu)) | Pilot |
| P-02 | Mua bán tiền thật, bot cày tài nguyên | C | C | Khóa giao dịch, vật phẩm trả phí ràng buộc, phát hiện đồ thị giao dịch | MVP |
| P-03 | Không tuân thủ Nghị định 147/2024 (xác thực điện thoại, thời gian chơi người dưới 18, giấy phép G1) | TB | C | Đánh giá pháp lý R-07; đưa vào thiết kế tài khoản | Trước MVP Online tại Việt Nam |
| P-04 | Thay đổi quy định trong tương lai (dự thảo sửa đổi 2026) | TB | TB | Thiết kế giới hạn thời gian cấu hình được | Liên tục |
| P-05 | Cộng đồng coi tiện ích trả phí là pay-to-win | TB | TB | Không bán tiện ích ảnh hưởng kinh tế ở bản ra mắt; khảo sát pilot | Ra mắt |

## 3. Danh sách không dùng (mức 0)

Không dùng dưới bất kỳ hình thức nào (tên, hình, cốt truyện, cơ chế, mỹ phẩm) trừ khi có quyết định mới kèm đồng thuận của cộng đồng liên quan:

- Thần linh đang được thờ phụng làm nhân vật chơi được, boss, quái hoặc vật phẩm (Hindu, Phật giáo, Thần đạo, Orisha, Đạo Mẫu, thờ Hùng Vương...).
- Câu chuyện Dreaming và Rắn Cầu Vồng của người Aboriginal; skinwalker (Navajo); katsina; tri thức nghi lễ hạn chế.
- Haka *Ka Mate* và các haka khác; hình xăm tā moko.
- Bảo vật hoàng gia và vật thiêng đang được giữ (ví dụ bộ ba thần khí Nhật Bản); cổng torii làm đạo cụ tùy tiện.
- Nhân vật lịch sử là anh hùng dân tộc (Hai Bà Trưng, Bà Triệu, Thánh Gióng...) làm nhân vật trong game.
- Biểu tượng bị chiếm dụng bởi nhóm thù hận (rune Othala, valknot, sonnenrad, chữ vạn ở mọi dạng) trong UI, mỹ phẩm, emblem.
- Hình dạng địa lý, ranh giới có thật trên bản đồ thế giới.

## 4. Quy trình duyệt văn hóa

Mở rộng [World Bible §9](../design/world-bible.md#9-cổng-nghiên-cứu-và-duyệt-văn-hóa) thành quy trình có bước, vai trò và đầu ra.

| Bước | Việc | Ai làm | Đầu ra |
| --- | --- | --- | --- |
| 1. Hồ sơ | Lập hồ sơ cho mỗi nội dung có tham chiếu thực tế (mẫu dưới) | Người thiết kế | Hồ sơ trong repo |
| 2. Phân mức | Gán mức 0–3 theo [khảo sát §1](mythology-survey.md#thang-mức-sử-dụng) | Người thiết kế + người phụ trách văn hóa | Mức được ghi rõ |
| 3. Review nội bộ | Kiểm tra khuôn mẫu, phe ác mặc định, nguồn, nhãn DK/ST | Người phụ trách văn hóa | Danh sách sửa |
| 4. Cố vấn | Cố vấn có chuyên môn hoặc thành viên cộng đồng liên quan, **có trả thù lao** | Cố vấn | Ý kiến bằng văn bản; quyền đề nghị bỏ |
| 5. Playtest nhạy cảm | Người chơi thuộc cộng đồng liên quan xem nội dung trong game | Nhóm QA + cố vấn | Ghi nhận phản ứng |
| 6. Ký duyệt | Ghi người duyệt, ngày, phiên bản, nguồn | Chủ dự án | Trạng thái "đã nghiên cứu và duyệt" |
| 7. Sau phát hành | Kênh phản hồi; cam kết sửa hoặc gỡ trong thời hạn công bố | Vận hành | Lịch sử thay đổi |

**Mẫu hồ sơ nội dung:**

```text
Tên nội dung (trong game):
Loại: nhân vật / sinh vật / địa điểm / vật phẩm / cơ chế / câu chuyện / mỹ phẩm
Tham chiếu thực tế: cộng đồng, nơi chốn, thời kỳ cụ thể (không ghi "châu Á", "châu Phi")
Nguồn: [S#] và hạng nguồn
Phần dữ kiện (DK) / phần sáng tác (ST):
Mức sử dụng: 0 / 1 / 2 / 3, lý do
Tri thức thiêng hoặc hạn chế? Biểu tượng quốc gia? Cộng đồng đang sống?
Cơ chế chơi có biến biểu tượng thành loot, trophy, quái farm không?
Người review, cố vấn, ngày:
Quyết định và điều kiện:
```

Hồ sơ này cũng là tư liệu cho hồ sơ phê duyệt nội dung trò chơi G1 tại Việt Nam ([monetization.md §7](../design/monetization.md#7-bảo-vệ-người-chơi-và-tuân-thủ)).

## 5. Cần kiểm chứng trước từng gate

| Gate | Phải có bằng chứng về |
| --- | --- |
| Duyệt ADR 0002 | Chủ dự án chấp nhận phân tầng khái niệm, năm truyền thừa, linh vực thứ hai; tra cứu nhãn hiệu sơ bộ |
| Graybox | Thoát Mạch, Hoang Sinh, Vọng/Neo không phá luật combat (T5, T8); điều khiển cảm ứng Tơ Vọng |
| Vertical slice | Ba truyền thừa khác biệt (G2); cố vấn văn hóa Đông Nam Á cho Vân Thủy |
| MVP Online | Pipeline animation chốt; tuân thủ Nghị định 147 trong thiết kế tài khoản; chống tài khoản phụ trong simulator |
| Bản ra mắt | T1–T6 đạt hoặc có giải trình; hồ sơ văn hóa Song Nguyên đã ký duyệt; tên đã tra nhãn hiệu |
