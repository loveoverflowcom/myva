# Báo cáo thiết kế — thế giới, thần thoại và hệ nhân vật

> **Đề xuất v0.2, 2026-10-09.** Giai đoạn Research & Game Design: chưa viết gameplay code, chưa sản xuất asset.
> Báo cáo được viết từ góc nhìn các vai trò Creative Director, Systems Designer, Narrative Designer, Cultural Researcher, Combat Designer và Economy Designer, trên nền tài liệu v0.1. Đây là bản đề xuất để chủ dự án và cố vấn chuyên môn review, không thay thế ý kiến của họ. Các quyết định đã chốt ở [quyết định 0001](../decisions/0001-project-foundation.md) được ưu tiên hơn mọi đề xuất mới.

## Cách đọc

| Nhãn | Ý nghĩa |
| --- | --- |
| **DK** | Dữ kiện có nguồn, liệt kê trong [khảo sát §13](../research/mythology-survey.md#13-nguồn-tham-khảo) |
| **GT** | Giả thuyết thiết kế, cần graybox, mô phỏng hoặc playtest |
| **ST** | Sáng tác mới của MyVa, không giới thiệu như truyền thuyết có thật |
| **⚠** | Nhạy cảm văn hóa, tôn giáo, pháp lý — phải qua [quy trình duyệt](../research/risk-register.md#4-quy-trình-duyệt-văn-hóa) |

## Tóm tắt

1. **"Hệ nhân vật" = truyền thừa**, tách khỏi linh vực (xuất thân), thế lực, nhánh, vai trò và Mạch tính. Xuất thân không cộng chỉ số.
2. **Năm truyền thừa ra mắt** dựa trên năm dòng mạch xuyên văn hóa: Long Lưu (Thủy), Sơn Cốt (Thạch), Phong Vũ (Phong) — giữ từ slice — cùng **Lâm Trảo** (Sinh, áp sát hồi phục) và **Tơ Vọng** (Ký, khống chế/hỗ trợ bằng tiếng vọng và sợi neo).
3. **Mỗi truyền thừa lấy cảm hứng từ một mô-típ lặp lại ở nhiều nền văn hóa độc lập** (núi chống nước, chim thần, người-thú, sợi dệt số mệnh...), nên không gom một châu lục thành một lớp nhân vật.
4. **Thần Mạch là ký ức chảy.** Đại Mạch bị Phân Dòng chia thành các linh vực và đang Hợp Dòng trở lại; đó là lý do các thần thoại giống nhau, lý do có nhiều người thức tỉnh, và lý do các linh vực phải liên minh.
5. **Phản diện chính là Người Đo Mạch**, người muốn ghép **Mạch Bạ** — sổ ghi trữ lượng mạch — để phân bổ tập trung và xóa ký ức "trái phép". Mạch Bạ cũng là cách trò chơi giải thích tài nguyên hữu hạn.
6. **Bản ra mắt 1.0** = Vân Thủy + **Song Nguyên** (cảm hứng Lưỡng Hà–Iran, đổi tên từ "Sa Đăng"), cấp 1–30, mùa chiến dịch đầu tiên nối hai linh vực.
7. **Học kỹ năng liên vùng** qua truyền thừa phụ: tối đa một thuật khách, không nhận nội tại, dùng bộ thi triển chung.
8. **Cân bằng là giả thuyết** cho tới khi qua harness headless với bot nhiều bậc (T1–T6). Không truyền thừa nào được tuyên bố là đã cân bằng.
9. **Thương mại hóa giữ nguyên 0001**: mỹ phẩm và dịch vụ; vật phẩm mua bằng tiền thật không giao dịch được; tuân thủ Nghị định 147/2024 ở Việt Nam.
10. **Không đưa thần linh đang được thờ, tri thức thiêng, biểu tượng quốc gia** thành nhân vật, boss hay vật phẩm.

## Bản đồ tài liệu

| Mục | Nội dung | Tài liệu |
| --- | --- | --- |
| A | Đánh giá Game Vision hiện tại | Báo cáo này, [phần A](#a-đánh-giá-game-vision-hiện-tại) |
| B | Bảng nghiên cứu thần thoại và văn hóa | [research/mythology-survey.md](../research/mythology-survey.md) |
| C | So sánh phương án phân chia hệ nhân vật | [lineages.md §1–3](lineages.md#2-so-sánh-phương-án-phân-chia-hệ-nhân-vật) |
| D | Năm hệ nhân vật cho bản ra mắt | [lineages.md §4–6](lineages.md#4-năm-truyền-thừa-cho-bản-ra-mắt) |
| E | Combat, progression và cân bằng | [combat-progression-balance.md](combat-progression-balance.md) |
| F | Thế giới, bản đồ, vùng khám phá | [world-atlas.md](world-atlas.md) |
| G | Cốt truyện, phản diện, chiến dịch | [narrative.md](narrative.md) |
| H | Kinh tế và thương mại hóa | [monetization.md](monetization.md) |
| I | Rủi ro thiết kế, văn hóa, bản quyền, kỹ thuật | [research/risk-register.md](../research/risk-register.md) |
| J | Roadmap triển khai | [production/content-roadmap.md](../production/content-roadmap.md) |
| — | Thuật ngữ | [glossary.md](glossary.md) |
| — | Quyết định đề xuất | [decisions/0002](../decisions/0002-lineages-world-structure.md) |

---

## A. Đánh giá Game Vision hiện tại

### A1. Những nguyên tắc giữ nguyên

Từ [quyết định 0001](../decisions/0001-project-foundation.md), [GDD](gdd.md), [World Bible](world-bible.md) và [review](review.md):

- Người chơi là **con người kế thừa** sức mạnh, không đóng vai thần; có giới hạn, sai lầm, nghĩa vụ.
- **Linh vực là địa lý hư cấu**; không coi châu lục là một nền văn hóa; không bonus theo nơi sinh hay chủng tộc.
- **Solo và cày chay có tuyến hoàn chỉnh**; bang hội thêm chiều sâu, không khóa tiến triển.
- **Kỹ năng quan trọng, chỉ số có giới hạn**: dải ±15% từ trang bị trong PvE cùng tier; PvP cạnh tranh chuẩn hóa.
- **Loadout 2 đòn cơ bản + 3 thuật**, đại thuật không thêm nút; mobile 8 nút.
- **Tài nguyên hữu hạn**, một ngân sách toàn thế giới, server authoritative.
- **Doanh thu từ mỹ phẩm**, không bán sức mạnh hay quota.
- **Phạm vi theo gate**: slice Vân Thủy 3 bản đồ, 3 truyền thừa, 1 boss trước mọi mở rộng.

Đánh giá chung: nền v0.1 **mạnh ở kỷ luật phạm vi, kinh tế và kỹ thuật**; phần thế giới và hệ nhân vật mới ở mức khung. Đợt này chủ yếu lấp khung đó, không đổi hướng sản phẩm.

### A2. Khoảng trống

| # | Khoảng trống | Hệ quả nếu để nguyên | Xử lý trong v0.2 |
| --- | --- | --- | --- |
| G1 | Chưa có bảng thuật ngữ; "truyền thừa" vừa là "kỹ năng của một vùng" ([GDD §3](gdd.md#3-các-trụ-cột-có-thể-kiểm-chứng)) vừa là lựa chọn độc lập với xuất thân ([World Bible §2](world-bible.md#2-những-quy-tắc-giữ-thế-giới-nhất-quán)) | Thiết kế và người chơi hiểu "hệ" theo nhiều nghĩa | [glossary](glossary.md), phân tầng sáu khái niệm |
| G2 | Thần Mạch chỉ được mô tả, chưa trả lời vì sao nhiều thần thoại cùng tồn tại, vì sao có nhiều người kế thừa cùng lúc, điều gì xảy ra khi vượt giới hạn | Cốt truyện khó dài hơi; MMO thiếu lý do cho dân số đông | [narrative §2](narrative.md#2-trả-lời-các-câu-hỏi-nền) |
| G3 | Phản diện chỉ là tổ chức; không có người dẫn dắt hay bí ẩn dài hạn | Nhiệm vụ dễ thành chuỗi đánh trạm lặp lại | Người Đo Mạch, Mạch Bạ, năm bí ẩn có lịch trả lời |
| G4 | Danh mục linh vực thiếu Nam Á (yêu cầu nghiên cứu nêu rõ), châu Phi, Thái Bình Dương; Mesoamerica gộp vào "Bắc Mỹ" | Nghiên cứu bỏ sót vùng lớn | [khảo sát](../research/mythology-survey.md) phủ 9 khu vực |
| G5 | Chưa định nghĩa "bản ra mắt" giữa MVP và World Expansion | Không biết cần bao nhiêu hệ, vùng, cấp khi ra mắt | Bản ra mắt 1.0 ([content-roadmap](../production/content-roadmap.md)) |
| G6 | Ba truyền thừa không phủ vai trò khống chế/hỗ trợ và áp sát | Co-op và PvP thiếu chiều sâu; dễ đồng nhất build | Thêm Lâm Trảo, Tơ Vọng; tầng vai trò |
| G7 | Học kỹ năng liên vùng có trần ô, nhưng chưa có luật giữ bản sắc và chưa tính chi phí animation | Build đồng nhất; chi phí animation nhân theo số tổ hợp | Luật thuật khách + bộ thi triển chung |
| G8 | Chưa phân biệt cơ chế theo PvE / PvP / bang hội; chưa thiết kế chiến tranh bang hội | Cân bằng một chế độ làm hỏng chế độ khác | [bảng cơ chế theo nội dung](combat-progression-balance.md#6-cơ-chế-theo-loại-nội-dung), Tranh Trạm |
| G9 | Có telemetry nhưng chưa có phương pháp mô phỏng cân bằng | "Cân bằng" không kiểm chứng được | Harness headless, bot B0–B3, bộ T1–T8 |
| G10 | Chưa xét quy định game Việt Nam (xác thực điện thoại, giới hạn giờ chơi người dưới 18, giấy phép G1) | Thiết kế tài khoản, sự kiện phải làm lại | [monetization §7](monetization.md#7-bảo-vệ-người-chơi-và-tuân-thủ) |
| G11 | Tạo nhân vật (dáng cơ thể, trang phục) và mỹ phẩm chưa tính vào chi phí animation | Mô hình doanh thu mỹ phẩm có thể không sản xuất nổi | Rủi ro K-01, spike R-04 |

### A3. Mâu thuẫn và điểm lệch

| # | Điểm lệch | Đề xuất |
| --- | --- | --- |
| C1 | Tên **"Sa Đăng"** (cát + đèn) gợi sa mạc và đèn thần Aladdin, trong khi chính World Bible ghi "không đồng nhất khu vực với sa mạc"; truyện Aladdin là lớp được thêm ở châu Âu thế kỷ 18 | Đổi thành Song Nguyên ([world-atlas §3](world-atlas.md#đổi-tên-sa-đăng)) |
| C2 | **"Hồng Nguyên"** (đồng đỏ) cho Australia gợi khuôn mẫu "vùng đỏ trung tâm"; **"Thạch Phong"** trùng hình vị với Phong Vũ, Sơn Cốt | Xem lại khi các linh vực đó vào nghiên cứu |
| C3 | Yêu cầu nghiên cứu nói "hậu duệ hoặc người kế thừa"; World Bible nói "người bình thường vừa thức tỉnh" | Hậu duệ = con cháu của mọi cộng đồng ký Giao Ước Phân Dòng; sức mạnh truyền qua thực hành → cả hai đều đúng |
| C4 | Mortal Kombat 3 được nêu làm cảm hứng combo, nhưng chuỗi định sẵn của MK3 không ngắt được — trái quy tắc "combo dài phải có cửa thoát" của combat.md | Mượn sự dễ đọc của chuỗi định sẵn, giữ cửa thoát; thêm Thoát Mạch (GT) |
| C5 | Yêu cầu cho phép "phát triển hệ thống nạp tiền"; 0001 chốt không bán sức mạnh | 0001 ưu tiên; nạp tiền chỉ cho mỹ phẩm và dịch vụ ([monetization §2](monetization.md#2-hòa-giải-nạp-tiền-với-quyết-định-0001)) |
| C6 | Mỗi linh vực ứng với một khu vực địa lý lớn, trong khi nguyên tắc nói không coi châu lục là một văn hóa | Giữ danh mục làm nhóm tham chiếu, nhưng mỗi linh vực chọn cộng đồng cụ thể và có thể tách khi nghiên cứu sâu |
| C7 | Yêu cầu muốn các hệ khác nhau về tầm, cơ động, hồi phục..., nhưng combat.md giới hạn 8 nút mobile | Mọi khác biệt đi qua nội tại, bộ pháp và thuật; không thêm nút |

Không có mâu thuẫn nào buộc thay đổi yêu cầu đã chốt.

### A4. Cải tiến đề xuất và lợi ích

| Cải tiến | Lợi ích rõ ràng |
| --- | --- |
| Năm dòng mạch làm gốc cho truyền thừa | Một nguồn duy nhất giải thích sức mạnh, tương tác bản đồ, tên gọi và vũ trụ quan |
| Mạch Bạ = sổ cái tài nguyên trong truyện | Người chơi hiểu vì sao tài nguyên hữu hạn; phản diện gắn trực tiếp với hệ thống kinh tế |
| Ngưỡng Mạch / Hóa Tướng / Thông Mạch | Lý do trong truyện cho trần cấp và tiến triển ngang; boss bi kịch thay vì quái vô hồn |
| Trả nhịp thay vì giết sinh vật thần thoại | Giảm rủi ro văn hóa; khớp chủ đề chăm sóc |
| Nhánh địa phương thay cho lớp nhân vật mới | Mở rộng bản sắc từng vùng với chi phí animation thấp |
| Harness cân bằng dùng lại lõi simulation | Biến "cân bằng" thành phép đo tái lập; tận dụng kiến trúc đã tách renderer |
| Thuật khách dùng bộ thi triển chung | Học chéo không nhân chi phí animation |

---

## B–J. Tóm tắt các phần

**B. Nghiên cứu thần thoại và văn hóa** — 9 khu vực, mỗi khu vực 8 hạng mục; thang mức sử dụng 0–3; 11 mô-típ xuyên văn hóa; bảng phân biệt gốc và chuyển thể (Lĩnh Nam chích quái, Kalevala, Popol Vuh, Aladdin, Paul Bunyan...); 59 nguồn có hạng. → [mythology-survey.md](../research/mythology-survey.md)

**C. So sánh phương án phân chia** — 5 phương án (châu lục, nguyên tố, vai trò, vũ khí, dòng mạch) chấm theo 6 tiêu chí; chọn dòng mạch xuyên văn hóa; giữ vai trò và vũ khí làm thuộc tính. → [lineages.md §2](lineages.md#2-so-sánh-phương-án-phân-chia-hệ-nhân-vật)

**D. Năm hệ nhân vật** — mỗi truyền thừa có tên + phương án tên, cảm hứng có nguồn và mức dùng, lịch sử, triết lý, đạo trường, phong cách, cơ chế đặc trưng, điểm mạnh/yếu, vũ khí, 11 kỹ năng, 2 nhánh, nhân vật, boss, quan hệ; ma trận khắc chế vòng 5. → [lineages.md §4–5](lineages.md#4-năm-truyền-thừa-cho-bản-ra-mắt)

**E. Combat, progression, cân bằng** — biến thể hành động chung, chuỗi combo có cửa thoát, Thoát Mạch, thuật khách, bảng PvE/PvP/bang hội, cấp 1–30 và Thông Mạch, chỉ số giả thuyết, 5 giả thuyết, harness headless, bot B0–B3, bộ T1–T8 với ngưỡng. → [combat-progression-balance.md](combat-progression-balance.md)

**F. Thế giới và bản đồ** — cấu trúc Thần Mạch/linh vực/Mạch Đạo, 8 quy tắc kết nối, danh mục linh vực, chấm điểm chọn linh vực thứ hai, 15 bản đồ (cảm hứng, cấp, địch và boss, tài nguyên, địa hình chiến đấu, nhiệm vụ và bí mật), khu PvP, quy chuẩn dựng bản đồ. → [world-atlas.md](world-atlas.md)

**G. Cốt truyện** — trả lời 9 câu hỏi nền, 5 thế lực, hồ sơ Người Đo Mạch, mở đầu → chương 1 → chương 2, 5 bí ẩn có lịch trả lời, mùa chiến dịch "Tiếp Mạch", Tranh Trạm, boss thế giới, 3 tầng hệ quả lựa chọn. → [narrative.md](narrative.md)

**H. Kinh tế và thương mại hóa** — hai tài nguyên mới, bảng nguồn/tiêu hao theo sổ cái, Dấu Boss chống xui, luật chợ, khởi động `q_i` chống tài khoản phụ, danh mục ✓/⚠/✗, tuân thủ Nghị định 147/2024, rủi ro doanh thu chỉ từ mỹ phẩm. → [monetization.md](monetization.md)

**I. Rủi ro** — 44 rủi ro theo 5 nhóm (thiết kế, văn hóa, bản quyền, kỹ thuật, kinh doanh–pháp lý), danh sách không dùng, quy trình duyệt văn hóa 7 bước và mẫu hồ sơ. → [risk-register.md](../research/risk-register.md)

**J. Roadmap** — 7 giai đoạn khớp roadmap v0.1, ngân sách nội dung, gate bản ra mắt, chiến lược tái sử dụng asset, 8 đầu việc R-01…R-08. → [content-roadmap.md](../production/content-roadmap.md)

---

## Quyết định cần chủ dự án duyệt

Chi tiết và phương án thay thế ở [ADR 0002](../decisions/0002-lineages-world-structure.md).

| Mã | Câu hỏi | Đề xuất |
| --- | --- | --- |
| D1 | Có tách "hệ nhân vật" thành sáu tầng khái niệm? | Có |
| D2 | Năm truyền thừa ra mắt và tên gọi? | Long Lưu, Sơn Cốt, Phong Vũ, Lâm Trảo, Tơ Vọng |
| D3 | Định nghĩa bản ra mắt 1.0? | 2 linh vực, cấp 1–30, 5 truyền thừa, mùa 1 |
| D4 | Linh vực thứ hai và đổi tên Sa Đăng? | Song Nguyên |
| D5 | Vũ trụ quan và phản diện chính? | Thần Mạch là ký ức chảy; Người Đo Mạch, Mạch Bạ |
| D6 | Chính sách thần linh và tri thức thiêng? | Mức 0 cho thần đang thờ, tri thức thiêng, biểu tượng quốc gia |
| D7 | Luật học kỹ năng liên vùng? | 1 thuật khách từ truyền thừa phụ |
| D8 | Điều kiện được gọi là "cân bằng"? | Qua T1–T6 hoặc có giải trình được review |
| D9 | Thương mại hóa và tuân thủ? | Giữ 0001; vật phẩm trả phí không giao dịch; tuân thủ Nghị định 147 |

## Những gì báo cáo này không làm

- Không viết code, không tạo asset, không tạo issue GitHub.
- Không sửa nội dung GDD, World Bible, combat, progression, economy v0.1 — chỉ thêm liên kết. Việc đồng bộ chờ ADR 0002 được duyệt.
- Không tuyên bố bất kỳ con số cân bằng hay kinh tế nào đã được kiểm chứng.
- Không thay thế ý kiến của cố vấn văn hóa hay luật sư.
