# MyVa — Thần Mạch: kinh tế bản ra mắt và định hướng thương mại hóa

> Đề xuất v0.2, 2026-10-09. Mục **H** của [báo cáo thiết kế](worldbuilding-report.md).
> [economy.md](economy.md) vẫn là nguồn sự thật cho sổ cái, bể tài nguyên, công thức ngân sách và bảo toàn. Tài liệu này mở rộng cho quy mô bản ra mắt (hai linh vực, năm truyền thừa) và đặt khung thương mại hóa.
> Mọi con số là giả thuyết cho simulator ([work-plan 030](../work-plan/030-economy-simulator.md)), không phải cấu hình vận hành.

## 1. Phạm vi và quan hệ với economy.md

| Đã có trong economy.md (giữ nguyên) | Tài liệu này bổ sung |
| --- | --- |
| Bể hữu hạn `M_r`, ngân sách `B_r`, sổ cái `R/N/I/X/Q/Z` | Hai tài nguyên vùng mới cho Song Nguyên |
| Cấp độ hiệu dụng `E`, lợi suất giảm dần, trần pilot | Khởi động `q_i` cho tài khoản mới, phát hiện chuỗi tài khoản phụ |
| Một tiền tệ, phí bán 1%, NPC thu mua hữu hạn | Luật thị trường, khóa giao dịch, vận chuyển liên vùng |
| Người mới, solo, độ khan hiếm | Trang bị hiếm, token chống xui, phần thưởng chiến dịch |
| — | Danh mục thương mại hóa, bảo vệ người chơi, tuân thủ pháp luật Việt Nam |

