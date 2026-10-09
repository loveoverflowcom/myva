# MyVa — Thần Mạch: kinh tế và tài nguyên

> Trạng thái: bản thiết kế đề xuất để mô phỏng, chưa phải thông số cân bằng đã kiểm chứng.
> Phạm vi đầu tiên: một linh vực hư cấu lấy cảm hứng Đông Nam Á; hai tài nguyên thường, một tài nguyên hiếm thử nghiệm; MVP online có giới hạn người tham gia.

## 1. Kết luận review bản định hướng

- Tổng cấp độ có thể quyết định nhịp phục hồi, nhưng không tự tạo quyền phát hành mới.
- Giới hạn lượng đang lưu hành và giới hạn tổng số lượt phát hành là hai yêu cầu khác nhau.
- Giới hạn nguồn cung không bảo đảm giá ổn định: nhu cầu, tích trữ, mất cân đối vật phẩm và vận tốc giao dịch vẫn có thể đẩy giá lên.
- Tiêu hao phải có mục đích chơi rõ ràng; phí sửa chữa quá lớn chỉ biến kinh tế thành nghĩa vụ cày cuốc.
- Phải có một ngân sách chung cho mọi bản đồ, kênh và instance; mở kênh không được nhân bản tài nguyên.
- Chống bot và tài khoản phụ là giảm khả năng khai thác, không phải giả định nhận diện hoàn hảo một người thật.
- Người chơi solo được tiếp cận nguyên liệu tiến triển thiết yếu; bang hội mở lựa chọn phối hợp và mục tiêu chung.

## 2. Hợp đồng nguồn cung

Chọn mặc định **bể tài nguyên hữu hạn toàn thế giới**, không phát hành thêm chỉ vì tổng cấp độ tăng.
Một phần nguyên liệu đã tiêu hao có thể quay về bể sau thời gian phục hồi; khai thác là chuyển quyền giữ vật liệu, không phải tạo vật liệu vô điều kiện.
Mọi số liệu dưới đây là giả định cho thử nghiệm và có thể thay đổi bằng phiên bản cấu hình được ghi nhận.

| Khái niệm | Ý nghĩa | Chính sách mặc định |
| --- | --- | --- |
| Trần vật liệu `M_r` | Tổng đơn vị của loại `r` trong các trạng thái sổ cái | Hữu hạn, toàn thế giới |
| Ngân sách phục hồi `B_r` | Lượng được chuyển ra điểm khai thác trong một cửa sổ | Có trần giờ và ngày |
| Tổng lượt tái phát hành `J_r` | Tổng lượng đã đi từ bể ra điểm khai thác hoặc hỗ trợ | Tăng kể cả khi vật liệu được tái chế |
| Trần suốt đời `J_max,r` | Trần tuyệt đối của tổng lượt tái phát hành | Chế độ bổ sung nếu chốt yêu cầu theo nghĩa đen |
| Phát hành mới `G_r` | Tăng trần vật liệu `M_r` | Bằng 0 trong pilot; không tự động |

Nếu vật liệu liên tục được tái chế, tồn lượng vẫn hữu hạn nhưng tổng lượt tái phát hành có thể tăng mãi theo thời gian.
Nếu “không phát hành vô hạn” nghĩa là **tổng lượt phát hành suốt đời cũng hữu hạn**, phải bật `J_max,r`; khi chạm trần, phục hồi phải giảm hoặc dừng.
Không thể đồng thời hứa tài nguyên tiêu hao phục hồi mãi và tổng lượt phục hồi hữu hạn.
GDD cần chốt cách hiểu này trước khi triển khai; thiết kế hiện chọn hữu hạn tồn lượng và ngân sách từng kỳ.

## 3. Tài nguyên của vertical slice

