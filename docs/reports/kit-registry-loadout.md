# PR2 — Data-driven Kit Registry và Loadout

**Ngày:** 2026-10-12. **Host:** Mac mini, aarch64-apple-darwin; Rust/Cargo 1.97.1, Bevy 0.20.0. **Base/head dependency:** PR27 `2f0a71269e9b428ba488b39c1fe4c67532adbe3e`; `origin/develop` xác minh là `e4bdba46a5750ab623c6fac1be120e5835fc093a`. PR27 vẫn draft/open/unmerged tại lúc bắt đầu. **Branch:** `feat/kit-registry-loadout`; worktree `/Users/manhblue/Documents/Codex/2026-10-12/task-2/myva-pr2`. Draft stacked base `feat/faction-lineage-domain`, đích tích hợp cuối `develop`, không tự merge PR1.

Không có AGENTS.md/.agents/skills trong repo/ancestors; đã đọc CONTRIBUTING, README, [xtask contract](../architecture/xtask-build-contract.md), [báo cáo PR1](faction-lineage-domain.md) và work-plan. Dùng Architecture-Aware Work Planner để duy trì queue có sẵn; review theo skill hệ thống review-agent.

## Kết quả

[Contract/API](../technical/kit-registry-loadout.md) có registry validated, quyền trang bị riêng metadata lore, loadout chọn ID/version/hash và khóa trong round. Default Long Lưu là unresolved legacy fixture; boss/quái ReplayOnly. Battle/default/rematch/CLI dùng registry, lỗi propagate; session loadout mới ghi replay v4 pins và kiểm trước parse/play. v3 legacy vẫn frozen/khớp từng byte. Không thêm dependency, kit cân bằng, lore binding, UI catalog hoặc asset selection.

Không sửa Kit definitions, World/hash encoding, input/60 Hz/damage/timing/hitbox/guard/Mạch hoặc economy. Graybox/native/WASM host kiểm startup error trước khi chạy hệ phụ thuộc; các thay đổi graybox khác chỉ là test callers theo constructor Result. WASM host source chưa được compile do prerequisite gate. Gameplay adapter thêm test thực thi loadout qua FixedUpdate, không thêm luật.

## Kiểm chứng local

| Gate | Trạng thái | Evidence/giới hạn |
| --- | --- | --- |
| `cargo xtask verify sim` trước tests mới | PASS | Replay boss giữ 3.763 tick/63 checkpoints/hash cuối `227d609f34dcd9e4` |
| Test authoring / repository lần đầu | FAIL, đã sửa | Borrow conflict trong catalog fixture tests; constructor/schema core đã compile. Các stage phía sau lần lỗi là NOT_RUN |
| `cargo xtask verify gameplay` sau sửa | PASS | Sim/registry/loadout + adapter tests; sáu fixture normal/Mạch seeds 1/7/12 × 3.600 tick và mọi replay verify |
| `cargo xtask verify repository` trước review | PASS | 246 Rust tests, native clippy -D warnings, 496 links/anchors và toàn bộ native contracts |
| Repository dưới sandbox | FAIL/BLOCKED, đã rerun | 5 xtask lifecycle/socket tests bị Operation not permitted; rerun ngoài sandbox được duyệt PASS |
| Repository sau review | PASS | 249 Rust tests (0 failed/ignored), clippy -D warnings, docs và toàn bộ native contracts. Lần trước FAIL do test module ordering lint, đã chuyển module về cuối file |
| CLI ID/version/hash/v4 + invalid selection | PASS | Demo-loadout ghi v4, verify 3.763 tick/63 checkpoints/hash cuối legacy; version 99 trả nonzero và không tạo file |
| Legacy byte comparison | PASS | `cmp` với `/tmp/myva-pr1-baseline.myva-replay` exit 0; SHA-256 `b9a13a910849b28879d7d92ac4068ac1b6c8239234c7950837889da7d76112dd` |
| `cargo xtask doctor` | BLOCKED | Thiếu wasm-bindgen 0.2.129 và local Playwright/browser; Rust/WASM target/Node/Python/Bash sẵn có |
| `preflight --changed-only --base origin/feat/faction-lineage-domain --dry-run` | NOT_RUN | Full plan được chọn; dry-run không là gate PASS |
| Full changed-only preflight trước review fixes | BLOCKED | Mọi native stage PASS (246 Rust tests); dừng vì thiếu wasm-bindgen 0.2.129. Các stage WASM/browser phía sau NOT_RUN; không tự provision |
| WASM/browser/native rendered UI/device/human/online | NOT_RUN | Không chứng minh platform/online/entitlement/balance bằng headless |
| GitHub CI | NOT_RUN | Không chạy, đọc hoặc chờ CI theo yêu cầu |

