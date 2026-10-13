# Equipment Registry & Headless Build Rules

**Trạng thái:** đã author offline prototype/data cho #20; Rust verification BLOCKED/NOT_RUN trên dot cloud host thiếu toolchain. [Contract](../technical/equipment-registry.md), [evidence](../reports/equipment-registry.md). Base develop `98e7ddc2c7fe47831907e76d0ac022b41373b030` sau merge PR27–29; không triển khai lại domain/kit/assets.

## Kết quả và ranh giới review

Ba item đầu mục 5 issue #20, immutable equipment registry, bounded budget/stacking, exact item/build pins, LockedLoadout và headless `build-check` consumer. Pure rules thử dash cost, guard pressure và guard recovery identity/cooldown. Combat/replay từ chối nonempty equipment trước spawn vì chưa có runtime/state contract; không silently ignore.

## Kiểm chứng và bước tiếp theo

Rust property/regression tests đã author cho registry/domain bounds, permutation/stacking/budget, refund loops, exact revalidation, runtime rejection, CLI và v3/v4 unknown-wire rejection. Host hiện tại không có Cargo/rustc/rustfmt; không tự tải toolchain. Chạy `cargo xtask verify repository`, `verify sim`, `verify gameplay` và determinism khi prerequisites đã provision. Không đổi timing/damage hoặc mở online PvP để hoàn thành headless data.

Runtime effect integration/replay mới và playtest được giữ riêng trong [backlog](backlog/equipment-runtime-effects.md). Đây là bước phụ thuộc cần review, không tự promote toàn issue #20 hoặc claim combat equipment đã xong.
