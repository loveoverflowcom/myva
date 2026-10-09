# Kiểm chứng Bevy native trong CMP

## Why

MyVa yêu cầu mobile native. [ADR 0003](../decisions/0003-bevy-engine-adoption.md) chốt Bevy; [D02 / #10](https://github.com/loveoverflowcom/myva/issues/10) thay spike Macroquad [#1](https://github.com/loveoverflowcom/myva/issues/1) đã superseded. Sample Android/iOS standalone không chứng minh renderer nhúng ổn định vào CMP. Cần phát hiện blocker trước khi sản xuất asset và flow mobile lớn.

## Scope

Một cảnh di chuyển hình/nhân vật bằng touch trong shell CMP, có UI overlay nhỏ, trên Android và iOS thật. Kiểm chứng packaging Rust library, surface/context/thread, input ownership, resize/safe area, background/resume và giải phóng tài nguyên. Báo riêng phương án A (native view nhúng CMP) và B (màn native độc lập do CMP điều hướng); mở được app Bevy độc lập không hoàn thành B.

## Non-goals

Không làm account, MMO, giao dịch, world assets hoặc voice service. Không dùng WebView để đánh dấu native gate đạt. Không chọn ABI, renderer backend hay plugin chỉ từ ví dụ khác version.

## Dependencies

[Architecture](../technical/architecture.md), [ADR / compatibility matrix](../decisions/0003-bevy-engine-adoption.md) và [assets-performance](../technical/assets-performance.md). Bevy `=0.20.0`, Rust `1.97.1`; pin thêm CMP/Kotlin/Gradle/NDK/Xcode trong implementation và báo cáo. Chọn thiết bị thật; dependency/sample chính thức phải cùng version trước khi quyết định bridge.

## Suggested sequence

1. Doctor toolchain/device, ghi `KNOWN / UNKNOWN / BLOCKED`; đọc runner Winit và entry point mobile. Phân biệt API native view của CMP với API engine thật sự nhận surface.
2. Thử A tối thiểu mỗi hệ; ghi phần runner/surface cần adapter hoặc patch. Nếu bị chặn, ghi nguyên nhân và chi phí rồi đánh giá B. Không kết luận embedding từ build standalone.
3. Tích hợp input/render và shell overlay; lặp vào/rời 30 lần, background/resume 20 lần, resize/orientation, touch cancel, IME và audio focus. Dọn tài nguyên có log/counter hoặc profile.
4. Xuất bằng chứng theo platform/device, cập nhật kiến trúc và kết luận feasibility. Môi trường thiếu thiết bị hoặc Xcode ghi BLOCKED, không PASS hoặc hỗ trợ giả định.

## Review boundary

Người khác build được theo hướng dẫn và lặp tình huống trên thiết bị thật. [Báo cáo native hiện tại](../reports/native-feasibility.md) ghi command, version, kết quả và blocker; phần chưa chạy không được dùng để đóng #10. Spike chưa đủ khẳng định gameplay production chạy tốt.

## Risks / unknowns

Thread/context ownership, runner của Bevy/Winit, native packaging, đồng bộ UI/render, audio và backend thực tế. CMP không được tự gọi Bevy loop như một composable mà bỏ qua lifecycle OS.

## Follow-ups

Bridge production, input accessibility và audio focus sau khi spike xác nhận đường đi. Blocker iOS/Android tách rõ; tiếp tục D03 web và economy độc lập. Nếu cần đổi cách tích hợp, ghi ADR với chi phí; engine Bevy giữ nguyên trừ khi có quyết định mới.
