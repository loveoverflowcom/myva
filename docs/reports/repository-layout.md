# Báo cáo refactor cấu trúc repository

**Ngày:** 2026-10-11. **Phạm vi:** structural refactor trên `refactor/repository-layout`, base `develop` tại `71e23aff17066c909dfa3ca7b66d181d4ab70524`. [Cây trước/sau, mapping và boundaries](../technical/repository-layout.md) là SSOT của layout. Không có kết quả browser/device/performance mới trong báo cáo này.

## Kết quả và scope

Chuyển bảy crate hiện hữu thành hai web entries trong `apps/web` và năm reusable packages; chuyển web static vào `apps/web/public`. Giữ nguyên `assets/fixture` thật từ D05-A, native feasibility prototype, CLI kiểm chứng, tests, scripts, benchmarks và CI hiện hữu. Không thêm crate, production server/mobile hoặc contract types trùng SSOT.

Khảo sát ban đầu trên `c96a7c4f…` đã ghi mapping trước khi di chuyển. Trong lúc làm, PR21 merge; checkpoint được lưu bằng Git stash, branch fast-forward đến `71e23aff…`, rồi hòa giải đường dẫn với asset pipeline mới. Thêm mapping `crates/assets` → `packages/assets` trước khi chuyển crate này. Giữ đầy đủ upstream art loader, fallback, validator, fixture và asset E2E. Review cuối dùng base mới, không dùng kết quả của base cũ.

Cantos được fetch vào snapshot chỉ đọc: `develop` tại `695ecbd062e7f00f7db6b3fe90673fe3ba483087`, đã merge PR27. Kế thừa phân biệt entries/packages, một workspace/lockfile và dependency hướng vào core. Không kế thừa topology hai sản phẩm, service/audio/publication, skeleton app/mobile, package renames hoặc CI additions của Cantos. Checkout gốc MyVa và Cantos không sửa; không chạm model/audio/evidence/data của Cantos.

## Các file sửa

| Nhóm | Nội dung |
| --- | --- |
| Moves | 74 file từ `crates/{assets,economy,gameplay,graybox,sim,web-game,web-shell}` và `web` theo mapping; ưu tiên `git mv` |
| Manifests | `Cargo.toml`: bảy member tường minh. `apps/web/game/Cargo.toml`: hai relative dependency paths. Những manifest khác và lockfiles giữ Git blob của base |
| Script | `scripts/build-web-shell.sh`: hai copy-source paths từ `apps/web/public`; giữ asset validate/copy của D05-A và mọi output name/path |
| Docs hiện hữu | README, CONTRIBUTING, docs index; architecture, gameplay foundation, asset pipeline; ADR0003; web/combat/asset reports; combat work item và work-plan index |
| Docs mới | Layout SSOT, báo cáo này, [backlog boundaries](../work-plan/backlog/repository-boundaries.md) |
| Consumer giữ nguyên | CI, npm/Playwright, serve/determinism/performance scripts, native Rust/Gradle/probes; package selectors và relative paths đã còn đúng |

Không đổi Rust/Bevy/Leptos/Kotlin versions/features so với base cuối. Rust/JS/CSS/HTML source, integration tests, root fixture/evidence JSON, bridge/session checks, replay/hash implementation, authority/reward/economy và streaming/asset loader giữ nguyên Git blob. Test expectations không sửa.

## Kiểm chứng trên Mac

Mac arm64, macOS 15.6; RAM 24 GiB do user cung cấp. Đĩa free lúc đầu khoảng 4,0 GiB, sau khảo sát khoảng 3,9 GiB. Snapshot `ps` không có Cargo/rustc/Gradle/Playwright build khác. Không tải toolchain/SDK/emulator/model, không dùng Docker hoặc cold Bevy build với ngân sách này.

Toolchain pin `1.97.1` chưa cài; `1.96` đã có và là `rustc 1.96.1`. `wasm-bindgen` không có trong PATH; dependency cache thiếu Bevy. Các lệnh pin dưới đây bị chặn ở rustup trước compile (sandbox không cho tạo temp trong rustup home). Kiểm tra HTTP của manifest pin trả 200 chỉ xác nhận availability, không là installation/build evidence. Dùng `1.96` cho metadata không resolve dependencies và formatter nguồn không thay pin trong repo, không chứng nhận MSRV hoặc compile.

