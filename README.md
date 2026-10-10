# MyVa — Thần Mạch

**MMORPG hành động 2D HD về những người kế thừa sức mạnh thần thoại.**

Người chơi trưởng thành từ một linh vực, học cách chiến đấu, khám phá các truyền thừa khác và tham gia những chiến dịch nối nhiều lục địa. Thế giới có chỗ cho người chơi solo, người cày chay và bang hội; sức mạnh nhân vật phải đi cùng kỹ năng điều khiển.

> Dễ bắt đầu, khó thành thạo; cày chay vẫn mạnh, kỹ năng tạo khác biệt, thế giới mở rộng trong một nền kinh tế có giới hạn và được kiểm chứng.

## Trạng thái dự án

- **Giai đoạn:** thiết kế tiền sản xuất; tài liệu draft v0.1, ngày 2026-10-09.
- **Hiện có:** review ý tưởng, GDD, World Bible, luật chiến đấu, kinh tế, kiến trúc đề xuất và kế hoạch kiểm chứng; lõi mô phỏng combat và kinh tế headless, nền ECS Bevy headless dùng chung luật lõi ([D04](docs/technical/gameplay-foundation.md)), graybox chiến đấu Bevy chơi được trên trình duyệt và native desktop (chưa qua playtest người thật), shell Leptos web và prototype native standalone. Phạm vi bằng chứng nằm trong báo cáo từng platform.
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

