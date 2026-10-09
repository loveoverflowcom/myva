# MyVa — Thần Mạch: Game Design Document

> Bản nháp thiết kế v0.1. Các con số là giả thuyết để làm prototype và đo kiểm, chưa phải thông số đã cân bằng.
> Tên tiếng Việt chính thức: **Thần Mạch**. Tên dự án và repository: **MyVa / myva**.

## 1. Đề xuất sản phẩm

MyVa là MMORPG hành động 2D nhìn ngang, nơi người chơi kế thừa một mạch sức mạnh thần thoại, học cách điều khiển nó và đi qua những vùng văn hóa khác nhau. Người chơi là con người có lựa chọn và giới hạn; không trực tiếp trở thành một vị thần có toàn quyền.

Trải nghiệm cốt lõi: một thế giới có người khác cùng khám phá, một trận đánh thắng bằng đọc tình huống, và một hành trình đủ ý nghĩa khi chơi một mình. Hạ boss giúp mở đường và hiểu thế giới; cấp độ, trang bị và tài nguyên hỗ trợ hành trình này.

Ngọc Rồng Online gợi ý nhịp khám phá và gắn bó cộng đồng; Mortal Kombat 3 gợi ý sự rõ ràng của combo và phản công; Mega Man gợi ý địa hình và quy luật boss. MyVa xây dựng nhân vật, hình ảnh, cốt truyện, luật và nội dung riêng, không dùng lại tài sản hay tên gọi của các IP này.

## 2. Review và bổ sung draft

| Điểm trong draft | Đánh giá | Quyết định đề xuất |
| --- | --- | --- |
| Thần thoại đa văn hóa | Có bản sắc, nhưng quá rộng để khởi đầu | Một linh vực hư cấu lấy cảm hứng Đông Nam Á; các vùng khác có hồ sơ nghiên cứu trước khi sản xuất |
| MMO + đối kháng + platformer | Hấp dẫn nhưng dễ xung đột về độ trễ, camera và số nút | Combat ngắn, tín hiệu rõ; thử thách chính xác đặt trong instance; không yêu cầu phản xạ từng frame |
| Solo, cày chay, bang hội | Phù hợp nhiều nhóm người chơi | Cốt truyện và tiến triển cơ bản hoàn thành solo; bang hội có nội dung hợp tác và ảnh hưởng xã hội riêng |
| Tài nguyên theo tổng cấp độ | Dễ bị tài khoản phụ và tích trữ thao túng | Cấp độ hiệu dụng chỉ là một tín hiệu; còn có hoạt động hợp lệ, trữ lượng, tiêu hao và ngân sách phát hành |
| Không lạm phát | Là mục tiêu tốt, không thể cam kết tuyệt đối bằng một công thức | Định nghĩa chỉ số, biên điều khiển và cơ chế xử lý sự cố; tách tài nguyên vật lý khỏi tiền tệ |
| Skeletal animation + 2D HD | Cần kiểm chứng chi phí và runtime | Prototype sprite atlas trước; thử skeletal cho một nhân vật nếu thực sự giảm chi phí sản xuất |
| Bevy native trong CMP | Đúng hướng nhưng là rủi ro tích hợp riêng | Spike render surface, input, vòng đời và bộ nhớ trước khi hứa ngang bằng web/mobile |
| Chiến tranh liên lục địa | Là đích dài hạn | Không đưa vào MVP; phải chứng minh combat, duy trì nội dung và vận hành kinh tế trước |

## 3. Các trụ cột có thể kiểm chứng

1. **Dễ nhập cuộc:** đi lại, đánh thường, né và nhận nhiệm vụ đầu trong 10 phút; người chơi hiểu vì sao bị trúng đòn.
2. **Khó thành thạo:** hướng đánh, vị trí, tài nguyên chiến đấu và lựa chọn phản công tạo khác biệt; lực chiến không giải quyết mọi tình huống.
3. **Truyền thừa có bản sắc:** bộ kỹ năng của một vùng có hình thái, vai trò và giới hạn riêng; học vùng khác tạo lựa chọn, không cộng vô hạn sức mạnh.
4. **Solo có đường đi:** có thể hoàn thành tuyến chính, mở khóa vùng và sở hữu trang bị đủ dùng mà không cần bang hội hay thanh toán.
5. **Thế giới có sức chứa:** tài nguyên tái sinh trong giới hạn, tài sản được tiêu hao có lý do, và quy mô người chơi không tự động cấp quyền phát hành vô hạn.

