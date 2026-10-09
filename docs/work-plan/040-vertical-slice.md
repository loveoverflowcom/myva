# Hoàn thiện vòng chơi Bến Lau

## Why

Các spike riêng lẻ chưa chứng minh quest, combat, tài nguyên và tải asset tạo thành một trải nghiệm người mới có thể hoàn thành.

## Scope

Slice một linh vực Vân Thủy, 3 bản đồ, 3 kit, boss Kẻ Giữ Đập, recipe và checkpoint. Tích hợp pipeline asset đã đo, onboarding điều khiển và lưu tiến trình prototype; chơi 20–30 phút.

## Non-goals

Không mở world trade, guild war, nhiều origin, PvP hoặc monetization. Local state không trở thành vật phẩm online thật.

## Dependencies

020 và 030 đạt gate; mobile phụ thuộc 010 trên hệ tương ứng. [Vertical slice](../production/vertical-slice.md) là nguồn scope và gate.

## Suggested sequence

1. Chốt một mẫu asset/export/render đúng; làm thêm 2 kit và 3 map trong ngân sách.
2. Ghép quest/craft/boss/respawn/loading; sửa các chỗ chờ hoặc không hiểu đường đi.
3. Playtest người mới, benchmark device thật; cập nhật GDD và kế hoạch online.

## Review boundary

Người mới hoàn thành chuỗi chính bằng solo, nhận thấy khác biệt kit và retry boss không cần farm cưỡng ép. Nội dung và platform phải có bằng chứng theo checklist slice.

## Risks / unknowns

Asset cuối có thể vượt GPU budget; progression có thể chặn combat; native spike không tương đương toàn slice.

## Follow-ups

Thêm online pilot vào queue chỉ sau khi gate đạt; vấn đề integrity/networking đi trước tính năng social lớn.
