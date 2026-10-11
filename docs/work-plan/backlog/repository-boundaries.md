# Boundary repository chờ consumer

## Why

[Refactor đường dẫn](../../technical/repository-layout.md) đã phân biệt entry web với packages dùng chung. Desktop graybox còn ghép library/bin; native là feasibility prototype. D05-A đã có fixture, schema/validator trong `packages/assets` và loader Bevy trong graybox. Việc tách tiếp contracts hoặc asset ownership cần consumer cụ thể để tránh wrapper và nguồn sự thật trùng.

## Scope

Khi promote, chia thành PR riêng theo consumer: entry desktop/native có lifecycle và packaging thật; contract dùng cho transport/ABI cụ thể; hoặc extraction loader/streaming có host thứ hai cần dùng chung. Tách module `graybox::fight` chỉ khi consumer cho thấy orchestration/replay và HUD state cần ownership riêng; giữ parity làm tiêu chí. Kết nối planner với `AssetServer` và acquisition performance là scope D05-B riêng.

## Non-goals

Không tạo app/server/mobile production, crate mới hoặc folder rỗng chỉ để hoàn chỉnh cây. Không đổi combat, tick, replay/hash, bridge, authority hoặc economics trong cleanup cấu trúc.

## Dependencies

- Entry desktop cần consumer phân phối và gate desktop phù hợp; mobile cần Android/iOS thật theo [D02](../010-native-feasibility.md) trước khi thay `prototypes/native` bằng app production.
- Contracts cần transport/ABI và owner cụ thể. `sim::{protocol,session,replay}` tiếp tục là SSOT của domain types; không sao chép types sang schema mới.
- Assets kế thừa [implementation D05-A](../../technical/asset-pipeline.md) và [evidence theo platform](../../reports/asset-pipeline.md). Xác minh consumer mới trước khi promote extraction; [D05-B](../../reports/asset-streaming-performance.md) tiếp tục cần runtime streaming, acquisition/fault/soak thật trước tuyên bố performance. Fixture D05-A không tự mở gate mobile hay budget production.

## Review boundary

Mỗi PR chỉ ra capability/consumer được bảo vệ, ownership/dependency trước và sau, mọi path consumer được cập nhật, rồi kiểm đúng target/platform. Extraction phải giữ hành vi và evidence cũ; thay protocol, authority hoặc pipeline thành nhiệm vụ riêng khi cần.

## Risks / unknowns

Native lifecycle, packaging iOS và phạm vi desktop production chưa được chốt; client mạng có thể cần snapshot chưa có trong mirror. Boundary loading/cache phụ thuộc pipeline và benchmark runtime đã được xác minh trên revision hiện hành. Không coi cây thư mục mới là bằng chứng các gate đã đạt.

## Follow-ups

Promote enabling boundary nhỏ có consumer tức thời trước tổng quát hóa; giữ [online pilot](online-pilot.md) và [world expansion](world-expansion.md) theo gate hiện có.
