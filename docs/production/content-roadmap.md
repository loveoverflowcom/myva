# Roadmap nội dung — truyền thừa, linh vực và câu chuyện theo giai đoạn

> Đề xuất v0.2, 2026-10-09. Mục **J** của [báo cáo thiết kế](../design/worldbuilding-report.md).
> Bám theo các giai đoạn và gate của [roadmap.md](roadmap.md); tài liệu này chỉ thêm **nội dung** (truyền thừa, bản đồ, boss, chương truyện) và việc nghiên cứu đi kèm. Không đặt ngày, ước lượng nhân sự hay deadline khi chưa có dữ liệu prototype.

## 1. Nguyên tắc

- Mỗi giai đoạn chỉ thêm nội dung khi gate của giai đoạn trước đạt. Nội dung trễ không được bù bằng cách bỏ qua playtest, mô phỏng cân bằng hay duyệt văn hóa.
- Thêm truyền thừa **từng cái một**, mỗi cái qua graybox và mô phỏng riêng.
- Linh vực mới chỉ vào sản xuất khi hồ sơ văn hóa đã ký duyệt ([risk-register §4](../research/risk-register.md#4-quy-trình-duyệt-văn-hóa)).
- Không hứa với người chơi nội dung chưa qua gate.

## 2. Tổng quan giai đoạn

| Giai đoạn ([roadmap](roadmap.md)) | Nội dung mới | Nghiên cứu, kiểm chứng đi kèm | Gate nội dung |
| --- | --- | --- | --- |
| 0. Tài liệu | Bộ docs v0.2; ADR 0002 | Duyệt quyết định; kế hoạch cố vấn; tra nhãn hiệu sơ bộ | Chủ dự án duyệt hoặc sửa ADR 0002 |
| 1. Feasibility | Graybox Long Lưu; spike Tơ Vọng; spike pipeline animation | Harness headless + bot B0–B1; thử Thoát Mạch | Combat graybox đạt; rủi ro Tơ Vọng và animation được đánh giá |
| 2. Vertical slice | Vân Thủy 3 bản đồ, 3 truyền thừa, mở đầu có bổ sung truyện | Bot B2, ma trận 3×3; cố vấn Đông Nam Á cho Vân Thủy | Gate G1–G8 của [vertical-slice](vertical-slice.md#tiêu-chí-hoàn-thành) |
| 3. MVP Online | Vân Thủy 8 bản đồ, chương 1, cấp 1–20; Lâm Trảo nếu đạt gate | Dãy boss T4; mở rộng simulator kinh tế; đánh giá pháp lý Việt Nam | Gate MVP + solo tới cấp 20 bằng mọi truyền thừa đã phát hành |
| 4a. Bản ra mắt 1.0 | Song Nguyên, Đèo Gió Mặn, chương 2, 5 truyền thừa, thuật khách, mùa 1, đấu trường | T1–T6 đầy đủ; duyệt văn hóa Song Nguyên; gate thương mại | §4.5 |
| 4b. Mở rộng 1 | Linh vực thứ ba, nhánh địa phương, có thể thêm truyền thừa thứ sáu | Đối tác văn hóa cho linh vực mới | Linh vực mới có tutorial, boss, nhánh và hồ sơ duyệt |
| 5. Chiến dịch liên lục địa | Nhiều mặt trận, cung Người Đo Mạch tiếp theo, Hợp Dòng | Thử tải, chống snowball | Theo [roadmap](roadmap.md#chiến-dịch-liên-lục-địa) |

## 3. Ngân sách nội dung

Số lượng là **giới hạn trên** để lập kế hoạch, không phải cam kết.

| Hạng mục | Slice | MVP Online | Bản ra mắt 1.0 | Mở rộng 1 |
| --- | --- | --- | --- | --- |
| Truyền thừa | 3 | 3 (+ Lâm Trảo nếu đạt gate) | 5 | 5 + tối đa 1 |
| Nhánh mỗi truyền thừa | Một lựa chọn nâng thuật mẫu | 2 | 2 | 2–3 (nhánh địa phương) |
| Linh vực | 1 | 1 | 2 | 3 |
| Bản đồ | 3 | 8 | 15 (8 + Đèo Gió Mặn + 6) | +6 |
| Boss instance | 1 | 5 | 9 | +4 |
| Boss thế giới | 0 | 1 | 2 | +1 |
| Archetype địch thường | 3 | 8 | 14 | +6 |
| Chương truyện | Mở đầu | Chương 1 | Chương 2 + mùa 1 | Chương 3 + mùa tiếp |
| Trần cấp | 10 | 20 | 30 | 30 (Thông Mạch) |
| Tài nguyên giao dịch | 3 | 3 | 5 | + tối đa 2 |
| PvP | — | Đấu tay đôi thử nghiệm | Đấu trường 1v1/2v2/3v3 chuẩn hóa | Xếp hạng theo mùa |
| Mỹ phẩm | — | Thử pipeline | Cửa hàng + Sổ Hành Trình mùa 1 | Theo nhịp mùa |

## 4. Chi tiết từng giai đoạn

### 4.1 Giai đoạn 0 — tài liệu (hiện tại)

- Chủ dự án duyệt [ADR 0002](../decisions/0002-lineages-world-structure.md): phân tầng khái niệm, năm truyền thừa, linh vực thứ hai, chính sách thần linh.
- Sau khi duyệt: cập nhật GDD, World Bible, progression-social, combat §8 trong cùng một PR (R-08).
- Bắt đầu R-01 (kế hoạch cố vấn) và R-02 (tra nhãn hiệu) song song với 010/020/030.

### 4.2 Giai đoạn 1 — feasibility

- **Graybox Long Lưu** (020) giữ nguyên phạm vi; bổ sung thử **Thoát Mạch** và chạy harness với bot B0–B1 (R-03).
- **Spike Tơ Vọng** (R-05) sau khi graybox Long Lưu ổn định: Vọng, Neo, ngắm trên cảm ứng, authority của tiếng vọng. Lý do làm sớm dù là nội dung bản ra mắt: đây là truyền thừa có độ bất định cao nhất; nếu không khả thi, cần thay sớm.
- **Spike pipeline animation** (R-04) cùng 010 và [assets-performance §2](../technical/assets-performance.md#2-skeletal-animation-là-lựa-chọn-cần-kiểm-chứng): rig module + attachment vũ khí/trang phục so với flipbook; đánh giá license runtime.

### 4.3 Giai đoạn 2 — vertical slice

- Ba truyền thừa như đã chốt; mở đầu bổ sung: An và học trò cũ, thiết bị đo lạ, lựa chọn số phận Kẻ Giữ Đập ([narrative §4.1](../design/narrative.md#41-mở-đầu--nước-về-muộn-vertical-slice-cấp-110)).
- 1–2 Hồi Ức mẫu để thử khám phá; không bắt buộc cho gate.
- Bot B2 và ma trận đối đầu 3×3; dãy boss Kẻ Giữ Đập cho ba truyền thừa.
- Cố vấn văn hóa Đông Nam Á review Vân Thủy trước khi công bố hình ảnh.

### 4.4 Giai đoạn 3 — MVP Online

- Vân Thủy 8 bản đồ, chương 1, cấp 1–20; boss Đốc Kè, Hổ Mất Nhịp, Én Đen, Bánh Xe Ngược; boss thế giới Giao Ngược.
- **Lâm Trảo** vào MVP nếu: G2 của slice đạt, pipeline animation đã chốt, và graybox Lâm Trảo qua T5 (không vòng lặp hồi máu). Nếu không, đẩy sang bản ra mắt.
- Đấu tay đôi thử nghiệm ở Võ Đài Thuyền khi gate combat mạng đạt.
- Mở rộng simulator kinh tế (R-06); đánh giá pháp lý (R-07) trước khi mở pilot tại Việt Nam.

### 4.5 Giai đoạn 4a — bản ra mắt 1.0

- Song Nguyên 6 bản đồ + Đèo Gió Mặn; chương 2; cấp 21–30; Tơ Vọng (và Lâm Trảo nếu chưa có).
- Truyền thừa phụ và thuật khách; Thông Mạch sau cấp 30.
- Mùa 1 "Tiếp Mạch": hai linh vực nối bằng một tuyến tiếp tế — đúng bước "thử hai linh vực kết nối bằng một tuyến tiếp tế" của [roadmap](roadmap.md#chiến-dịch-liên-lục-địa).
- Đấu trường chuẩn hóa; cửa hàng mỹ phẩm và Sổ Hành Trình sau gate thương mại.

**Gate bản ra mắt:**

| Điều kiện | Bằng chứng |
| --- | --- |
| Cân bằng | T1–T6 đạt ngưỡng hoặc có giải trình được review ([combat-progression-balance §11](../design/combat-progression-balance.md#11-phương-pháp-kiểm-chứng)) |
| Solo | Mọi truyền thừa + nhánh qua tuyến chính tới cấp 30 bằng nguồn solo |
| Văn hóa | Hồ sơ Vân Thủy và Song Nguyên đã ký duyệt |
| Pháp lý | Tuân thủ Nghị định 147 ở thị trường Việt Nam; giấy phép G1 |
| Tên | Tra cứu nhãn hiệu xong cho tên công bố |
| Kinh tế | Simulator với hai tài nguyên mới, chợ và ngân sách mùa không vi phạm invariant |

### 4.6 Giai đoạn 4b — mở rộng 1

- Chọn linh vực thứ ba theo mức sẵn sàng nghiên cứu: Sương Hải (rủi ro thấp) hoặc linh vực Nam Á (nếu đã có cố vấn).
- Mỗi linh vực mới: một nhánh địa phương cho 1–2 truyền thừa; cân nhắc truyền thừa thứ sáu từ [danh sách ứng viên](../design/lineages.md#6-mở-rộng-sau-ra-mắt).
- Sự kiện Kình Nuốt Trăng chỉ khi có cố vấn văn hóa Philippines.

### 4.7 Giai đoạn 5 — chiến dịch liên lục địa

- Nhiều mặt trận với giới hạn tác nhân; cung Người Đo Mạch tiếp theo; Hợp Dòng thay đổi trạng thái thế giới theo mùa (cần ADR nếu thay đổi bản đồ vĩnh viễn).

## 5. Chiến lược tái sử dụng asset

| Nhóm | Chia sẻ | Phần riêng |
| --- | --- | --- |
| Cơ thể người chơi | 2 dáng cơ thể, một bộ xương chung | Vũ khí (5 họ), trang phục dạng attachment |
| Kẻ địch và boss người | Cùng rig người với người chơi | Trang phục, vũ khí, VFX |
| Thuật khách | Bộ thi triển chung 3 tư thế | VFX của thuật gốc |
| Biến hình Lâm Trảo, tiếng vọng Tơ Vọng | Shader và VFX phủ | Không cần model mới |
| Sợi neo | Line renderer theo dữ liệu | — |
| Sinh vật lớn | Rig rắn (Giao Ngược, Thuồng Luồng), rig mèo lớn (Hổ Mất Nhịp), rig chim (Chim Bão, Cò Sậy thu nhỏ) | Mỗi rig mới là chi phí lớn — tối đa 1–2 rig mới mỗi linh vực |
| Máy móc Hắc Triều | Bộ phận module (bánh răng, ống, thùng mực) | Hình khối tổng thể từng boss máy |
| Môi trường | Bộ tile + 4 lớp parallax mỗi linh vực | ≤ 30% asset riêng mỗi bản đồ |

## 6. Đầu việc đề xuất

Các việc dưới đây **chưa** là issue GitHub. Theo [planning.md](../work-plan/planning.md), tạo issue và đưa lên Kanban sau khi chủ dự án đồng ý.

| Mã | Việc | Gắn với | Giai đoạn |
| --- | --- | --- | --- |
| R-01 | Kế hoạch cố vấn văn hóa: lĩnh vực cần cố vấn, ngân sách, mẫu hồ sơ, quy trình ký duyệt | World Bible §9, risk-register §4 | 0 |
| R-02 | Tra cứu nhãn hiệu cho tên truyền thừa và linh vực | lineages §3 | 0 |
| R-03 | Đặc tả và dựng harness cân bằng headless, bot B0–B1 | 020, combat-progression-balance §11 | 1 |
| R-04 | Spike pipeline animation: rig module, attachment, license runtime | 010, assets-performance §2 | 1 |
| R-05 | Spike cơ chế Tơ Vọng: Vọng, Neo, cảm ứng, authority | 020 | 1 |
| R-06 | Mở rộng simulator kinh tế: 2 tài nguyên, chợ, chuỗi tài khoản phụ, ngân sách mùa | 030, monetization §11 | 2–3 |
| R-07 | Đánh giá pháp lý Nghị định 147/2024 và thiết kế tài khoản | backlog online-pilot | 3 |
| R-08 | Đồng bộ GDD, World Bible, progression-social, combat với ADR 0002 sau khi duyệt | ADR 0002 | 0 |

## 7. Bước tiếp theo ngay

1. Chủ dự án đọc [báo cáo thiết kế](../design/worldbuilding-report.md#quyết-định-cần-chủ-dự-án-duyệt) và trả lời các quyết định D1–D9.
2. Nếu duyệt: thực hiện R-08 trong một PR riêng, cập nhật trạng thái ADR 0002.
3. Đưa R-03 vào phạm vi 020 (harness chạy cùng graybox) và R-04 vào 010.
4. Tạo issue cho R-01, R-02 để chạy song song với các prototype kỹ thuật.
