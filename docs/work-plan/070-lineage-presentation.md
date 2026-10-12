# PR3 — Presentation metadata và asset mapping

**Trạng thái:** TODO, phiên3; phụ thuộc [PR2](060-kit-registry-loadout.md). Chưa triển khai UI catalog hoặc asset selection trong PR1/PR2.

## Kết quả và lý do

Client hiển thị tên/provenance/trạng thái và ánh xạ asset bằng identity ổn định, giúp presentation phát triển mà không sửa combat/replay bytes.

## Phạm vi và acceptance

- Đọc metadata ngoài `Kit`/`World`; giữ legacy labels hashed và replay IDs.
- Hiển thị rõ fixture chưa ánh xạ và ứng viên chưa duyệt; không giả là roster chính thức, không khóa theo quê hương/bản đồ.
- Mapping asset có fallback/lỗi rõ, giữ luật tải theo vùng và provenance/license; không đổi damage/economy/skill rules.
- Kiểm tra browser/native phù hợp và parity; report phần chưa chạy, review độc lập rồi normal push/draft PR. Không merge hoặc tự triển khai phiên này từ PR1.

## Rủi ro còn mở

Tên/hình tượng/roster còn chờ quyết định. UI layout và asset contracts phải lấy bằng chứng từ consumer thực tế ở head PR2.

## Bàn giao phiên2 → phiên3

- Worktree: `/Users/manhblue/Documents/Codex/2026-10-12/task-2/myva-pr2`; branch `feat/kit-registry-loadout`. Head được read-back sau publication trong kết quả phiên2; xác minh `git rev-parse HEAD`/remote trước bắt đầu, không lấy local cache làm trạng thái merge.
- Đọc [API/contract registry](../technical/kit-registry-loadout.md) và [evidence/limitations](../reports/kit-registry-loadout.md). PR2 phụ thuộc PR27; nếu chưa merge, PR3 tiếp tục stack từ head PR2/base nhánh PR2 theo yêu cầu người dùng. Đích cuối là develop; không tự merge các PR phụ thuộc.
- `KitRegistry::baseline()` trả Result; `catalog()`/`entries()` cho identity, metadata, approval và implementation/equip permission. Catalog record proposed có thể không có implementation; UI phải phân biệt hai điều này.
- `KitRequest` nhận ID runtime + `KitRevision`; `resolve_loadout` trả `LockedLoadout`/typed error. `Bout::loadout()` trả `KitPin` và binding đã khóa; constructors/rematch trả Result. Chọn trước trận qua `Bout::start_with_loadout` hoặc `Battle::with_loadout`.
- `UnresolvedLegacyFixture` là trạng thái Long Lưu hiện có; hierarchy vẫn trả `UnmappedFixture`. Không suy ra thế lực từ tên, CombatTeam, map hoặc asset. Giữ `Kit.lineage`/action names đang hashed; tên UI/provenance đọc từ catalog.
- Replay explicit loadout là v4 có rules/kit pins; default legacy giữ v3. Không sửa source file luật/hash để đổi presentation. WASM/browser và rendered/device/online evidence chưa chạy; phiên3 phải kiểm prerequisites/gates thực tế.
- Equipment/item/proc/budget/weapon/alternate ultimate slots, canon roster/approved binding và historical replay migration vẫn TODO; PR3 không nhận chúng thành scope UI/asset mapping.

Không tự mở phiên3 từ phiên2.
