# D05 — Assets và baseline Bevy

**Trạng thái:** prerequisite chưa triển khai, ngày 2026-10-11. Issue [#13](https://github.com/loveoverflowcom/myva/issues/13). D04 và combat đã tích hợp vào `develop` ở `33c77e1e7e507c8a76723c14fa9731f4b5b45875`; merge không xác nhận gate art/mobile/performance.

## Kết quả cần review

Cảnh chuẩn dùng bundle hữu hạn, giữ tín hiệu combat rõ và có phép đo tái lập. Streaming/benchmark không đổi luật, replay hoặc mạng. D05-B hiện chỉ chuẩn bị planner và hợp đồng báo cáo độc lập; chưa có backend tải asset hay cảnh benchmark chạy được.

## Vì sao

`develop` chỉ có presentation bằng hình khối. Không có atlas fixture, manifest, validator hoặc D05-A; benchmark hình khối không thể chứng minh bộ nhớ/decode/upload của pipeline art. WASM combat lịch sử có kích thước 84.842.609 byte, gzip ước tính 11.182.853 byte, chưa phải payload P1 truyền thực tế.

## Phụ thuộc và thứ tự

1. **D05-A có phạm vi hẹp, chờ duyệt:** một fixture có provenance cho character, map, boss và VFX; atlas/export metadata có ID, hash, kích thước, dependency, pivot/padding; validator reference/frame/budget; smoke loading Bevy native/web. Không mở rộng roster/art production, CDN, cache production hoặc skeletal runtime.
2. **D05-B:** nối planner với `AssetServer` và strong handles; scene preload, optional prefetch, exit/cancel/retry/fallback có giới hạn; benchmark render-only riêng với gameplay; capture RAM/decode/GPU và QA trên runtime thật.
3. **Art slice:** chỉ mở rộng sau báo cáo budget-vs-actual và Go/No-Go theo [assets-performance](../technical/assets-performance.md). Android/iOS phải qua [native gate](../reports/native-feasibility.md) riêng.

## Kiểm chứng

Logic và report contract có test độc lập; các test ấy không xác nhận GPU upload/unload. Pin Rust 1.97.1 / Bevy 0.20.0. Chỉ chạy build/download sau khi parent xếp lịch chung với Cantos; measured runs không overlap. Đọc [báo cáo D05-B](../reports/asset-streaming-performance.md) để biết command và gate chưa chạy. Không đóng #13 khi còn acceptance chưa đạt.

## Rủi ro / chưa biết

- D05-A chưa được tìm thấy trên local hoặc remote; phần predecessor cần scope riêng, không giả là đã xong.
- Mac chỉ còn khoảng 6 GiB sau cleanup; toolchain 1.97.1 và wasm-bindgen/wasm-opt chưa cài. Cần dự toán disk trước heavy window.
- Hardware GPU/presentation, 30 asset load/unload, soak 10 phút, lỗi texture/cancel, và mobile thật chưa có evidence D05-B.
- Repo chưa có `AGENTS.md` hoặc `.agents/skills`; không đánh dấu skill review đã chạy.
