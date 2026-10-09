# Kiểm chứng Macroquad native trong CMP

## Why

MyVa yêu cầu mobile native. Macroquad hỗ trợ build Android/iOS chưa chứng minh rằng renderer có thể nhúng ổn định vào shell CMP. Cần phát hiện blocker trước khi sản xuất asset và flow mobile lớn.

## Scope

Một cảnh di chuyển hình/nhân vật bằng touch trong shell CMP, có UI overlay nhỏ, trên Android và iOS thật. Kiểm chứng packaging Rust library, render surface/context, input ownership, resize/safe area, background/resume và giải phóng tài nguyên.

## Non-goals

Không làm account, MMO, giao dịch, world assets hoặc voice service. Không dùng WebView để đánh dấu native gate đạt.

## Dependencies

[Architecture](../technical/architecture.md) và [assets-performance](../technical/assets-performance.md). Chọn thiết bị thật và toolchain; đọc mã/version dependency trước quyết định bridge.

## Suggested sequence

1. Thử lifecycle tối thiểu mỗi hệ; ghi rõ phần Macroquad/miniquad cần adapter hoặc patch.
2. Tích hợp input/render; đo lặp vào/rời, xoay/resize, background và khôi phục.
3. Xuất bằng chứng, cập nhật kiến trúc và quyết định feasibility theo từng hệ.

## Review boundary

Một người khác build được từ hướng dẫn và lặp các tình huống trên. Chưa đủ để khẳng định toàn bộ gameplay production chạy tốt.

## Risks / unknowns

Thread/context ownership, lifecycle của engine, native packaging, đồng bộ UI/render và hỗ trợ backend đồ họa có thể cần thay đổi có phạm vi.

## Follow-ups

Bridge production, input accessibility và audio focus sau khi spike xác nhận đường đi. Blocker của iOS/Android phải tách rõ, không giấu bằng kết quả web.
