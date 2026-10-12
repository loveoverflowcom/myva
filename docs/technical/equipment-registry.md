# Equipment Registry & Headless Build Rules

**Trạng thái:** offline data/prototype cho [#20](https://github.com/loveoverflowcom/myva/issues/20), 2026-10-12. Chưa tích hợp hiệu ứng vào combat, chưa compile/run Rust trên host hiện tại; xem [bằng chứng](../reports/equipment-registry.md). Năm thế lực, catalog và kit của PR27–29 giữ nguyên.

## Phạm vi và nguồn dữ liệu

`packages/sim/src/equipment.rs` thuộc simulation thuần Rust, không thêm crate/dependency, renderer, inventory, shop, economy, entitlement hoặc online PvP. Chọn **ba ứng viên đầu** của mục 5 issue #20 để thử luật build; issue không chốt một bộ ba production. Tên là hư cấu/GT, các con số dưới đây là GT offline, chưa được playtest hoặc cân bằng.

| ID ổn định | Tên | Budget | Pure effect prototype | Item version/hash |
| --- | --- | ---: | --- | --- |
| `an-phan-chan` | Ấn Phản Chấn | 3 | Một guard chính xác phía trước, blockable: hồi tối đa 8 sức bền, cooldown 300 tick/5 s, tối đa một proc/source action | 1 / `67bf57e86c58af0e` |
| `giap-tran-mach` | Giáp Trấn Mạch | 3 | Giảm 15% guard pressure phía trước khi đòn blockable; không giảm HP damage, không chống sau lưng/unblockable | 1 / `8683fcf7fed50f68` |
| `phu-vu-bo` | Phù Vũ Bộ | 2 | Giảm 20% dash cost, làm tròn lên, sàn 15 điểm; không tăng chi phí nếu base dưới sàn | 1 / `782d9b33f5a4124b` |

`EquipmentRegistry::new(entries, rules)` nhận dữ liệu immutable borrowed và validate một lần trước lookup: ID, duplicate, tên, cost dương, numeric domains, effect caps, cooldown dương/tối thiểu, version/hash item và build rules. Không leak chuỗi runtime; `entry(&str)` trả identity authored. `resolve(ItemPin)` chỉ chấp nhận exact version/hash. Hash FNV-1a canonical v1 dùng integer fixed-width little-endian, string UTF-8 length-prefixed u64 và explicit effect tags; display name được loại khỏi gameplay hash. Hash là fingerprint, không là chữ ký hoặc quyền sở hữu item.

## Luật build và stacking

Baseline rules version 1 / `9dc0378661ca5573`:

- Tối đa 3 ô, tổng ngân sách 6; không bắt buộc dùng hết ô/budget. Ba item cùng lúc tốn 8, bị từ chối; recovery + guard dùng đủ 6, một trong hai + dash dùng 5.
- Không duplicate item ID; mỗi nhóm GuardRecovery / GuardPressure / DashCost tối đa 1.
- Refund cộng trực tiếp, trần 10 điểm; nếu authored rules cho nhiều item, dùng cooldown **lớn nhất**, không reset/multiply.
- Reduction cộng basis points, trần guard 20% và dash 20%; vượt cap bị từ chối thay vì silently clamp. Percent không nhân chồng.
- Refund cooldown tối thiểu 180 tick/3 s; dash cost sàn 15. Numeric domain cấm 100% reduction, cost sàn 0, cooldown 0 và recovery trên 100 điểm.
- Số học trung gian dùng u64 cho budget và scaling, làm tròn lên. Positive pressure/cost vẫn dương; reduction không tăng base value. Không sửa startup/recovery/hitbox/i-frames.

`resolve_build(&[&str])` trả `LockedEquipment` private fields, canonical ID order, exact item/rules pins, budget và fingerprint; đổi thứ tự catalog/input không đổi build. Registry tùy chỉnh dùng cùng validator. `revalidate` kiểm exact pins và rules trước khi lock lại, không nâng revision ngầm.

## Loadout, headless và runtime boundary

`KitRegistry::resolve_loadout` validate baseline equipment cùng kit/binding/lore permission; `resolve_loadout_with_equipment` nhận authored equipment registry khác để làm fixture. `LockedLoadout::equipment()` đọc build đã khóa. `request()` chỉ là ID request; `revalidate_loadout()` giữ exact pins cho baseline rematch. Lock không thay tài nguyên hoặc grant entitlement.

Consumer CLI không spawn World hoặc xuất replay:

```bash
cargo xtask run replay -- build-check
cargo xtask run replay -- build-check an-phan-chan phu-vu-bo
cargo xtask run replay -- build-check an-phan-chan giap-tran-mach phu-vu-bo # nonzero: budget
```

CLI in budget, build fingerprint, item/rules pins và `NOT_IMPLEMENTED` cho combat integration. Invalid IDs, unknown items, duplicates, slot/budget/group/cap/version/hash errors trả typed errors và nonzero.

`LockedEquipment::dash_cost` / `guard_pressure` chỉ là pure integer calculations. `guard_recovery(build, previous_state, contact, stamina, cap)` trả state/resource mới, không mutate input; target/build pin, tick monotonic, cooldown và per-source action watermark được giữ trong owned prototype state. Item-origin contacts không proc; cùng action dù khác hit index không refund thêm, contact được thử trong cooldown không được replay sau cooldown để proc. Nguồn mới không lách cooldown toàn item. Tick cooldown dùng u64 để không wrap cuối u32. Contact booleans/identities là fixture inputs, **chưa phải xác nhận authoritative từ combat**. Đây không là chứng minh exploit prevention trong game đang chạy.

`Bout::start_locked`, được cả Battle/Bout/rematch đi qua, từ chối valid nonempty equipment với `EquipmentError::RuntimeNotImplemented` **trước spawn**. Registry, LockedLoadout và headless calculations vẫn sử dụng được. Không equip thành công rồi im lặng bỏ hiệu ứng.

## Replay compatibility và TODO

Không đổi `Kit`, World, state hash, damage/timing, resource laws, `RULES_VERSION`, replay parser/encoder hoặc v3/v4 wire. Mười file được `rules_config_hash` fingerprint giữ nguyên từng byte so develop; baseline laws fingerprint vẫn `93755e498de17ca3`. Existing v3/v4 regression tests tiếp tục có trong suite nhưng chưa chạy lại trên host này. Dòng equipment/item thêm vào replay v3/v4 vẫn bị từ chối; không fallback hoặc tự promote.

Trước khi mở equipment combat:

1. Tích hợp effect vào core tại đúng cost/guard/verified-contact phase; không refund sau action bị từ chối, không bỏ qua guard break, không lách attack identity/cooldown.
2. Đặc tả state/proc identity và reset theo round; duy trì core/ECS/headless parity. Watermark prototype hiện từ chối action instance cũ/wrap, không hỗ trợ spawn hoặc reuse actor IDs giữa round.
3. Xuất **replay version mới tường minh** để ghi equipment/rules pins và proc state cần thiết; so v3/v4 golden và không thay playback lịch sử bằng luật mới.
4. Chạy Rust format/clippy/tests, native/WASM determinism rồi playtest matchup/side-swap, device và latency theo gate #20/#5. Headless formulas không chứng minh balance hoặc PvP.

Trảo Truy Kích, Linh Ấn Tĩnh Tâm, Hộ Phù Giải Trói, build Lâm Trảo, inventory/shop, respec/weapon switching và online competitive authority tiếp tục TODO, không được nhận thành implemented item bằng ID fallback.
