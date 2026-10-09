# Hàng đợi công việc MyVa

Repository có simulation/economy Rust thuần và client graybox **Macroquad lịch sử**. Engine hiện hành là **Bevy** theo [ADR 0003](../decisions/0003-bevy-engine-adoption.md); graybox chờ nền ECS #12 rồi combat #2 chuyển đổi. Spike tích hợp không khẳng định gameplay hoặc native bridge production đã hoàn thành.

[Planning và Kanban](planning.md) liên kết các issue triển khai với project GitHub; trạng thái hiện tại được cập nhật trên board.

Số `010`, `020`, ... biểu thị **thứ tự khuyến nghị hiện tại**, có thể đánh lại khi ưu tiên thay đổi; không phải ID cố định. Luật chuyên môn nằm trong docs/design và docs/technical, không sao chép thành một hệ thống quản lý trạng thái khác.

| Thứ tự | Công việc | Kết quả cần review | Phụ thuộc |
| --- | --- | --- | --- |
| D01 / [#9](https://github.com/loveoverflowcom/myva/issues/9) | ADR và kiến trúc Bevy | Pin, ownership, lifecycle, compatibility matrix | Quyết định #8 |
| D02 / [010](010-native-feasibility.md) / [#10](https://github.com/loveoverflowcom/myva/issues/10) | Bevy native trong CMP | Android/iOS thật; surface, shell navigation, input, lifecycle, packaging | D01; thay #1 cũ |
| D03 / [#11](https://github.com/loveoverflowcom/myva/issues/11) | Bevy WASM + Leptos | Load, resize/focus, bridge, route teardown và browser evidence | D01; song song D02 |
| D04 / [#12](https://github.com/loveoverflowcom/myva/issues/12) | ECS + simulation headless | Adapter core, schedule/schema, replay/invariant không GPU | Boundary D01; không chờ mobile production |
| D05 / [#13](https://github.com/loveoverflowcom/myva/issues/13) | 2D pipeline + performance/QA | Assets, memory/build size/frame time đo thật | D02/D03 theo platform đã qua gate |
| [020](020-combat-graybox.md) / #2 | Combat và boss graybox Bevy | Một kit, một arena, replay và điều khiển touch | D04; web qua D03, mobile qua D02 tương ứng |
| [030](030-economy-simulator.md) / #3 | Simulator tài nguyên | Invariant, công thức, demand shocks và ledger được đo | Độc lập engine; có thể song song |
| [040](040-vertical-slice.md) / #4 | Slice Bến Lau | Vòng chơi 20–30 phút, 3 kit, tải asset và solo progress | 020/030 và D02/D03/D05 cho platform tương ứng |

[#7 tooling/skills](https://github.com/loveoverflowcom/myva/issues/7) chạy song song D01, phải dùng Bevy và phân biệt headless, browser, standalone mobile, CMP embedding, thiết bị thật. Issue #1 chỉ giữ lịch sử Macroquad; không tiếp tục triển khai theo hợp đồng cũ. Một gate chưa đạt không được đánh dấu phần phụ thuộc đã xong.

## Sau slice

- [Online pilot](backlog/online-pilot.md) chỉ được đưa vào active queue khi slice và integrity prerequisites đạt.
- [World expansion và chiến dịch](backlog/world-expansion.md) chờ pilot có bằng chứng tải, kinh tế và solo progression.

Sau mỗi prototype, cập nhật các giả thuyết bị bác bỏ và follow-up cần thiết. Việc này ưu tiên lỗi ảnh hưởng người chơi trước tối ưu hoặc tổng quát hóa kiến trúc.
