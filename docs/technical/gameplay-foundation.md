# Nền gameplay ECS và simulation headless — D04

**Trạng thái:** draft đã có implementation, ngày **2026-10-10**. Issue [#12](https://github.com/loveoverflowcom/myva/issues/12), quyết định [ADR 0003](../decisions/0003-bevy-engine-adoption.md), nhánh `feat/bevy-gameplay-foundation`. Bằng chứng ở mục 9 phủ chạy headless native, lõi WASM qua Node và (từ 2026-10-11) client Bevy render của #2 chạy trên `GameplayPlugin`; chưa có mạng hai client hay thiết bị. Mọi thông số quái, Slow và tầm tương tác là giả thuyết (GT) để kiểm chứng boundary, chưa cân bằng.

## 1. Một bộ luật, hai lớp

Luật combat chỉ nằm trong `myva-sim`. `myva-gameplay` là adapter ECS: nhận lệnh, gọi `Session::step` đúng một lần mỗi tick cố định, mirror kết quả sang entity/component và phát sự kiện. Không có system ECS nào tính di chuyển, va chạm, damage, cooldown hay hiệu ứng.

| Crate / module | Sở hữu | Không sở hữu |
| --- | --- | --- |
| `myva-sim::world` | `World::step`: `input → movement → collision → combat → status → events` | Vai trò người chơi/quái, phiên mạng |
| `myva-sim::protocol` | Schema lệnh/sự kiện có version, `EntityRef`, `CommandQueue`, `TickReport` | Định dạng truyền tải (chưa chọn) |
| `myva-sim::session` | `World` + cổng lệnh + bộ não AI + replay; trait `Controller` | Phần thưởng, ledger |
| `myva-sim::monster`, `status`, `npc` | Quái bùn và `MonsterBrain`, Slow, NPC tương tác | Hội thoại, nhiệm vụ |
| `myva-sim::snapshot` | Khung nhìn tối thiểu cho mirror/hòa giải | Khôi phục `World` |
| `myva-sim::fixture` | Phòng thử dùng chung cho test, runner, WASM | Nội dung phát hành |
| `myva-sim::battle` | Trận graybox #2: đội hình, boss/bot đấu tập là `Controller`, bot lái hộ, trọng tài, tự kiểm chứng replay | Vòng tick, luật |
| `myva-gameplay` | `GameplayPlugin`, mirror component và khung nhìn đọc lại (`view`), `LocalInput`, `Authority`, `RewardOutbox`, `LoadSession`, runner `myva-headless` | Renderer, window, asset, audio |

`myva-gameplay` chỉ phụ thuộc `bevy_app`, `bevy_ecs`, `bevy_time` `=0.20.0` với `default-features = false, features = ["std"]`; `cargo tree -p myva-gameplay` không có wgpu, winit, render, asset, audio hay window. Server dùng được crate này mà không kéo renderer. Lõi `myva-sim` vẫn không phụ thuộc Bevy.

## 2. Tick và schedule

Tick 60 Hz qua `Time<Fixed>::from_hz(60)`. `FixedUpdate` chạy 0..n lần mỗi frame theo thời gian đã trôi, nên FPS chỉ đổi số tick mỗi frame, không đổi tốc độ luật. `Time<Virtual>` giữ trần delta 250 ms mặc định của Bevy: frame dừng lâu hơn làm mô phỏng chậm lại thay vì chạy dồn.

| `GameplaySet` (chain trong `FixedUpdate`) | Việc |
| --- | --- |
| `Input` | `LocalInput` (client) hoặc `ScriptedPlayer` (headless) gửi một `CommandEnvelope` vào phiên |
| `Simulate` | `Session::step`: bộ não AI theo thứ tự `FighterId` → lấy lệnh của tick → `World::step` → ghi replay |
| `Mirror` | `Snapshot` → spawn/thay/xóa entity và component |
| `Events` | `SimEvent`, `TickCompleted`; chỉ server: `RewardIssued` |

Trong `World::step`, các pha là hàm riêng theo đúng thứ tự ADR: **input** nhận tối đa một khung mỗi nhân vật, xử lý tương tác NPC theo trạng thái đầu tick, bắt đầu hành động; **movement** di chuyển nhân vật, đạn cũ bay rồi đạn mới sinh; **collision** gom mọi va chạm từ cùng một trạng thái; **combat** áp dụng đỡ/phản công/damage/hiệu ứng; **status** tiến bộ đếm đòn, hitstun, tài nguyên, hồi chiêu, hiệu ứng; **events** trả theo thứ tự phát sinh. ECS không chạy riêng từng pha nên không thể đổi thứ tự luật.

Thiết bị chạy theo frame ở `Update` và chỉ ghi vào `LocalInput`: hướng/đỡ là trạng thái đang giữ, nút bấm được chốt tới tick kế tiếp rồi xóa. 30 FPS không biến một cú bấm thành hai hành động; 240 FPS không làm mất cú bấm giữa hai tick. `release_all()` dùng khi mất focus hoặc mở bàn phím chat.

## 3. Entity và component

| Loại | Component |
| --- | --- |
| Mọi entity | `SimId(EntityRef)` |
| Người chơi / quái | `Player` hoặc `Monster`, `KitRef`, `Faction`, `Position`, `Facing`, `Health`, `Stamina`, `Energy`, `Mach`, `Stance`, `CurrentAction`, `Cooldowns`, `StatusEffects`, `Hurtbox`, `AttackBox`, `Invulnerable`, `Grounded`, `GuardWindow`, `AckedSeq`; quái có thêm `Ai` |
| Đạn | `Projectile`, `Owner`, `SourceAction`, `Heading`, `Position`, `Volume` |
| NPC | `Npc`, `NpcKind`, `Position` |

**Built-in và miền.** Adapter không dùng component built-in của Bevy. `Transform`, sprite, camera, animation và VFX thuộc client presentation, suy ra từ `Position` (mili-pixel, y hướng lên) ở `Update`. Component miền là mirror số nguyên của lõi, đều `#[component(immutable)]`: system khác không lấy được `&mut` để sửa luật qua ECS. Mirror chỉ thay component khi giá trị đổi, nên change detection phản ánh thay đổi thật của lõi. Đọc `SimEvent` bằng `MessageReader` để phát hiệu ứng; `EventId { tick, index }` là khóa bỏ bản lặp.

Entity có ngay khi phiên được nạp (mirror ở `Startup`); phiên chèn muộn thì có từ tick đầu tiên. Vào instance khác trong cùng app (đấu lại, đổi chế độ) dùng lệnh `LoadSession`: xóa entity mirror cũ cùng component client gắn thêm, nạp phiên mới và mirror ngay. `SimSession::halt` dừng tick khi instance kết thúc, entity giữ trạng thái cuối. Rời game thì web shell vẫn tháo cả runtime như D03.

Presentation đọc lại mirror qua `view::FighterMirror`/`ProjectileMirror` thành `FighterView`/`ProjectileView`, cùng kiểu với `Session::snapshot()`; test kiểm hai nguồn trùng nhau mỗi tick, nên hàm vẽ test được bằng snapshot không cần GPU.

## 4. Lệnh và sự kiện

`CommandEnvelope { protocol, session, actor, seq, tick, frame }`, `PROTOCOL_VERSION = 1`. `CommandFrame` gồm **Move** (`move_x` −1/0/1), **Guard** (giữ) và tối đa một `Action`: **Jump**, **Dash**, **Attack** (`Light`/`Heavy`), **Skill** (ô 0..=2), **Interact**. Không có trường damage, kết quả trúng, phần thưởng hay số dư.

| Kiểm tra trong `CommandQueue::submit` | Từ chối |
| --- | --- |
| Version khác `PROTOCOL_VERSION` | `UnsupportedProtocol` |
| `move_x` ngoài −1..=1, ô thuật > 2 | `Malformed` |
| Tác nhân chưa gắn phiên | `UnknownActor` |
| Phiên không điều khiển tác nhân, kể cả epoch cũ sau reconnect hoặc client giả epoch AI | `NotController` |
| `seq` không tăng | `StaleSeq` |
| Tick client dự đoán lệch quá `INPUT_WINDOW` = 30 tick | `Late` / `TooEarly` |
| Lệnh dồn vượt `MAX_BACKLOG` = 8 tick | `Backlog` |

Server tự gán `accepted_tick = max(tick hiện tại, tick trống kế tiếp của tác nhân)` và trả `CommandAck`; không lùi về tick client đề xuất (combat.md §6). Lệnh của một tick được lấy theo `FighterId`, nên thứ tự đến giữa các tác nhân không đổi kết quả. Bộ não quái dùng epoch `SessionEpoch::AI` và đi qua cùng cổng; phiên tự đánh số `seq` lệnh AI, nên tắt/chỉnh bộ não giữa trận (`Session::controller_mut`) không làm lệnh bị từ chối và replay vẫn ghi đủ lệnh đã áp dụng. Reconnect (`Session::rebind`) cấp epoch mới, hủy lệnh chưa chạy của epoch cũ; `seq` tiếp tục sau `last_seq` trong snapshot.

Sự kiện lõi: `InputRejected`, `ActionStarted`, `Hit`, `Blocked`, `Countered`, `GuardBroken`, `Downed`, `StatusApplied`, `StatusEnded`, `Interacted`. `TickReport { protocol, tick, events: [EventRecord { id, event }], hash }` là kết quả một tick. `damage` trong sự kiện là damage danh nghĩa của đòn.

## 5. Định danh, lưu/khôi phục và hòa giải

- `EntityRef::{Fighter(FighterId), Projectile(ProjectileId), Npc(NpcId)}` là ID miền: ổn định suốt `World`, không tái sử dụng (`ProjectileId` tăng dần). Handle `Entity` của Bevy chỉ sống trong một process và có thể được tái cấp sau khi xóa; nó không xuất hiện trong lệnh, sự kiện, replay hay snapshot. `EntityIndex` là bảng `EntityRef → Entity` do mirror quản lý.
- **Lưu/khôi phục trong process:** `World: Clone`; test chứng minh khôi phục bản lưu rồi phát lại lệnh đã ghi cho cùng hash. Replay v2 thêm `seed`, phe và NPC, vẫn đọc được bản 1. Spawn hiện chỉ ở tick 0; spawn giữa trận cần dòng replay riêng. Định dạng lưu bền vững chưa làm.
- **Hòa giải mạng (thiết kế, chưa triển khai):** server gửi snapshot kèm `last_seq` từng nhân vật; client bỏ input đã xác nhận, giữ input có `seq > last_seq`, rồi dự đoán lại từ trạng thái server. `Snapshot` hiện chỉ là khung nhìn cho presentation, không đủ để dựng lại `World` (thiếu buffer, tập mục tiêu đã trúng, bộ đếm hồi tài nguyên); prediction cần snapshot đầy đủ có version ở prototype online.

## 6. Authority và phần thưởng

`Authority::Server` và `Authority::Client` chạy cùng plugin, cùng luật. Chỉ server đăng ký `issue_rewards` và có `RewardOutbox`. Khi lõi phát `Downed` cho nhân vật vai trò quái, server ghi một `RewardClaim` với khóa `RewardKey { instance, defeated }`; khóa đã cấp không cấp lại, kể cả sau `drain`. Server nghiệp vụ dùng khóa này làm unique key khi commit ledger; nội dung phần thưởng và người nhận do bên đó quyết định.

Client không có đường tạo claim: constructor là nội bộ crate, `RewardOutbox`/`Messages<RewardIssued>` không tồn tại ở chế độ client, và lệnh không mang damage. Lõi cũng chặn nguồn nhân đôi: mục tiêu đã bị hạ trong cùng tick không nhận thêm đòn, `Downed` chỉ phát một lần. Trước PR này, hai đòn chí mạng cùng tick phát `Downed` hai lần cho cùng mục tiêu; test hồi quy tái hiện lỗi đó khi gỡ chốt chặn. Hai bên trúng nhau cùng tick vẫn đều được tính.

## 7. AI, kỹ năng, cooldown, damage, status

- `MonsterBrain` có `AiState::{Idle, Chase, Attack, Recover, Defeated}` công khai qua component `Ai`; luật không đọc trạng thái này. Bộ não chỉ đọc trạng thái đầu tick và sinh input như người chơi; nhịp nghỉ và lựa chọn thuật lấy từ `Rng` có seed dẫn xuất theo seed phiên và `FighterId`.
- Quái bùn (GT, hư cấu): 300 HP, Cào Bùn (startup 450 ms) và một thuật Phun Bùn (startup 550 ms, 30 năng lượng, hồi 4 s): đạn chậm 35 damage, trúng thì Slow 20% trong 1,5 s. Mọi đòn ≥ 450 ms tín hiệu.
- Slow theo combat.md §6: không cộng dồn, mạnh nhất thắng, trần 25%; cùng độ mạnh thì làm mới thời lượng. Slow yếu hơn đến khi đang có Slow mạnh hơn bị bỏ qua (đơn giản hóa GT). Slow giảm tốc đi bộ, không giảm lướt; chỉ áp khi trúng, không áp khi bị đỡ; bị hạ xóa mọi hiệu ứng. Thời lượng tính như hitstun: đếm từ tick trúng.
- Phe: cùng phe không đánh trúng nhau; không phe đánh được mọi người như phòng đấu tập. NPC chỉ xác nhận tương tác trong tầm từ trạng thái trung tính trên nền; hội thoại/nhiệm vụ thuộc server nghiệp vụ.

## 8. Mở rộng cho combat graybox #2

| Muốn thêm | Làm ở | Không cần sửa |
| --- | --- | --- |
| Kỹ năng/đòn mới | `ActionSpec` trong kit (`myva-sim::kit`/`monster`) | Vòng tick, plugin |
| Quái mới | Kit + một `Controller`, spawn qua `Session::spawn_controlled` | Plugin, mirror |
| Hiệu ứng mới | `StatusKind` + điểm áp dụng trong lõi | ECS |
| Presentation, HUD, VFX | System `Update` đọc mirror component và `SimEvent` | Lõi |
| Thiết bị input | Ghi `LocalInput` trước vòng fixed | Schema lệnh |
| Bot lái hộ người chơi | `LocalInput::set_frame` trước `GameplaySet::Input` | Cổng lệnh |
| Đọc trạng thái riêng của bộ não (pha boss) | `Session::controller::<T>()` | Snapshot |

#2 đã dựng client Bevy theo đúng đường này (2026-10-11): `DefaultPlugins` + `GameplayPlugin::new(Authority::Client)` + `MatchPlugin` của graybox. Boss Kẻ Giữ Đập (`BossController`) và bot đấu tập B0 (`SparringBot`) là `Controller` của phiên; bot B1/B0 lái hộ người chơi chạy phía client và gửi lệnh qua `LocalInput`. Trọng tài (`Bout::observe`) đọc `TickReport` trong `GameplaySet::Events`; cảnh và HUD đọc khung nhìn từ mirror. Graybox không ghi component mirror và không có system tính luật. Test `crates/graybox/tests/parity.rs` chạy đúng các plugin đó không cửa sổ và so từng tick với `Battle` chạy thẳng trên lõi.

## 9. Bằng chứng

Ngày **2026-10-10**, Linux x86_64, Rust `1.97.1`, Bevy crates `=0.20.0`, Node `v24.21.0`, wasm-bindgen CLI `0.2.129`, base `develop` `c8c9875`.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --locked -p myva-web-shell -p myva-web-game -p myva-sim -p myva-gameplay \
  --target wasm32-unknown-unknown -- -D warnings
cargo run -p myva-gameplay --bin myva-headless -- --seed 7 --ticks 3600 --replay d04.myva-replay
cargo run -p myva-sim --bin myva-replay -- verify d04.myva-replay
./scripts/check-wasm-determinism.sh 3600 1 7 12
```

| Kiểm tra | Kết quả | Phạm vi |
| --- | --- | --- |
| `cargo test --workspace` | PASS | Lõi: phase/regression, protocol, status, quái, phiên; 24 seed × 3.600 tick invariant: không `Downed` hai lần, đã hạ không ra đòn/không bị trúng, hồi chiêu không vượt trần, status hợp lệ, sự kiện không lặp, ID đạn không tái dùng |
| Adapter ECS | PASS | 6 seed × 1.800 tick: lõi server-only = ECS server = ECS client từng hash; mirror khớp snapshot mỗi tick; 30/60/144 FPS và jitter 5–48 ms cùng hash, tick = 60/giây mô phỏng ±1; chỉ server cấp claim, một claim mỗi quái bị hạ |
| `myva-headless` seed 1, 7, 12 | PASS | 3.600 tick không cửa sổ/GPU/audio; ECS khớp lõi 3.600/3.600 hash; replay ghi ra được `myva-replay verify` khớp 60 mốc |
| Native ECS (release) vs lõi WASM (release, Node) | PASS, 0 sai lệch | 3 seed × 3.600 hash; hash cuối trùng bản debug native. `target/wasm-determinism/report.json` |
| Client Bevy render trên `GameplayPlugin` (#2, 2026-10-11) | PASS | Hệ của client (không cửa sổ) = `Battle` trên lõi từng hash: bot B1 đánh boss tới thắng, 2.400 tick input thiết bị ngẫu nhiên, đổi chế độ/tắt bot giữa trận; replay giống hệt từng byte; E2E trình duyệt trong [báo cáo combat](../reports/combat-graybox.md) |
| Prediction/rollback, hai client qua mạng, thiết bị | NOT_RUN | Thuộc #5 và gate platform |

Native và WASM khớp vì lõi chỉ dùng số nguyên có độ rộng cố định, RNG SplitMix64 và hash FNV ghi little-endian độ rộng cố định; kết luận chỉ đúng cho fixture và build đã chạy, không suy ra cho code float sau này. Hash vẫn chỉ so được trong cùng build. Adapter ECS build được cho wasm32; ở #12 nó chưa chạy trong browser, từ #2 nó chạy trong bản web graybox (xem báo cáo combat).

## 10. Giới hạn đã biết

- Chưa có transport, snapshot mạng đầy đủ, prediction/rollback, spawn giữa trận, DR khống chế, persistence.
- Pha boss chỉ đọc được qua `Session::controller::<BossController>()`, chưa có trong snapshot/mirror; client mạng sẽ cần trường riêng.
- Interact chưa giới hạn tần suất; server nghiệp vụ phải rate-limit trước khi tin sự kiện.
- `TickHistory` của app headless tăng theo số tick; chỉ dùng cho test/runner.
