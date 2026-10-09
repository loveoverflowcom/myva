# MyVa — Thần Mạch

**MMORPG hành động 2D HD về những người kế thừa sức mạnh thần thoại.**

Người chơi trưởng thành từ một linh vực, học cách chiến đấu, khám phá các truyền thừa khác và tham gia những chiến dịch nối nhiều lục địa. Thế giới có chỗ cho người chơi solo, người cày chay và bang hội; sức mạnh nhân vật phải đi cùng kỹ năng điều khiển.

> Dễ bắt đầu, khó thành thạo; cày chay vẫn mạnh, kỹ năng tạo khác biệt, thế giới mở rộng trong một nền kinh tế có giới hạn và được kiểm chứng.

## Trạng thái dự án

- **Giai đoạn:** thiết kế tiền sản xuất; tài liệu draft v0.1, ngày 2026-10-09.
- **Hiện có:** review ý tưởng, GDD, World Bible, luật chiến đấu, kinh tế, kiến trúc đề xuất và kế hoạch kiểm chứng; lõi mô phỏng combat và kinh tế headless, client graybox lịch sử (chưa qua playtest), spike Bevy + Leptos web và prototype native standalone. Phạm vi bằng chứng nằm trong báo cáo từng platform.
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

Workspace Rust dùng toolchain `1.97.1` theo `rust-toolchain.toml`; Bevy `0.20.0` yêu cầu phiên bản này. Lõi simulation và economy hiện có được giữ độc lập renderer. **Client Macroquad hiện hữu là graybox lịch sử**, chưa được chuyển sang Bevy; nền ECS [#12](https://github.com/loveoverflowcom/myva/issues/12) rồi combat [#2](https://github.com/loveoverflowcom/myva/issues/2) chịu trách nhiệm chuyển đổi. Các spike tích hợp không hoàn thành gate combat/playtest.

| Crate | Vai trò |
| --- | --- |
| `crates/sim` (`myva-sim`) | Mô phỏng combat thuần Rust: tick 60 Hz, số nguyên, kit Long Lưu, đạn, boss Kẻ Giữ Đập, bot B0/B1, replay và công cụ `myva-replay`. Không phụ thuộc renderer; build được cho wasm32. |
| `crates/economy` (`myva-economy`) | Sổ cái R/N/I/X/Q/Z, ngân sách theo giờ và bộ chạy headless `myva-econ-sim`. |
| `crates/web-shell` (`myva-web-shell`) | Leptos CSR shell của spike #11: tên local, vào/rời game, loading/error, chat/IME. Chưa có tài khoản hoặc server. |
| `crates/web-game` (`myva-web-game`) | Cảnh Bevy WASM 2D tối thiểu trong game document cùng origin; bridge có version, canvas/runtime được hủy cùng iframe. |
| `crates/graybox` (`myva-graybox`) | **Legacy Macroquad**, chỉ giữ để đối chiếu luật/replay trước khi chuyển đổi: đánh boss (mặc định) hoặc đấu tập với bot; phím theo [combat.md §12](docs/design/combat.md#12-điều-khiển-và-khả-năng-tiếp-cận). Chạy native hoặc trên trình duyệt. Chưa có touch hay gamepad. |

```bash
cargo test --workspace
cargo run -p myva-sim --bin myva-replay -- verify graybox-r1-t600.myva-replay
cargo run -p myva-economy --bin myva-econ-sim -- --seed 7 --days 90 > econ.csv
```

Chạy spike **Bevy + Leptos** (#11):

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

Lệnh tái hiện **graybox Macroquad lịch sử** (không dùng làm mẫu phát triển engine mới):

```bash
cargo run -p myva-graybox
./scripts/build-web.sh && python3 -m http.server -d target/web 8080
```

Trong graybox legacy, F9 lưu replay của trận đang chơi; `myva-replay events <file>` in lại từng đòn ra, trúng, đỡ theo tick. Không dùng kết quả build/chơi graybox này để báo gate Bevy đã đạt.

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
