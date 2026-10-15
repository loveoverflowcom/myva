# MyVa — Thần Mạch

**MMORPG hành động 2D HD về những người kế thừa sức mạnh thần thoại.**

Người chơi kế thừa sức mạnh theo trực hệ, học cách chiến đấu, khám phá các linh vực và tham gia những chiến dịch nối nhiều vùng. Năm thế lực đã chốt lấy nền tảng từ **Kitô giáo, Hồi giáo, Ấn Độ giáo, Phật giáo và Đạo giáo**; nơi sinh không khóa thế lực, trực hệ hoặc kit. Thế giới có chỗ cho người chơi solo, người cày chay và bang hội; sức mạnh nhân vật phải đi cùng kỹ năng điều khiển.

> Dễ bắt đầu, khó thành thạo; cày chay vẫn mạnh, kỹ năng tạo khác biệt, thế giới mở rộng trong một nền kinh tế có giới hạn và được kiểm chứng.

## Trạng thái dự án

- **Giai đoạn:** thiết kế tiền sản xuất; nền v0.1, đồng bộ worldbuilding #18 ngày 2026-10-12. [ADR 0004](docs/decisions/0004-five-factions-lineage-hierarchy.md) ghi phần đã chốt; tên phe, trực hệ, skill và cân bằng vẫn là đề xuất.
- **Hiện có:** review ý tưởng, GDD, World Bible, luật chiến đấu, kinh tế, kiến trúc đề xuất và kế hoạch kiểm chứng; lõi mô phỏng combat và kinh tế headless, nền ECS Bevy headless dùng chung luật lõi ([D04](docs/technical/gameplay-foundation.md)), graybox chiến đấu Bevy chơi được trên trình duyệt và native desktop (chưa qua playtest người thật), shell Leptos web và prototype native standalone. Phạm vi bằng chứng nằm trong báo cáo từng platform.
- **Pipeline asset (D05-A):** fixture placeholder có provenance cho Long Lưu, Kẻ Giữ Đập, bản đồ Đập Cổ và VFX; manifest có hash/budget, validator và loader `AssetServer` trên native/web ([đặc tả](docs/technical/asset-pipeline.md), [báo cáo](docs/reports/asset-pipeline.md)).
- **Chưa có:** gameplay hoàn chỉnh, server, asset thành phẩm hoặc tích hợp Bevy native vào CMP đã được xác nhận trên Android/iOS thật.
- **Nhánh đầu tiên và mặc định:** `develop`. Các nhánh công việc và PR sau này lấy `develop` làm base.
- **Tên tiếng Anh:** MyVa. **Tên tiếng Việt:** Thần Mạch.

Các con số trong tài liệu là giả thuyết thiết kế cần đo bằng prototype. Không coi chúng là kết quả benchmark hoặc lời hứa vận hành.

## Nền tảng và stack

| Phần | Định hướng |
| --- | --- |
| Gameplay và render | Rust + Bevy `=0.20.0`, quyết định hiện hành [ADR 0003](docs/decisions/0003-bevy-engine-adoption.md) |
| Web | Bevy WebAssembly; Leptos `=0.8.22` CSR quản lý shell; spike dùng hai bundle và game document cùng origin |
| Android, iOS | Kotlin Compose Multiplatform quản lý shell; kiểm chứng Bevy native qua cầu nối và bề mặt render |
| Online | Server authoritative; simulation tách khỏi renderer |
| Persistence | PostgreSQL là đề xuất ban đầu; xác nhận ở prototype persistence |
| Assets | Atlas, tải theo nhu cầu, cache bền vững và ngân sách bộ nhớ; skeletal runtime cần thử nghiệm riêng |

## Mã nguồn

[Cấu trúc repository](docs/technical/repository-layout.md) là nguồn đúng về đường dẫn, ownership và dependency boundaries; [báo cáo refactor](docs/reports/repository-layout.md) ghi mapping, kiểm chứng và giới hạn của lần di chuyển.

