# MyVa — Thần Mạch: chiến đấu và điều khiển

> Bản nháp v0.1. Timing, damage, cooldown và ngưỡng mạng dưới đây phục vụ prototype; mọi thay đổi cần cập nhật cùng dữ liệu và phép đo.
> Thuật ngữ **frame animation** không đồng nghĩa **tick mô phỏng**. Server quyết định hành động và kết quả; client thể hiện, dự đoán và hòa giải trạng thái.

## 1. Mục tiêu

Chiến đấu thưởng cho việc đọc đối thủ, chọn khoảng cách và tiết kiệm tài nguyên. Cấp độ/trang bị giúp làm quen với nội dung PvE; không được phép che lấp mọi sai lầm. Combo ngắn và dễ đọc; các chuỗi dài cần cam kết, có khoảng trống và có cách thoát.

Ba loại quyết định diễn ra cùng nhau: **vị trí** (đất/không trung, gần/xa), **thời điểm** (đỡ/né/phản công), **tài nguyên** (sức bền/năng lượng/thanh Mạch). Mỗi đòn có startup, active, recovery, chi phí, hitbox và quy tắc cancel được khai báo rõ.

## 2. Bộ hành động và trạng thái

Mỗi build có **2 đòn cơ bản + 3 thuật được trang bị**. Di chuyển, nhảy, lướt và đỡ là hành động điều khiển chung. Một thuật được chọn để có phiên bản đại thuật; đại thuật dùng chính ô đó và không là kỹ năng thứ sáu.

| Trạng thái | Có thể thực hiện | Giới hạn |
| --- | --- | --- |
| Grounded/idle/move | Đánh, thuật, nhảy, lướt, đỡ | Gia tốc, tốc độ và đổi hướng có giới hạn |
| Airborne | Điều hướng giới hạn, đòn không trung hợp lệ, một lần lướt nếu dữ liệu cho phép | Không đỡ; không reset lướt chỉ vì chạm hitbox |
| Attack startup | Giữ hướng đã chọn, chờ đòn | Chỉ cancel nếu đòn khai báo cửa sổ trước cam kết |
| Attack active | Hitbox theo timeline | Không đổi mục tiêu hoặc kéo hitbox theo camera |
| Attack recovery | Chờ hoặc cancel theo dữ liệu | Không tự do né khỏi mọi đòn bị hụt |
| Guard | Chặn đòn từ phía trước | Không hồi sức bền; tốc độ di chuyển giảm |
| Hitstun/guard break | Chờ hồi, dùng cửa thoát hợp lệ nếu có | Áp dụng giảm hiệu lực khống chế trên server |
| Downed | Hồi sinh theo checkpoint | Không nhận thêm đòn hoặc dùng vật phẩm trong trạng thái này |

Địch không va chạm cứng với người chơi trong hub. Trong combat, pushbox và hitbox của địch có thể khác; không đẩy nhân vật ra ngoài vùng hợp lệ hoặc xuyên qua rào chắn để ghép combo.

## 3. Tài nguyên chiến đấu

| Tài nguyên mẫu | Vai trò | Giá trị prototype |
| --- | --- | --- |
| Sinh lực | Khả năng sống sót | Chuẩn hóa 1.000 trong phòng thử; PvE tính theo tiến triển |
| Sức bền | Lướt, đỡ, đòn nặng | 100; hồi 20/giây sau 600 ms không tiêu; không hồi khi đỡ |
| Năng lượng | Ba thuật | 100; hồi 6/giây sau 1.000 ms không dùng thuật |
| Mạch | Đại thuật | 0–100; nhận khi tác động hợp lệ trong chiến đấu; tiêu 100 một lần |

Đánh thường không cần năng lượng để người chơi luôn còn phương án. Lướt tiêu 25 sức bền; đòn nặng mẫu tiêu 12; đỡ tiêu 12/giây và `8 + 0,5 × áp lực đòn` mỗi hit được chặn. Chi phí không bao giờ âm; cạn sức bền dẫn tới guard break, không trừ sang thanh năng lượng.