Workspace Rust dùng toolchain `1.97.1` theo `rust-toolchain.toml`; Bevy `0.20.0` yêu cầu phiên bản này. Lõi simulation và economy được giữ độc lập renderer. Graybox combat đã chuyển từ Macroquad sang **Bevy** ở [#2](https://github.com/loveoverflowcom/myva/issues/2): `Battle` của lõi chạy trong `FixedUpdate` 60 Hz qua một adapter mỏng của graybox; nền ECS dùng chung [#12](https://github.com/loveoverflowcom/myva/issues/12) có adapter headless riêng. Bằng chứng tự động không thay playtest người thật; xem [báo cáo combat](docs/reports/combat-graybox.md).

| Crate | Vai trò |
| --- | --- |
| `crates/sim` (`myva-sim`) | Mô phỏng combat thuần Rust, bộ luật duy nhất: tick 60 Hz, số nguyên, kit Long Lưu, đạn, Slow, quái bùn có AI, NPC, boss Kẻ Giữ Đập, bot B0/B1, schema lệnh/sự kiện có version, phiên authoritative, replay và công cụ `myva-replay`. Không phụ thuộc Bevy hay renderer; build được cho wasm32. |
| `crates/gameplay` (`myva-gameplay`) | Adapter Bevy ECS headless (`bevy_app`/`bevy_ecs`/`bevy_time` `=0.20.0`): `FixedUpdate` 60 Hz, mirror component bất biến, authority client/server, runner `myva-headless`. Không renderer/window/asset/audio. Xem [D04](docs/technical/gameplay-foundation.md). |
| `crates/economy` (`myva-economy`) | Sổ cái R/N/I/X/Q/Z, ngân sách theo giờ và bộ chạy headless `myva-econ-sim`. |
| `crates/graybox` (`myva-graybox`) | Client graybox **Bevy**: đánh boss Kẻ Giữ Đập (mặc định) hoặc đấu tập với bot; hình khối, vùng cảnh báo boss, HUD HP/sức bền/năng lượng/Mạch/hồi chiêu; bàn phím, gamepad và nút cảm ứng theo [combat.md §12](docs/design/combat.md#12-điều-khiển-và-khả-năng-tiếp-cận). Lib dùng chung cho web, bin cho native desktop. |
| `crates/web-shell` (`myva-web-shell`) | Leptos CSR shell: tên local, vào/rời trận, tạm dừng, đấu lại, đổi chế độ, bot đánh mẫu, tải replay, trạng thái tiếng Việt, chat/IME. Chưa có tài khoản hoặc server. |
| `crates/web-game` (`myva-web-game`) | Entry WASM của graybox trong game document cùng origin; bridge v2 có version/session, canvas/runtime được hủy cùng iframe. |

```bash
cargo test --workspace
cargo run -p myva-gameplay --bin myva-headless -- --seed 7 --ticks 3600
cargo run -p myva-sim --bin myva-replay -- verify myva-boss-r1-t3766.myva-replay
cargo run -p myva-economy --bin myva-econ-sim -- --seed 7 --days 90 > econ.csv
```

`myva-headless` chạy phòng thử qua app ECS không cửa sổ/GPU và thoát lỗi nếu hash từng tick lệch lõi server-only. `./scripts/check-wasm-determinism.sh` (cần wasm-bindgen CLI `0.2.129` và Node) so cùng fixture giữa native và WASM.

Chơi **trên trình duyệt** (Bevy WASM trong shell Leptos):

```bash
cargo install wasm-bindgen-cli --version 0.2.129 --locked
./scripts/build-web-shell.sh
python3 scripts/serve-web-shell.py
```

Mở `http://127.0.0.1:8080/`. Script build dùng profile `web`; `PROFILE=dev ./scripts/build-web-shell.sh` dùng bản debug. Chạy browser check sau khi server sẵn sàng:

```bash
npm ci
CHROME_PATH=/usr/bin/google-chrome npm run test:web
```

Đọc [báo cáo web](docs/reports/web-feasibility.md) và [báo cáo native](docs/reports/native-feasibility.md) để phân biệt compile, browser interaction, giả lập touch và thiết bị thật. Android/iOS thật còn blocker; không dùng bản standalone để kết luận CMP đã tích hợp.

Chơi **native desktop** (Linux cần `libudev-dev` để build gamepad; `--no-default-features` bỏ gamepad):

```bash
cargo run -p myva-graybox            # thêm --duel, --demo, --screenshot <file.png>
```

Phím: A/D di chuyển, Space nhảy, Shift lướt, giữ L đỡ, J/K nhẹ/nặng, Q/E/R thuật; Enter đấu lại, M đổi chế độ, H hitbox. Bản native lưu replay bằng F9, bản web bằng nút “Tải replay”. Khi trận kết thúc, client tự chạy lại replay và hiện kết quả kiểm chứng; `myva-replay verify <file>` chạy lại ngoài client, `myva-replay events <file>` in từng đòn ra, trúng, đỡ theo tick. Replay ghi trên WASM kiểm chứng được bằng bản native cùng revision.

## Đọc tài liệu

1. [Review draft và quyết định cần kiểm chứng](docs/design/review.md).
2. [GDD](docs/design/gdd.md), [World Bible](docs/design/world-bible.md), [chiến đấu](docs/design/combat.md).
3. [Tiến triển, solo và bang hội](docs/design/progression-social.md), [kinh tế](docs/design/economy.md).
4. [Kiến trúc](docs/technical/architecture.md), [assets và hiệu năng](docs/technical/assets-performance.md).
5. [Vertical slice](docs/production/vertical-slice.md), [roadmap](docs/production/roadmap.md), [QA](docs/production/qa.md).
6. [Hàng đợi công việc](docs/work-plan/README.md), [quyết định Bevy hiện hành](docs/decisions/0003-bevy-engine-adoption.md) và [quyết định nền tảng](docs/decisions/0001-project-foundation.md).
7. Đề xuất v0.2 về thế giới, thần thoại và hệ nhân vật: [báo cáo thiết kế](docs/design/worldbuilding-report.md), [khảo sát thần thoại](docs/research/mythology-survey.md) và [quyết định 0002 (chờ duyệt)](docs/decisions/0002-lineages-world-structure.md).

[Mục lục đầy đủ](docs/README.md) phân biệt yêu cầu đã chốt, phương án đề xuất và câu hỏi còn mở.

Theo dõi công việc tại [Thần Mạch — Planning](https://github.com/users/loveoverflowcom/projects/3) và [Thần Mạch — Kanban](https://github.com/users/loveoverflowcom/projects/4/views/1). [Hướng dẫn planning](docs/work-plan/planning.md) liên kết các issue, phụ thuộc và điều kiện chuyển cột.

## Điểm bắt đầu

Tầm nhìn dài hạn gồm nhiều linh vực và lựa chọn xuất thân. Bản thử đầu tiên chỉ có **Vân Thủy**, một linh vực hư cấu lấy cảm hứng từ cảnh quan Đông Nam Á: ba bản đồ, ba truyền thừa, một boss và một vòng chơi 20–30 phút. Kiểm chứng cảm giác chiến đấu, khả năng tích hợp native và luật tài nguyên trước khi mở rộng nội dung online.

Không khởi tạo một workspace với hàng chục crate hoặc service khi chưa có prototype chứng minh ranh giới cần thiết. Các sơ đồ kỹ thuật là thiết kế dự kiến, không mô tả mã nguồn đang tồn tại.