## 4. Người chơi và phiên chơi

| Kiểu phiên | Thời lượng mục tiêu | Hoạt động |
| --- | --- | --- |
| Ngắn trên mobile | 5–10 phút | Một nhiệm vụ, thu thập có mục tiêu, thử một đoạn boss |
| Khám phá/co-op | 20–35 phút | Đi qua vùng, tìm đường phụ, hoàn thành instance |
| Hoạt động cộng đồng | 45–60 phút | Sự kiện hoặc nhiệm vụ bang hội có điểm kết thúc rõ |

Không dùng energy theo ngày để chặn chơi. Những giới hạn về khai thác và nguồn cung thuộc thế giới và vận hành chống lạm dụng, không bán quyền bỏ qua chúng.

## 5. Vòng lặp gameplay

| Nhịp | Hành động | Kết quả có ý nghĩa |
| --- | --- | --- |
| Vài giây | Đọc động tác → di chuyển/đỡ/né → chọn đòn → quản lý sức bền | Tạo khoảng trống, bảo toàn sinh lực, hiểu đối thủ |
| Vài phút | Khám phá → nhiệm vụ/thử thách → thu thập → về điểm an toàn | Mở đường, nâng độ thành thạo, chế tạo hoặc sửa trang bị |
| Một phiên | Chọn mục tiêu → chuẩn bị bộ kỹ năng → boss/co-op → nhận kết quả | Tiến triển truyện hoặc cải thiện một lựa chọn build |
| Nhiều phiên | Phát triển truyền thừa → học kỹ năng giao thoa → góp phần vào vùng | Mở cách chơi, quan hệ và tuyến nội dung mới |

Nhiệm vụ đầu giải thích hành động qua tình huống: cứu người ở cầu, chặn một đợt nước và tìm vật liệu sửa đường. Không đưa bảng thuật ngữ dài lên trước thao tác chơi.

## 6. Phạm vi nội dung đầu tiên

Linh vực đầu: **Vân Thủy**. Điểm xuất phát: **Bến Lau**. Đây là vùng hư cấu, không đại diện cho toàn bộ văn hóa Đông Nam Á. Chi tiết văn hóa và nhân vật được phát triển trong World Bible.

| Bản đồ slice | Chức năng | Nội dung chính |
| --- | --- | --- |
| Bến Lau | Hub, hướng dẫn, chỗ nghỉ | NPC nhiệm vụ, điểm hồi sinh, chế tạo thử nghiệm, luyện đỡ/né |
| Rừng Bậc Nước | Khám phá ngoài trời | Quái thường, tuyến thu thập, nhánh platform ngắn và đường dễ thay thế |
| Đập Cổ | Instance thử thách | 2 nhóm địch, checkpoint, boss **Kẻ Giữ Đập** |

Ba truyền thừa đề xuất: **Long Lưu** (dòng chảy/điều tiết), **Sơn Cốt** (kiên định/áp lực cận chiến), **Phong Vũ** (chuyển động/tầm đánh). Người chơi thử cả ba trước khi chọn; đổi truyền thừa tại hub trong slice để hỗ trợ kiểm thử.

Slice có một tuyến truyện ngắn, 6–8 nhiệm vụ gồm các bước hướng dẫn, 3 archetype địch thường, một boss và một chuỗi chế tạo mẫu. Không suy rộng số lượng nội dung này thành cam kết MMO hoàn chỉnh.

## 7. Nhân vật, trang bị và tiến triển