Không nhận Mạch từ đánh vật trang trí, lặp đòn vào mục tiêu bất tử, đồng đội hoặc tài khoản đứng yên để farm. Trong prototype PvP, nguồn nhận Mạch gắn với sát thương/đỡ hợp lệ có trần theo thời gian và theo mục tiêu. Khi ra khỏi combat đủ lâu, Mạch giảm về mức khởi đầu của hoạt động; không tích đầy trước trận xếp hạng.

Không có hồi năng lượng bằng thanh toán. Vật phẩm hồi trong PvE, nếu thêm sau slice, dùng cooldown nhóm và ngân sách rõ; chưa đưa vào PvP chuẩn hóa.

## 4. Timeline và simulation

Đề xuất thử server **60 Hz** và snapshot **20 Hz**; chọn chính thức sau benchmark CPU, băng thông và latency. Thiết kế lưu timing bằng mili giây; ở 60 Hz, đổi sang tick bằng `ceil(ms × 60 / 1000)` để hành động không xảy ra sớm hơn mô tả.

Ví dụ: startup 180 ms trở thành 11 tick, active 100 ms thành 6 tick, recovery 240 ms thành 15 tick. Dữ liệu runtime phải công bố timing thực sau lượng tử hóa. Visual marker theo timeline này; thay tốc độ phát sprite không tự thay sát thương hoặc cửa phòng thủ.

Input có sequence, hướng và hành động; server xác thực trạng thái, cooldown, tài nguyên, vị trí và thứ tự trước khi thực thi ở tick hợp lệ. Client không gửi một kết quả “đã trúng”, damage, vật phẩm thưởng hay số dư mới.

Mỗi lần ra đòn có action ID và tập mục tiêu đã trúng; active kéo dài nhiều tick không tự gây damage mỗi tick. Đòn nhiều hit phải khai báo từng hit index, khoảng cách thời gian và số hit tối đa. Reconnect không làm mất tập hit hoặc tái tạo đạn đã hết hiệu lực.

Có buffer hành động tối đa 120 ms khi đang ở cuối recovery: hành động được server chấp nhận sẽ chạy ở cửa sổ hợp lệ tiếp theo. Buffer không phát lại thao tác cũ khi đổi trạng thái, bị hạ, mở UI hoặc mất kết nối; mỗi input chỉ được áp dụng một lần.

Slice không dùng rollback hoặc rewind để sửa kết quả combat theo timestamp client. Thử bù trễ PvP sau này phải có giới hạn và kiểm thử riêng; không đưa damage hoặc quyền phòng thủ về client.

## 5. Hướng đánh, đỡ và né

Hướng được xác lập khi bắt đầu startup; có thể đổi trước điểm cam kết nếu đòn khai báo, nhưng không xoay tức thời sau active để bắt mục tiêu phía sau. Auto-facing là tùy chọn khi đánh ở trạng thái trung tính, chỉ chọn mục tiêu trong vùng và không đổi hướng giữa một đòn.

Đỡ chặn đòn trong hình nón phía trước ±75° và chỉ hoạt động khi đứng trên nền. Startup đỡ mẫu 100 ms. Đòn phía sau, hazard môi trường và đòn được đánh dấu unblockable không bị chặn; chúng phải có tín hiệu khác nhau, không chỉ đổi màu.

Perfect guard là 90 ms đầu sau khi đỡ active, không phải 90 ms từ lúc chạm nút. Đỡ thường là công cụ chính; perfect guard thưởng sức bền/giảm recovery hợp lý, không bắt buộc để hạ boss truyện. Chỉ một hiệu ứng perfect guard cho mỗi pha đánh đã xác nhận.

Lướt có startup 50 ms, di chuyển 220 ms, recovery 140 ms; khoảng bất tử mẫu 100 ms nằm trong đoạn di chuyển. Khoảng bất tử không áp dụng cho vực, rào bản đồ hoặc mọi hazard. Hitbox và trạng thái lướt do server tính; không dịch chuyển bằng vị trí client đề xuất.

Guard break có recovery mẫu 500 ms và tín hiệu âm thanh/tư thế rõ. Đòn unblockable nguy hiểm có startup dài hơn; không tạo một loại đòn vừa không đỡ được vừa bao phủ tất cả hướng thoát.

