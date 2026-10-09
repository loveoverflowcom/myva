# Hàng đợi công việc MyVa

Repository hiện ở giai đoạn docs-only. Các bước dưới đây là công việc prototype cần review riêng; chưa có mã nguồn để khẳng định cấu trúc crate, schema hoặc native bridge cuối cùng.

Số `010`, `020`, ... biểu thị **thứ tự khuyến nghị hiện tại**, có thể đánh lại khi ưu tiên thay đổi; không phải ID cố định. Luật chuyên môn nằm trong docs/design và docs/technical, không sao chép thành một hệ thống quản lý trạng thái khác.

| Thứ tự | Công việc | Kết quả cần review | Phụ thuộc |
| --- | --- | --- | --- |
| [010](010-native-feasibility.md) | Macroquad native trong CMP | Một cảnh render tương tác trên Android/iOS thật, lifecycle và packaging có bằng chứng | Baseline docs |
| [020](020-combat-graybox.md) | Combat và boss graybox | Một kit, một arena, replay và điều khiển touch | Baseline; web làm độc lập với 010, mobile phụ thuộc 010 |
| [030](030-economy-simulator.md) | Simulator tài nguyên | Invariant, công thức, demand shocks và ledger được đo | Baseline; có thể song song với 010/020 |
| [040](040-vertical-slice.md) | Slice Bến Lau | Vòng chơi 20–30 phút, 3 kit, tải asset và solo progress | Gate 010/020/030 cho phần platform tương ứng |

010 ưu tiên sớm vì có thể ảnh hưởng platform. Thứ tự khuyến nghị không cấm làm độc lập song song; một gate chưa đạt không được đánh dấu phần phụ thuộc đã xong.

## Sau slice

- [Online pilot](backlog/online-pilot.md) chỉ được đưa vào active queue khi slice và integrity prerequisites đạt.
- [World expansion và chiến dịch](backlog/world-expansion.md) chờ pilot có bằng chứng tải, kinh tế và solo progression.

Sau mỗi prototype, cập nhật các giả thuyết bị bác bỏ và follow-up cần thiết. Việc này ưu tiên lỗi ảnh hưởng người chơi trước tối ưu hoặc tổng quát hóa kiến trúc.
