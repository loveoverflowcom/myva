# Cấu trúc repository MyVa

**Trạng thái:** structural refactor, ngày 2026-10-11. Đây là SSOT về vị trí mã nguồn; [architecture](architecture.md) và [gameplay foundation](gameplay-foundation.md) giữ SSOT về ownership và luật. Kết quả kiểm chứng của lần di chuyển nằm trong [báo cáo](../reports/repository-layout.md).

## Cơ sở và quyết định trước khi di chuyển

MyVa base cuối: `develop` tại `71e23aff17066c909dfa3ca7b66d181d4ab70524`, đã fetch origin sau merge PR21 lúc 08:19 UTC. Checkpoint ban đầu là `c96a7c4f6f4beaad826cd2c0c7907af588cab992` sau PR17; trước bàn giao đã tích hợp fast-forward base mới và bổ sung mapping cho asset pipeline thật. Checkout người dùng sạch trên `feat/bevy-asset-streaming-performance`; refactor dùng bản Git độc lập để bảo vệ checkout đó. Cả hai base không có `AGENTS.md` hoặc `.agents/skills`; runtime không có `memory_summary.md`.

Cantos tham khảo: `develop` tại `695ecbd062e7f00f7db6b3fe90673fe3ba483087`, đã fetch riêng; PR27 đã merge vào develop lúc 08:02 UTC. Không lấy nhánh PR chưa merge làm SSOT. Tham khảo cách tách entry platform khỏi package dùng chung và giữ tài liệu/công cụ ở root. Không sao chép topology audio, publication, storage hoặc service của Cantos sang game.

Bảng này được chốt trước khi sửa mã nguồn:

| Current path | Proposed path | Reason | Impact |
| --- | --- | --- | --- |
| `crates/web-shell` | `apps/web/shell` | Entry Leptos CSR sở hữu DOM, navigation, loading/error và IME | Workspace member đổi; crate name và module `/web/shell-bridge.js` giữ nguyên |
| `crates/web-game` | `apps/web/game` | Entry Bevy WASM sở hữu game canvas/runtime/input | Workspace member và hai relative dependency paths đổi; bundle/bridge API giữ nguyên |
| `web` | `apps/web/public` | Static shell/game documents, CSS và bootstrap | Build copy source đổi; serving URLs và `target/web-shell` giữ nguyên |
| `crates/sim` | `packages/sim` | Luật miền thuần Rust, protocol/session/replay dùng chung | Member/source path đổi; tên, target và format/hash giữ nguyên |
| `crates/gameplay` | `packages/gameplay` | Adapter ECS headless dùng chung, không renderer | Member/source path đổi; dependency `../sim` vẫn đúng |
| `crates/economy` | `packages/economy` | Ledger/budget và mô phỏng headless thuần Rust | Member/source path đổi; không thêm renderer hoặc quyền client |
| `crates/assets` | `packages/assets` | Schema/hash/validator dùng chung đã có trong PR21, không phụ thuộc Bevy | Member/source path đổi; relative fixture paths giữ đúng cùng độ sâu |
| `crates/graybox` | `packages/graybox` | Client library/input/presentation dùng cho web và desktop | Member/source path đổi; dependencies giữa packages vẫn đúng |
| `assets/fixture` | Giữ nguyên | Fixture PNG/metadata/manifest/provenance thật từ D05-A | Root source và output `game/assets/fixture` giữ nguyên, không tạo content giả |
| `prototypes/native` | Giữ nguyên | Probe Bevy/Winit và CMP vẫn chưa qua gate Android/iOS thật | Nested workspaces và Gradle relative paths giữ nguyên |
| `docs`, `benchmarks`, `scripts`, `tests` | Giữ nguyên | Tài liệu, budget config, automation và cross-package/browser tests | Links tới source đổi; script/output/test paths còn hợp lệ |

### Cây trước

```text
myva/
├── Cargo.toml, Cargo.lock, rust-toolchain.toml
├── crates/
│   ├── sim/          # lib, myva-replay, fixture, integration tests
│   ├── gameplay/     # lib, myva-headless, adapter tests
│   ├── economy/      # lib, myva-econ-sim
│   ├── assets/       # schema/validator, myva-assets, validator tests
│   ├── graybox/      # shared lib, desktop bin, parity tests
│   ├── web-shell/    # Leptos lib + web/shell-bridge.js
│   └── web-game/     # WASM entry
├── web/             # shell static files + game/ static files
├── assets/fixture/  # real D05-A generated fixture and provenance
├── prototypes/native/  # Rust probes + Android/CMP Gradle projects
├── docs/
├── benchmarks/
├── scripts/
├── tests/           # browser E2E + Python report tests
└── .github/workflows/ci.yml
```

### Cây sau

```text
myva/
├── Cargo.toml, Cargo.lock, rust-toolchain.toml
├── apps/web/
│   ├── shell/       # myva-web-shell, web/shell-bridge.js
│   ├── game/        # myva-web-game WASM entry
│   └── public/      # shell static files + game/ static files
├── packages/
│   ├── sim/         # myva-sim, myva-replay, fixture, integration tests
│   ├── gameplay/    # myva-gameplay, myva-headless, adapter tests
│   ├── economy/     # myva-economy, myva-econ-sim
│   ├── assets/      # myva-assets schema/validator and CLI
│   └── graybox/     # myva-graybox shared lib, desktop bin, parity tests
├── assets/fixture/
├── prototypes/native/
├── docs/
├── benchmarks/
├── scripts/
├── tests/
└── .github/workflows/ci.yml
```

## Ownership và dependency boundaries