## 6. Combo và cancel

| Kết nối mẫu | Cửa sổ | Điều kiện |
| --- | --- | --- |
| Nhẹ 1 → nhẹ 2 → nhẹ 3 | 100 ms cuối recovery | Nhập tiếp; tối đa ba nhịp; không tự lặp khi giữ nút |
| Nhẹ xác nhận trúng → thuật | 80 ms sau hit trên timeline server; cho buffer ý định trước hit | Server xác nhận trúng và điều kiện thuật; không áp dụng khi đánh hụt |
| Đòn nặng → thuật | Không mặc định | Chỉ mở cho dữ liệu đã kiểm thử, không toàn bộ kit |
| Đòn đánh → lướt | 120 ms cuối recovery trên hit confirm | Tiêu sức bền; đòn hụt phải chịu recovery đầy đủ |
| Đỡ → phản công | Sau perfect guard hoặc hết blockstun | Có thời gian ra đòn; không gây sát thương ngay khi guard thành công |
| Thuật → thuật/đại thuật | Chỉ khi khai báo | Mọi nhánh vẫn chịu ngân sách damage, CC và tài nguyên |

Input ở ngoài cửa sổ không bị biến thành combo bí mật. UI luyện tập hiện trạng thái “trúng/hụt/đã chặn” và cửa nối bằng tín hiệu nhẹ; không ép học chuỗi mệnh lệnh dài kiểu đối kháng arcade.

**Hit confirm có dự đoán:** client có thể hiện phản hồi trúng dự đoán và gửi ý định nối thuật kèm action ID trước đó ngay khi người chơi nhập; không chờ round trip nhận hit confirm từ server. Ý định chỉ là yêu cầu, không là bằng chứng đã trúng. Server ghi `acceptedTick` sau khi nhận và xác thực input, giữ ý định trước hit tối đa 120 ms; chỉ cancel sau khi chính server xác nhận hit và điều kiện thuật. Ý định đến trong 80 ms sau hit có thể được thực thi ở tick hợp lệ trong cửa sổ; đánh hụt, hết buffer, hết cửa sổ hoặc thiếu tài nguyên thì bị từ chối, client hòa giải animation/tài nguyên đã dự đoán.

Server không nhận `acceptedTick` do client chỉ định và không lùi cửa sổ theo timestamp client. 80 ms là giả thuyết về cửa cancel, không là thời gian đủ để chờ xác nhận qua mạng; đo tỷ lệ ý định hợp lệ bị từ chối và chênh lệch cảm giác nối đòn ở các profile RTT/jitter. Nếu buffer cùng prediction vẫn không cho kết quả ổn định ở profile mục tiêu, nới cửa sổ hoặc đổi nhánh combo trước khi chốt, thay vì âm thầm trao hit authority cho client.

Trong PvP prototype: một chuỗi không cho phản ứng có mục tiêu ≤800 ms và ≤25% sinh lực chuẩn; không xem đây là trần damage của mọi chiến thuật. Combo dài hơn phải có đoạn có thể đỡ/né/thoát. Không có true combo lặp vô hạn khi đủ cooldown hoặc khi nhiều người thay nhau nâng mục tiêu.

Khống chế mạnh dùng bộ đếm chung cho mọi nguồn trong cửa sổ trượt 8 giây: lần 1 đủ thời gian, lần 2 70%, lần 3 40%, lần 4 gây 0 ms và cho miễn khống chế mạnh 1 giây. Thời gian mẫu mỗi lần stun ≤300 ms trong PvP; knockup tối đa một lần mỗi 6 giây. Displacement vẫn chịu kiểm tra bản đồ và không là cách né bộ đếm.

Slow không xếp cộng vô hạn: chỉ hiệu lực mạnh nhất, giới hạn 25% ở PvP prototype. Root, stun, launch và kéo cưỡng bức đều thuộc nhóm mạnh; boss dùng luật riêng nhưng không bị khóa vô hạn bởi 4 người chơi.

## 7. Kit mẫu: Long Lưu

