# D05 — Assets và baseline Bevy

**Trạng thái:** D05-A đã merge vào develop; D05-B runtime/harness triển khai trên `feat/bevy-asset-runtime`, qualification render/device còn BLOCKED, ngày 2026-10-11. Issue [#13](https://github.com/loveoverflowcom/myva/issues/13). D04 và combat đã tích hợp vào `develop` ở `33c77e1e7e507c8a76723c14fa9731f4b5b45875`; merge không xác nhận gate art/mobile/performance.

## Kết quả cần review

Cảnh chuẩn dùng bundle hữu hạn, giữ tín hiệu combat rõ và có phép đo tái lập. Streaming/benchmark không đổi luật, replay hoặc mạng. D05-B đã nối planner với AssetServer và có benchmark atlas 50/200/500 × High/Low; chưa có actual rendered/device qualification trên dot cloud.

## Vì sao

Trước D05-A, `develop` chỉ có presentation bằng hình khối, không có atlas fixture, manifest hay validator; benchmark hình khối không thể chứng minh bộ nhớ/decode/upload của pipeline art. WASM combat lịch sử có kích thước 84.842.609 byte, gzip ước tính 11.182.853 byte, chưa phải payload P1 truyền thực tế.

## Phụ thuộc và thứ tự

1. **D05-A, đã merge:** fixture có provenance cho Long Lưu, Kẻ Giữ Đập, bản đồ Đập Cổ và VFX; atlas/sheet có pivot, padding, extrude, scale và clip `idle`/`run`/`jump`/`attack` với mốc combat; manifest có ID, version, hash, kích thước, dependency, bundle budget, provenance/license; validator `myva-assets` với test asset lỗi; loader `AssetServer` trong graybox chạy native và web. Đặc tả: [asset-pipeline](../technical/asset-pipeline.md); kết quả: [báo cáo D05-A](../reports/asset-pipeline.md). Không mở rộng roster/art production, CDN, cache production hoặc skeletal runtime.
2. **D05-B code/harness, đã triển khai:** AssetServer/strong-handle ownership, required preload, optional VFX, exit/draining cancellation, retry/fallback hữu hạn, six render-only presets và High/Low thực.
3. **D05-B qualification, bước tiếp theo:** chạy harness trong browser/native window được hỗ trợ; capture 60 giây × six presets, 30 cycle, scene/fault/Low target, 600 giây soak và mở/inspect ảnh; đo GPU/presentation/RSS/network startup đúng scope. Không mở rộng framework, CDN hoặc art roster trước gate này.
4. **Art slice:** chỉ mở rộng sau báo cáo budget-vs-actual và Go/No-Go theo [assets-performance](../technical/assets-performance.md). Android/iOS phải qua [native gate](../reports/native-feasibility.md) riêng.

## Kiểm chứng

Logic và report contract có test độc lập; các test ấy không xác nhận GPU upload/unload. Pin Rust 1.97.1 / Bevy 0.20.0. Task hiện hành chạy trên dot cloud với dependencies chính thức; không dùng Mac hoặc đo chồng workload. Đọc [báo cáo D05-B](../reports/asset-streaming-performance.md) để biết command và gate chưa chạy. Không đóng #13 khi còn acceptance chưa đạt.

## Rủi ro / chưa biết

- D05-A là fixture sinh bằng mã: chứng minh định dạng, validator, loader và fallback, không chứng minh chất lượng hay chi phí art vẽ tay, mipmap hoặc mobile thật.
- Dot cloud đã có toolchain pin; browser shell Unix socket và CUA-to-localhost bị block, native window thiếu display/GPU device. Giữ qualification BLOCKED thay vì dùng CPU tests làm GPU PASS.
- CPU AssetServer 30 load/unload/fault/cancel/scene tests có evidence; hardware GPU/presentation, rendered lifecycle/soak 10 phút, visual review và mobile thật còn thiếu.
- Repo chưa có `AGENTS.md` hoặc `.agents/skills`; review source độc lập theo yêu cầu đã sửa ba regression và chạy lại final checks. Qualification rendered/device vẫn là bước tiếp theo.