| Mã | Tài nguyên hư cấu | Vai trò | `M_r` đề xuất cho pilot |
| --- | --- | --- | ---: |
| `fiber` | Linh thảo | Bình hồi phục, dụng cụ, chế tác thường | 100.000 đơn vị |
| `ore` | Mạch khoáng | Vũ khí, nâng cấp ngang, công trình | 60.000 đơn vị |
| `shard` | Mảnh Thần Mạch | Thử nghiệm một nhánh chế tác hiếm | 500 đơn vị |

- Không bán tài nguyên hoặc quyền khai thác vượt ngân sách bằng tiền thật.
- Linh thảo và Mạch khoáng có điểm khai thác tiếp cận được bằng solo.
- Mảnh Thần Mạch không bắt buộc để hoàn thành vùng khởi đầu hoặc học bộ kỹ năng cơ bản.
- Chưa tạo hàng chục loại nguyên liệu, tiền vùng hoặc tiền bang hội ở vertical slice.
- Truyền thừa không được nhân bản tài nguyên; cả ba truyền thừa dùng chung quy tắc phát hành.
- Hai kỹ năng cơ bản và ba thuật đang trang bị không trả phí mỗi lần sử dụng; chiến đấu cần thuận tay và dễ thử nghiệm.

## 4. Sổ cái vật liệu

Với mỗi loại `r`, duy trì các trạng thái đo bằng **đơn vị của chính loại đó**:

- `R_r`: bể chưa phân bổ, có thể được cấp cho phục hồi.
- `N_r`: lượng trên điểm khai thác, kể cả đã cấp cho kênh nhưng chưa thu hồi.
- `I_r`: lượng trong kho người chơi, bang hội, thư và ký quỹ thị trường.
- `X_r`: lượng đang nằm trong vật phẩm hoặc công trình, ghi theo thành phần nguyên liệu.
- `Q_r`: lượng đã tiêu hao và đang chờ quay về bể.
- `Z_r`: lượng bị loại bỏ vĩnh viễn, chưa được phép phục hồi.

```text
M_r = R_r + N_r + I_r + X_r + Q_r + Z_r
```

- Hái cây: `N → I`; chế tác: `I → X`; tháo dỡ: `X → I + Q + Z` theo tỷ lệ đã cấu hình.
- Sửa chữa và dùng vật phẩm: phần nguyên liệu được tiêu hao chuyển sang `Q` hoặc `Z`.
- Sau thời gian phục hồi: `Q → R`; tỷ lệ quay lại không được lớn hơn lượng đã tiêu hao.
- Vật phẩm gồm hai nguyên liệu giữ hai thành phần; không cộng trực tiếp một đơn vị thảo với một đơn vị khoáng.
- Vật phẩm giao dịch được giữ cùng thành phần nguyên liệu; bán lại không tạo bản sao thành phần.
- Hủy điểm khai thác hoặc đóng kênh phải trả lượng còn lại về `R`, sau khi đối soát quyền sở hữu.
- Prototype hiếm chọn tỷ lệ về `Q = 0`; đây là thí nghiệm khan hiếm, cần đánh giá nguy cơ hết nguồn trước khi mở rộng.
- Mỗi chuyển trạng thái có mã sự kiện duy nhất, nguyên nhân, người thực hiện và phiên bản công thức.
- Ghi sổ và chuyển quyền sở hữu phải nguyên tử; retry cùng mã trả kết quả cũ, không chuyển lần hai.

## 5. Tổng cấp độ hiệu dụng đang hoạt động

Không lấy tổng cấp độ toàn bộ tài khoản từng được tạo hoặc thời gian đăng nhập để treo máy làm tín hiệu.
Đo mức hoạt động trong cửa sổ trượt 24 giờ; không tăng trọng số bằng thời gian chơi vô hạn.
Hoạt động hợp lệ trong cửa sổ vẫn được tính sau khi đăng xuất; tài khoản không còn hoạt động trong cửa sổ đóng góp 0.

```text
l_i = L_cap × sqrt(min(L_i, L_cap) / L_cap)
a_i = min(t_i / T_active, 1)
e_i = l_i × a_i × q_i
E = min(sum(e_i), E_pilot_cap)
```