Chủ đề: điều tiết dòng chảy, giữ khoảng cách vừa và phản công. Không phải bộ kỹ năng thuần hồi máu hoặc phiên bản của một vị thần có thật. Tất cả giá trị dưới đây là tương đối ở phòng thử với 1.000 HP.

| Hành động | Startup / active / recovery | Chi phí / cooldown | Hiệu ứng mẫu |
| --- | --- | --- | --- |
| Đòn nhẹ — Gợn Sóng | 180 / 100 / 240 ms | 0 / theo timeline | 45 damage, hitstun 120 ms; ba nhịp có nhịp 3 đẩy lùi nhẹ |
| Đòn nặng — Phá Lưu | 400 / 130 / 360 ms | 12 sức bền | 90 damage, áp lực guard 24; hụt phải chịu recovery |
| Thuật 1 — Lưu Tiễn | 300 / 100 / 300 ms | 20 năng lượng / 3 s | Đạn thẳng 80 damage, một hit mỗi mục tiêu, biến mất ở vật cản |
| Thuật 2 — Hồi Thế | 160 / 300 / 300 ms | 25 năng lượng / 7 s | Tư thế phản công có giới hạn phía trước; bắt được một đòn thì gây 65 damage; không phản hazard/unblockable |
| Thuật 3 — Triều Dâng | 450 / 150 / 450 ms | 35 năng lượng / 10 s | Sóng cận-trung 100 damage; đẩy, không launch mặc định; đứng yên khi cam kết |
| Triều Dâng tăng cường | 650 / 200 / 550 ms | 100 Mạch + chi phí thuật / cooldown chung | 160 damage theo một đợt rộng hơn; đỡ/né được; không xuyên mọi vật cản |

Đại thuật cần xác nhận chủ động: giữ nút thuật được chọn hoặc dùng chế độ toggle rõ trong cài đặt; chạm ngắn vẫn dùng thuật thường. Nếu thiếu năng lượng/cooldown, không tiêu Mạch. Server trừ cả hai tài nguyên trong một lần thực thi; mạng chập chờn không tạo phiên bản thường và tăng cường cùng lúc.

Lưu Tiễn không homing trong slice. Giới hạn khoảng cách/projectile lifetime được ghi trong data. Hồi Thế không tự tìm mục tiêu phía sau; thất bại vẫn trả chi phí và chịu recovery. Người chơi cần chọn đúng tình huống thay vì bấm theo cooldown.

## 8. Khung hai kit còn lại

| Truyền thừa | Thế mạnh | Hạn chế bắt buộc | Ba thuật gợi ý |
| --- | --- | --- | --- |
| Sơn Cốt | Giữ vị trí, áp lực cận chiến, bảo vệ khoảng trống | Startup lớn, khó đổi hướng, không cộng giáp vô hạn | Thạch Chấn, Trấn Sơn, Cốt Kích |
| Phong Vũ | Chuyển động, đổi góc và đạn nhẹ | HP/áp lực guard thấp hơn; lướt không miễn phí | Phong Tiễn, Vũ Bộ, Gió Quẩn |

Ba kit phải đi qua cùng phòng thử: quái cận chiến, quái tầm xa, platform, boss và tổ hợp co-op. Không thiết kế một kit chỉ khả dụng khi có đồng đội hoặc kit có thể bỏ qua toàn bộ hazard bằng bay liên tục.

## 9. Boss: Kẻ Giữ Đập

Boss bảo vệ một cơ chế điều tiết đã hỏng; người chơi khôi phục dòng chảy thay vì chỉ giết mục tiêu có nhiều HP. Đấu trường có nền chính đủ rộng, hai bệ cao đọc được và van điều tiết. Không dùng phá nền ngẫu nhiên dưới chân người chơi.

| Pha | Điều kiện | Mẫu hành vi và thời gian báo |
| --- | --- | --- |
| 1 — Nhận diện | 100–70% HP | Quét ngang 800 ms; đập đất 1.000 ms; nhịp nghỉ đủ thử phản công |
| 2 — Đổi địa hình | 70–35% HP | Nước dâng 1.600 ms, chỉ một phần nền; đạn vòng cung 900 ms; một van mở cửa phản công |
| 3 — Phối hợp quy luật | <35% HP | Nối hai đòn đã học; mỗi tín hiệu ≥700 ms, không rút ngắn startup bất ngờ; kích van làm lộ điểm yếu |