```text
apps/web/
├── shell/      # Leptos: DOM, navigation, IME
├── game/       # Bevy WASM: canvas, runtime, input
└── public/     # HTML, CSS, bootstrap và game document
packages/
├── sim/        # luật combat, session, protocol, replay
├── gameplay/   # adapter Bevy ECS headless
├── assets/     # format/validator 2D thuần Rust, CLI
├── economy/    # ledger, budget và mô phỏng kinh tế
└── graybox/    # client library và entry desktop hiện hữu
assets/fixture/     # PNG, atlas/layout/manifest và provenance D05-A
prototypes/native/  # Bevy/Winit và CMP feasibility
docs/               # thiết kế, quyết định, báo cáo và work-plan
benchmarks/         # cấu hình budget; không tự là phép đo
scripts/            # build, serve và kiểm chứng
xtask/              # entrypoint development, orchestration và lifecycle
tests/              # E2E web và report contract tests
```

`apps` chứa entry theo platform; `packages` chứa luật hoặc thư viện dùng chung. Tám member Rust gồm năm package, hai entry web và `xtask`. CLI kiểm chứng và desktop graybox hiện đi cùng package của mình; native vẫn là prototype đến khi có bằng chứng Android/iOS thật. Contract types tiếp tục thuộc SSOT hiện có trong `sim`; việc tách contracts cần consumer/ABI cụ thể. `packages/assets` sở hữu format/validator, root `assets/fixture` giữ nội dung D05-A đã có; D05-B có bounded AssetServer runtime và render-only benchmark harness, còn qualification GPU/performance theo [gate hiện hành](docs/reports/asset-streaming-performance.md).