| Giai đoạn | Cấp độ đề xuất | Quyền mở khóa |
| --- | --- | --- |
| Vertical Slice | 1–10 | Hai đòn cơ bản, ba thuật, sửa/chế tạo, boss và một lựa chọn nâng thuật |
| MVP Online | 1–20 | Tuyến truyện Vân Thủy mở rộng, co-op, biến thể bộ kỹ năng, giao thương giới hạn |
| Sau MVP | Chưa chốt | Vùng thứ hai, kỹ năng giao thoa, sự kiện liên vùng |

Cấp độ mở quyền dùng và giúp đọc tiến độ; không liên tục cộng mọi chỉ số theo cùng một hệ số. Trang bị tạo đánh đổi, ví dụ hồi thuật tốt hơn đổi lấy sức bền thấp hơn. Vật phẩm mạnh phải nằm trong ngân sách thuộc tính, không có mọi ưu điểm cùng lúc.

Người chơi trang bị **5 hành động chiến đấu: 2 đòn cơ bản + 3 thuật**. Đại thuật là phiên bản tăng cường của một thuật đã chọn, tiêu hao thanh Mạch; không thêm nút kỹ năng thứ sáu. Chi tiết tài nguyên, giới hạn combo và đối đầu trong [combat.md](combat.md).

Kỹ năng từ vùng khác chỉ được dùng trong số ô hiện có. Một nhân vật có thể học nhiều thuật nhưng không mang toàn bộ vào trận. Không buộc đổi linh vực gốc để học hoặc chơi với bạn bè.

Tiến độ cốt truyện của mỗi người được lưu riêng. Co-op cho phép người đã hoàn thành chơi lại; chỉ thưởng tiến triển một lần, phần thưởng lặp có ngân sách riêng. MVP chưa có trao đổi vật phẩm ràng buộc nhiệm vụ.

## 8. Cấu trúc thế giới và đông người

Hub và bản đồ ngoài trời là không gian chia sẻ theo shard/channel. Người chơi không va chạm thân thể với nhau, không chắn lối hoặc làm mất nền đứng của người khác. Admission limit đề xuất cho pilot là 50 phiên/channel và 200 phiên đồng thời toàn pilot, thống nhất với [kiến trúc](../technical/architecture.md); đây là giới hạn thử nghiệm, chưa phải capacity đã đạt hay giới hạn MMO cuối cùng.

Platform khó, boss truyện và co-op dùng instance với 1–4 người. Camera giữ vùng nguy hiểm trong tầm nhìn; người vào sau xuất hiện tại checkpoint, không rơi thẳng vào đòn đang diễn ra. Bản đồ công cộng có thể có chiến đấu nhưng không dùng platform chính xác làm cửa bắt buộc đi qua một đám đông.

Khai thác không cho người đứng trước độc chiếm cả một tuyến mới chơi. Vật liệu hướng dẫn được bảo đảm bằng phần thưởng nhiệm vụ giới hạn; node công cộng và tài nguyên giao dịch tuân theo ngân sách vùng. Không dùng cơ chế nhân bản vật phẩm cho mỗi người mà bỏ qua tổng phát hành.

## 9. Solo, co-op và bang hội

| Chế độ | Phạm vi | Quy tắc |
| --- | --- | --- |
| Solo PvE | Slice/MVP | Tuyến chính hoàn thành một mình; boss có checkpoint và nhắc lại tín hiệu sau thất bại |
| Co-op PvE | MVP | Nhóm 1–4 người, tăng độ khó theo thành viên hợp lệ; tránh boss chỉ tăng HP |
| Bang hội cơ bản | MVP | Tạo/tham gia, vai trò, bảng nhiệm vụ hợp tác, chat và báo cáo; chưa có kho tài sản dùng chung |
| Duel opt-in | Sau khi combat/network đạt kiểm thử | PvP chuẩn hóa, không mất vật phẩm; không ảnh hưởng tuyến truyện |
| Cứ điểm/raid/world boss | Sau MVP | Cần thiết kế đóng góp, tránh tranh last-hit, quá tải và nuôi tài khoản phụ |
| Liên lục địa | Dài hạn | Nhiều mặt trận với lịch, điều kiện thắng và tài nguyên chiến dịch có giới hạn |

