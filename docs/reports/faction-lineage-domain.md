# PR1 — Evidence domain thế lực và trực hệ

**Ngày:** 2026-10-12. **Host:** Mac mini, aarch64-apple-darwin; Rust/Cargo 1.97.1; Bevy 0.20.0. **Base:** `e4bdba46a5750ab623c6fac1be120e5835fc093a` (`origin/develop` sau PR26). **Branch:** `feat/faction-lineage-domain`. Phạm vi tại [catalog boundary](../technical/faction-lineage-catalog.md); các tests tự động không là playtest/balance/online evidence.

Không có `AGENTS.md` hoặc `.agents/skills` trong MyVa tại base này. Đã đọc README, CONTRIBUTING, Cargo manifests và [xtask build contract](../architecture/xtask-build-contract.md). Repository-level gates đi qua xtask. Review đọc toàn bộ diff và call sites theo skill `review-agent` hệ thống.

## Baseline và compatibility

`cargo xtask verify sim` trên develop trước chỉnh sửa: PASS. Boss B1 round 1 ghi/verify 3.763 tick, 63 checkpoint; hash cuối `227d609f34dcd9e4`. Replay SHA-256 `b9a13a910849b28879d7d92ac4068ac1b6c8239234c7950837889da7d76112dd`; 179.328 byte. Bản baseline lưu local `/tmp/myva-pr1-baseline.myva-replay`, không nhập file replay lớn vào Git.

Regression mới PASS, khóa digest mọi replay byte và hash cuối, thay label/provenance catalog rồi so từng byte; không thay protocol/hash implementation. `cmp` replay mới với baseline trả exit 0; SHA-256 giữ nguyên. Test validator/identity dùng synthetic source/lineage/branch, không nhập mười ứng viên. Adapter test kiểm CombatTeam/None và metadata độc lập.

## Gate lần chạy

| Gate | Trạng thái | Evidence/giới hạn |
| --- | --- | --- |
| `cargo xtask fmt` | PASS | Format toàn workspace |
| `cargo xtask verify repository` | PASS | Native contract cuối: 481 local links/anchors trong 58 Markdown files, clippy/tests/headless/replay/economy/assets/report contracts |
| `cargo xtask verify sim` (catalog validator ban đầu) | PASS | Core/replay và 10 catalog tests; regression mới PASS trong preflight |
| `cargo xtask verify gameplay` lần đầu | FAIL | Sandbox DNS chặn download crates; đã chuyển preflight sang execution có network được duyệt, không sửa dependency pins |
| `cargo xtask preflight --changed-only --dry-run` | NOT_RUN | Plan đầy đủ được chọn; dry-run không phải test |
| `cargo xtask preflight --changed-only` | BLOCKED | Native stages PASS; dừng ở web prerequisite thiếu wasm-bindgen. Các stage sau là NOT_RUN |
| `cargo xtask doctor` | BLOCKED | Thiếu wasm-bindgen 0.2.129, local Playwright và browser; Rust/format/clippy/wasm target/Node/Python/Bash đã có |
| Native clippy + workspace Rust tests | PASS | 233 tests, 0 failed/ignored; gồm 10 validator/identity, 1 golden compatibility, 15 adapter và 5 graybox parity tests |
| ECS/core per-tick + replay | PASS | Seeds 1/7/12 × 3.600 tick × fixture thường/Mạch; mỗi replay verify PASS |
| Economy/assets/report contracts | PASS | Native invariant/generator/projection/manifest budgets; 38 Python contract tests là synthetic, không benchmark |
| Browser/E2E, native/WASM hash comparison và WASM compilation | NOT_RUN | Stage phía sau BLOCKED; doctor cũng xác nhận thiếu local Playwright/browser. Không tự cài bộ browser/wasm-bindgen |
| Native rendered UI, người thật, điện thoại, online | NOT_RUN | PR1 chỉ metadata + headless tests, không thay UI/combat |
| GitHub CI | NOT_RUN | Không kiểm tra hoặc chờ CI theo yêu cầu |

Logs local: `/tmp/myva-pr1-baseline-sim.log`, `/tmp/myva-pr1-sim.log`, `/tmp/myva-pr1-gameplay.log`, `/tmp/myva-pr1-plan.log`, `/tmp/myva-pr1-preflight.log`, `/tmp/myva-pr1-final-repository.log`, `/tmp/myva-pr1-review-fix-repository.log`. Build cache khoảng 7 GiB, còn khoảng 40 GiB free tại lần chạy; không thêm build profile/cache ngoài workspace. Full native build phát một warning linker unwind table khi link graybox với debug symbols; native clippy/tests vẫn exit 0. Generated local evidence trong `target/xtask-evidence` là output của gate, không lấy evidence revision cũ làm PASS cho PR1.

## Review độc lập

Reviewer fresh dùng skill hệ thống `review-agent` đọc toàn bộ staged diff và call sites: một finding P2, `TeachingTradition` bị allow-list cho phép huyết thống. Đã xác nhận và sửa guard; regression bổ sung role pháp hệ thuộc Buddhism. Không có finding khác về Team, replay/hash, roster hoặc scope. Sau sửa, `cargo xtask verify repository` exit 0: 233 Rust tests, clippy, 481 links/anchors, golden replay và sáu lượt ECS/core parity cùng toàn bộ native contract PASS. Reviewer kiểm bản sửa và xác nhận **No findings**. WASM/browser limits giữ nguyên.

## Giới hạn và tiếp theo

PR1 không có kit registry/loadout/version, metadata UI hoặc mapping asset. Sources/lineages/branches mặc định rỗng; Long Lưu trả `UnmappedFixture` khi yêu cầu hierarchy, unknown kit trả `UnknownKit`, catalog sai trả `InvalidCatalog`; không có runtime panic/fallback cho boundary chưa hỗ trợ.

Giữ worktree/nhánh PR1 để [PR2](../work-plan/060-kit-registry-loadout.md) dựa trên head này mà không merge trái phép. Roster, tên, mức hiển thị nguồn thật và ánh xạ lore vẫn chờ quyết định. Không đóng #20, không tạo phiên2 tự động.

## Files thay đổi

17 files: `packages/sim/src/{catalog,kit,lib}.rs`, `packages/sim/tests/{catalog,catalog_compatibility}.rs`; `packages/gameplay/src/{components,mirror,view}.rs`, `packages/gameplay/tests/adapter.rs`; `docs/README.md`, `docs/technical/{faction-lineage-catalog,gameplay-foundation}.md`, báo cáo này, `docs/work-plan/README.md` và ba work items `050-faction-lineage-domain.md`, `060-kit-registry-loadout.md`, `070-lineage-presentation.md`. Cargo manifests/lockfile, combat engine, economy, asset và UI không thay đổi.

## Publication boundary

Normal Git push trên Mac: **BLOCKED** — HTTPS credential `loveoverflowdev` bị GitHub 403; credential `loveoverflowcom` chưa có; SSH không có key được chấp nhận. GitHub connector đã xác nhận tài khoản `loveoverflowcom`; publication dùng GitHub Git-data API tạo tree/commit/nhánh mới, không force update hoặc merge. Tree remote phải trùng tree local đã kiểm thử; commit local trước xuất được giữ trên backup branch `local/faction-lineage-domain-before-publication`. Worktree PR1 theo remote head sau đối chiếu để phiên2 có cùng SHA. Maintainer edit permission tắt khi tạo draft PR.
