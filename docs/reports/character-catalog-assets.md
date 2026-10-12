# CharacterCatalog & Asset Mapping — evidence

**Trạng thái:** offline PR3/3, 2026-10-12, Mac mini / aarch64 macOS, Rust 1.97.1, Bevy 0.20.0, Leptos 0.8.22. Base đã xác minh: PR28 head `97fc27a36a3dfa2733b9871d3ba939aefc1b2b13`; PR27/PR28 draft/open/unmerged, develop `e4bdba46a5750ab623c6fac1be120e5835fc093a`. Không merge hoặc kiểm/chờ GitHub CI.

## Kết quả và bằng chứng

Projection metadata dùng chung ở `sim::character`; Leptos/Bevy chọn danh tính và nhãn từ stable IDs. Runtime tải unit của selected KitPin, map hiện tại và boss/VFX cần thiết. Không thêm combat implementation, crate, raster/canon artwork hoặc roster mới. [Contract](../technical/character-catalog-assets.md) ghi errors và ranh giới.

| Gate | Trạng thái / phạm vi |
| --- | --- |
| `cargo xtask verify repository` | PASS final từ cache package sạch: 261 Rust tests, clippy `-D warnings`, 514 links/anchors, native replay/ECS/economy/asset/report contracts |
| Catalog regression | PASS: lookup invalid/unknown/missing/duplicate/orphan; metadata không có implementation, ReplayOnly/Disabled, pin/version/hash guards |
| Metadata/hash isolation | PASS: thay name/description/icon/skills giữ từng byte replay v4 ở boss/duel × 3.600 tick; legacy golden test giữ v3 |
| Selected asset plan/lifecycle | PASS: chỉ một Character unit; manifest có Character chưa chọn và payload không tồn tại vẫn ready, 4 loads/0 failures; mapping thiếu không phát payload hoặc loop retry; cancel/stale completion không giữ scene owners |
| Existing AssetServer tests | PASS: 30 load/unload cycles; required missing + retry recovery; optional missing VFX fallback; boss→duel retain common/drop boss; rapid cancellation và late optional install |
| WASM consumers | PASS: focused clippy `-D warnings` target wasm32 cho Leptos + Bevy; không là optimized bundle hoặc browser runtime |
| Native rendered screenshot | FAIL: capture PNG được mở inspect nhưng toàn đen; PR2 baseline/art/no-art có SHA-256 giống hệt. HUD/layout visual BLOCKED, nguyên nhân render chưa xác định |
| Full preflight, optimized web bundle, browser E2E | BLOCKED bởi wasm-bindgen CLI 0.2.129 thiếu; Playwright thiếu. Build/E2E bodies/visual responsive/focus/touch là NOT_RUN |
| Android/iOS, người thật, online, GPU memory/performance qualification | NOT_RUN; không suy từ compile/headless/screenshot |

Lượt repository trong sandbox thất bại ở năm test xtask socket/process với `Operation not permitted`. Chạy lại ngoài sandbox qua approval môi trường đã PASS; không bỏ test hoặc đổi TLS/credentials. WASM offline probe thiếu dependency cache; focused build được phép resolve dependencies đã pin trong lockfile, không cài tool mới. Các lỗi consumer cfg WASM phát hiện ở lượt đầu đã sửa; native check một mình không chứng minh đường WASM.

Native `cargo xtask run graybox -- --kit long-luu --screenshot target/xtask-evidence/pr3-native-boss.png` chạy Apple M4/Metal, cửa sổ 1280×720, dev profile. Selected replay v4 951 tick/16 checkpoint verify PASS, hash `05db500123b1b1f5` (log `/tmp/myva-pr3-native-capture-replay.log`). PNG PR3/art, PR3/no-art và baseline PR2 `97fc27a` đều được mở inspect và toàn đen; SHA-256 cả ba `7f0fcfad3151bf17c4ba0faa396039ee753bfd086f998f751361bedcb6a3e4b4`. Baseline được archive vào `/tmp/myva-pr3-render-baseline`, không sửa worktree PR1/PR2. Đây là lỗi capture/render trước PR3; không kết luận UI đã đạt, không sửa engine/camera ngoài scope. Follow-up diagnosis được giữ trong backlog.

Legacy CLI replay `cmp` PASS so với capture PR1/develop, 3.763 tick/63 checkpoint, final hash `227d609f34dcd9e4`, SHA-256 `b9a13a910849b28879d7d92ac4068ac1b6c8239234c7950837889da7d76112dd`. Node VM mocked shell bridge PASS exact pin URL, invalid selection giữ frame, label theo ID, unknown ID explicit, v3/v4 download và teardown; scope chỉ mocked JS, browser/WASM runtime NOT_RUN.

Review độc lập theo `/Users/manhblue/.codex/skills/.system/review-agent/SKILL.md`: **No findings** trên complete diff/call sites; không coi compile/headless là rendered proof. Native HUD và shell status cũng hiển thị lỗi art/fallback tường minh, không đổi kit/identity khi mapping lỗi.

Logs cục bộ: `/tmp/myva-pr3-final-repository-clean.log`, `/tmp/myva-pr3-final-wasm-clippy.log`, `/tmp/myva-pr3-bridge-probe.log`, `/tmp/myva-pr3-final-preflight-escalated.log`; final review/retest và publication được bổ sung vào bàn giao phiên3. CPU lifecycle artifact: `target/xtask-evidence/headless-asset-cycles.json`, không là GPU/RSS/driver evidence. Web spec `character-catalog.spec.mjs` có pinned selection/export v4/re-entry/unknown/mismatch/partial guards; Node syntax không là E2E PASS.

## TODO và cách review/merge

Giữ [backlog nội dung](../work-plan/backlog/character-content-approval.md), equipment/approved binding/historical replay TODO từ PR2 và các gate browser/device/D05-B đang thiếu. Metadata boss/quái unresolved và art/icon placeholder được ghi rõ trong consumer. Không gọi năm nền tảng hoặc mười trực hệ đề xuất là roster chơi được.

Review/merge tuần tự **PR27 → PR28 → PR3**. PR28 base PR27; PR3 base `feat/kit-registry-loadout` để chỉ thấy diff PR3. Người có quyền merge xử lý dependency, retarget/rebase nếu cần và rerun gate trên head tích hợp; đích cuối develop. Chưa có quyền merge, không auto-close issues.