| Command / oracle | Status | Kết quả và giới hạn |
| --- | --- | --- |
| `git fetch origin --prune`, `git rev-parse origin/develop` | PASS | Base cuối `71e23aff…`; initial base `c96a7c4f…` được thay bằng base mới trước review |
| `cargo metadata --locked --format-version 1` | BLOCKED | Pin `1.97.1` chưa cài; rustup không tạo được temp; chưa resolve graph đầy đủ |
| `rustup run 1.96 cargo metadata --locked --offline --format-version 1` | BLOCKED | Cache không có Bevy; không dùng lỗi này như lỗi layout |
| `rustup run 1.96 cargo metadata --locked --no-deps --format-version 1` | PASS | So metadata base export với tree sau, normalize đúng mapping: 7 members, 19 targets, names/features/versions/dependency requirements giữ nguyên; chỉ đường dẫn đổi |
| Hai `cargo metadata --locked --no-deps` qua `rustup run 1.96`, `--manifest-path prototypes/native/Cargo.toml` và `prototypes/native/probes/event-loop/Cargo.toml` | PASS | Native nested workspaces còn độc lập; không chứng minh native build/device |
| Git blob differential theo mapping + kiểm source/static anchors | PASS | Đủ 201 file của base, không mất file; 74 moved files, 73 blob-identical; file moved duy nhất đổi nội dung là web-game manifest (hai paths). Mọi compatibility-critical source/test/fixture/lockfile/native/CI blob giữ nguyên |
| `cargo fmt --all -- --check` | BLOCKED | Pin toolchain chưa cài |
| `rustup run 1.96 cargo fmt --all -- --check` | PASS | Format check của source trên 1.96.1; chưa gọi là pin-format PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | BLOCKED | Pin/dependencies và ngân sách cold Bevy build |
| `cargo test --workspace` | BLOCKED | Cùng blocker; Rust tests chưa chạy trên revision refactor |
| `python3 scripts/check-docs.py` | PASS | 403 local links/anchors trong 46 Markdown files |
| `python3 scripts/check-docs.py docs prototypes/native assets/fixture` | PASS | 371 local links/anchors trong 46 Markdown files; check thêm prototype/fixture |
| `python3 -m unittest discover -s tests -p 'test_*.py' -v` | PASS | 34 asset-performance report contract tests; synthetic unit fixtures, không phải performance measurement |
| `python3 scripts/generate-asset-fixture.py --check` | PASS | 5 PNG tái lập pixel-identical, max channel difference 0; không chứng nhận runtime/performance |
| `cargo run --locked -p myva-assets -- validate assets/fixture/manifest.json` | BLOCKED | Pin/dependency chưa sẵn sàng; validator source/fixture path đã static-check |
| `./scripts/check-wasm-determinism.sh 3600 1 7 12` | BLOCKED | Thiếu wasm-bindgen 0.2.129; không có hash native/WASM mới |
| `./scripts/build-web-shell.sh` | BLOCKED | Thiếu wasm-bindgen; chưa có generated WASM/bundle trên Mac |
| `npm run test:web` | NOT_RUN | Không có bundle đã build hoặc Playwright install trong checkout này; E2E kể cả assets chưa chạy tại refactor |
| `bash -n scripts/build-web-shell.sh scripts/check-wasm-determinism.sh prototypes/native/scripts/build-android.sh` | PASS | Shell syntax; không là build |
| `node --check` cho moved JS, root Playwright và scripts JS | PASS | Syntax/source parsing; không là browser execution |
| `python3 prototypes/native/scripts/preflight.py` | PASS (tool), gate BLOCKED/NOT_RUN | Android không có adb/SDK/physical device; iOS có Xcode 16.3 nhưng device/signing chưa kiểm tra. Không build hoặc chạy native lifecycle |
| Asset runtime/performance/device matrix | NOT_RUN | D05-A evidence cũ giữ revision; D05-B measurements và mobile gates không được đóng bởi rename |
| `git diff --check`, `git diff --cached --check` | PASS | Không whitespace error ở checkpoint; chạy lại trước commit |

Metadata/byte verifier được chạy bằng `python3 ../verify-layout.py` trong workspace riêng; helper chỉ đọc Git và metadata, không là test ứng dụng. Nó so tất cả trường metadata sau khi normalize paths, rồi so Git-cleaned blob của từng file base (tôn trọng `.gitattributes`, gồm CRLF wrapper native). Có thể tái lập riêng oracle 74 moved files từ root repo bằng Python:

