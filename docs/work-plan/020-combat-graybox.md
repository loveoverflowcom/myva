# Kiểm chứng combat và boss bằng graybox

## Why

MMO tạo tiến triển dài hạn nhưng từng phút chiến đấu phải vui, dễ đọc và chơi được bằng touch. Cần thử nhịp input/combo trước roster và mỹ thuật cuối.

## Scope

Client **Bevy** dùng foundation [D04 / #12](https://github.com/loveoverflowcom/myva/issues/12): một kit, một arena, một boss mẫu với di chuyển/nhảy/dash/guard, 2 đòn căn bản nhẹ/nặng + 3 thuật; thanh Mạch mở đại thuật biến thể một thuật. Simulation tách renderer, có replay. Web playable; mobile kiểm tra khi gate native tương ứng đạt.

## Non-goals

Không làm PvP ranked, nhiều lục địa, guild, market hoặc asset thành phẩm toàn roster.

## Dependencies

[Combat](../design/combat.md), [vertical slice](../production/vertical-slice.md) và [ADR 0003](../decisions/0003-bevy-engine-adoption.md); core/adapter ECS theo #12, web theo #11, mobile theo 010/#10 trên hệ tương ứng. Giữ simulation/headless độc lập renderer.

`crates/graybox` hiện dùng Macroquad là baseline lịch sử để đối chiếu luật/replay, chưa hoàn thành chuyển đổi Bevy. Không mở rộng engine legacy; D04 dựng boundary tối thiểu, #2 chịu trách nhiệm port và playtest kit/boss.

## Suggested sequence

1. Chuyển presentation/input từ baseline lịch sử sang Bevy + adapter #12; đối chiếu replay. Sau đó kiểm graybox movement/hit volumes, action timing và boss telegraph.
2. Thử keyboard/gamepad/touch, đo input delay và sai thao tác.
3. Playtest kit/boss, cập nhật cửa sổ cancel/guard và policy latency cần prototype online.

## Review boundary

Người chơi có thể thắng bằng đọc pattern; replay giải thích được hit/cancel. Tham số thay đổi có bằng chứng playtest và thông số render rõ.

## Risks / unknowns

Tap/hold có thể tạo trễ; màn hình mobile thiếu chỗ; simulation float không tự bảo đảm deterministic mọi target.

## Follow-ups

Mở 3 kit sau khi một kit đạt nhịp; authoritative 2-client/network impairment trước co-op MVP.