Workspace Rust dùng toolchain `1.98.1` theo `rust-toolchain.toml`; Bevy `0.20.0` yêu cầu Rust tối thiểu `1.97.1`. Lõi simulation và economy được giữ độc lập renderer. Graybox combat đã chuyển từ Macroquad sang **Bevy** ở [#2](https://github.com/loveoverflowcom/myva/issues/2) và chạy trên nền ECS dùng chung của [#12](https://github.com/loveoverflowcom/myva/issues/12): `GameplayPlugin` gọi `Session::step` trong `FixedUpdate` 60 Hz, boss là `Controller` của phiên, cảnh và HUD đọc component mirror. Bằng chứng tự động không thay playtest người thật; xem [báo cáo combat](docs/reports/combat-graybox.md).

| Đường dẫn / crate | Vai trò |
| --- | --- |
| `packages/sim` (`myva-sim`) | Mô phỏng combat thuần Rust, bộ luật duy nhất: tick 60 Hz, số nguyên, kit Long Lưu, đạn, Slow, quái bùn có AI, NPC, boss Kẻ Giữ Đập, bot B0/B1, schema lệnh/sự kiện có version, phiên authoritative, replay và công cụ `myva-replay`. Không phụ thuộc Bevy hay renderer; build được cho wasm32. |
| `packages/gameplay` (`myva-gameplay`) | Adapter Bevy ECS headless (`bevy_app`/`bevy_ecs`/`bevy_time` `=0.20.0`): `FixedUpdate` 60 Hz, mirror component bất biến, authority client/server, runner `myva-headless`. Không renderer/window/asset/audio. Xem [D04](docs/technical/gameplay-foundation.md). |
| `packages/assets` (`myva-assets`) | Schema manifest/sprite sheet/bản đồ format v1 và validator asset 2D (hash, PNG, frame, pivot, clip, bundle, budget, provenance), CLI `myva-assets validate`. Không phụ thuộc Bevy; client dùng chung luật tài liệu. Xem [D05-A](docs/technical/asset-pipeline.md). |
| `packages/economy` (`myva-economy`) | Sổ cái R/N/I/X/Q/Z, ngân sách theo giờ và bộ chạy headless `myva-econ-sim`. |
| `packages/graybox` (`myva-graybox`) | Client graybox **Bevy** trên `GameplayPlugin`: đánh boss Kẻ Giữ Đập (mặc định) hoặc đấu tập với bot; fixture art D05-A theo bundle qua `AssetServer`/StreamingPlanner (G unload/reload, retry hữu hạn, lỗi giữ hình khối), vùng cảnh báo boss, HUD HP/sức bền/năng lượng/Mạch/hồi chiêu; bàn phím, gamepad và nút cảm ứng theo [combat.md §12](docs/design/combat.md#12-điều-khiển-và-khả-năng-tiếp-cận). Lib dùng chung cho web, bin cho native desktop; test đối chứng từng tick với lõi. |
| `apps/web/shell` (`myva-web-shell`) | Leptos CSR shell: tên local, vào/rời trận, tạm dừng, đấu lại, đổi chế độ, bot đánh mẫu, tải replay, trạng thái tiếng Việt, chat/IME. Chưa có tài khoản hoặc server. |
| `apps/web/game` (`myva-web-game`) | Entry WASM của graybox trong game document cùng origin; bridge v2 có version/session, canvas/runtime được hủy cùng iframe. |

```bash
cargo xtask doctor
cargo xtask verify repository
cargo xtask run headless -- --seed 7 --ticks 3600
cargo xtask run replay -- verify myva-boss-r1-t3766.myva-replay
cargo xtask run economy -- --seed 7 --days 90 > econ.csv
cargo xtask verify assets
cargo xtask verify determinism
cargo xtask preflight --changed-only
```

[`cargo xtask`](xtask/README.md) là entrypoint development theo [Single Surface Rule và build contract](docs/architecture/xtask-build-contract.md); scripts hiện hữu là primitives. `myva-headless` chạy phòng thử qua app ECS không cửa sổ/GPU và thoát lỗi nếu hash từng tick lệch lõi server-only. `verify determinism` (cần wasm-bindgen CLI `0.2.129` và Node) so cùng fixture giữa native và WASM. `preflight` đầy đủ và `ci` dùng chung planner; `--changed-only` chọn theo Git/dependency impact. `--dry-run` in plan với `NOT_RUN`; build tuần tự, mặc định hai job và tôn trọng `CARGO_BUILD_JOBS`.

Chơi **trên trình duyệt** (Bevy WASM trong shell Leptos):

```bash
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cargo xtask run web
# Terminal khác, sau khi server đã ready:
cargo xtask app web
```

`run web` build cả hai bundle rồi serve `http://127.0.0.1:8080/`, in URL/PID/log sau readiness; Ctrl+C dừng owned server. `app web` chỉ mở bundle/server đã có. Dùng `--port` khi port bận; không kill process khác. Build mặc định profile `web`; `cargo xtask build web --profile dev` dùng debug, `--release` dùng release. Provision browser prerequisites rồi chạy E2E ở port riêng (xtask tự quản lý server của test):

```bash
npm ci
CHROME_PATH=/usr/bin/google-chrome cargo xtask verify web --port 8081
```

Đọc [báo cáo web](docs/reports/web-feasibility.md) và [báo cáo native](docs/reports/native-feasibility.md) để phân biệt compile, browser interaction, giả lập touch và thiết bị thật. Android/iOS thật còn blocker; không dùng bản standalone để kết luận CMP đã tích hợp.

Chơi **native desktop** (Linux cần `libudev-dev` để build gamepad; `--no-default-features` bỏ gamepad):

```bash
cargo xtask run graybox -- --duel    # thêm --demo, --no-art, --screenshot <file.png>
```

Phím: A/D di chuyển, Space nhảy, Shift lướt, giữ L đỡ, J/K nhẹ/nặng, Q/E/R thuật; Enter đấu lại, M đổi chế độ, H hitbox, G bật/tắt art. Bản native lưu replay bằng F9, bản web bằng nút “Tải replay”. Khi trận kết thúc, client tự chạy lại replay và hiện kết quả kiểm chứng; `myva-replay verify <file>` chạy lại ngoài client, `myva-replay events <file>` in từng đòn ra, trúng, đỡ theo tick. Replay ghi trên WASM kiểm chứng được bằng bản native cùng revision.

Long Lưu có đại thuật ở ô 3: chạm rồi thả R / RB+B / nút Triều dùng thường, giữ 450 ms yêu cầu tăng cường (100 Mạch + 35 năng lượng). Nhấn đỡ mới sau khi vào hitstun để Thoát Mạch (50 Mạch, hồi 20 s). DR/budget theo mục tiêu và feedback trúng/bị đỡ/hụt dùng cùng lõi; thông số là GT. [Báo cáo #19](docs/reports/combat-mach.md) ghi bằng chứng và các gate người thật/mạng còn thiếu.

EquipmentRegistry offline ([contract](docs/technical/equipment-registry.md), [evidence](docs/reports/equipment-registry.md)) validate ba item GT, budget/stacking và lock build; CLI `cargo xtask run replay -- build-check an-phan-chan phu-vu-bo` kiểm headless. Hiệu ứng là pure prototypes, chưa active trong combat/replay; Rust checks của phần mới chưa chạy vì host thiếu toolchain.

CharacterCatalog offline ([contract](docs/technical/character-catalog-assets.md), [evidence](docs/reports/character-catalog-assets.md)) dùng chung metadata Leptos/Bevy và chọn asset theo stable KitId. Long Lưu vẫn fixture chưa ánh xạ, icon/art placeholder; shell chọn exact pin trước trận, native dùng `--kit long-luu`. Chưa có roster trực hệ đã duyệt.

## Đọc tài liệu

1. [Review draft và quyết định cần kiểm chứng](docs/design/review.md).
2. [GDD](docs/design/gdd.md), [World Bible](docs/design/world-bible.md), [chiến đấu](docs/design/combat.md).
3. [Tiến triển, solo và bang hội](docs/design/progression-social.md), [kinh tế](docs/design/economy.md).
4. [Kiến trúc](docs/technical/architecture.md), [assets và hiệu năng](docs/technical/assets-performance.md), [D05-B runtime và gate còn thiếu](docs/reports/asset-streaming-performance.md).
5. [Vertical slice](docs/production/vertical-slice.md), [roadmap](docs/production/roadmap.md), [QA](docs/production/qa.md).
6. [Hàng đợi công việc](docs/work-plan/README.md), [quyết định Bevy hiện hành](docs/decisions/0003-bevy-engine-adoption.md) và [quyết định nền tảng](docs/decisions/0001-project-foundation.md).
7. [Năm thế lực — ADR 0004](docs/decisions/0004-five-factions-lineage-hierarchy.md), [danh mục trực hệ ứng viên và bàn giao #20](docs/design/lineages.md), [nguồn đối chiếu](docs/research/five-factions-sources.md), [báo cáo đồng bộ](docs/design/worldbuilding-report.md) và [lịch sử ADR 0002](docs/decisions/0002-lineages-world-structure.md).

[Mục lục đầy đủ](docs/README.md) phân biệt yêu cầu đã chốt, phương án đề xuất và câu hỏi còn mở.

Theo dõi công việc tại [Thần Mạch — Planning](https://github.com/users/loveoverflowcom/projects/3) và [Thần Mạch — Kanban](https://github.com/users/loveoverflowcom/projects/4/views/1). [Hướng dẫn planning](docs/work-plan/planning.md) liên kết các issue, phụ thuộc và điều kiện chuyển cột.

## Điểm bắt đầu

**Thế lực → Nguồn bảo trợ → Trực hệ → Nhánh chiến đấu → Kit** là trục nhân vật; linh vực/bản đồ là chiều địa lý độc lập. Tầm nhìn dài hạn gồm nhiều linh vực và lựa chọn xuất thân. Bản thử đầu tiên chỉ có **Vân Thủy**, một linh vực hư cấu lấy cảm hứng từ cảnh quan Đông Nam Á: ba bản đồ, ba kit (Long Lưu, Sơn Cốt, Phong Vũ), một boss và một vòng chơi 20–30 phút. Long Lưu là baseline graybox hiện có; hai kit còn lại theo gate slice. Năm tên kit/mẫu cơ chế cũ không ánh xạ một–một sang năm tôn giáo. Kiểm chứng cảm giác chiến đấu, khả năng tích hợp native và luật tài nguyên trước khi mở rộng nội dung online.

Không khởi tạo một workspace với hàng chục crate hoặc service khi chưa có prototype chứng minh ranh giới cần thiết. Các sơ đồ kỹ thuật là thiết kế dự kiến, không mô tả mã nguồn đang tồn tại.