- `L_i`, `l_i`, `e_i`, `L_cap` và `E` dùng cùng đơn vị cấp độ quy ước.
- `t_i`: số phút hoạt động hợp lệ; `T_active` đề xuất 60 phút mỗi cửa sổ; `a_i` không có đơn vị.
- `L_cap` đề xuất 30; căn bậc hai cho lợi suất giảm dần, mỗi tài khoản đóng góp tối đa 30.
- `q_i ∈ [0, 1]`: trọng số đủ điều kiện; tài khoản bị hạn chế có thể giảm hoặc không đóng góp.
- `E_pilot_cap` đề xuất 6.000 cấp độ hiệu dụng; đây là trần điều khiển, không phải số người chơi thực tế.
- Hoạt động hợp lệ gồm chiến đấu, nhiệm vụ và thu thập có tiến triển; không chỉ đếm đầu vào hoặc uptime.
- Giới hạn hoạt động giúp tránh thưởng cho chơi quá lâu, nhưng không tự ngăn được nhiều tài khoản.
- Không dùng thu nhập, nạp tiền hoặc việc tham gia bang hội để tăng `q_i`.
- `q_i` và cơ chế hạn chế cần có lý do, thời hạn, khả năng khiếu nại; không công khai chi tiết giúp né phát hiện.
- Người mới có gói hỗ trợ riêng, không phải đạt cấp cao để hệ thống cấp đủ nguyên liệu khởi đầu.

## 6. Ngân sách tái sinh và bộ điều khiển

Tính mục tiêu theo giờ; mọi hệ số trong công thức là đề xuất để mô phỏng.

```text
target_r = r_base,r × [1 + beta_r × ln(1 + E / E_ref)] × d_r × s_r
candidate_r = clamp(target_r, previous_r × 0.95, previous_r × 1.05)
B_r = floor(max(0, min(candidate_r × DeltaT, H_r, D_remaining,r, R_r,
                      J_remaining,r, NodeSpace_r)))
```

- `target_r`, `candidate_r`, `previous_r` là đơn vị tài nguyên mỗi giờ; `DeltaT = 1 giờ` trong pilot.
- Nhân tốc độ với `DeltaT` trước khi so sánh với tồn lượng; nếu đổi cửa sổ phải đổi các trần tương ứng.
- `H_r`: trần lượng mỗi giờ; `D_remaining,r`: ngân sách ngày chưa dùng; cả hai là đơn vị tài nguyên.
- `J_remaining,r`: lượng còn lại dưới trần suốt đời nếu bật; nếu không bật thì bỏ toán hạng này.
- `NodeSpace_r`: tổng chỗ trống của điểm khai thác và gói hỗ trợ được cấp trong giờ.
- `E_ref` đề xuất 3.000 cấp độ hiệu dụng; `E/E_ref` không có đơn vị.
- `beta_r` đề xuất 0,4; hàm log khiến tăng cấp độ không dẫn tới tăng tuyến tính nguồn cung.
- `s_r = clamp(R_r / R_target,r, 0, 1)` làm chậm phục hồi khi bể cạn; `R_target,r` đo bằng đơn vị vật liệu.
- `d_r ∈ [0,8; 1,2]` phản ánh tiêu thụ và tồn kho; không dùng riêng giá giao dịch để quyết định.
- Đề xuất `d_r = clamp(1 + 0,2 × (u_r - 1) - 0,2 × (w_r - 1), 0,8, 1,2)`; `u_r` là tiêu hao 24 giờ / tiêu hao tham chiếu, `w_r = I_r / I_target,r`.
- Các tỷ số không có đơn vị; tham chiếu phải dương. Dùng trung bình trượt, trễ dữ liệu 6 giờ và không tính mua bán như tiêu hao.
- Trần tăng/giảm 5% là giảm dao động mục tiêu, **không** cản các giới hạn cứng làm ngân sách giảm ngay về 0.
- Chu kỳ đầu tiên khởi tạo `previous_r = r_base,r`; không lấy số ngẫu nhiên hoặc dữ liệu chưa có.
- Thiếu dữ liệu hoạt động: dùng mức cơ sở bảo thủ dưới mọi trần; lỗi đối soát bể: dừng phát hành liên quan.