| Location | Owner / dependency direction |
| --- | --- |
| `packages/sim` | Combat/damage/tick/RNG/AI/session/protocol/replay là luật duy nhất. Không Bevy, UI, renderer, DB. |
| `packages/gameplay` | Chỉ adapter `bevy_app`/`bevy_ecs`/`bevy_time` và `sim`; gọi `Session::step`, mirror domain state, giữ fixed schedule. ECS không sở hữu combat rules. |
| `packages/economy` | Ledger, issuance budget, economy invariants và headless runner. Không renderer; server online tương lai mới có quyền xác nhận reward/economy. |
| `packages/assets` | Schema manifest/atlas/map, hash và validator không Bevy; feature `fs` chỉ dành đọc file/PNG ở tool native. Graybox dùng chung schema/luật với `default-features = false`. |
| `packages/graybox` | Phụ thuộc `assets`/`gameplay`/`sim` và Bevy render/input; scene/HUD đọc mirror, art loader dùng `AssetServer`, match orchestration dùng luật core. Không mint reward, không sửa ledger authoritative. |
| `apps/web/game` | Bevy canvas và browser game lifecycle/input; phụ thuộc `gameplay`/`graybox`. |
| `apps/web/shell` | Leptos DOM/navigation/IME và shell lifecycle; giao tiếp qua bridge, không import combat rules. |
| `apps/web/public` | Documents, CSS, bootstrap/runtime JS phục vụ cùng origin. Không chứa domain state SSOT. |
| `prototypes/native` | CMP native shell và Bevy/Winit feasibility probes; Android/iOS có gate riêng, chưa production. |
| `xtask` | Repository orchestration/CLI/process ownership; không chứa luật gameplay. Xem [build contract](../architecture/xtask-build-contract.md). |
| `assets/fixture` | Nội dung placeholder tái lập được, metadata/manifest/provenance của D05-A; không là chứng nhận art production hoặc performance D05-B. |

Packages có thể sở hữu CLI nhỏ dùng để kiểm chứng chính library đó: `myva-replay`, `myva-headless`, `myva-econ-sim`, `myva-assets` và desktop graybox. Giữ các target này cùng crate tránh thêm wrapper crate không có consumer mới. Khi desktop/native production có lifecycle và packaging thật, entry đó có thể chuyển sang app trong PR riêng.

Không tạo `contracts/`: domain types đã ở `sim::{protocol,session,replay}`; bridge v2 nằm trong JS/Rust entry hiện hữu. Chỉ tách contract khi có consumer/ABI cụ thể và một SSOT rõ, không sao chép types. Root `assets/fixture` đã có nội dung thật trên base mới nên giữ; static web thuộc `public`, evidence thuộc reports và budget config thuộc benchmarks. Không tạo app/server/mobile production hoặc folder rỗng để giống Cantos.

`graybox` hiện gộp reusable presentation và executable desktop; `fight` gộp match orchestration/replay với HUD state. Đây là scope prototype hiện hữu, không đổi systems, events hoặc authority trong refactor đường dẫn. Streaming/telemetry giữ nguyên behavior; tối ưu hoặc tách tiếp phải có bằng chứng và PR riêng.

## Consumer và output compatibility

- Workspace sau refactor dùng bảy member; PR unified xtask thêm member thứ tám `xtask` và dependencies tooling vào lockfile. Package/game target names, versions/features và domain logic giữ nguyên.
- Dependencies trong `packages` vẫn dùng sibling paths. `apps/web/game` dùng `../../../packages/gameplay` và `../../../packages/graybox`.
- `scripts/build-web-shell.sh` copy từ `apps/web/public`; output vẫn là `target/web-shell/{index.html,style.css,bootstrap.js,shell/,game/}`. WASM vẫn là `myva_web_shell.wasm` và `myva_web_game.wasm`; generated imports vẫn là `shell/myva_web_shell.js` và `game/pkg/myva_web_game.js`.
- `#[wasm_bindgen(module = "/web/shell-bridge.js")]` là file relative với crate shell, giữ tại `apps/web/shell/web/shell-bridge.js`. Generated wrapper tham chiếu `snippets/`; không hard-code opaque folder hash hoặc dùng hash đó thay cho kiểm chứng bundle.
- URL `/game/`, bridge version/session/origin/source checks và replay bytes không đổi.
- CI provision tools rồi gọi `cargo xtask ci`, planner chọn package bằng name và reuse root scripts; Playwright giữ `tests/web`, root npm manifest và `target/web-evidence`. Web server/readiness/cleanup thuộc xtask, upload artifacts vẫn thuộc Actions.
- Native workspaces không phụ thuộc moved packages. Probe-root anchors, CMP `../app/src/main/jniLibs`, Android settings và Gradle paths còn nguyên.
- D05-A asset paths giữ nguyên: validator test `../../assets/fixture`, graybox default `CARGO_MANIFEST_DIR/../../assets`, art test `../../../assets/fixture/long_luu/sheet.json`. `crates` → `packages` không đổi độ sâu. Build vẫn validate root manifest rồi copy sang `target/web-shell/game/assets/fixture`; URLs, hash/provenance và loading/fallback/streaming behavior giữ nguyên.
- Crate-local integration tests/examples đi cùng package; browser/report tests ở root `tests`. Xóa cây cũ bằng `git mv` sau khi cập nhật consumer và kiểm tra không còn active path.

Raw evidence JSON, commands và paths trong báo cáo có revision lịch sử giữ nguyên. Mapping trên giải thích vị trí hiện tại; không chuyển kết quả PASS của base thành runtime evidence cho refactor.

Các đề xuất tách scope nằm trong [work-plan backlog](../work-plan/backlog/repository-boundaries.md), không thay gate gameplay/platform hoặc trạng thái issue.