Không bắt người chơi solo phải gia nhập bang hội để mở linh vực. Bang hội nhận quyền tổ chức, phối hợp và xây dựng danh tiếng; vật phẩm sức mạnh vượt trội độc quyền không phải phần thưởng mặc định.

## 10. Thất bại và phục hồi

Khi bị hạ trong PvE, hồi sinh ở checkpoint gần nhất. Giữ trang bị và vật phẩm đã được server xác nhận; không làm rơi đồ cho người khác nhặt. Boss reset có kiểm soát khi cả nhóm thất bại, tài nguyên chiến đấu được phục hồi theo checkpoint.

Độ bền và chi phí sửa là hệ tiêu hao cho MVP, phải đặt trần và luôn có bộ đồ căn bản sử dụng được. Slice không dùng mất độ bền vì chết để tránh nhiễu việc đo combat. Không thu phí để hồi sinh hoặc giữ phần thưởng.

Ngắt mạng không được dùng để xóa thiệt hại hay nhân đôi phần thưởng. Chính sách reconnect và xử lý nhân vật đang trong trận thuộc tài liệu kiến trúc; thông điệp trong game phải nói rõ nhân vật đang được giữ hay đã trở về checkpoint.

## 11. Kinh tế, thương mại và doanh thu

Tài nguyên thu thập là hàng hóa trong thế giới; tiền giao dịch là sổ cái khác. Khai thác một node không tự đúc tiền. Chế tạo, sửa chữa, vận chuyển và xây dựng có thể tạo tiêu hao, nhưng phải tạo tiện ích và được kiểm thử để không trở thành phí bắt buộc quá mức.

Tài liệu [kinh tế](economy.md) chốt các loại tồn kho, tốc độ hồi phục, điều kiện hoạt động hợp lệ, sink, giới hạn giao dịch, chỉ số giá và phản ứng khi hệ thống lệch mục tiêu. Không dùng toàn bộ người đăng ký hoặc tổng cấp độ trọn đời làm đầu vào phát hành.

Đề xuất doanh thu: bán mỹ phẩm trực tiếp với giá hiển thị rõ, màu hiệu ứng, ngoại hình và vật trang trí. Mỹ phẩm phải giữ silhouette, hitbox và tín hiệu combat. Không bán chỉ số, vật liệu hiếm, ưu tiên khai thác, lượt reset boss, EXP boost, loot box hay quyền bỏ qua hạn mức kinh tế.

## 12. Hình ảnh, âm thanh và khả năng tiếp cận

2D HD theo hệ hình ảnh riêng; atlas có quy tắc pivot, chân đứng, kích thước và vùng va chạm tách khỏi sprite. Chuyển động và đòn đánh cần đọc được ở màn hình điện thoại trước khi tăng chi tiết hoặc hiệu ứng. 60 FPS là mục tiêu đo trên danh sách thiết bị, không là lời hứa khi chưa benchmark.

Đòn nguy hiểm có ít nhất hai kênh tín hiệu: hình dáng/chuyển động + âm thanh hoặc biểu tượng. Không dùng màu đỏ/xanh làm tín hiệu duy nhất. Cho giảm rung, flash, camera shake, motion và mật độ hiệu ứng của người khác; giữ nguyên dấu hiệu có ảnh hưởng gameplay.

Web hỗ trợ keyboard và gamepad; mobile chơi landscape với điều khiển trực tiếp. Có đổi nút, chọn tap/hold thay thế, điều chỉnh kích thước/vị trí nút, vùng an toàn và thao tác một tay khi đang ở hub. Bố cục combat cần kiểm thử theo [combat.md](combat.md).

## 13. Phân kỳ và cổng thông qua

