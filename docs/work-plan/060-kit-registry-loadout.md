# PR2 — Kit registry và loadout

**Trạng thái:** TODO, phiên2; chưa triển khai trong PR1. Phụ thuộc head hoàn thành của [PR1](050-faction-lineage-domain.md); các PR lấy develop làm base theo yêu cầu, giữ dependency trong mô tả/diff review nếu chưa merge.

## Kết quả và lý do

Registry/loadout/version nhỏ có consumer thật, sử dụng `KitId` ổn định và validator catalog. Consumer phải phân biệt metadata proposed, kit có implementation và quyền trang bị; `Catalog::hierarchy()` chỉ phân giải lore.

## Phạm vi và acceptance

- Đối chiếu `kit.rs`, `battle.rs`, `replay.rs` và fixtures tại head PR1 trước khi thiết kế registry; không tự duyệt roster hay gán Long Lưu sang tôn giáo.
- Giữ `long-luu`, `Kit.lineage` và tên đòn đang tham gia hash; registry/version migration phải có regression replay cũ, per-tick parity và explicit unsupported errors.
- Nếu có parser/owned data, chọn ownership theo consumer; catalog PR1 đang author tĩnh, không dùng leak để ép `&'static str`.
- Không đổi damage, skill rules, 60 Hz, guard/hitbox/Mạch hoặc economy chỉ để chuyển registry. Không tạo UI/asset mapping thuộc PR3.
- Chạy xtask gates; thiếu tool phải BLOCKED/NOT_RUN, không fake PASS. Review độc lập, sửa findings, normal push/draft PR; không merge/force push.

## Rủi ro còn mở

Version/hash contract phải được chứng minh từ code thực tế. Roster/mapping lore chưa được duyệt. Runtime chưa hỗ trợ phải fail tại boundary bằng lỗi có kiểu, không panic hoặc chọn fixture khác. Browser/WASM hiện thiếu prerequisites trên Mac; xem [evidence PR1](../reports/faction-lineage-domain.md).

Tiếp theo là [PR3](070-lineage-presentation.md), không mở phiên mới từ PR1.