Không đổi pha giữa một đòn theo cách hủy tín hiệu đang hiển thị; chờ đòn kết thúc rồi chuyển. Pha mới có một nhịp giới thiệu an toàn. Sát thương đã gây và điều kiện thắng do server tính; thanh HP chỉ là thể hiện.

Co-op 1–4 người: số đạn hoặc vị trí áp lực có thể tăng, nhưng luôn có lối thoát. Van không cần hai người đứng đồng thời, nên solo vẫn giải được. Mục tiêu boss ưu tiên rõ bằng tư thế; không quay đòn ngang sang người khác ở active.

Chết trước boss cho thử lại tại checkpoint với tài nguyên chiến đấu đầy; cắt đoạn đi bộ lặp. Sau ba thất bại cùng cơ chế, gợi ý ngắn có thể bật; không tự tăng chỉ số hoặc bán quyền bỏ qua.

## 10. Công bằng tín hiệu và độ trễ

700 ms là mức báo tối thiểu đề xuất cho đòn boss bắt buộc người mới phải phản ứng, không phải startup tối thiểu mọi đòn PvP. Đòn cơ bản ngắn dùng dự đoán và khoảng cách; không thể yêu cầu phản xạ luôn thắng mọi input đối thủ.

Prototype đo với RTT 50/100/150/250 ms, jitter 0/30/60 ms và mất gói 0/1/3%. Mốc chơi mục tiêu ban đầu là RTT ≤150 ms, jitter ≤30 ms; 250 ms dùng để đánh giá suy giảm và thông báo chất lượng kết nối.

Với boss, thời gian từ tín hiệu người chơi nhìn thấy đến deadline phòng thủ phải dư cho phản ứng, truyền input và startup phòng thủ. Ngân sách mẫu ở profile mục tiêu: 250 ms phản ứng + 200 ms dự phòng mạng/snapshot + 100 ms startup guard + 150 ms dư = 700 ms. Đây là giả thuyết đo, không bảo đảm phản ứng của mọi người hoặc mọi mạng.

Thu thập timestamp server về telegraph, input tới, guard active và damage; không ghi dữ liệu phím ngoài game. Nếu profile mạng mục tiêu liên tục khiến input hợp lệ đến sau deadline, tăng tín hiệu hoặc đổi cơ chế trước khi thu hẹp cửa phản ứng.

## 11. Platform và đông người

Nhảy có coyote time mẫu 80 ms, jump buffer 100 ms; trạng thái và mốc nền vẫn được server xác thực. Độ cao theo hold là tùy chọn, có chế độ nhảy một lần với độ cao cố định; các đường bắt buộc không đòi hold chính xác vài mili giây.

Phòng thử có nhảy qua khe, nền nâng, đạn theo chu kỳ và một tuyến dễ thay thế. Vực đưa về điểm an toàn gần nhất và trừ lượng HP nhỏ đã chốt; không rơi mãi hoặc mất đồ. Checkpoint cần giữ cảm giác tiến triển, không tua lại cả bản đồ sau một lỗi platform.

Người chơi không dùng thân thể để chắn nền/đường; player crowd không thay hitbox hazard. Thử thách nhảy chính xác ở instance; nền công cộng và tương tác van không bị một người giữ vô hạn. Trợ giúp khóa mục tiêu không tự nhảy hoặc tự né qua cơ chế.

## 12. Điều khiển và khả năng tiếp cận

| Hành động | Web mặc định | Gamepad mặc định |
| --- | --- | --- |
| Di chuyển | A/D hoặc trái/phải | Stick trái / D-pad |
| Nhảy | Space | A |
| Lướt | Shift | B |
| Đỡ | L | LB |
| Nhẹ / nặng | J / K | X / Y |
| Thuật 1/2/3 | Q/E/R | RB + X/Y/B |

