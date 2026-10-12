# Character content và rendered qualification

**Trạng thái:** deferred sau [PR3](../070-lineage-presentation.md). Không đưa ứng viên vào baseline để lấp dữ liệu thiếu.

## Kết quả cần có

Một kit có source/lineage/branch và metadata/art được duyệt bằng hồ sơ cụ thể, chỉ promote sau gate implementation/playability và rendered platform tương ứng. Web/device chứng minh layout, focus, touch, lifecycle và lỗi thật; không thay bằng headless CPU tests.

## Vì sao / phụ thuộc

PR3 chỉ có Long Lưu offline unresolved, text icon và art D05-A placeholder; boss/quái thiếu catalog lore. Năm nền tảng được chốt không duyệt roster hoặc hình tượng. [ADR 0004](../../decisions/0004-five-factions-lineage-hierarchy.md), #18/#20 sở hữu approval; [D05-B](../../reports/asset-streaming-performance.md) giữ rendered/performance gate; browser prerequisites thiếu được báo trong [PR3 evidence](../../reports/character-catalog-assets.md).

## Việc tiếp theo khi đủ dữ liệu/công cụ

- Diagnose capture/render native toàn đen: source PR2 nguyên bản và PR3 cùng SHA PNG; chưa xác định nguyên nhân. Giữ visual BLOCKED, không suy HUD/layout PASS từ screenshot file.
- Provision đúng bindgen/browser theo workflow dự án rồi chạy web pipeline/E2E, mở screenshots ở desktop/narrow/touch để inspect; thiết bị Android/iOS theo gate riêng.
- Duyệt source/lineage/branch/presentation/icon và asset mapping theo từng hồ sơ; thêm metadata boss/quái nếu có consumer lore được duyệt. Không tự tạo mười trực hệ hoặc màn chọn năm phe.
- Nếu consumer mới cần live mapping reload, multi-map selection hay bundle dependencies, thiết kế từ nhu cầu ấy; runtime hiện reject unsupported dependencies và không hot-reload catalog.

Equipment, inventory/server nhân vật, online entitlement và replay migration tiếp tục backlog PR2; không promote chỉ vì đã có UI chọn kit.
