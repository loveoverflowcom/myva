# Kit registry và loadout

**Trạng thái:** PR2/3, offline implementation boundary cho [#20](https://github.com/loveoverflowcom/myva/issues/20) và [#19](https://github.com/loveoverflowcom/myva/issues/19), 2026-10-12. Phụ thuộc [catalog PR1](faction-lineage-catalog.md); không duyệt roster, lore hoặc balance. Thông số Long Lưu giữ nguyên GT.

## Dữ liệu và quyền trang bị

`packages/sim/src/registry.rs` nằm trong `myva-sim`, không thêm dependency/crate. `KitRegistry::new(catalog, entries)` nhận slices immutable và chỉ trả registry sau khi validate catalog, ID, trùng ID, version khác 0, hash thông số, body/timing/ultimate slot và giới hạn số học/geometry của arena hiện tại và catalog record cho kit người chơi. Một metadata record có thể chưa có implementation; lookup gameplay trả `UnknownKit`, không chọn kit khác.

| Kit ID | Version | Config hash v1 | Quyền |
| --- | --- | --- | --- |
| `long-luu` | 1 | `a624f529d63112f0` | Player; mặc định chỉ fixture offline chưa ánh xạ |
| `ke-giu-dap` | 1 | `f4c34792bf8d1ae4` | ReplayOnly; boss |
| `quai-bun` | 1 | `e8aac515a7849264` | ReplayOnly; quái |

`KitRegistry::baseline()` validate dữ liệu tĩnh hiện có. `entry(&str)` và `resolve(KitRequest)` nhận chuỗi runtime, dùng cùng luật slug của catalog mà không leak/allocation để ép ID thành static. Identity lấy từ `Kit.id`, độc lập thứ tự registry. `resolve` yêu cầu version/hash chính xác; `InvalidId`, `UnknownKit`, `VersionMismatch`, `HashMismatch` và `InvalidRegistry` là lỗi explicit.

`resolve_loadout(LoadoutRequest)` tạo `LockedLoadout` với private fields và getters `kit()`, `pin()`, `binding()`. Nhánh yêu cầu phải bằng `KitPlacement::Branch` trong catalog; kit và toàn bộ parent chain phải có metadata approval. Permission `Player` riêng biệt với approval lore; boss/quái hoặc `Disabled` trả `NotEquippable`.

Long Lưu vẫn là `UnmappedFixture` và metadata proposed. Caller phải chọn rõ `LoadoutBinding::UnresolvedLegacyFixture`; nhánh bất kỳ trả `BranchMismatch`. Đây là ngoại lệ cho fixture offline, không là xác nhận roster/theology hoặc quyền dùng trong chế độ cạnh tranh. Năm nền tảng vẫn giữ nguyên, sources/lineages/branches mặc định rỗng. Các branch/approval tests dùng dữ liệu synthetic; không nhập mười ứng viên hoặc ánh xạ Long Lưu sang tôn giáo. Team PvP, địa lý và presentation không nằm trong loadout contract.

## Consumer và khóa round

`Bout::start_with_loadout(mode, round, request)` và `Battle::with_loadout(...)` resolve trước khi spawn. `Bout::loadout()` trả dữ liệu đã khóa; không có setter giữa round. Rematch giữ kit/pin/binding, kiểm lại registry trước khi tạo session mới, rồi giữ trạng thái autopilot. `GameplayPlugin` vẫn nhận `Session`, chạy đúng một luật authoritative ở 60 Hz; không có combat implementation thứ hai.

`Bout::start` / `Battle::new` giữ fixture mặc định và replay v3 nhưng cũng resolve qua registry. Các constructor/rematch nay trả `Result<..., RegistryError>`; CLI propagate lỗi, graybox báo `GrayboxStartupError`; native kiểm lỗi trước khi đăng ký host systems/runner, WASM `start_game` trả lỗi cho JS try/catch khi startup bị từ chối, rematch bị từ chối giữ trận hiện tại. Test callers unwrap dữ liệu fixture đã kiểm chứng. Các API spawn raw trong `World`/`Session` vẫn dành cho authored fixtures và adapter; chúng không là boundary nhận loadout từ client.

CLI là consumer cho chọn ID/version/hash mà không thêm UI catalog:

```bash
cargo xtask run replay -- demo-loadout /tmp/long-luu-v4.myva-replay long-luu 1 a624f529d63112f0 --legacy-fixture
cargo xtask run replay -- verify /tmp/long-luu-v4.myva-replay
```

Loadout hiện chỉ chọn toàn kit. Danh sách equipment không rỗng trả `UnsupportedEquipment`; chưa có item/proc/budget/vũ khí, nội tại Dư Ba, chọn một ultimate slot khác, respec production, online loadout authority hoặc competitive entitlement. Không silently ignore đồ/đổi cost. Các phần #20 đó là TODO theo gate của issue, không nằm trong PR2 hoặc PR3.

## Version và replay

`KitRevision { version, config_hash }` author bằng pin cố định. `kit_config_hash` v1 dùng FNV-1a 64-bit, số nguyên có độ rộng cố định little-endian, string UTF-8 với độ dài u64 và Option/enum tag tường minh. Nó bao phủ ID, legacy lineage/action names, body, ba light/heavy/ba skill, projectile/status/counter và ultimate/slot/Mạch. Đổi thông số phải cập nhật revision có chủ ý; pin cũ sai sẽ bị từ chối. Hash không là chữ ký bảo mật, không xác minh người dùng/approval và không thay `World::state_hash`.

`RULES_VERSION = 3`; `rules_config_hash = 93755e498de17ca3` tại baseline. Đây là fingerprint bảo thủ của các source file luật playback và state encoding (control/fighter/input/kit/meter/NPC/projectile/status/tick/world), để phát hiện cả thay đổi thuật toán/global constants ngoài kit. Catalog/registry/AI/presentation không nằm trong fingerprint; AI đã có input trong replay. Đổi comment/tests/refactor trong các file được fingerprint cũng có thể làm replay bị từ chối; không hứa semantic compatibility hoặc cross-build migration. Dùng build gốc nếu mismatch; chưa có loader lịch sử. Native/WASM equivalence phải kiểm bằng harness, hash contract không tự chứng minh equivalence.

| Định dạng | Hành vi |
| --- | --- |
| v3 legacy | Giữ header/encoding/từng byte. Không có pins trên wire; chỉ chấp nhận frozen v3 luật và ba revision cũ trong `legacy_replay_request`. Không tự chạy kit version mới bằng ID cũ |
| v4 loadout | Thêm `rules <version> <hash>` và một `kit <id> <version> <hash>` cho mỗi kit dùng trong spawns. Fighter/input/check/end giữ encoding cũ |
| v1/v2/unknown | Vẫn bị từ chối rõ; cần build gốc |

`Replay::parse` validate registry và contract trước khi trả replay; `Replay::world()` nay trả `Result`, `play()` kiểm lại cả replay dựng/sửa trực tiếp bằng API. v4 thiếu/duplicate/extra pins, unknown ID, version/hash/rules mismatch hoặc spawn chứa spec khác registry đều bị từ chối. Session giữ các revision đã validate và xuất chỉ pins của spawns thực tế; actor registered được thêm trước tick đầu cũng được pin; seed/input AI vẫn ghi như cũ. Không có fallback về revision đang chạy hoặc nâng v3 lên v4 âm thầm.

Regression legacy khóa 179.328 byte, 3.763 tick, 63 checkpoint, hash cuối `227d609f34dcd9e4` và SHA-256 `b9a13a910849b28879d7d92ac4068ac1b6c8239234c7950837889da7d76112dd`. Selection v4 so từng tick và replay với v3 khi bỏ phần contract, cùng seeds 1/7/12 ở boss/duel. [Báo cáo PR2](../reports/kit-registry-loadout.md) ghi gate/giới hạn thực tế; [bàn giao PR3](../work-plan/070-lineage-presentation.md) nhận API metadata/pins mà không sửa legacy labels.