```python
import pathlib, subprocess
base = "71e23aff17066c909dfa3ca7b66d181d4ab70524"
moves = {"crates/web-shell": "apps/web/shell", "crates/web-game": "apps/web/game", "web": "apps/web/public"}
moves.update({f"crates/{p}": f"packages/{p}" for p in ("assets", "economy", "gameplay", "graybox", "sim")})
def git(*args): return subprocess.check_output(["git", *args]).decode().strip()
same = total = 0
for old in git("ls-tree", "-r", "--name-only", base).splitlines():
    prefix = next((p for p in moves if old.startswith(p + "/")), None)
    if prefix is None: continue
    new = moves[prefix] + old[len(prefix):]
    assert pathlib.Path(new).is_file() and not pathlib.Path(old).exists()
    identical = git("rev-parse", base + ":" + old) == git("hash-object", "--path=" + new, new)
    assert identical or old == "crates/web-game/Cargo.toml"
    total += 1; same += identical
print(total, same)  # 74 73
```

## Review độc lập

Review read-only dùng BASE cuối, trace source/manifest/static/native/CI consumers và đọc tiêu chí assurance của Cantos engineering/code-review như tham khảo; MyVa architecture/ADR là owner của product rules. Kết luận: **`no-actionable-findings`**, không có regression P0–P3 được xác nhận. Coverage **`complete-for-declared-scope`** cho diff cấu trúc, manifests, moved blobs, consumer paths, docs và evidence claims; không review lại toàn bộ logic upstream D05-A hoặc native lifecycle vốn không đổi. Snapshot reviewer: SHA256 `47a2ca7a75d92cbeaf92a2559dcf5830befad85a004eb2e1ce11e3666d14c176`; sau snapshot chỉ ghi kết quả review vào báo cáo này.

| Claim | Owner | Evidence level / disposition |
| --- | --- | --- |
| Không mất source/member/target | Workspace | 201 base files; 7 libraries, 5 binaries, 1 example, 6 integration targets; `statically-checked`, held |
| Combat/tick/replay/hash và authority/economy | Sim/gameplay/graybox/economy | Source/test blob không đổi; `statically-checked`, held cho source, runtime còn gap |
| Web bundle/bridge/static/CI consumers | Shell/game/public và scripts | Trace paths/names/session/origin/source; `statically-checked`, held cho paths |
| Asset fixture/default/include/copy paths | Assets/graybox | Độ sâu giữ đúng, fixture/source không đổi; `statically-checked`, held cho paths |
| Native nested workspaces/JNI/probes | Native prototype | Git diff rỗng và consumer tracing; `statically-checked`, held cho paths |
| Docs giữ revision và giới hạn evidence | README/layout/report/backlog | Source-backed mapping và labels; `documented`/`statically-checked`, held |
| Full compile/determinism/browser/device | CI/platform gates | `gap-required`; cần checks hiện hữu trên exact PR head |

Reviewer độc lập chạy lại metadata no-deps (7/19), docs links (403/371), 34 report tests, shell syntax và blob/path/diff checks: PASS trong phạm vi các oracle đó. Không có generated bundle/device/performance evidence. Một correction đã xác nhận và sửa: bỏ diễn giải rằng move source tự làm đổi wasm-bindgen snippet hash; wrapper là consumer, source pin dùng package name/version và module identifier.

## Rủi ro và PR khác

Static metadata/blob checks không chứng minh compile, generated bundle, browser lifecycle hoặc native/WASM hash trên revision refactor. Các checks pin/full web hiện có trong CI là đường kiểm chứng tiếp theo; chỉ status theo exact head mới được dùng, không suy PASS từ base hoặc report lịch sử. Không sửa/thêm CI để che missing local evidence. Không tự merge hoặc đóng issue gameplay/platform.

Desktop/mobile production entries, native adapter/lifecycle/iOS runner, transport/ABI contracts, snapshot networking/persistence/ledger, extraction `graybox::fight` và asset performance D05-B thuộc các PR riêng theo [backlog](../work-plan/backlog/repository-boundaries.md) và gates hiện hữu. Cây mới không làm thay quyền server reward/economy.
