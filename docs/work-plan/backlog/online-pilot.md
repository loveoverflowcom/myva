# MVP Online có giới hạn

## Why

Solo slice chưa xác nhận ownership server, độ bền inventory và co-op dưới latency. MMO cần trạng thái có thể phục hồi trước trade và guild.

## Scope

Một linh vực, authoritative 2-client trước, persistence/reward/idempotency/reconnect, rồi co-op 2–4, trade giới hạn và guild cơ bản. Admission cap lấy từ load test; ngân sách tài nguyên chung toàn world.

## Non-goals

Không guild war, nhiều origin hoặc scale đa vùng trước bằng chứng pilot.

## Dependencies

Slice đạt gate; đọc [roadmap](../../production/roadmap.md), [architecture](../../technical/architecture.md), [QA](../../production/qa.md).

## Review boundary

Fault injection không mất/nhân reward; retry trade/craft trả kết quả đã commit; nhân vật chỉ có một session owner; solo progression không bị guild khóa.

## Risks / unknowns

Combat lag policy, storage workload, moderation và admission capacity chưa đo. Khi promote, chia theo capability có giá trị, không tạo PR khổng lồ bao gồm tất cả.

## Follow-ups

World expansion chỉ sau pilot và review mô hình kinh tế từ dữ liệu người chơi thật.