| Tài nguyên | `r_base` / giờ | `H` / giờ | Trần ngày `D` | Phục hồi đề xuất |
| --- | ---: | ---: | ---: | --- |
| Linh thảo | 600 | 800 | 10.000 | 100% tiêu hao về `Q`, chờ 24 giờ |
| Mạch khoáng | 300 | 450 | 5.000 | 100% tiêu hao về `Q`, chờ 48 giờ |
| Mảnh Thần Mạch | 0 | 2 | 10 | Lịch sự kiện hữu hạn, không dùng công thức cấp độ |

Nguyên liệu hiếm có ngân sách sự kiện riêng trong cùng `M_r`; mở thêm boss không được tạo thêm ngân sách.
Ngân sách giờ/ngày chưa dùng hết sẽ hết hạn, không cộng dồn thành đợt xả lớn hôm sau.
Lượng chưa phát vẫn nằm ở `R`; lượng đã cấp tới điểm còn tồn ở `N` và chiếm trữ lượng điểm.
Vật liệu trong `Q` chỉ quay lại khi đủ thời gian; không ứng trước dựa trên tiêu hao dự kiến.
Tài nguyên thường mặc định tiêu hao theo thời gian thay vì hủy vĩnh viễn; nếu thử tỷ lệ phục hồi dưới 100%, phải đo và chấp nhận thời điểm bể cạn.

## 7. Phân bổ trên bản đồ, kênh và instance

- Một bộ cấp ngân sách phát hành cho toàn thế giới; máy bản đồ chỉ sử dụng phần đã được cấp.
- Mỗi cấp phát có `grant_id`, loại, lượng, thời hạn và trạng thái thu hồi.
- Chia ngân sách theo nhu cầu vùng đã làm mượt, có phần tối thiểu cho vùng khởi đầu; tổng phần không vượt `B_r`.
- Không tính lại `E` và phát trọn `B_r` trên từng kênh; không dùng số kênh như số người chơi.
- Chuyển kênh không đặt lại quota hỗ trợ hoặc kết quả boss của cùng nhân vật.
- Grant hết hạn không tự trả vào bể nếu chưa biết lượng nào đã dùng; phải đối soát trước khi cấp lại.
- Thời gian lỗi hoặc phân vùng mạng: chỉ tiêu phần grant còn hợp lệ; hết phần thì dừng tạo điểm mới.
- Điểm chung có lượng và nhịp phục hồi hữu hạn; hạn chế một người khóa điểm bằng cửa sổ khai thác liên tiếp.
- Instance cá nhân phải rút từ ngân sách chung nếu sản phẩm giao dịch được.
- Vật liệu thực hành không giao dịch, không chuyển thành vật liệu thị trường và không ghi vào chỉ báo giá.

## 8. Người mới, solo và độ khan hiếm

- Dự kiến dành 20% ngân sách thường cho hỗ trợ khởi đầu; phần này nằm trong `B_r`, không cộng thêm.
- Gói thử nghiệm: 20 Linh thảo và 10 Mạch khoáng một lần cho tài khoản đủ điều kiện trong thế giới pilot; nhân vật phụ dùng cùng dấu đã nhận.
- Gói khóa giao dịch ở giai đoạn đầu; vẫn ghi sổ và giới hạn số lượt nhận trong pilot.
- Lập kế hoạch slot onboarding từ ngân sách đã giữ trước; không mời vô hạn rồi hứa cấp gói vô hạn.
- Nếu ít người mới, chỉ chuyển phần chưa cam kết sang điểm công cộng sau thời hạn cấu hình.
- Nhiệm vụ, chiến đấu và học kỹ năng không bị chặn vì thị trường đã hết nguyên liệu hiếm.
- Raid bang hội tăng lựa chọn cách kiếm và phối hợp, không độc quyền toàn bộ nguồn nguyên liệu thiết yếu.
- Khi thiếu tài nguyên thường: ưu tiên tiếp cận khởi đầu và giảm chi phí thiết yếu theo bản cân bằng; không tạo thêm vật liệu âm thầm.
- Vật liệu hiếm dùng cho lựa chọn ngang, ngoại hình hoặc thành tựu thử nghiệm; không tạo đường thắng PvP bắt buộc.

