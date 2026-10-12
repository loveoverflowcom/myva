# CharacterCatalog và asset mapping

**Trạng thái:** PR3/3 cho [#20](https://github.com/loveoverflowcom/myva/issues/20) và [#13](https://github.com/loveoverflowcom/myva/issues/13), offline fixture, 2026-10-12. Phụ thuộc [registry/loadout PR2](kit-registry-loadout.md); không duyệt roster hoặc mỹ thuật canon.

## Metadata chung

`sim::character::CharacterCatalog` là projection nhỏ của `KitRegistry::catalog()` và `CharacterDetails`. Tên, review/provenance và quan hệ truyền thừa lấy từ catalog PR1; description, icon placeholder dạng chữ và ba nhãn thuật có stable KitId. Lookup nhận runtime string, không dùng index/tên hiển thị. Không có Team, map, nơi sinh hay tham số chiến đấu trong details. Dữ liệu nằm ngoài `Kit`, `World`, protocol, rules/config hash và replay encoding.

`lookup()` trả `CharacterView` với `Inheritance::Mapped(KitHierarchy)` hoặc `UnmappedFixture`, cùng implementation riêng. Invalid/unknown ID, details thiếu/trùng/orphan/rỗng là lỗi explicit. Record metadata có thể tồn tại khi chưa có implementation; nó không cấp quyền chơi. `select(KitRequest)` kiểm exact version/hash, permission và approval/binding qua registry trước khi tạo `LockedLoadout`. Fixture được ghi rõ là offline khi client đưa ra lựa chọn; đây không là competitive entitlement.

Baseline vẫn có một lựa chọn **Long Lưu**, ID `long-luu`, v1 `a624f529d63112f0`, fixture chưa gán thế lực/nguồn/trực hệ/nhánh. Không nhập mười ứng viên vào roster. Boss/quái giữ ReplayOnly; boss HUD hiện `ke-giu-dap [metadata unresolved]`, không tự tạo hồ sơ lore cho actor thiếu catalog.

Leptos dùng các component nhỏ `CharacterCard` và projection `character_choices()`, tái dùng palette/layout hiện có, label/select native, focus ring, provenance disclosure và wrap chuỗi dài. Lựa chọn không còn ở trang vào trận sau khi chơi; muốn đổi kit phải rời iframe. Shell gửi ID/version/hash qua URL, không gửi gameplay spec. Bevy native và WASM dùng cùng projection, HUD đọc tên/icon/nhãn thuật và truyền thừa ngoài combat. Native hỗ trợ `--kit long-luu`; không có `--kit` giữ legacy v3. Shell lựa chọn tường minh ghi v4, rematch giữ pin; export bridge nhận v3/v4. Hash đi qua JS dạng 16 ký tự hex, không mất precision u64. Re-entry cùng iframe và invalid/partial selection bị từ chối, không chọn mặc định âm thầm.

## Đơn vị tải theo kit đang chọn

`graybox::mapping::KIT_ASSETS` nối exact kit ID sang bundle và sheet ID. Mapping có review riêng: `PlaceholderFixture`, `Approved { decision }`, `Unapproved`. Placeholder D05-A có provenance/license trong manifest; không là hình tượng canon. Unknown/duplicate/unapproved mapping, missing bundle/sheet, kind sai, dependency chưa hỗ trợ hoặc sheet subject/role sai trả lỗi. Không tải kit khác để che lỗi.

| Consumer | Required | Optional |
| --- | --- | --- |
| Long Lưu đánh boss | `character.long_luu`, `map.dap_co`, `boss.ke_giu_dap` | `vfx.combat` |
| Long Lưu đấu tập | `character.long_luu`, `map.dap_co` | `vfx.combat` |

Hai actor duel dùng cùng unit Long Lưu. Map hiện tại vẫn là Đập Cổ của graybox, độc lập với FactionId; thêm metadata Character vào manifest không đưa các bundle ấy vào kế hoạch. CatalogLoader chỉ đọc metadata; `scene_keys()` resolve selected KitPin thay cho quét mọi Character/Map/VFX. ArtLoader và projection/hash/provenance/budget validation vẫn chạy trước install. Loaded body sheet phải khớp mapping subject/role; map/VFX không được lén cung cấp body cho kit khác.

Tái dùng StreamingPlanner + AssetServer: tối đa 4 live unit, 2 in-flight, 3 attempts; required missing payload block presentation với lỗi và geometric fallback, optional VFX failure giữ required art. Native HUD và shell status hiển thị lỗi art/fallback bằng mã rõ. G/unload, explicit retry, mode switch và iframe teardown giữ boundary hiện có. Mapping lỗi trước payload I/O hủy quyền sở hữu scene cũ; completion cũ không install. Pending cancellation vẫn giữ strong handle/slot tới terminal state; Bevy không có transport abort công khai. Không tuyên bố GPU/driver release từ CPU ownership tests.

## Phần chưa hỗ trợ

- Approved lore/portrait/icon, metadata boss/quái, chọn source/lineage/branch và roster mới: [backlog](../work-plan/backlog/character-content-approval.md).
- Live content/mapping reload, multi-map selection, dependency bundle graph ngoài self-contained units: lỗi/TODO, không thêm framework.
- Equipment/inventory/server nhân vật/online quyền loadout, economy, timing/damage/hitbox/guard/Mạch và authority: theo contract PR2; PR3 không thay.
- Historical replay migration và cross-revision playback: giữ reject mismatch của PR2.

[Báo cáo kiểm chứng](../reports/character-catalog-assets.md) phân biệt native, WASM compile, browser, rendered/device và TODO.
