# MyVa — Thần Mạch: combat theo truyền thừa, tiến triển và cân bằng

> Đề xuất v0.2, 2026-10-09. Mục **E** của [báo cáo thiết kế](worldbuilding-report.md).
> [combat.md](combat.md) vẫn là nguồn sự thật cho luật lõi (tick, timing, guard, cancel, DR khống chế, điều khiển). Tài liệu này bổ sung lớp truyền thừa, thuật khách, tiến triển và **phương pháp kiểm chứng cân bằng**.
> **Không truyền thừa nào được tuyên bố là đã cân bằng.** Mọi chỉ số dưới đây là giả thuyết (GT) để graybox và mô phỏng.

## 1. Nguyên tắc

1. **Kỹ năng quyết định trong cùng dải; chỉ số tạo lợi thế có giới hạn.** Giữ dải ±15% từ trang bị trong PvE cùng tier ([combat.md §13](combat.md#13-chỉ-số-pvp-và-nghiệm-thu)).
2. **Không thêm nút.** Mọi khác biệt truyền thừa đi qua 8 nút mobile: di chuyển, nhảy, lướt, đỡ, nhẹ, nặng, 3 thuật.
3. **Cơ chế chung trước, cơ chế riêng sau.** Người mới học di chuyển–đỡ–lướt–đánh giống nhau ở mọi truyền thừa; khác biệt nằm ở nội tại, bộ pháp và thuật.
4. **Khắc chế mềm.** Mỗi cặp đối đầu có chênh lệch nhỏ và có công cụ phản ứng; không có trận "biết thua trước khi đánh".
5. **Mỗi đòn mạnh có điểm yếu đọc được:** startup, hướng, khoảng cách, tài nguyên hoặc hồi chiêu.

## 2. Hành động chung và biến thể theo truyền thừa

| Hành động chung | Long Lưu | Sơn Cốt | Phong Vũ | Lâm Trảo | Tơ Vọng |
| --- | --- | --- | --- | --- | --- |
| Nhảy | Chuẩn | Thấp hơn 10%, không nhảy kép | Nhảy kép, giữ để lượn 0,8 s | Bám vách, bật vách | Chuẩn |
| Lướt | Xa hơn 30% trên mặt nước | Ngắn hơn, không mất đà khi trúng đạn nhẹ | Chuẩn; −20% sức bền ở nhánh Phong Bộ | Nhanh, ngắn; nối được Vồ | Chuẩn |
| Đỡ | Đỡ hoàn hảo phản đạn thường | Tiến chậm khi đỡ (+20% sức bền) | Sức bền đỡ thấp hơn | Sức bền đỡ thấp hơn; đỡ hoàn hảo mở Hoang Sinh 1 s | Đỡ hoàn hảo để lại một tiếng vọng đứng yên |
| Di chuyển đặc biệt | — | Lao xuống phá nền nứt | — | Leo vách | Kéo mình tới neo (Neo Mạch) |

Các biến thể này là **nội tại**, không cần thao tác mới. Timing gốc của nhảy, lướt, đỡ vẫn theo [combat.md §5](combat.md#5-hướng-đánh-đỡ-và-né) và §11.

## 3. Combo, phản đòn và kỹ thuật chiến đấu

### Chuỗi định sẵn có đường thoát

Mortal Kombat 3 (1995) nổi tiếng với nút chạy và các chuỗi combo định sẵn ("dial-a-combo") mà khi trúng nhịp đầu thì đối thủ không ngắt được ([nguồn](https://en.wikipedia.org/wiki/Mortal_Kombat_3)). MyVa mượn **sự dễ đọc** của chuỗi định sẵn nhưng **không** mượn tính không thể ngắt, vì [combat.md §6](combat.md#6-combo-và-cancel) yêu cầu chuỗi dài phải có đoạn thoát.

Mỗi truyền thừa có 2–3 chuỗi gợi ý (GT):

| Truyền thừa | Chuỗi khởi đầu | Chuỗi nâng cao | Đường thoát của đối thủ |
| --- | --- | --- | --- |
| Long Lưu | Nhẹ ×3 → Lưu Tiễn | Nhẹ ×2 → Dòng Xiết → Triều Dâng | Giữa Dòng Xiết và Triều Dâng có khoảng 180 ms đỡ được |
| Sơn Cốt | Nhẹ ×2 → Cốt Kích | Đỡ hoàn hảo → Sơn Băng (giáp) → Thạch Chấn | Thạch Chấn chỉ trúng mục tiêu trên nền: nhảy là thoát |
| Phong Vũ | Nhẹ ×3 → Gió Ngược | Phong Tiễn (trên không) → Vũ Bộ → Phong Tiễn | Vũ Bộ có recovery trên không, có thể bị phạt |
| Lâm Trảo | Nhẹ ×3 → Vồ Mồi | Lướt → Nanh Kép (đổi bên) → Nhẹ ×2 → Rễ Siết | Rễ Siết tính DR chung; nhịp đổi bên có tín hiệu bóng |
| Tơ Vọng | Neo Mạch → Lưới Tơ → Kéo Sợi | Thuật → Hồi Vọng (đổi chỗ) → thuật (tiếng vọng chồng) | Tiếng vọng không gây khống chế mạnh; xác định được vị trí trước 0,8 s |

### Thoát Mạch — cơ chế thoát combo (GT)

- **Thao tác:** nhấn **đỡ** trong lúc hitstun khi thanh Mạch ≥ 50. Không thêm nút.
- **Hiệu ứng:** đẩy lùi ngắn quanh nhân vật, không gây sát thương, cắt hitstun; 200 ms bất tử với đòn đánh, không với hazard.
- **Cái giá:** tiêu 50 Mạch (nửa thanh đại thuật). PvP: hồi 20 s. PvE: dùng được với địch thường, không dùng trong đòn nắm của boss.
- **Lý do:** biến Mạch thành lựa chọn công/thủ (đại thuật hay thoát), giống quyết định tài nguyên ở [combat.md §1](combat.md#1-mục-tiêu); bảo vệ người bị chuỗi nhiều người.
- **Rủi ro:** làm combo vô nghĩa nếu quá rẻ, hoặc vô dụng nếu quá đắt. Đo bằng mô phỏng (§11) và graybox trước khi chốt.

### Phản đòn

Đỡ hoàn hảo là cơ chế phản đòn chung ([combat.md §5](combat.md#5-hướng-đánh-đỡ-và-né)). Mỗi truyền thừa nhận một phần thưởng khác nhau (bảng §2). Thuật phản đòn riêng (Hồi Thế, Trấn Sơn) có tư thế và startup rõ; không truyền thừa nào có phản đòn tự động.

### Đổi bên và đánh lừa

Hai cơ chế đổi bên (Nanh Kép của Lâm Trảo, Hồi Vọng của Tơ Vọng) tạo tình huống đoán hướng đỡ kiểu game đối kháng. Giới hạn: một lần đổi bên trong mỗi chuỗi; có tín hiệu bóng 100 ms (GT); không dùng được khi mục tiêu sát vách hoặc mép bản đồ.

## 4. Khắc chế giữa các phong cách

Ma trận khắc chế mềm ở [lineages.md §5](lineages.md#5-ma-trận-quan-hệ). Cơ sở thiết kế:

| Phong cách | Thắng khi | Thua khi | Công cụ của bên bất lợi |
| --- | --- | --- | --- |
| Tầm xa (Phong Vũ) | Giữ được khoảng cách và tầng cao | Bị kéo vào hành lang, bị phản đạn | Vũ Bộ, Gió Ngược, đổi tầng |
| Giáp, tiền tuyến (Sơn Cốt) | Ép được đối thủ phải đỡ | Bị thả diều, bị bẫy làm chậm | Vách Đá chặn đạn, Cốt Kích phá sợi |
| Áp sát (Lâm Trảo) | Bám được và ép đoán | Gặp giáp hoặc phản đòn đúng lúc | Nanh Kép đổi bên, Gầm Rừng ngắt startup |
| Phản đòn (Long Lưu) | Đối thủ tấn công đoán trước được | Bị phá thủ, bị đe dọa trễ | Dòng Xiết chủ động, Xoáy Nước |
| Bày thế (Tơ Vọng) | Có thời gian đặt neo | Bị áp sát nhanh, neo bị bắn hạ | Hồi Vọng thoát, Kéo Sợi ngắt áp sát |

## 5. Thuật khách — học kỹ năng từ linh vực khác

### Các phương án

| Phương án | Bản sắc truyền thừa | Cân bằng | Chi phí animation | Đề xuất |
| --- | --- | --- | --- | --- |
| Không cho học chéo | Giữ tối đa | Dễ | Thấp | Loại: mâu thuẫn tầm nhìn "học truyền thừa khác" |
| Học tự do mọi thuật | Mất | Rất khó | Rất cao | Loại |
| Đổi hẳn truyền thừa | Giữ | Dễ | Thấp | Đã có (đổi tại hub); chưa đủ cho "giao thoa" |
| **Một truyền thừa phụ, tối đa 1 thuật khách** | Giữ khung, thêm lựa chọn | Kiểm soát được | Trung bình | **Đề xuất cho bản ra mắt** |
| Truyền thừa phụ mở cả nội tại nhánh | Phai | Khó | Cao | Xem lại sau khi có dữ liệu |

Phương án đề xuất khớp với "truyền thừa phụ" đã có ở [progression-social.md §3](progression-social.md#3-truyền-thừa-và-bộ-kỹ-nghệ): truyền thừa phụ là **nguồn** của thuật khách.

### Luật thuật khách (GT)

1. Học truyền thừa phụ qua nhiệm vụ của mentor ở linh vực khác (cấp ~22). Đổi truyền thừa phụ tại hub.
2. Tối đa **1** thuật khách trong 3 ô thuật. Không làm đại thuật.
3. Thuật khách **không nhận** nội tại truyền thừa và nội tại nhánh của người dùng, cũng không nhận nội tại của truyền thừa gốc của nó.
4. Đường cancel: chỉ đường chung "đòn nhẹ trúng → thuật" ([combat.md §6](combat.md#6-combo-và-cancel)); không nối vào chuỗi riêng của truyền thừa.
5. Mỗi thuật có **thẻ ngân sách** (sát thương, khống chế, cơ động, phòng thủ). Mô phỏng sinh **danh sách cặp cấm** khi một tổ hợp vượt ngân sách.
6. **Animation:** thuật khách dùng **bộ thi triển chung** (3 tư thế: phóng trước, đập đất, hướng lên) + VFX của thuật gốc, không làm lại animation cho từng cơ thể và vũ khí.
7. PvP xếp hạng: danh sách thuật khách được phép theo mùa. Chiến dịch bang hội: như PvE.

### Học từ boss (cảm hứng Mega Man)

Mega Man (1987) cho người chơi nhận vũ khí của boss sau khi thắng. MyVa dùng ý tưởng chung này theo cách của thế giới: một số boss sau khi được **trả nhịp** trao một **Ấn**; mentor dùng Ấn để dạy một biến thể thuật (sidegrade). Không rơi ngẫu nhiên, không mua bán, không thêm sức mạnh dọc.

### Theo dõi sau ra mắt

- Nếu > 40% người chơi một truyền thừa dùng cùng một thuật khách: review thuật đó và điểm yếu của truyền thừa.
- Nếu một truyền thừa gần như luôn lấy thuật khách để vá cùng một điểm yếu: xem lại điểm yếu thay vì tăng sức mạnh thuật khách.

## 6. Cơ chế theo loại nội dung

| Cơ chế | PvE | PvP (đấu trường, xếp hạng) | Chiến dịch bang hội |
| --- | --- | --- | --- |
| Chỉ số trang bị | Đầy đủ, dải ±15% cùng tier | Template chuẩn hóa | Nén chênh lệch còn 50% (GT) |
| Khống chế mạnh | Luật boss riêng; DR với địch thường | DR chung, stun ≤ 300 ms | DR chung |
| Vật phẩm hồi máu | Có, hồi chiêu nhóm | Không | Giới hạn theo mục tiêu |
| Nhận Mạch | Bình thường | Có trần theo thời gian và mục tiêu | Có trần |
| Thoát Mạch | Với địch thường | Có, hồi 20 s | Có, hồi 20 s |
| Thuật khách | 1 ô | 1 ô, danh sách theo mùa | 1 ô |
| Hoang Sinh (Lâm Trảo) | 50% | 40%, giảm dần | 40% |
| Nhắc Nhịp (Tơ Vọng) | Có | Cần quyết định (câu hỏi mở) | Có |
| Hồi sinh | Checkpoint | Theo trận | Điểm hồi sinh có thời gian chờ |
| Phần thưởng | Tiến triển, vật liệu trong ngân sách | Danh hiệu, mỹ phẩm | Điểm mặt trận, danh vọng, Biên Niên |

## 7. Tiến triển

### Cấp độ (GT)

| Dải | Mở khóa | Giai đoạn |
| --- | --- | --- |
| 1–3 | 2 đòn cơ bản, thuật lõi 1 | Slice |
| 4–6 | Thuật lõi 2, 3; thanh Mạch và đại thuật | Slice |
| 7–10 | Thoát Mạch (khi đạt graybox); chọn **Nhánh** ở cấp 10 | Slice |
| 11–20 | Thuật thay thế của nhánh; trang bị tier II–III; co-op | MVP |
| 21–25 | Truyền thừa phụ và thuật khách (~22); tier IV | Bản ra mắt |
| 26–30 | Nội tại nhánh nâng cao; thử thách Ngưỡng Mạch | Bản ra mắt |
| Sau 30 | **Thông Mạch**: biến thể thuật ngang, thử thách, mỹ phẩm; không cộng chỉ số | Bản ra mắt |

Trần 30 khớp `L_cap = 30` của [economy.md §5](economy.md#5-tổng-cấp-độ-hiệu-dụng-đang-hoạt-động). Không tăng trần mỗi mùa theo thói quen; tăng trần cần ADR.

### Trang bị

- **4 ô:** Vũ khí (quyết định đòn cơ bản), Giáp (sinh lực, sức bền), Bùa (biến đổi thuật), Ấn (nội tại phụ trong ngân sách). Ít ô để đọc được trên mobile ([progression-social.md §4](progression-social.md#4-trang-bị-chế-tạo-và-đổi-build)).
- **Tier I–IV** ở bản ra mắt; mỗi món có ngân sách thuộc tính và một đánh đổi.
- **Rèn +0 → +5:** tất định (không thất bại ngẫu nhiên, không vỡ đồ), tiêu vật liệu và Đồng; mỗi mức nhỏ hơn mức trước.
- **Luyện thuật hạng 1–3:** đạt bằng sử dụng hợp lệ trong chiến đấu + thử thách; mỗi hạng thêm **thuộc tính** (ví dụ thêm một nhịp đẩy), không thêm % sát thương thuần.

### Các trục tiến triển khác

| Trục | Phần thưởng | Không cho |
| --- | --- | --- |
| Khám phá (Hồi Ức, Mạch cơ) | Lore, sổ đối chiếu, mỹ phẩm, đường tắt | Chỉ số |
| Danh vọng thế lực | Nhiệm vụ, cửa hàng mỹ phẩm, lựa chọn truyện | Vật phẩm sức mạnh độc quyền |
| Truyền thụ (dẫn người mới) | Danh hiệu, mỹ phẩm | Vật liệu giao dịch được |
| Chiến dịch | Danh hiệu, Biên Niên, mỹ phẩm | Bonus vĩnh viễn |

### Vòng lặp chơi bổ sung

Giữ bốn nhịp của [GDD §5](gdd.md#5-vòng-lặp-gameplay). Bản ra mắt thêm:

- **Theo tuần:** Sổ Hành Trình (mục tiêu đa dạng, cộng dồn tối đa 2 tuần, không bắt đăng nhập liên tục).
- **Theo mùa:** chiến dịch 4 phần; mỗi phiên tham gia ≤ 45 phút có điểm kết thúc.

## 8. Cân bằng PvE

- Mỗi boss có cấu hình solo; co-op tăng cơ chế (đạn, vị trí áp lực), không chỉ tăng máu ([combat.md §9](combat.md#9-boss-kẻ-giữ-đập)).
- Mỗi tổ hợp truyền thừa + nhánh phải solo được mọi boss tuyến chính bằng trang bị nhiệm vụ.
- Điểm lệch của boss phản ứng với Mạch tính bằng **trạng thái** (choáng, lộ điểm yếu), không nhân sát thương; luôn có ≥ 2 cách mở bằng hành động chung.

## 9. Cân bằng PvP

- Template chuẩn hóa: cùng cấp, cùng ngân sách thuộc tính; khác biệt chỉ đến từ truyền thừa, nhánh, thuật.
- Ngân sách chuỗi: ≤ 800 ms không có cửa phản ứng, ≤ 25% sinh lực chuẩn ([combat.md §6](combat.md#6-combo-và-cancel)).
- Mỹ phẩm giữ silhouette, hitbox và telegraph.

## 10. Giả thuyết cân bằng ban đầu

Chỉ số tương đối, mốc 100 = mức trung bình. **Đây là điểm xuất phát cho graybox, không phải kết luận.**

| Chỉ số (PvP chuẩn hóa) | Long Lưu | Sơn Cốt | Phong Vũ | Lâm Trảo | Tơ Vọng |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sinh lực | 100 | 115 | 90 | 95 | 95 |
| Hiệu quả sức bền khi đỡ | 100 | 125 | 85 | 85 | 100 |
| Tốc độ mặt đất | 100 | 85 | 105 | 115 | 95 |
| Cơ động trên không | 100 | 75 | 130 | 105 | 105 |
| Tầm hiệu quả | Trung 2–6 m | Gần 0–3 m | Xa 5–12 m | Sát 0–2 m | Trung 3–8 m, gián tiếp |
| Sát thương bộc phát (cửa sổ 2 s) | 95 | 110 | 90 | 120 | 80 |
| Sát thương duy trì | 100 | 90 | 105 | 105 | 90 |
| Khống chế | 105 | 110 | 95 | 90 | 125 |
| Tự hồi phục | 100 | 100 | 85 | 125 | 90 |
| Hỗ trợ đồng đội | 100 | 115 | 95 | 90 | 125 |
| Độ khó điều khiển (1–5) | 3 | 2 | 3 | 3 | 4 |
| Độ khó phối hợp kỹ năng (1–5) | 3 | 2 | 3 | 4 | 5 |

**Ngân sách sức mạnh:** với trọng số mặc định bằng nhau, tổng điểm có trọng số của mỗi truyền thừa chênh ≤ 5%. Trọng số thật sẽ được ước lượng từ dữ liệu mô phỏng (§11), không đoán.

**Giả thuyết cần bác bỏ hoặc xác nhận:**

- H1: Không cặp đối đầu nào lệch quá 55/45 khi hai bên cùng trình độ.
- H2: Không truyền thừa nào kém nhất ở **mọi** loại nội dung (solo boss, co-op, đấu trường, chiến dịch).
- H3: Người chơi kỹ năng cao thắng người kỹ năng thấp có trang bị cao hơn một tier trong đa số trường hợp.
- H4: Mỗi nhánh có lý do để chọn trong ít nhất một loại nội dung.
- H5: Hoang Sinh, Vọng và Thế Núi không tạo vòng lặp bất tử hoặc khóa vô hạn.

## 11. Phương pháp kiểm chứng

### Kiến trúc

Simulation Rust thuần đã được tách khỏi renderer ([architecture.md §2](../technical/architecture.md#2-phân-chia-trách-nhiệm)). Dùng chính lõi đó để chạy **headless**:

```text
kịch bản (YAML/RON: bản đồ, truyền thừa, build, bot, seed, cấu hình)
  → bộ chạy headless (nhiều tiến trình, không render)
  → log sự kiện theo tick (đòn, trúng, đỡ, tài nguyên, chết)
  → bảng chỉ số (CSV/Parquet) + replay tái lập được
```

- Mỗi lần chạy ghi: build/commit, hash cấu hình, seed, phiên bản bot. Kết quả không có các trường này không được dùng làm bằng chứng.
- Chỉ so sánh kết quả trên **cùng một target** (native server build); không suy ra từ khác biệt float giữa WASM và native ([architecture.md §5](../technical/architecture.md#5-tick-prediction-và-chiến-đấu-online)).
- Bot gửi **cùng loại input** như người chơi qua hàng đợi command; không gọi thẳng hàm gây sát thương.

### Các bậc bot

| Bậc | Cách hoạt động | Dùng để | Thời điểm |
| --- | --- | --- | --- |
| B0 — Ngẫu nhiên hợp lệ | Chọn hành động hợp lệ ngẫu nhiên | Tìm crash, trạng thái kẹt, input lặp | Graybox |
| B1 — Kịch bản | Cây hành vi/utility; mô hình phản xạ 150–400 ms, tỷ lệ sai thao tác, độ chính xác đọc đòn | Đối chiếu trình độ người chơi; PvE boss | Graybox → slice |
| B2 — Tìm kiếm | MCTS/beam search trên mô hình tiến tới tất định | Tìm chuỗi mạnh nhất, vòng lặp khống chế, kẽ hở tài nguyên; đối đầu "trình độ cao" | Slice → MVP |
| B3 — Học tăng cường (tùy chọn) | Self-play; cầu nối Python nếu cần | Tìm chiến thuật thoái hóa mà người thiết kế không nghĩ ra | Trước bản ra mắt, nếu ngân sách cho phép |

Cách tiếp cận có tiền lệ học thuật: playtest tự động bằng "persona" dựa trên MCTS ([Holmgård và cộng sự, IEEE Transactions on Games, 2019](https://arxiv.org/abs/1802.06881)); đánh giá cân bằng luật chơi bằng self-play quy mô lớn ([Tomašev và cộng sự, 2020](https://arxiv.org/abs/2009.04374)).

### Bộ kiểm thử

| Bộ | Nội dung | Ngưỡng đề xuất (GT) |
| --- | --- | --- |
| T1 — Ma trận đối đầu | 5×5 truyền thừa × 2×2 nhánh; B2 vs B2 cùng trình độ; ≥ 2.000 trận mỗi cặp | Mỗi cặp 45–55% (khoảng tin cậy 95%); tỷ lệ thắng tổng mỗi truyền thừa 48–52% |
| T2 — Tách biệt kỹ năng | B2 vs B1 cùng trang bị; B2 vs B1 có trang bị +1 tier | ≥ 75% cùng trang bị; ≥ 55% khi đối thủ hơn một tier |
| T3 — Tác động trang bị | Cùng bot, chênh một tier | Bên mạnh hơn thắng 60–70%: trang bị quan trọng nhưng không tuyệt đối |
| T4 — Dãy boss PvE | Mọi truyền thừa + nhánh, solo và các tổ hợp co-op | Tỷ lệ qua boss của B1 trong ±10 điểm % trung vị; thời gian trong ±15% trung vị |
| T5 — Tìm combo nguy hiểm | B2 tối đa hóa sát thương không cho cửa phản ứng; tìm khóa khống chế, vòng lặp tài nguyên | Không chuỗi nào vượt ngân sách combat.md; không khóa vô hạn; không hồi phục vượt sát thương nhận |
| T6 — Tiến hóa meta | Quần thể ≥ 20 build, động lực replicator qua nhiều thế hệ đối đầu | Không build nào chiếm > 30% ở trạng thái cân bằng; không truyền thừa nào < 10% |
| T7 — Mạng | Replay dưới ma trận RTT/jitter/mất gói ([combat.md §10](combat.md#10-công-bằng-tín-hiệu-và-độ-trễ)) | Tỷ lệ ý định nối đòn hợp lệ bị từ chối được báo cáo theo truyền thừa |
| T8 — Cơ chế đặc thù | Hoang Sinh, Vọng, Neo, Thế Núi, Thoát Mạch trong tình huống nhiều người | Không nhân hit, không nhân hồi máu, tiếng vọng có owner đúng sau reconnect |

### Kết hợp với người chơi thật

- Bot không cảm thấy ức chế, không hiểu sai UI và không mệt. Mô phỏng chỉ **loại bỏ** thiết kế hỏng; quyết định "vui" và "công bằng" cần playtest.
- Mỗi vòng playtest ghi: truyền thừa chọn, tỷ lệ dùng đòn, nguyên nhân chết, lý do đổi truyền thừa, tỷ lệ kích nhầm trên cảm ứng.
- Khi bot và người chơi mâu thuẫn, ưu tiên dữ liệu người chơi và điều tra mô hình bot.
- Sau ra mắt: tỷ lệ chọn mỗi truyền thừa ngoài 12–30% là tín hiệu cần xem xét, không phải trigger tự động.

### Báo cáo

Mỗi báo cáo cân bằng gồm: câu hỏi kiểm chứng, cấu hình, số trận, khoảng tin cậy, các ngưỡng đạt/không đạt, thay đổi đề xuất, và **những gì mô phỏng không chứng minh được**.

## 12. Câu hỏi mở

- Thoát Mạch có cần trong PvE với boss không, hay chỉ PvP?
- Nén chênh lệch trang bị 50% trong chiến dịch có đủ chống snowball mà vẫn giữ động lực nâng đồ?
- B3 (học tăng cường) có đáng chi phí hạ tầng so với B2?
- Có công khai số liệu cân bằng cho cộng đồng không, và ở mức nào?
