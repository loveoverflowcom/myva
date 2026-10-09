# Planning và Kanban — MyVa · Thần Mạch

Khởi tạo ngày **2026-10-09**. Nhánh đầu tiên và mặc định: **`develop`**; các PR công việc lấy `develop` làm base.

## Không gian làm việc

- [Repository và tài liệu](https://github.com/loveoverflowcom/myva/tree/develop).
- [Thần Mạch — Planning](https://github.com/users/loveoverflowcom/projects/3): xem tổng thể, thứ tự và các đầu việc chưa hoàn thành.
- [Thần Mạch — Kanban](https://github.com/users/loveoverflowcom/projects/4/views/1): theo dõi trạng thái thực thi.
- [Danh sách issue](https://github.com/loveoverflowcom/myva/issues): checklist nghiệm thu, kết quả thử và thảo luận từng đầu việc.

Hai project dùng chung các issue của repository. Project hiện có quyền truy cập riêng tư; mở bằng tài khoản được cấp quyền. Các file Markdown là nguồn phạm vi và luật thiết kế; **Status trên Kanban là nguồn trạng thái thực thi**. Không sao chép trạng thái hiện tại vào bảng Markdown để tránh lệch khi kéo thẻ. `Todo` trong Planning chỉ biểu thị chưa hoàn thành, không khẳng định công việc đã vượt gate.

## Đầu việc và phụ thuộc

| Issue | Kế hoạch nguồn | Điều kiện bắt đầu |
| --- | --- | --- |
| [#8 — Bevy adoption](https://github.com/loveoverflowcom/myva/issues/8) | [ADR 0003](../decisions/0003-bevy-engine-adoption.md) | Parent D01–D05; engine đã chốt, từng gate còn độc lập |
| [#9 — D01 ADR](https://github.com/loveoverflowcom/myva/issues/9) | [ADR / architecture](../technical/architecture.md) | Đồng bộ ownership và pin trước integration |
| [#10 — D02 native](https://github.com/loveoverflowcom/myva/issues/10) | [010 — Bevy native trong CMP](010-native-feasibility.md) | D01; thiết bị/toolchain và báo cáo riêng Android/iOS |
| [#11 — D03 web](https://github.com/loveoverflowcom/myva/issues/11) | [Web architecture](../technical/architecture.md#3-web-tách-shell-và-game-runtime) | D01; song song D02 |
| [#12 — D04 ECS/headless](https://github.com/loveoverflowcom/myva/issues/12) | [Core boundary](../technical/architecture.md#2-phân-chia-trách-nhiệm) | Boundary D01; không ép renderer vào core/server |
| [#13 — D05 2D/performance](https://github.com/loveoverflowcom/myva/issues/13) | [Assets/performance](../technical/assets-performance.md) | Platform D02/D03 đã qua gate tương ứng |
| [#7 — Bevy tooling/skills](https://github.com/loveoverflowcom/myva/issues/7) | [ADR 0003](../decisions/0003-bevy-engine-adoption.md) | Song song D01; instructions engine-specific dùng Bevy |
| [#1 — Native feasibility cũ](https://github.com/loveoverflowcom/myva/issues/1) | Lịch sử Macroquad; **superseded bởi #10** | Không dùng làm hợp đồng mới hoặc gate Bevy |
| [#2 — Combat graybox](https://github.com/loveoverflowcom/myva/issues/2) | [020 — Combat và boss Bevy](020-combat-graybox.md) | Foundation #12; web qua #11, mobile qua #10 trên hệ tương ứng |
| [#3 — Economy simulator](https://github.com/loveoverflowcom/myva/issues/3) | [030 — Kinh tế và tài nguyên](030-economy-simulator.md) | Độc lập engine, song song integration/combat |
| [#4 — Vertical slice](https://github.com/loveoverflowcom/myva/issues/4) | [040 — Bến Lau](040-vertical-slice.md) | #2/#3; gate #10/#11/#13 theo platform công bố |
| [#5 — Online pilot](https://github.com/loveoverflowcom/myva/issues/5) | [MVP Online](backlog/online-pilot.md) | Slice #4 và integrity prerequisites; không kéo sớm vì đổi engine |
| [#6 — World expansion](https://github.com/loveoverflowcom/myva/issues/6) | [Linh vực và chiến dịch](backlog/world-expansion.md) | Pilot #5 có bằng chứng load/integrity/solo; văn hóa có nguồn và review |

Số `010`, `020`, ... là thứ tự khuyến nghị, không phải ID cố định. Issue GitHub giữ ID theo dõi lâu dài. Chưa đặt deadline, estimate hoặc assignee khi chưa có dữ liệu và người nhận việc.

## Cách vận hành Kanban

| Cột | Điều kiện |
| --- | --- |
| To triage | Issue mới cần đối chiếu tài liệu, phạm vi và phụ thuộc |
| Backlog | Chưa ưu tiên hoặc còn chờ gate; ghi rõ blocker trong issue |
| Ready | Có phạm vi, tiêu chí nghiệm thu và đủ prerequisite để bắt đầu phần việc đã nêu |
| In progress | Đang thực hiện; liên kết nhánh công việc và kết quả thử khi có |
| In review | Có PR hoặc artifact để review và bằng chứng theo checklist của issue |
| Done | Tiêu chí nghiệm thu đạt, gate tương ứng được kết luận và tài liệu liên quan đã cập nhật |

Một issue có thể chỉ đạt gate cho web hoặc một hệ mobile. Ghi kết quả riêng từng platform; không dùng kết quả web để đóng gate native. Khi chỉ hoàn thành một phần, tách follow-up có phạm vi rõ trước khi đóng issue gốc.

Sau prototype, cập nhật giả thuyết bị bác bỏ và bằng chứng trong tài liệu nguồn. Thay đổi stack, platform hoặc luật kinh tế cốt lõi cần quyết định mới theo [quyết định nền tảng](../decisions/0001-project-foundation.md).