**Diễn giải trong truyện:** trần tài nguyên hữu hạn chính là **Mạch Bạ** — sổ cổ ghi trữ lượng mạch của thế giới ([narrative.md](narrative.md#3-thế-lực-và-phản-diện)). Người chơi hiểu vì sao tài nguyên có hạn; phản diện muốn độc quyền cuốn sổ đó.

## 2. Hòa giải "nạp tiền" với quyết định 0001

Yêu cầu nghiên cứu cho phép "phát triển hệ thống nạp tiền". [Quyết định 0001](../decisions/0001-project-foundation.md) đã chốt: doanh thu từ mỹ phẩm, **không bán sức mạnh hoặc quota tài nguyên**. Quyết định đã chốt được ưu tiên, nên:

- "Nạp tiền" được hiểu là **mua mỹ phẩm và dịch vụ tài khoản**, với giá hiển thị rõ.
- Không có tiền tệ trả phí đổi được sang Đồng, vật liệu hay chỉ số.
- Vật phẩm mua bằng tiền thật **không giao dịch được**, để tiền thật không đi vòng qua chợ thành sức mạnh.
- Mọi thay đổi khỏi các điểm trên cần một ADR mới với bằng chứng pilot. Nhiều MMO mobile bán tiền tệ cao cấp dùng được cho sức mạnh; MyVa cố ý không theo mô hình đó.

## 3. Tài nguyên và tiền tệ của bản ra mắt

| Mã | Tên | Nguồn chính | Dùng cho | Giao dịch | Trạng thái |
| --- | --- | --- | --- | --- | --- |
| `fiber` | Linh thảo | Vân Thủy, Vườn Bốn Dòng | Bình hồi, dụng cụ, chế tác thường | Có | Đã có |
| `ore` | Mạch khoáng | Hang, kênh ngầm, cao nguyên | Vũ khí, Rèn, công trình | Có | Đã có |
| `shard` | Mảnh Thần Mạch | Sự kiện có ngân sách | Lựa chọn ngang, mỹ phẩm, thành tựu | Giới hạn (GT) | Đã có |
| `reed` | Sợi Lau | Đầm Sậy Nổi, Vườn Bốn Dòng | Giáp nhẹ, Bùa, sợi cho thoi Tơ Vọng | Có | Mới |
| `salt` | Muối Mạch | Cao Nguyên Tháp Gió | Vật phẩm tiêu hao, hàng vận chuyển của đoàn xe | Có | Mới |
| — | Đồng | Kho hệ thống (`M_coin`) | Giao dịch, phí, sửa chữa | Có | Đã có |

Tài nguyên ràng buộc (không giao dịch): **Danh vọng** theo thế lực, **Dấu Boss** (§5), **Hồi Ức**, **Điểm mặt trận** (xóa cuối mùa).

- `M_reed`, `M_salt`: chốt bằng simulator. Điểm khởi đầu đề xuất: `M_reed ≈ 0,8 × M_fiber`, `M_salt ≈ 0,7 × M_ore` (GT).
- Bản ra mắt chỉ thêm **hai** tài nguyên giao dịch được và **không** thêm tiền tệ vùng.
- Tài nguyên Song Nguyên không bắt buộc để hoàn thành tuyến Vân Thủy; tài nguyên Vân Thủy vẫn có giá trị ở Song Nguyên (không làm phế vùng cũ).

## 4. Nguồn sinh và tiêu hao

Mọi dòng đều đi qua sổ cái của [economy.md §4](economy.md#4-sổ-cái-vật-liệu); không có nguồn nào in vật liệu hay Đồng ngoài sổ.

| Hoạt động | Sinh | Tiêu hao | Chuyển trạng thái |
| --- | --- | --- | --- |
| Nhiệm vụ | Đồng (từ ngân sách thưởng), vật liệu ràng buộc | — | Treasury → Wallet |
| Khai thác | Vật liệu | Thời gian, rủi ro | `N → I` |
| Quái, boss | Đồng có trần, vật liệu, trang bị, Dấu Boss | — | Theo ngân sách vùng |
| Chế tác | Trang bị, vật phẩm | Vật liệu, Đồng | `I → X` |
| Tháo dỡ | Một phần vật liệu | Phần còn lại | `X → I + Q + Z` |
| Rèn +1…+5 | Mức Rèn | Vật liệu, Đồng | `I → Q` |
| Sửa chữa | — | Đồng, vật liệu, có trần mỗi buổi | `I → Q` |
| Vận chuyển qua Mạch Đạo | — | Đồng theo khối lượng | Wallet → Treasury |
| Phí chợ | — | 1% giá bán (giữ), phí đăng nhỏ (GT) | Wallet → Treasury |
| Duy trì trạm bang hội | — | Đồng, vật liệu theo tuần | `I → Q` |
| Dự án tái thiết chiến dịch | Mục Biên Niên, mỹ phẩm | Vật liệu đóng góp | `I → Q` |
| Đổi nhánh, truyền thừa phụ | — | Phí nhỏ (miễn phí trước cấp 10) | Wallet → Treasury |
| Nhuộm mỹ phẩm (kiếm trong game) | Màu nhuộm | Vật liệu | `I → Q` |

Tiêu hao phải tạo tiện ích hoặc lựa chọn; không có phí chỉ để đăng nhập hay giữ tiến trình.

## 5. Trang bị hiếm và phần thưởng boss

- **Hiếm là ngang, không dọc:** món hiếm cho biến thể chiến thuật và ngoại hình trong cùng ngân sách tier.
- **Trang bị boss cao nhất ràng buộc khi nhận**; trang bị **chế tác** giao dịch được. Chợ sống nhờ chế tác, còn áp lực mua bán tiền thật giảm ở đồ đỉnh.
- **Dấu Boss chống xui:** mỗi lần qua boss nhận 1 Dấu (ràng buộc). Đủ N Dấu đổi một món chọn trong danh sách của boss đó (GT: N = 5). Không có cảm giác "đánh 40 lần không ra".
- **Trần phần thưởng theo tuần** cho mỗi boss (ngân sách), nhưng **chơi lại không giới hạn** để luyện tập; không bán lượt reset.
- Không vỡ đồ, không mất mức Rèn khi thất bại (Rèn là tất định).

## 6. Chế tác, giao dịch và thị trường

| Quy tắc | Đề xuất (GT) | Lý do |
| --- | --- | --- |
| Hình thức | Sổ lệnh ký gửi chung toàn thế giới, quầy ở Chợ Nổi và Cảng Hai Dòng | Một thị trường, không chia nhỏ thanh khoản |
| Nhận hàng | Hàng đăng ở linh vực khác: tự đi lấy (miễn phí) hoặc trả phí vận chuyển theo khối lượng | Cho vận chuyển có ý nghĩa; tạo sink |
| Mở khóa giao dịch | Tài khoản ≥ 7 ngày, xong chương 1, đã xác thực số điện thoại (bắt buộc với thị trường Việt Nam, §7) | Giảm tài khoản phụ nuôi hàng |
| Giới hạn | Số lệnh mở mỗi tài khoản; giá trị gửi thư/tặng mỗi ngày thấp với tài khoản mới | Chống chuyển tài sản hàng loạt |
| Phí | 1% khi bán (giữ từ economy.md) + phí đăng nhỏ, hoàn một phần khi bán được | Giảm spam lệnh |
| Thông tin giá | Hiển thị trung vị và khối lượng gần đây | Minh bạch; không để NPC cố định giá |
| Không giao dịch | Trang bị boss đỉnh, vật phẩm mua bằng tiền thật, vật liệu nhiệm vụ, Dấu Boss, Danh vọng | Chống RMT, giữ ý nghĩa thành tựu |

## 7. Bảo vệ người chơi và tuân thủ

### Pháp luật Việt Nam (cần luật sư xác nhận)

Nghị định **147/2024/NĐ-CP** (ban hành 9/11/2024, hiệu lực 25/12/2024, thay Nghị định 72/2013) — theo các nguồn báo chí pháp lý ([hoatieu.vn](https://hoatieu.vn/phap-luat/nguoi-duoi-18-tuoi-khong-duoc-choi-game-qua-60-phut-ngay-226987), [lsvn.vn](https://lsvn.vn/du-kien-gioi-han-thoi-gian-choi-game-trong-ngay-voi-tre-em-a176381.html)):

- Tài khoản game phải **xác thực bằng số điện thoại di động tại Việt Nam**; chỉ tài khoản đã xác thực mới được chơi. Người dưới 16 tuổi do cha mẹ/người giám hộ đăng ký.
- Người dưới 18 tuổi: tối đa **60 phút mỗi trò chơi** và **180 phút tổng mỗi ngày**; nhà cung cấp phải có hệ thống kỹ thuật quản lý thời gian chơi.
- Game nhiều người qua server thuộc nhóm G1, cần giấy phép và phê duyệt nội dung trước khi phát hành.
- Tháng 7/2026 có **dự thảo sửa đổi** đề xuất giảm thời gian chơi của người dưới 16 tuổi còn 60 phút/ngày; chưa xác nhận đã ban hành.

**Hệ quả thiết kế:**

- Phiên hoạt động cộng đồng và cửa sổ Tranh Trạm ≤ 45 phút; không thiết kế sự kiện bắt buộc dài hơn 60 phút.
- Cảnh báo trước khi hết thời gian (ví dụ 10 và 5 phút); không cho bắt đầu instance mà thời gian còn lại không đủ hoàn thành hợp lý; khi hết giờ, giữ phần thưởng đã server xác nhận.
- Hồ sơ nội dung văn hóa ([risk-register](../research/risk-register.md#4-quy-trình-duyệt-văn-hóa)) cũng phục vụ hồ sơ phê duyệt nội dung G1.
- Kiến trúc tài khoản cần trường xác thực điện thoại, độ tuổi, giám hộ — đưa vào thiết kế persistence trước MVP Online.

### Bảo vệ chi tiêu (đề xuất)

- Giá hiển thị bằng tiền thật (VND và nội tệ khác); nếu có tiền tệ trung gian thì quy đổi cố định, không gói lẻ khiến luôn dư.
- Không đếm ngược giả, không "chỉ còn 2 suất" giả, không ép mua để giữ tiến trình.
- Giới hạn chi tiêu tháng mặc định cho tài khoản dưới 18 tuổi, cha mẹ điều chỉnh được (GT).
- Chính sách hoàn tiền rõ ràng; không cơ chế quay thưởng, gacha hay loot box.

## 8. Điều chỉnh theo quy mô cộng đồng và chống tài khoản phụ

Nguyên tắc: **sức mạnh cộng đồng không in thêm tài nguyên.** Nó chỉ thay đổi nhịp tái sinh trong trần và cách phân bổ giữa các vùng.

| Cơ chế | Mô tả (GT) | Quan hệ với economy.md |
| --- | --- | --- |
| Cấp độ hiệu dụng `E` | Giữ nguyên công thức, lợi suất giảm dần, trần pilot | [§5](economy.md#5-tổng-cấp-độ-hiệu-dụng-đang-hoạt-động) |
| Khởi động `q_i` | Tài khoản mới đóng góp 0 vào `E` cho tới khi có 10 giờ hoạt động hợp lệ và xong chương 1; sau đó tăng dần tới 1 | Bổ sung cho `q_i` |
| Gói khởi đầu một lần | Theo tài khoản, khóa giao dịch | Giữ §8 |
| Sức khỏe nút mạch | Sự kiện cộng đồng phục hồi nút làm **dịch phân bổ** `B_r` sang vùng được chăm sóc; tổng không đổi | Bổ sung cho §7 |
| Ngân sách mùa | Phần thưởng chiến dịch rút từ ngân sách mùa có trần riêng, nằm trong `M_r` | Bổ sung |
| Phát hiện chuỗi nuôi hàng | Phân tích đồ thị giao dịch: nhiều tài khoản mới → một người nhận; gắn cờ để xem xét, không tự động phạt | Bổ sung §10 |
| Xác thực điện thoại | Ma sát thêm cho việc tạo hàng loạt tài khoản; **không** coi là chứng minh một người thật | Bổ sung |
| Không thưởng hiện diện | Treo máy, đứng yên không tạo XP, Mạch hay đóng góp | Giữ |

Khi dân số giảm, nguồn cung giữ mức cơ sở bảo thủ; khi tăng, tăng theo log và bị trần chặn. Người chơi thật vẫn có thể bị giảm tiếp cận khi bot lọt qua — đây là giới hạn đã nêu ở [economy.md §10](economy.md#10-tiêu-hao-có-ích-và-giới-hạn-khai-thác), cần đo trong simulator.

## 9. Danh mục thương mại hóa

| Hạng mục | Trạng thái | Điều kiện |
| --- | --- | --- |
| Mỹ phẩm bán trực tiếp (trang phục, màu, hiệu ứng thuật, dáng đứng) | ✓ | Giữ silhouette, hitbox, telegraph; có bản đọc rõ khi giảm hiệu ứng |
| Sổ Hành Trình theo mùa | ✓ | Bản miễn phí có phần thưởng chơi theo ngân sách; bản trả phí **chỉ thêm mỹ phẩm** |
| Gói người ủng hộ, gói khởi đầu | ✓ | Chỉ mỹ phẩm và danh hiệu |
| Đổi tên, đổi diện mạo | ✓ | — |
| Thêm ô nhân vật | ✓ | Gói khởi đầu và đóng góp `E` tính theo tài khoản, không theo nhân vật |
| Mở rộng kho đồ | ⚠ | Mặc định kho miễn phí đủ dùng; không bán ở bản ra mắt; cần ADR nếu muốn bán |
| Thuê bao mỹ phẩm hàng tháng | ⚠ | Chỉ mỹ phẩm; quyết định sau pilot |
| Bản mở rộng trả tiền | ⚠ | Nguy cơ chia cộng đồng; quyết định sau khi có dữ liệu |
| Tiền tệ trả phí trung gian | ⚠ | Nếu dùng: tỷ giá cố định, không đổi sang Đồng, không gói lẻ dư thừa |
| Chỉ số, vật liệu, EXP boost, lượt boss, ưu tiên khai thác, bỏ qua hạn mức | ✗ | Quyết định 0001 |
| Loot box, gacha, quay thưởng | ✗ | Quyết định 0001; rủi ro pháp lý nhiều nước |
| Mỹ phẩm trả phí giao dịch được, bán Đồng | ✗ | Chống chuyển tiền thật thành sức mạnh qua chợ |
| "VIP" có lợi ích gameplay | ✗ | — |

**Mua bán tài khoản và vàng ngoài game (RMT):** cấm trong điều khoản; phát hiện qua đồ thị giao dịch và lịch sử đăng nhập; vật phẩm trả phí ràng buộc làm RMT kém hấp dẫn.

## 10. Tính bền vững doanh thu

Doanh thu chỉ từ mỹ phẩm là **giả thuyết rủi ro cao** chưa có nghiên cứu thị trường ([roadmap](../production/roadmap.md#cổng-quyết-định-thương-mại)).

- **Phụ thuộc sản xuất:** mỹ phẩm phải phủ 5 truyền thừa × 2 dáng cơ thể. Nếu mỗi bộ trang phục phải vẽ lại từng khung hình, chi phí không bền vững → cần rig module với attachment thay được (quyết định pipeline ở [risk-register K-01](../research/risk-register.md#k--kỹ-thuật)).
- **Chỉ số pilot cần đo:** tỷ lệ người trả phí, doanh thu trên người trả phí, chi phí sản xuất mỗi bộ mỹ phẩm, nhịp nội dung cần để giữ chân, tỷ lệ người chơi chỉ chơi miễn phí vẫn tiến triển tới trần.
- **Nếu không đủ:** ưu tiên bản mở rộng trả tiền hoặc thuê bao mỹ phẩm — vẫn trong khuôn 0001. Bán sức mạnh không phải phương án dự phòng; nếu chủ dự án muốn cân nhắc, cần ADR mới.

## 11. Mô phỏng và chỉ số theo dõi

Mở rộng phạm vi simulator 030 trước bản ra mắt:

- Thêm `reed`, `salt`, sổ lệnh thị trường, vận chuyển liên vùng, ngân sách mùa, phí duy trì trạm.
- Kịch bản: chuỗi tài khoản phụ nuôi một người; bang hội tích trữ một tài nguyên; mùa chiến dịch kéo nhu cầu đột biến; dân số giảm 50% sau mùa đầu.

| Chỉ số | Ngưỡng cảnh báo (GT) |
| --- | --- |
| Biến động giá rổ thiết yếu 7 ngày | ±10% (giữ từ economy.md) |
| Thời gian người solo chế tác bộ tier mới | Tăng > 25% so với mục tiêu |
| Tỷ lệ tiêu hao / sinh của Đồng | Ngoài 0,8–1,1 trong 2 tuần |
| Tỷ trọng tài sản của 1% người giữ nhiều nhất | Tăng liên tục 4 tuần |
| Giao dịch từ tài khoản < 7 ngày tới cùng một người nhận | Vượt ngưỡng → xem xét |
| Tỷ lệ người chỉ chơi miễn phí tới trần cấp | Giảm giữa các mùa |

## 12. Câu hỏi mở

- Mảnh Thần Mạch có nên giao dịch được ở bản ra mắt không?
- Bán mở rộng kho đồ có bị xem là pay-to-win không? Cần khảo sát người chơi pilot.
- Một thị trường toàn thế giới hay chợ theo linh vực (tăng vai trò vận chuyển nhưng giảm thanh khoản)?
- Mức giới hạn chi tiêu mặc định cho người dưới 18 tuổi.
