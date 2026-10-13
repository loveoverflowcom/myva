# Equipment Registry & Headless Build Rules — evidence

**Ngày:** 2026-10-12. **Host:** dot cloud computer, Linux; làm trong isolated worktree, không dùng Mac. **Fresh fetched base:** develop `98e7ddc2c7fe47831907e76d0ac022b41373b030`, sau merge PR27–29. **Branch:** `feat/equipment-registry-headless`. Existing source/uncommitted work của các task khác giữ nguyên; checkout mới từ develop rồi worktree riêng. Không có AGENTS.md hoặc `.agents/skills` trong MyVa; đã đọc CONTRIBUTING, README, architecture, kit/replay và xtask build contract.

## Deliverable và giới hạn

[Contract](../technical/equipment-registry.md) ghi ba ứng viên đầu của mục 5 #20: Ấn Phản Chấn, Giáp Trấn Mạch, Phù Vũ Bộ. Immutable registry, item/build revision pins, typed errors, ngân sách/stacking, LockedLoadout integration và CLI headless `build-check`. Pure prototypes tính dash cost, guard pressure và bounded guard recovery state-in/state-out. Combat từ chối nonempty build trước spawn vì runtime/proc/replay state chưa implement; không nhận đồ rồi im lặng bỏ qua hiệu ứng.

Không đổi kit data, 60 Hz, core damage/timing/resource laws, World/state hash, session, replay v3/v4 parser/encoding, dependencies hoặc faction/character/assets. Không shop, inventory, economy, online PvP hoặc balance/entitlement claims. Không đóng toàn issue #20.

## Verification thực tế

| Check | Status | Evidence / giới hạn |
| --- | --- | --- |
| `cargo xtask verify repository` | BLOCKED; stages NOT_RUN | Exit 127, `cargo: command not found`; orchestrator chưa khởi chạy |
| `cargo xtask verify sim` | BLOCKED; tests/CLI NOT_RUN | Exit 127, cùng thiếu Cargo |
| `cargo xtask verify gameplay` | BLOCKED; ECS tests NOT_RUN | Exit 127, cùng thiếu Cargo |
| `cargo xtask fmt --check` | BLOCKED; format validation NOT_RUN | Exit 127; rustfmt cũng không có; chỉ manual readability cleanup |
| Rust compile/clippy/property/integration tests | NOT_RUN | Cargo/rustc/rustfmt không có trong PATH hoặc existing toolchain locations đã kiểm; không tải toolchain theo yêu cầu |
| Docs primitive fallback | PASS | Checker Python exit 0, 528 local links/anchors trong 67 Markdown files; không là repository PASS |
| Source invariance + Python source fingerprint | PASS, static only | Mười playback-law files, replay/session và Cargo manifest/lock khớp develop từng byte; independent hash `93755e498de17ca3` |
| Canonical pin authoring | PASS, static only | Independent Python FNV authoring calculation; rules `9dc0378661ca5573`, items `67bf57e86c58af0e` / `8683fcf7fed50f68` / `782d9b33f5a4124b`; không thay thế Rust hash tests |
| `git diff --check` | PASS | Whitespace sanity, không là rustfmt/compile |
| Rust CLI execution, v3/v4 playback/golden/parity | NOT_RUN | Existing regression suite và tests mới còn cần host có toolchain |
| Browser/WASM/renderer/mobile/human/online | NOT_RUN | Ngoài bằng chứng data/pure prototype; không tự provision |
| GitHub CI | NOT_RUN | Không inspect hoặc chờ CI theo yêu cầu |

Initial docs run FAIL vì evidence report chưa được viết; đã sửa và rerun checker exit 0 sau khi có file. Final checks được rerun sau review updates. Không tính lỗi authoring này thành completed gate. Logs của attempted xtask checks nằm trong thư mục evidence riêng của task, không commit cache/binary hoặc log máy vào source.

## Tests đã author, chưa chạy

Mười sáu tests tại `packages/sim/tests/equipment_registry.rs` đã author, chưa compile/run. Suite kiểm frozen hashes/metadata independence; runtime IDs/registry and input permutations; duplicate/slot/budget errors; invalid authored registry/domain bounds; group caps/additive stacking/slowest cooldown; exact item/rules revalidation; usable headless loadout và explicit runtime rejection; integer boundary properties; bounded recovery/cap/source-action identity/cooldown; proc origin/front/blockable/perfect eligibility; target/build/time rejection; refund-rate/tick overflow; v3/v4 unknown-wire rejection; valid/invalid headless CLI.

Current registry/loadout test được đổi từ unsupported equipment sang unknown-item typed error. Existing v3/v4 golden, per-tick legacy equivalence và adapter tests giữ nguyên. Authoring tests không là PASS hoặc native/WASM balance evidence.

## Review và publication

Independent source reviewer đã kiểm complete implementation, consumer call sites, nguồn playback giữ nguyên, pin hashes và tests; không có confirmed blocking correctness defect trong source pass. Theo góp ý coverage, bổ sung test full-loadout stale item/rules pins, guard reduction additive cap/exclusions và wide budget 3 × u32::MAX. Test readability được chỉnh thủ công; rustfmt vẫn NOT_RUN. Final snapshot được reviewer kiểm lại; source review không thay compile hoặc chạy tests.

Shell push dry-run bị chặn vì không có HTTPS username/credential (`could not read Username ... No such device or address`). Không đọc/chuyển credential hoặc force-push. Checkpoint đã publish bằng GitHub connector: [draft PR30](https://github.com/loveoverflowcom/myva/pull/30), target develop, commit `5b2f43545a9356318be3b1ccf2fd9a927ba5ef54`, tree `09eab84dffab3ced0f196d0350cd12d917fab0b3`, parent exact fresh develop. Fetch/read-back xác nhận remote tree bằng staged local tree trước khi fast-forward local branch. Follow-on test/evidence updates dùng commit nối tiếp và normal fast-forward publication; final SHA/tree được verify read-back trong bàn giao. Không merge/force push, không chờ hoặc inspect CI.

## Follow-up

Provision toolchain theo lựa chọn chủ dự án rồi thực thi gates ở exact final SHA trước khi coi build ready. [Runtime effects/replay contract](../work-plan/backlog/equipment-runtime-effects.md) là phạm vi riêng; phải có explicit new replay version nếu wire/state thay đổi. Không sửa combat hoặc tải tooling để lách blocker của task này.