## 9. Tiền tệ và thị trường

- Pilot dùng một tiền tệ nội bộ, không quy đổi ra tiền thật, không thiết kế như token đầu tư.
- Đề xuất bể tiền 12.000.000 đơn vị; phần chưa cấp nằm ở kho hệ thống, phần đã cấp nằm ở ví và ký quỹ.
- Bảo toàn tiền: `M_coin = Treasury + Wallets + Escrow + Burned`; mọi trạng thái đều cùng đơn vị tiền, không tính ký quỹ đồng thời trong ví.
- Thưởng nhiệm vụ, quái và bán cho NPC đều rút từ ngân sách tiền; không được tạo tiền ngoài sổ cái.
- Phí thị trường, mua từ NPC và dịch vụ tùy chọn chuyển tiền về kho hệ thống; tiêu hủy là trạng thái riêng nếu có.
- Ngân sách thưởng đề xuất tối đa 40.000 đơn vị/ngày toàn pilot; ngày mới không tăng trần bể tiền.
- Giao dịch người chơi chuyển tiền giữa hai ví; không tính toàn bộ giá bán như tiền mới phát hành.
- Phí bán đề xuất 1%; cách làm tròn phải nhất quán và không cho chia nhỏ giao dịch để né toàn bộ phí.
- Hủy lệnh hoàn ký quỹ đúng một lần; khớp lệnh, chuyển vật phẩm và thanh toán cùng giao dịch nguyên tử.
- NPC không mua vô hạn nguyên liệu tự tái sinh; ngân sách mua và lượng thu mua hữu hạn, công khai cho người chơi.
- Chưa có vay nợ, lãi suất, đấu giá phức tạp hoặc tiền riêng theo linh vực trong MVP.
- Theo dõi giá trung vị có trọng số khối lượng và rổ vật phẩm thiết yếu; bỏ mẫu thanh khoản thấp khỏi điều khiển tự động.
- Giao dịch tự mua bán, tài khoản liên quan và thao túng giá có thể làm sai chỉ báo; giá chỉ là tín hiệu bổ trợ.

## 10. Tiêu hao có ích và giới hạn khai thác

- Chế tạo nâng cấp ngang, thử bộ trang bị, công trình bang hội và vật phẩm tiện ích là nhu cầu chính.
- Sửa chữa có trần chi phí theo buổi chơi, mức thấp cho solo; chưa chốt tỷ lệ trước khi đo trải nghiệm.
- Không mất kỹ năng đã học, không phá trang bị do thất bại ngẫu nhiên để ép tiêu hao, không thu phí chỉ để đăng nhập.
- Thu thập có nhịp, sức chứa và khoảng di chuyển; server xác thực vị trí, thời gian, quyền tương tác và lượng còn lại.
- Giới hạn mềm theo tài khoản và nhóm tín hiệu giúp giảm farm, nhưng không làm tăng trần phát hành nếu bot lọt qua.
- Một người tạo nhiều tài khoản vẫn có thể chiếm phân bổ; trần toàn thế giới chỉ chặn tăng cung, không bảo đảm công bằng.
- Tách phát hiện bot khỏi quyền xử phạt; lưu bằng chứng, thử thách phù hợp và đường khôi phục khi nhận diện sai.
- Không coi CAPTCHA thường xuyên, nhận diện thiết bị hoặc xác minh danh tính là giải pháp mặc định hoàn chỉnh.