Modifier gamepad phải phân biệt chord trước khi phát đòn cơ bản; bỏ RB không vô tình tạo đòn X/Y/B. Tất cả mapping đổi được, có deadzone và thiết bị theo thứ tự input thực tế; không bắt gamepad có tên nút A/B giống nhau.

Mobile landscape dùng joystick bên trái, cụm đòn/thuật bên phải và nhóm nhảy/lướt/đỡ gần ngón cái. Mặc định prototype có **8 nút**: nhẹ, nặng, ba thuật, nhảy, lướt, đỡ. Chia hai cụm có vị trí tùy chỉnh, phân cấp nút thường dùng; không thêm một nút đại thuật hoặc thanh nút phụ bắt buộc.

Bố cục gọn tùy chọn có **7 nút**, gộp nhẹ/nặng: thả dưới 180 ms tạo nhẹ; giữ đến 180 ms bắt đầu startup nặng, thả sau đó không phát thêm nhẹ. Nó tăng thời gian nhận input so với nút riêng; đo kích nhầm và độ trễ trước khi khuyến nghị cho PvP. Không bắn đòn nhẹ ở touch-down rồi tự đổi thành nặng. Nút thuật có lựa chọn giữ tăng cường hoặc toggle đại thuật; trạng thái toggle hiện rõ và reset ở checkpoint/đổi hoạt động.

Cho đổi vị trí/kích thước/độ mờ nút, giữ vùng chạm ít nhất 48 dp, safe area và cách nhấn đỡ: giữ hoặc toggle. Toggle đỡ vẫn áp dụng cùng sức bền/startup; mở chat/menu tự hủy đỡ sau quy tắc an toàn đã công bố, không giữ mãi ngoài tầm nhìn.

Không yêu cầu double-tap để lướt, vuốt để combo hoặc phân biệt màu để phản ứng. Có giảm flash/rung, dùng biểu tượng/âm thanh rõ, và bộ luyện tập với tốc độ mô phỏng chậm chỉ ở phòng offline; không đổi thời gian server khi vào trận online.

## 13. Chỉ số, PvP và nghiệm thu

PvE cùng dải nhiệm vụ: ngân sách prototype giới hạn chênh damage cuối từ trang bị ±15% quanh bộ nhiệm vụ chuẩn và chênh HP ±15%; không cộng các nguồn thành hệ số không giới hạn. Dải này phải đo lại ở MVP; không áp dụng cứng một công thức cho toàn bộ cấp độ.

PvP cạnh tranh, nếu làm sau MVP, dùng template HP/thuộc tính chuẩn và ngân sách biến thể ngang cấp. Mỹ phẩm không đổi tầm đòn hoặc độ rõ tín hiệu; không mang item hồi PvE vào đấu. Duel chưa chuẩn hóa phải gắn nhãn rõ và không có phần thưởng kinh tế có thể farm.

| Tình huống nghiệm thu | Bằng chứng cần có |
| --- | --- |
| Input lặp/mất kết nối | Một input chỉ gây một hành động; không tiêu hai lần hoặc nhân đôi damage |
| Tấn công hụt | Không cancel tự do hoặc xoay hitbox sau active |
| Đỡ sai hướng | Đòn sau lưng trúng; hình nón và timeline trùng mô tả |
| Combo nhiều nguồn | Stun DR dùng chung; không nối launch vô hạn với 4 người |
| Boss ở profile mạng mục tiêu | Input đúng thời điểm theo ngân sách đạt kết quả nhất quán; không hidden hitbox |
| Solo cả ba kit | Đánh boss bằng đồ tuyến nhiệm vụ; mọi cơ chế có phương án không cần party |
| Touch | Đo cả kích nhầm và trễ thả; hoàn thành thử thách bằng layout gọn và mở rộng |
| Visual | Hitbox, telegraph và cue luôn tồn tại khi giảm hiệu ứng hoặc thiếu asset mỹ phẩm |

Trước khi mở rộng kit, ghi dữ liệu damage, tỷ lệ dùng đòn, thời gian boss, nguyên nhân chết, hiệu lực guard/dash và lỗi mạng. Review các combo tốt nhất lẫn hành động không ai dùng; không chỉ cân bằng theo DPS trung bình.