| Mốc | Bằng chứng để thông qua | Ngoài phạm vi |
| --- | --- | --- |
| GDD/World Bible | Một tuyến chơi nhất quán, glossary và các giả thuyết cần đo | Chưa sản xuất tất cả linh vực |
| Prototype kỹ thuật | Di chuyển, input, render web/native; đo bộ nhớ, frame time, vòng đời | Chưa gọi là MMO |
| Vertical Slice | 3 map, 3 kit, 1 boss, checkpoint, telemetry, tài nguyên có trữ lượng và hồi phục | Không giao dịch người chơi, không bang hội, không PvP cạnh tranh |
| MVP Online | Server authority, tài khoản, persistence, party, co-op, nhiệm vụ, giao thương giới hạn, bang hội cơ bản | Chưa liên lục địa hoặc đại chiến đông người |
| World Expansion | Nội dung vùng thứ hai, nghiên cứu văn hóa, kỹ năng giao thoa trong ngân sách build | Không phát hành hàng loạt vùng trước khi đủ năng lực sản xuất |
| Intercontinental War | Kiểm thử tải, luật đóng góp, kinh tế chiến dịch, vận hành mùa | Chưa chốt thời điểm phát hành |

Slice được thử với authoritative server tối thiểu để đo độ trễ và tài nguyên; đây chưa phải MVP tài khoản, thương mại và vận hành đầy đủ.

## 14. Tiêu chí nghiệm thu thiết kế

| Giả thuyết | Phép đo đề xuất | Ngưỡng để tiếp tục |
| --- | --- | --- |
| Người mới hiểu điều khiển | 10 người chưa đọc hướng dẫn ngoài game | ≥8 người tự hoàn thành nhiệm vụ đầu trong 10 phút |
| Thua có thể học | Hỏi người chơi về đòn vừa trúng sau boss | ≥8/10 mô tả được tín hiệu và một cách phòng tránh |
| Solo khả thi | Bộ đồ nhiệm vụ, không trade, không bang | Toàn bộ tuyến slice hoàn thành; boss không đòi đồ ngoài tuyến |
| Co-op không chỉ là tăng HP | Thử nhóm 1, 2, 4 người | Mỗi quy mô có cơ hội xử lý cơ chế; không khóa một người chờ đồng đội bắt buộc |
| Touch đủ rõ | Hai cỡ màn hình và ít nhất 10 người chơi | Tỷ lệ kích nhầm <5%; không có nút bắt buộc bị che bởi ngón tay hoặc safe area |
| Hiệu ứng không che chiến đấu | Trận 4 người, chế độ giảm hiệu ứng | Tín hiệu boss luôn có cùng thời điểm và vùng nguy hiểm đọc được |
| Tái sinh hữu hạn | Simulation tải thấp/cao, bot giả lập, thiếu sink | Không vượt trữ lượng vùng và ngân sách phát hành; log giải thích được từng đợt cấp |

Các ngưỡng trên là cổng review, không thay thế phỏng vấn người chơi. Ghi riêng thiết bị, latency và kinh nghiệm của người thử để tránh kết luận từ trung bình che mất nhóm gặp lỗi.

## 15. Quyết định còn mở

- Hình thức tạo nhân vật, lựa chọn giới tính/ngoại hình và giọng nói; không gắn quyền chọn linh vực với địa chỉ thực hay dân tộc người chơi.
- Cách giữ bản sắc vùng khi học kỹ năng giao thoa; kiểm thử tổ hợp trước khi mở nhiều truyền thừa.
- Chi phí tích hợp render native CMP và lựa chọn sprite/skeletal; chỉ chốt sau spike.
- Bố cục touch tap/hold hay hai nút đòn cơ bản; quyết định từ prototype, không từ hình mockup riêng lẻ.
- Nhịp tiến triển MVP, độ bền, danh mục item có thể trade và giới hạn thương mại.
- Danh sách thiết bị mục tiêu, mức tải channel và quy mô vận hành; chưa cam kết thiết bị hoặc CCU cụ thể.

Đề xuất trả lời một phần các câu hỏi trên (giữ bản sắc khi học kỹ năng giao thoa, phạm vi bản ra mắt, tạo nhân vật) nằm ở [báo cáo thiết kế thế giới v0.2](worldbuilding-report.md), chờ [quyết định 0002](../decisions/0002-lineages-world-structure.md).