Fourteen registry/loadout regression tests kiểm invalid IDs, registry/schema, permission/equipment, exact approved branch, proposed data thiếu implementation, metadata independence, toàn kit fingerprint, per-tick legacy/v4 equivalence, rematch, version/hash/rules và contract tampering. ECS test dùng boss/duel × seeds 1/7/12 × 600 tick, so toàn TickReport và replay. Startup failure regression inject lỗi registry và kiểm host gate/AppExit mà không chạy hệ thiếu resource; 18 specs có hash đúng nhưng vượt miền số học bị từ chối; actor thêm trước tick0 có pin ở replay zero/one-tick. Golden PR1 vẫn kiểm mọi v3 byte. Synthetic branch data không là binding lore hay roster được duyệt.

Logs local: `/tmp/myva-pr2-{sim,gameplay,repository,doctor,plan,preflight,cli,review-fix-repository}.log`; failures trước sửa trong `/tmp/myva-pr2-{sim,repository}-before-test-fix.log`. Lần lint fail sau review trong `/tmp/myva-pr2-review-fix-before-lint.log`; sandbox failures trong `/tmp/myva-pr2-repository-sandbox.log`. Temporary hash-capture test phục vụ authoring pins đã được loại; nó cố ý fail để in fingerprint, không được tính là PASS. Cache PR1 copy bằng APFS clone vào target PR2; gates thực thi lại binary/tests ở worktree này.

## Review và publication

Reviewer độc lập theo review-agent đã đọc complete diff và call sites, reproduce ba P2: host systems chạy sau startup rejection, specs có hash đúng nhưng gây arithmetic panic, và Session pre-tick spawn thiếu pin. Đã sửa host gate, numeric/arena validator và pins theo spawns, bổ sung regressions. Reviewer kiểm lại complete diff/bản sửa/call sites: **No findings**. `cargo xtask verify repository` retest exit 0 sau sửa. WASM/browser host startup source chỉ được review, chưa compile/runtime; không gọi native regression là browser PASS. Normal Git push dry-run trên Mac bị GitHub 403 với credential `loveoverflowdev`, không đổi credential. GitHub connector xác nhận `loveoverflowcom`; publication dùng Git-data API tạo tree/commit/nhánh mới từ PR1 và draft stacked, tắt maintainer edit. Tree local đã kiểm thử phải bằng tree remote trước tạo nhánh; fetch/read-back commit/PR để xác nhận parent/head/base. Local branch chỉ fast-forward từ PR1 tới commit xuất, không thay published history. SHA/URL read-back thực tế được ghi trong kết quả/bàn giao phiên2; không merge/force/rebase hoặc đóng issue.

## Giới hạn và tiếp theo

Equipment/procs/budgets/vũ khí/alternate ultimate slots, nội tại mới, server transport/entitlement và competitive ruleset vẫn TODO/chưa hỗ trợ. Nonempty equipment trả UnsupportedEquipment, branch lore mặc định trả BranchMismatch, unknown/mismatched revision/hash trả lỗi explicit. Source fingerprint bảo thủ có thể reject replay khi chỉ đổi comment/test trong file luật; v3 không có pins trên wire, chỉ frozen legacy configuration được chấp nhận. Không có replay migration/historical loader hay parser kit stats từ JSON.

[PR3 CharacterCatalog & AssetMapping](../work-plan/070-lineage-presentation.md) đọc registry/catalog/loadout pins và binding unresolved, giữ Kit labels/replay encoding. Không tự mở phiên3, không đóng #19/#20. Giữ PR1/PR2 worktrees và history để phiên sau kiểm base remote thực tế.