## 11. Ví dụ tính ngân sách và bảo toàn

Giả định một giờ Linh thảo có `E = E_ref = 3.000`, `d = s = 1`, `previous = 700`, `R_target = 800`.
Bể hiện có `R = 900`; trần giờ là 800, trần ngày còn 700, chỗ trống đủ và chưa bật trần suốt đời.

```text
target = 600 × (1 + 0,4 × ln(2)) ≈ 766,36 đơn vị/giờ
candidate = min(766,36; 700 × 1,05) = 735 đơn vị/giờ
B = min(735 × 1 giờ; 800; 700; 900) = 700 đơn vị
Hỗ trợ = 20% × 700 = 140; điểm công cộng = 560
```

Bảy gói 20 đơn vị được nhận và bảy điểm 80 đơn vị được cấp: toàn giờ chuyển đúng 700 từ bể.
Trước cấp: `(R, N, I, X, Q, Z) = (900, 4.000, 60.000, 25.000, 10.100, 0)`; tổng bằng 100.000.
Sau cấp và nhận gói: `(200, 4.560, 60.140, 25.000, 10.100, 0)`; tổng vẫn bằng 100.000.
Giờ tiếp theo `s = 200/800 = 0,25`; nguồn cung bị chặn bởi bể và trần ngày dù người chơi tăng.

## 12. Mô phỏng và tiêu chí nghiệm thu

Các mục sau là kiểm thử cần thực hiện, không phải kết quả đã đạt. Mô phỏng đề xuất 90 ngày với seed cố định và nhiều tập hành vi.

| Tình huống | Tiêu chí bắt buộc |
| --- | --- |
| Solo, co-op, bang hội và người mới | 95% lượt onboarding đủ điều kiện nhận gói trong 10 phút với số slot pilot đã cam kết |
| Số tài khoản tăng 10 lần trong một giờ | Mọi trần cấp phát giờ/ngày và `M_r` vẫn giữ; đo độ giảm tiếp cận của người thật |
| Bot giả làm người chơi hoạt động | Thử cả bot không bị phát hiện; nguồn cung không tăng vượt trần; báo cáo mức chiếm phân bổ |
| Mở kênh từ 1 lên 20 | Tổng lượng được cấp không thay đổi chỉ vì số kênh; không có grant hoặc điểm bị nhân bản |
| Retry, crash, phân vùng mạng | Không cấp/thu/hoàn hai lần; mọi đơn vị giữ trạng thái xác định hoặc bị khóa chờ đối soát |
| Tích trữ 80% kho, ít tiêu hao | Không in thêm để bù thiếu; đo nhu cầu starter, thời gian kiếm và lựa chọn giảm chi phí |
| Thị trường mất thanh khoản hoặc giá bị thao túng | Điều khiển dùng tín hiệu dự phòng; không vượt ±5% mục tiêu mỗi giờ và các trần cứng |
| Bể cạn hoặc `J_max` đã hết | Dừng phát hành đúng quy tắc; học kỹ năng và tiến triển thiết yếu không phụ thuộc vật liệu hiếm |
| Chế tạo, tháo dỡ và làm tròn | Sai số bảo toàn bằng 0 cho từng loại; không có vòng biến đổi sinh thêm nguyên liệu |

Mục tiêu giá đề xuất: đo biến động rổ thiết yếu 7 ngày trong kịch bản ổn định, thử ngưỡng ±10%; đây là cảnh báo để chỉnh thiết kế, không phải lời bảo đảm.
Theo dõi lượng mới cấp, lượt tái cấp, tiêu hao, `Q`, `Z`, tồn kho, tốc độ lưu thông tiền và thời gian kiếm của người mới mỗi ngày mô phỏng.
Trước MVP online phải chốt: nghĩa của giới hạn suốt đời, kích thước bể, công thức chế tạo, danh sách tiêu hao và số slot onboarding có thể đáp ứng.
