# MyVa — Thần Mạch: thế lực, trực hệ và kit

> Cập nhật 2026-10-12 cho [#18](https://github.com/loveoverflowcom/myva/issues/18). **Đã chốt:** năm nền tảng và trục phân chia theo [ADR 0004](../decisions/0004-five-factions-lineage-hierarchy.md). **Chưa duyệt:** mọi tên trực hệ/nhánh/skill, roster, ánh xạ và thông số dưới đây. Chưa triển khai năm thế lực trong gameplay.
> **DK** = dữ kiện có nguồn; **ST** = sáng tác MyVa; **GT** = giả thuyết cần prototype/playtest. ID ứng viên chỉ dùng trong tài liệu, không phải ID runtime. Nguồn C/I/H/B/T ở [hồ sơ năm thế lực](../research/five-factions-sources.md).

## 1. Phân tầng khái niệm

**Thế lực → Nguồn bảo trợ → Trực hệ → Nhánh chiến đấu → Kit.** “Nguồn bảo trợ” viết gọn cho nguồn thần linh, bảo trợ hoặc truyền pháp cụ thể, tùy truyền thống.

| Tầng | Định nghĩa | Ví dụ đề xuất | Quyết định gameplay |
| --- | --- | --- | --- |
| Thế lực | Nhóm lấy nền tảng từ một trong năm tôn giáo đã chọn | Thế lực lấy nền tảng Kitô giáo, chưa đặt tên riêng | Có nhiều trực hệ/lối chơi; không bonus theo số tín đồ |
| Nguồn bảo trợ | Ai/nguồn nào truyền hoặc dẫn sức mạnh, với vai trò đúng theo hồ sơ | Thiên Chúa — ân sủng qua tuyến Michael; Michael là thiên thần | Nguồn bản sắc; không phải một kit hoặc avatar của thần |
| Trực hệ | Quan hệ kế thừa cụ thể: từ ai, qua ai, bằng cách nào | C-MIC “Hộ Ước”: người được chọn/ban phước qua tuyến bảo trợ hư cấu | Đặt bản sắc và giới hạn truyền thừa, không mặc định huyết thống |
| Nhánh chiến đấu | Cách chuyên sâu của cùng trực hệ | “Giữ Tuyến” hoặc “Phản Kích” | Chọn ưu tiên/đánh đổi; vai trò có thể thay đổi |
| Kit | Bộ hành động cụ thể để thử hoặc trang bị | Kit ứng viên Giữ Tuyến: 2 đòn + 3 thuật, một thuật tăng cường | Có nội tại, chi phí, telegraph, hit/block/whiff và counterplay |

**Các chiều độc lập:** linh vực/bản đồ (xuất thân, nhiệm vụ, địa hình, tải asset); tổ chức chính trị/danh vọng (Hắc Triều, Liên Bến...); vai trò phát sinh trong trận; Mạch tính là thẻ tương tác hư cấu của thuật. Không dùng chúng thay năm thế lực. Bản đồ có thể chứa nhiều thế lực, nguồn và trực hệ; mỗi trực hệ có thể được học ở nhiều bản đồ khi nội dung được phát hành.

Một quan hệ ban phước/truyền pháp **trong MyVa** là ST, kể cả khi tên nguồn là DK. Huyết thống chỉ được đề xuất khi hồ sơ cụ thể phù hợp, không dùng cho Allah/thiên thần/Bồ-tát một cách mặc định. Thần, hóa thân, thánh, ngôn sứ, Phật/Bồ-tát và tiên có vai trò riêng. Không phân loại người chơi bằng tôn giáo ngoài đời, dân tộc, quốc tịch, nơi sinh hoặc ngoại hình.

## 2. Năm thế lực đã chốt và phần còn mở

| Nền tảng đã chốt | Hướng nghiên cứu | Những phân biệt phải giữ | Độ đa dạng đề xuất trong §4 |
| --- | --- | --- | --- |
| Kitô giáo | Ân sủng, bảo trợ, gìn giữ và chữa lành | Một Thiên Chúa theo Ba Ngôi; thiên thần/thánh không phải thần ngang hàng | Giữ tuyến/phản công và du kích/hồi phục có điều kiện |
| Hồi giáo | Đức tin, dẫn dắt, ban phước | Allah không có con; Jibril/Mikail là thiên thần; Muhammad là ngôn sứ | Tầm trung/dẫn hướng và cận chiến/giữ nhịp |
| Ấn Độ giáo | Nguồn thần cụ thể, ban phước và truyền thừa | Phân biệt vị thần, hóa thân và người kế thừa; ghi hệ phái nguồn | Khống chế đường đi và áp sát cam kết |
| Phật giáo | Pháp hệ Phật/Bồ-tát, thầy–trò và tu học | Phật không phải thần sáng thế; pháp hệ không là phả hệ huyết thống | Điều tiết khoảng cách/hỗ trợ và đấu sĩ đọc nhịp |
| Đạo giáo | Thần, tiên, pháp phái và thầy–trò cụ thể | Phân biệt Đạo giáo, tín ngưỡng dân gian giao thoa và hư cấu tu tiên | Giữ thế/phản kích và kiếm thuật cơ động hữu hạn |

Tên nguồn thật được nghiên cứu trực tiếp theo #18. Chưa chọn tên hiển thị, hình tượng, nghi thức hoặc mức xuất hiện. Không đổi toàn bộ thành Cổ Linh vô danh; Cổ Linh vẫn có thể là sinh vật ST riêng. Không suy mọi nguồn trong năm truyền thống là các mảnh của một vị thần chung trong lore.

## 3. Quy tắc và phương án đặt tên

- Trong docs, dùng đúng tên nguồn + ID ứng viên để truy vết; trong game có thể dùng tên hư cấu của trực hệ nhưng vẫn ghi nguồn rõ ở hồ sơ. Tên tạm dưới đây **chưa duyệt**, không phải thuật ngữ tôn giáo có thật.
- Tên phe, trực hệ, nhánh và kit không dùng lẫn nhau. Không đổi tên nguồn thành tên skill rồi trình bày như phép thuật có thật.
- Tên quốc tế, quyền dùng hình/âm thanh và tra nhãn hiệu cần hồ sơ riêng trước công bố ([R-02](../production/content-roadmap.md#6-đầu-việc-đề-xuất)). Chưa tra nhãn hiệu cho tên mới.
- Đổi presentation sau này không tự đổi ID kit, protocol, replay hoặc asset hiện tại; cần kế hoạch migration nếu thực sự đổi dữ liệu runtime.

## 4. Danh mục trực hệ ứng viên

**Toàn bộ §4 là đề xuất ST/GT chưa duyệt.** Mỗi hồ sơ gồm phần giữ từ nguồn (DK), quan hệ hư cấu, hai nhánh khác ưu tiên và một cơ chế đặc trưng. Tên kỹ năng là ví dụ để #20 phát triển thành kit; chưa có timing/damage, chưa là true combo hoặc lời hứa cân bằng. Nhánh thứ hai không tự được cộng vào kit của nhánh thứ nhất.

| Thế lực theo nền tảng | Nguồn tham chiếu | Trực hệ ứng viên (tên ST) | Hai nhánh chiến đấu đề xuất |
| --- | --- | --- | --- |
| Kitô giáo | Thiên Chúa — tuyến Michael | C-MIC / Hộ Ước | Giữ Tuyến / Phản Kích |
| Kitô giáo | Thiên Chúa — tuyến Raphael | C-RAP / Đồng Hành | Dẫn Lộ / Hồi Sức |
| Hồi giáo | Allah — tuyến Jibril | I-JIB / Dẫn Quang | Định Hướng / Chuyển Tuyến |
| Hồi giáo | Allah — tuyến Mikail | I-MIK / Kiên Tín | Giữ Nhịp / Tiếp Bước |
| Ấn Độ giáo | Vishnu | H-VIS / Trì Thế | Hộ Trì / Luân Hồi Kích |
| Ấn Độ giáo | Shiva/Shankara | H-SHI / Chuyển Vũ | Phá Thế / Thu Vũ |
| Phật giáo | Avalokiteśvara/Quán Âm | B-AVA / Từ Hành | Giải Áp / Hộ Hành |
| Phật giáo | Mañjuśrī/Văn Thù | B-MAN / Minh Kiến | Kiến Nhịp / Khai Lộ |
| Đạo giáo | Chân Vũ/Zhenwu | T-ZHE / Trấn Vũ | Trấn Tuyến / Phản Chấn |
| Đạo giáo | Lữ Động Tân/Lü Dongbin | T-LU / Vân Kiếm | Du Kiếm / Định Kiếm |

### 4.1 Kitô giáo — C-MIC / Hộ Ước

- **Nguồn/giữ lại (DK):** Thiên Chúa — tuyến bảo trợ Michael, thiên thần có vai trò chiến đấu trong Khải Huyền 12 (C1, C2). Không coi Michael là thần hoặc người chơi là Michael.
- **Quan hệ (ST):** người được chọn nhận ân sủng qua tuyến bảo trợ Michael, luyện cùng mentor con người. Không quan hệ huyết thống; việc trao sức mạnh này là hư cấu MyVa.
- **Nhánh (ST):** **Giữ Tuyến** ưu tiên bảo vệ một hướng; **Phản Kích** ưu tiên trừng phạt đòn cam kết, giảm khả năng giữ vùng.
- **Đặc trưng (GT): Hộ Ước** — phòng thủ chính xác tích tối đa một nhịp, tiêu bằng đòn thường kế tiếp trúng để đẩy đối thủ. Không tự proc khi giữ guard, không phản mọi hướng.
- **Thuật ứng viên (ST/GT):** **Khiên Ước** che phía trước trong cửa ngắn; **Bước Hộ** dịch ngắn để giữ tuyến; **Phá Vây** quét gần có startup rõ. Đại thuật là Phá Vây tăng cường trên cùng ô, vẫn cần điều kiện Mạch/cost chung.
- **Mạnh:** giữ trung cự, phạt đòn đoán trước được. **Yếu:** bị nhử guard, đánh sau lưng hoặc kéo rời vị trí. **Khắc chế mềm:** gây khó cho lao thẳng; đối thủ có thể feint/lùi rồi phạt recovery. Người dùng đổi vị trí bằng Bước Hộ hữu hạn, không có miễn CC/damage mặc định.

### 4.2 Kitô giáo — C-RAP / Đồng Hành

- **Nguồn/giữ lại (DK):** Thiên Chúa — tuyến Raphael, thiên thần được sai đến chữa lành trong Tobit (C1, C3); phạm vi Công giáo của nguồn được ghi riêng.
- **Quan hệ (ST):** người nhận ban phước, học qua người hướng dẫn; không con cháu Raphael và không tuyên bố nghi thức game là bí tích thật.
- **Nhánh (ST):** **Dẫn Lộ** ưu tiên dịch chuyển/tầm trung; **Hồi Sức** ưu tiên trao đổi dài, giảm burst. Cả hai có đường solo, không healer bắt buộc.
- **Đặc trưng (GT): Dưỡng Nhịp** — một lần đánh trúng trong cửa phản công hợp lệ tạo một lần hồi nhỏ có trần/cooldown; không hồi từ whiff/guard/hazard hoặc proc tự kích proc.
- **Thuật ứng viên:** **Dẫn Lộ** bước ngắn; **Hộ Hành** tạo khoảng an toàn ngắn, không bất tử; **Hoàn Nhịp** đòn tầm trung tiêu nhịp để tự hồi nhỏ, khi co-op có thể chọn trợ đồng đội thay phần tự hồi. Đại thuật tăng cường Hoàn Nhịp cùng ô.
- **Mạnh:** trao đổi bền khi phản công đúng. **Yếu:** hồi cần trúng, burst thấp, bị ép khi mất nhịp. **Khắc chế mềm:** hiệu quả với cấu rỉa đơn nhịp; đối thủ trì hoãn để hết cửa hồi hoặc ép một đợt có telegraph. Người dùng có thể rút bằng Dẫn Lộ, trả giá mất áp lực.

### 4.3 Hồi giáo — I-JIB / Dẫn Quang

- **Nguồn/giữ lại (DK):** Allah — tuyến Jibril, thiên thần truyền đạt mặc khải theo phép Allah (I1–I3). Không ngôn sứ/thần độc lập.
- **Quan hệ (ST):** người được ban phước qua tuyến dẫn dắt hư cấu gắn Jibril, luyện kỹ thuật qua mentor; không tự nhận mặc khải thật và không hậu duệ Allah.
- **Nhánh (ST):** **Định Hướng** kiểm soát đường đạn trung cự; **Chuyển Tuyến** đổi góc bắn, giảm độ bền vùng.
- **Đặc trưng (GT): Tuyến Dẫn** — đặt tối đa một mốc nhìn thấy được; thuật kế tiếp cam kết hướng qua mốc rồi tiêu nó. Mốc không bám mục tiêu hoặc tự đọc input.
- **Thuật ứng viên:** **Quang Tuyến** đạn thẳng; **Đổi Góc** bước ngắn không miễn đòn; **Dẫn Kích** phát đạn trễ qua mốc có báo trước. Đại thuật tăng cường Dẫn Kích cùng ô. Quang/mốc/đạn đều là ST, không thuộc tính tôn giáo được I2 xác nhận.
- **Mạnh:** ép đường tiếp cận, tạo góc trên nền ngang. **Yếu:** cần bố trí, yếu khi bị vượt mốc/áp sát. **Khắc chế mềm:** gây khó cho giữ thế đứng yên; đối thủ nhảy/đổi tầng hoặc đỡ đạn, ép hồi chiêu Đổi Góc. Người dùng vẫn có đòn nhẹ gần và đường rút ngắn.

### 4.4 Hồi giáo — I-MIK / Kiên Tín

- **Nguồn/giữ lại (DK):** Allah — tuyến Mikail, thiên thần được nêu trong Qur'an 2:98 (I1, I2). Chưa gán quyền quản mưa/nước/sét khi nguồn này không xác nhận chức trách đó.
- **Quan hệ (ST):** người nhận ban phước theo tuyến Mikail hư cấu, học thầy–trò; không huyết thống Allah/thiên thần.
- **Nhánh (ST):** **Giữ Nhịp** cận chiến bền; **Tiếp Bước** tiến ngắn có cam kết, giảm phòng thủ tại chỗ.
- **Đặc trưng (GT): Kiên Nhịp** — dừng chuỗi đúng cửa hồi động tác tạo một nhịp giảm chi phí đòn kế tiếp có sàn; spam không tạo thêm nhịp, không hồi tài nguyên dương vô hạn.
- **Thuật ứng viên:** **Trụ Bước** giữ một hướng ngắn; **Tiếp Kích** tiến đánh không homing; **Chấn Nhịp** đòn gần phá áp lực guard với startup rõ. Đại thuật tăng cường Chấn Nhịp cùng ô. Đây là cơ chế ST, không khẳng định Mikail dạy võ thuật thật.
- **Mạnh:** duy trì áp lực bằng quản lý tài nguyên. **Yếu:** tầm ngắn, có khoảng nghỉ dễ đoán. **Khắc chế mềm:** gây khó cho đối thủ tiêu sức bền quá sớm; đối thủ lùi giữ khoảng cách hoặc đổi nhịp để phạt lúc nghỉ. Người dùng có Tiếp Kích để tiếp cận hữu hạn, không miễn đạn.

### 4.5 Ấn Độ giáo — H-VIS / Trì Thế

- **Nguồn/giữ lại (DK):** Vishnu được nêu và phân biệt với hóa thân Vamana trong H1. Chọn mô-típ gìn giữ thế trận là diễn giải ST, không khẳng định mọi hệ phái đồng nhất.
- **Quan hệ (ST):** người được ban phước theo tuyến Vishnu, học qua mentor; chưa chọn hóa thân hoặc huyết thống. Người chơi không là Vishnu/Vamana.
- **Nhánh (ST):** **Hộ Trì** kiểm soát vùng nhỏ; **Luân Hồi Kích** đòn đi–về trung cự, giảm khả năng giữ nền. Tên nhánh là sáng tác gameplay.
- **Đặc trưng (GT): Đường Về** — một đòn có đường đi và đường về cố định, mỗi mục tiêu chỉ nhận ngân sách hit đã định; không nhân damage bằng đứng chồng quỹ đạo.
- **Thuật ứng viên:** **Luân Kích** phóng vật lực hư cấu; **Trì Vực** đánh dấu vùng nhỏ; **Thu Thế** thu hồi vật lực với recovery. Đại thuật tăng cường Luân Kích cùng ô, không triệu chính vị thần.
- **Mạnh:** ép đường ngang, bảo vệ một vị trí. **Yếu:** khó đổi hướng khi đã phóng, dễ bị đánh từ tầng khác. **Khắc chế mềm:** gây khó cho lao theo cùng một tuyến; đối thủ đổi tầng, chờ đòn đi qua rồi vào recovery. Người dùng có thể giữ đòn thường để tự vệ thay vì phóng tiếp.

### 4.6 Ấn Độ giáo — H-SHI / Chuyển Vũ

- **Nguồn/giữ lại (DK):** Shiva/Shankara trong H2; không coi là phe ác. Vòng động tác chiến đấu là ST, không tái hiện nghi lễ/vũ điệu thiêng.
- **Quan hệ (ST):** người nhận ban phước tuyến Shiva, rèn qua thầy–trò; không tự xưng hóa thân hoặc con cháu thần.
- **Nhánh (ST):** **Phá Thế** cam kết đòn nặng; **Thu Vũ** cơ động gần, giảm áp lực guard.
- **Đặc trưng (GT): Nhịp Chuyển** — sau khi hoàn tất recovery hợp lệ, một đòn thường trúng mở lựa chọn tiếp nhịp hoặc thu vị trí; không cancel cứu đòn hụt miễn phí.
- **Thuật ứng viên:** **Khai Vũ** quét gần; **Chuyển Bộ** bước cam kết; **Đoạn Thế** đòn chậm ép guard. Đại thuật tăng cường Đoạn Thế cùng ô, vẫn đọc được startup.
- **Mạnh:** punish recovery, ép quyết định ở cự ly gần. **Yếu:** đòn lớn hụt bị phạt, truy đuổi xa kém. **Khắc chế mềm:** gây khó cho tấn công quá cam kết; đối thủ giữ tầm hoặc nhử đòn nặng. Người dùng chọn Thu Vũ để đổi vị trí với chi phí mất burst.

### 4.7 Phật giáo — B-AVA / Từ Hành

- **Nguồn/giữ lại (DK):** Avalokiteśvara/Quán Âm, Bồ-tát gắn với từ bi trong B2; B1 đối chiếu việc không ép Phật giáo thành phả hệ thần sáng thế.
- **Quan hệ (ST):** pháp hệ tu học hư cấu lấy cảm hứng Quán Âm, qua mentor và thực hành; không huyết thống, không chứng nhận truyền pháp thật.
- **Nhánh (ST):** **Giải Áp** kiểm soát khoảng cách; **Hộ Hành** hỗ trợ có đường tự vệ solo, giảm sát thương bộc phát.
- **Đặc trưng (GT): Khoảng Lặng** — đỡ/né đúng đòn có cam kết tạo một nhịp dùng cho đẩy lùi hoặc khiên nhỏ, chỉ chọn một. Không tích lũy vô hạn hoặc xóa mọi CC.
- **Thuật ứng viên:** **Mở Khoảng** đẩy gần; **Dẫn Bước** di chuyển ngắn; **An Vực** vùng nhỏ có báo trước để tiêu nhịp phòng thủ. Đại thuật tăng cường Mở Khoảng cùng ô. Không tụng kinh/dhāraṇī làm chiêu đánh.
- **Mạnh:** cắt nhịp lao liên tục, duy trì khoảng trống. **Yếu:** burst thấp, bị nhử công cụ đẩy. **Khắc chế mềm:** gây khó cho áp sát vô điều kiện; đối thủ chờ vùng hết hoặc đánh từ ngoài tầm. Người dùng có Dẫn Bước và đòn thường để tiếp tục solo, không cần đồng đội nhận buff.

### 4.8 Phật giáo — B-MAN / Minh Kiến

- **Nguồn/giữ lại (DK):** Mañjuśrī/Văn Thù, Bồ-tát gắn với trí tuệ trong B3; không thần sáng thế.
- **Quan hệ (ST):** pháp hệ thầy–trò/thực hành lấy cảm hứng Văn Thù, không nhận gen hoặc điều khiển Bồ-tát. “Đọc nhịp” là kỹ năng người chơi, không biết input bí mật.
- **Nhánh (ST):** **Kiến Nhịp** phản công trung cự; **Khai Lộ** chủ động áp sát ngắn, giảm cửa phòng thủ.
- **Đặc trưng (GT): Minh Nhịp** — chỉ đánh trúng recovery hợp lệ mới có một nhịp để biến đổi đòn thường kế tiếp; hụt/bị đỡ không tạo nhịp.
- **Thuật ứng viên:** **Thăm Thế** đòn thăm tầm; **Khai Bộ** tiến ngắn; **Đoạn Nhịp** đòn punish có startup. Đại thuật tăng cường Đoạn Nhịp cùng ô, không auto counter.
- **Mạnh:** thưởng đọc đúng tình huống. **Yếu:** khó tạo lợi thế nếu đoán sai, không giữ vùng lâu. **Khắc chế mềm:** gây khó cho chuỗi đều nhịp; đối thủ dừng sớm/đổi nhịp hoặc giữ tầm. Người dùng có Thăm Thế để chủ động thăm dò thay vì chờ phản công mãi.

### 4.9 Đạo giáo — T-ZHE / Trấn Vũ

- **Nguồn/giữ lại (DK):** Chân Vũ/Zhenwu, thần bảo hộ trong hồ sơ T1; dùng phạm vi Đạo giáo, ghi nhận giao thoa với truyền thống khác.
- **Quan hệ (ST):** người nhận bảo trợ tuyến Chân Vũ qua đạo trường hư cấu; không tự công bố pháp phái lịch sử hoặc hậu duệ thần.
- **Nhánh (ST):** **Trấn Tuyến** giữ nền; **Phản Chấn** phản kích gần, giảm thời gian giữ thế.
- **Đặc trưng (GT): Trấn Điểm** — chọn một điểm đứng có tín hiệu, tăng ổn định phía trước khi ở điểm; bị buộc rời điểm thì mất lợi thế. Không miễn damage/hazard.
- **Thuật ứng viên:** **Lập Trấn** đặt điểm; **Hộ Thế** chắn phía trước ngắn; **Phản Chấn** trả đòn có cam kết. Đại thuật tăng cường Phản Chấn cùng ô.
- **Mạnh:** giữ tuyến, phạt lao thẳng. **Yếu:** cơ động thấp, điểm bị tránh/ép bỏ. **Khắc chế mềm:** gây khó cho cận chiến nóng vội; đối thủ đổi tầng/đánh hướng khác hoặc ép di chuyển. Người dùng có thể bỏ điểm, mất buff để dùng đòn thường và tái bố trí.

### 4.10 Đạo giáo — T-LU / Vân Kiếm

- **Nguồn/giữ lại (DK):** Lữ Động Tân/Lü Dongbin, một trong Bát Tiên; T2 ghi quan hệ nhận tri thức từ Chung Ly Quyền. Không đồng nhất với mọi truyện tu tiên.
- **Quan hệ (ST):** truyền thừa tiên qua chuỗi thầy–trò hư cấu đến người chơi; không tuyên bố chuỗi này là lịch sử pháp phái thật.
- **Nhánh (ST):** **Du Kiếm** đổi vị trí gần; **Định Kiếm** kiếm đòn trung cự, giảm khả năng nối bước.
- **Đặc trưng (GT): Hồi Kiếm** — sau một bước cam kết, chọn một đường kiếm trở về có tín hiệu, dùng cùng ô; chỉ một lần, không teleport hoặc reset dash vô hạn.
- **Thuật ứng viên:** **Xuất Kiếm** đòn trung cự; **Vân Bộ** bước hữu hạn; **Hồi Kiếm** đường trở về cam kết, hụt chịu recovery. Đại thuật tăng cường Xuất Kiếm cùng ô.
- **Mạnh:** đổi khoảng cách và góc giao tranh. **Yếu:** yếu khi bị ép vào góc, hết bước không thoát miễn phí. **Khắc chế mềm:** gây khó cho giữ một hướng cố định; đối thủ giữ đòn chống không trung/đường về và phạt điểm đáp. Người dùng chọn Định Kiếm để giữ tầm thay cho tiếp tục lao.

## 5. Khắc chế theo cơ chế

**Không có vòng tôn giáo A khắc B.** Bảng dưới là câu hỏi matchup GT giữa các nhánh/kit, không phải bảng ưu thế của thế lực. Hai trực hệ trong cùng thế lực cũng có thể là đối trọng; mirror cần kiểm tra tương tự.

| Cơ chế gây áp lực | Mục tiêu dễ bất lợi | Phản ứng của bên bất lợi | Tình huống đảo chiều cần thử |
| --- | --- | --- | --- |
| Giữ tuyến/phản công (C-MIC, T-ZHE) | Lao thẳng (H-SHI, T-LU) | Feint, đổi tầng/hướng, chờ công cụ phòng thủ hồi | Người giữ tuyến bị kéo khỏi điểm hoặc đánh sau lưng |
| Đường đạn/vùng (I-JIB, H-VIS) | Giữ nền ít cơ động | Đỡ/né, đổi tầng, rút khỏi vùng rồi vào recovery | Người bắn bị áp sát lúc đã cam kết quỹ đạo |
| Áp sát đổi nhịp (I-MIK, H-SHI, B-MAN) | Đối thủ cần chuẩn bị mốc/vùng | Dùng đòn thường chống áp sát, giữ tài nguyên rút | Áp sát hụt hoặc hết sức bền trước mục tiêu |
| Hồi có điều kiện/giải áp (C-RAP, B-AVA) | Cấu rỉa hoặc lao đều nhịp | Ngắt nhịp, không trao hit hồi miễn phí | Kit hồi bị ép khi công cụ đẩy/di chuyển đang hồi |
| Cơ động hữu hạn (T-LU, nhánh Chuyển Tuyến) | Giữ một hướng, đòn lớn | Giữ công cụ chống không trung/điểm đáp | Hết bước trong góc hoặc bị dụ đi qua vùng đã đặt |

Mục tiêu là bất lợi vừa phải và có đáp trả, không định trước win rate hoặc damage multiplier theo tôn giáo. Ma trận 5 kit cũ là giả thuyết lịch sử của ADR 0002; không dùng làm ma trận năm thế lực. #20 sẽ chọn số ít matchup để đo với side-swap, map, trình độ, touch/keyboard và latency; chưa có kết quả playtest cho các ứng viên §4.

## 6. Giữ kit hiện tại và mở rộng theo gate

| Kit/mẫu cơ chế cũ | Giữ cho việc gì | Trạng thái thực tế | Ánh xạ nguồn/trực hệ |
| --- | --- | --- | --- |
| Long Lưu | Phản công/giữ trung cự; Lưu Tiễn, Hồi Thế, Triều Dâng | Baseline core/graybox #2, #19; bằng chứng tại [combat](combat.md#14-lát-cắt-19-và-trạng-thái-triển-khai) | **Chưa chọn**; giữ ID, replay và asset `long_luu` |
| Sơn Cốt | Giữ vị trí/áp lực guard | Kit thiết kế mở rộng theo gate slice #4 | **Chưa chọn** |
| Phong Vũ | Đổi tầng/tầm xa | Kit thiết kế mở rộng theo gate slice #4 | **Chưa chọn** |
| Lâm Trảo | Áp sát có điều kiện | Mẫu thiết kế; cải biến Dấu Săn thuộc #20, chưa implement | **Chưa chọn**; không mang hồi máu cũ vào #20 mặc định |
| Tơ Vọng | Neo/bày thế | Mẫu thiết kế và backlog; chưa vào prototype đầu | **Chưa chọn** |

Không ghép một–một năm tên cũ với năm tôn giáo. Các số liệu, nội tại/nhánh cũ trong [combat-progression-balance](combat-progression-balance.md) là GT chưa duyệt, không được #18 nâng thành luật core. Muốn dùng một cơ chế cũ cho trực hệ mới cần bảng mapping riêng, review DK/ST rồi prototype theo gate; không sửa code/asset để đủ bảng lore.

Thêm bản đồ là thêm không gian chơi, không bắt buộc thêm trực hệ/kit. Mentor của cùng trực hệ có thể hiện diện ở nhiều vùng; học chéo theo hồ sơ nguồn và thử thách, không khóa theo nơi sinh. Luật một thuật khách của D7 vẫn chưa duyệt; tính tương thích giữa các trực hệ/thế lực cần quyết định riêng. Ngân sách tài nguyên và cơ chế tải asset theo vùng giữ nguyên.

## 7. Hợp đồng bàn giao cho issue #20

#18 bàn giao danh mục ứng viên, quan hệ nguồn và giới hạn lore. [#20](https://github.com/loveoverflowcom/myva/issues/20) chọn kit cụ thể, item và ruleset; [#19](https://github.com/loveoverflowcom/myva/issues/19) / [combat.md](combat.md) giữ luật chung. Không chép timing mới vào hồ sơ lore.

Mỗi kit được chọn cần một hồ sơ sau (đây là **mẫu thiết kế chưa triển khai**, không đổi schema runtime):

| Trường | Nội dung tối thiểu |
| --- | --- |
| Danh tính | Nền tảng thế lực, nguồn + ID hồ sơ, trực hệ ứng viên, nhánh, kit; trạng thái duyệt riêng từng lớp |
| Truyền thừa | Ai trao/ai dẫn, người chơi nhận bằng cách nào, DK giữ lại, ST thêm mới, tên/hình tượng còn mở |
| Vòng quyết định | Một cơ chế đặc trưng hữu hạn, nội tại → đòn thường → 3 thuật; điều kiện tạo/tiêu tài nguyên |
| Hành động | 2 đòn cơ bản + 3 thuật; đại thuật biến thể cùng ô; không thêm nút; vũ khí/build khóa trong giao tranh theo luật chung |
| Phản ứng | Tầm/hướng/tầng cao, telegraph, hit/block/whiff/recovery, cách chống trên không, cửa đối thủ đáp trả |
| Solo và nhóm | Thắng bằng gì, thiếu gì, đường solo boss bằng đồ nhiệm vụ; lợi ích nhóm không biến thành hỗ trợ bắt buộc |
| Trang bị | Loại item phù hợp và chi phí cơ hội; không xóa điểm yếu, không vòng proc/refund; catalog/số ô do #20 thiết kế |
| Kiểm chứng | Kit/seed/build/config, matchup và side-swap, map, trình độ, touch/keyboard; tách simulation, local người thật và online |
| Migration | Nếu gắn lore vào kit cũ: mapping tên hiển thị, giữ ID/protocol/replay/asset; thay dữ liệu thật cần kế hoạch riêng |

Trình tự: tiếp tục Long Lưu baseline → Sơn Cốt/Phong Vũ theo gate #4 → chọn một ứng viên nguồn nếu được duyệt, thử offline nhỏ. Không đòi làm 10 trực hệ hoặc năm roster trước khi một kit có cảm giác tốt. PvP thật chờ #5 hai client authoritative/reconnect; Đấu trường và Chiến tuyến thuộc thiết kế #20, không mở vào solo slice bởi PR tài liệu này.
