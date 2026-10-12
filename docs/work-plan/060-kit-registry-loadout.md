# PR2 — Kit registry và loadout

**Trạng thái:** đã triển khai trong phiên2; [contract](../technical/kit-registry-loadout.md), [evidence](../reports/kit-registry-loadout.md). Phụ thuộc PR27 head `2f0a71269e9b428ba488b39c1fe4c67532adbe3e`; stacked draft base `feat/faction-lineage-domain` khi PR27 chưa merge, đích tích hợp cuối `develop`. Không tự merge PR1.

## Kết quả và lý do

Registry/loadout/version nhỏ có consumer thật, sử dụng `KitId` ổn định và validator catalog. Consumer phải phân biệt metadata proposed, kit có implementation và quyền trang bị; `Catalog::hierarchy()` chỉ phân giải lore.

## Phạm vi và acceptance

- Đối chiếu `kit.rs`, `battle.rs`, `replay.rs` và fixtures tại head PR1 trước khi thiết kế registry; không tự duyệt roster hay gán Long Lưu sang tôn giáo.
- Giữ `long-luu`, `Kit.lineage` và tên đòn đang tham gia hash; registry/version migration phải có regression replay cũ, per-tick parity và explicit unsupported errors.
- Nếu có parser/owned data, chọn ownership theo consumer; catalog PR1 đang author tĩnh, không dùng leak để ép `&'static str`.
- Không đổi damage, skill rules, 60 Hz, guard/hitbox/Mạch hoặc economy chỉ để chuyển registry. Không tạo UI/asset mapping thuộc PR3.
- Chạy xtask gates; thiếu tool phải BLOCKED/NOT_RUN, không fake PASS. Review độc lập, sửa findings, normal push/draft PR; không merge/force push.

## Rủi ro còn mở

Registry validated và loadout consumer Battle/CLI đã có; revision/hash/rules pins dùng replay v4 mới, v3 giữ byte legacy và chỉ nhận frozen baseline. Long Lưu vẫn unresolved legacy fixture; source fingerprint bảo thủ có thể từ chối replay khi đổi comment/refactor file luật. Equipment/procs/budgets, alternate ultimate slots, roster lore, historical loader và online competitive entitlement chưa hỗ trợ. Lỗi explicit tại boundary; browser/WASM thiếu prerequisites, xem evidence PR2.

Tiếp theo là [PR3](070-lineage-presentation.md), không mở phiên mới từ PR1.
