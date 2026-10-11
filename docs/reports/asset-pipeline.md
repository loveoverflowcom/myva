# D05-A — Pipeline asset 2D

**Trạng thái:** fixture, validator và loader chạy được trên native desktop và Chrome desktop (WebGL2 software), ngày 2026-10-11. Issue [#13](https://github.com/loveoverflowcom/myva/issues/13). Run revision `ac6c6c491adf2cdd4803eef4d82f71fc3c0dbd32`, `dirty=false`, base `develop` `c96a7c4`. Đặc tả: [asset-pipeline](../technical/asset-pipeline.md). Evidence: [JSON](evidence/d05a-2026-10-11.json). Không đóng #13: D05-B, mobile thật và đo hiệu năng vẫn còn.

## Kết quả

- **Asset mẫu:** Long Lưu (idle 4, run 6, jump 3 có mốc `apex`, attack 5 có mốc `hit`), Kẻ Giữ Đập (idle 4, run 4, attack 5), bản đồ Đập Cổ (2 tile + 4 lớp phông + layout) và VFX (hit, guard, slash, dust, đạn Lưu Tiễn). Sinh bằng mã trong repo, không có đầu vào bên thứ ba; license `LicenseRef-MyVa-Unreleased` vì chủ dự án chưa chọn license.
- **Chuẩn hóa:** 2 texel/px thiết kế cho mặt phẳng gameplay, 1 cho phông; pivot đế chân giống nhau ở mọi frame; padding 4, extrude 2 cho tile/phông; texel trong suốt mang màu nét viền.
- **Manifest:** 11 asset, 4 bundle (`character`, `boss`, `map`, `vfx` trùng `BundleKind` của D05-B), mỗi asset có ID, version, bytes, SHA-256, dependency, kích thước texture và provenance/license.
- **Validator:** 37 mã lỗi; 16 test tích hợp làm hỏng fixture thật theo từng kiểu và yêu cầu đúng mã lỗi, phần lớn yêu cầu *chỉ* mã đó.
- **Runtime:** client graybox tải fixture qua `AssetServer`, kiểm size/hash trước khi giải mã, animation đòn theo pha của lõi; lỗi tải giữ hình khối, trận vẫn chạy. Phím G bật/tắt.

Validator trên fixture đã commit, chưa có mipmap:

| Bundle | Texture RGBA8 / budget | File / budget |
| --- | --- | --- |
| `character.long_luu` | 7,38 / 8,00 MiB | 161.165 / 1.048.576 B |
| `boss.ke_giu_dap` | 9,02 / 12,00 MiB | 233.700 / 1.048.576 B |
| `vfx.combat` | 0,78 / 2,00 MiB | 48.994 / 524.288 B |
| `map.dap_co` | 1,77 / 4,00 MiB | 22.546 / 1.048.576 B |
| Tổng | 18,94 / 32,00 MiB | 466.405 / 4.194.304 B |

`texture_bytes` là ước lượng `w × h × 4` base level, không phải GPU allocation đo được. Budget bundle là giá trị thiết kế cho fixture, không phải budget art slice.

## Kiểm chứng

Host: Linux 7.0.0-38, 22 luồng CPU, 22 GiB RAM, Intel Graphics (MTL) Mesa 26.0.8 Vulkan. Rust 1.97.1, Bevy 0.20.0, wasm-bindgen 0.2.129, Python 3.14.4, Chrome 153.0.8010.36, Playwright 1.64.0. Mọi lệnh dưới đây chạy trên run revision; log lệnh và mã thoát nằm trong evidence JSON.

| Hạng mục | Kết quả | Ghi chú |
| --- | --- | --- |
| `python3 scripts/generate-asset-fixture.py --check` | PASS | 5 atlas tái lập, chênh kênh lớn nhất 0; sheet/layout trùng |
| `myva-assets validate assets/fixture/manifest.json` | PASS | 0 lỗi, bảng budget ở trên |
| Test asset lỗi (`cargo test -p myva-assets`) | PASS | 2 unit + 16 tích hợp: hash, size, PNG hỏng/cắt cụt, thiếu/thừa file, ID trùng, path traversal, dependency lạ/chu trình, kích thước texture, limit, budget, bundle mồ côi/tham chiếu chéo, license, provenance, format/trường lạ, frame ngoài texture/chồng/padding, pivot, pivot rung, thiếu clip/mốc, `hit` ở frame 0, frame ma, timing, scale, layout, hình chạm mép ô, extrude |
| `cargo fmt --check`, clippy native và wasm32 `-D warnings` | PASS | Graybox native chạy `--no-default-features` (xem BLOCKED) |
| `cargo test --workspace` | PASS | Trừ graybox mặc định; graybox chạy riêng `--no-default-features`: 32 unit + 4 parity |
| Test art headless | PASS | Frame `hit` hiện đúng lúc `Active`, frame không lùi, startup trước `hit`; idle/run/jump chọn đúng; art thay thân/nền nhưng số hình vùng báo không đổi |
| `check-wasm-determinism.sh 3600 1 7 12` | PASS | 3 seed khớp 3600 hash native/WASM |
| `myva-headless --seed 7 --ticks 3600` | PASS | ECS khớp lõi server-only |
| Native có art / không art, cùng bot B1 | PASS | Art sẵn sàng 5 sheet. Hai replay trùng 1.900 dòng input đầu và 15 mốc hash chung; mốc cuối khác vì chụp ở tick 950 và 951. `myva-replay verify` khớp cả hai |
| Web build `PROFILE=web` | PASS | `dirty=false`; validator chạy trước khi chép asset |
| Playwright toàn bộ (`npm run test:web`) | PASS | 9/9: 3 test art + 6 test có sẵn (30 vòng vòng đời, touch/DPR/context loss, giao thức, bàn phím, gamepad giả lập, bot thắng boss và replay kiểm native) |
| Web: tải art | PASS | `ready`, 5 sheet; 5 PNG + 7 JSON, mỗi file một request 200, không request `.meta`, không lỗi console |
| Web: atlas sai hash | PASS | `failed` / `hash_mismatch`, hình khối, tick tiếp tục tăng |
| Web: thiếu manifest (404) | PASS | `failed` / `io`, trận tiếp tục |
| Phím G bật/tắt art trên web | PASS | `visible` đổi hai chiều, tick tăng |
| Gamepad native Linux (`gilrs`) | BLOCKED | Máy thiếu `libudev-dev`, không có quyền cài; CI cài sẵn và build feature mặc định |
| Android/iOS thật, Safari, Firefox, GPU phần cứng trên web | NOT_RUN | Chưa có thiết bị/gate D02; Chrome dùng SwiftShader |
| Frame time, RAM/VRAM, upload GPU, load/unload lặp, soak | NOT_RUN | Phạm vi D05-B; không suy từ fixture |

## Kích thước web

Build cùng máy, cùng `PROFILE=web` và wasm-bindgen. Gzip là `gzip.compress(mtime=0)`, không phải byte truyền đo được.

| File | `develop` `c96a7c4` | D05-A `ac6c6c4` | Chênh |
| --- | --- | --- | --- |
| Game WASM | 84.864.336 B (gzip 11.183.569) | 86.218.814 B (gzip 11.496.394) | +1.354.478 B (+1,60%), gzip +312.825 B (+2,80%) |
| Fixture `game/assets/` | — | 477.336 B (gzip 431.355) | PNG gần như không nén thêm |

Phần tăng của WASM đến từ decoder PNG của Bevy/`image` trên web cùng `serde_json`/`sha2` của `myva-assets`; chưa tách riêng từng phần. Mốc 84.842.609 B trong [báo cáo D05-B](asset-streaming-performance.md) được build ở revision và máy khác nên không dùng để so.

## Ảnh đã mở và kiểm tra

- [Native có art](evidence/d05a-native-art.png), 2133×1200, tick 950: boss đang báo Quét Ngang. Vùng báo vàng và thanh tiến trình nằm trên phông, nhìn xuyên qua nhân vật; chân Long Lưu và boss đặt đúng vạch nền; nét viền rõ khi phóng to; HUD, nhật ký và nhãn "Boss" đọc được.
- [Web có art](evidence/d05a-web-art.png) và [fallback do atlas sai hash](evidence/d05a-web-fallback.png): cùng bố cục; fallback là cảnh hình khối cũ, không lẫn sprite nào.
- [Giả lập cảm ứng ngang 844×390](evidence/d05a-touch-landscape.png): silhouette và màu hai bên vẫn phân biệt được nhưng Long Lưu chỉ cao khoảng 40 px CSS; ảnh giả lập không chứng minh điện thoại thật.

Lỗi đã phát hiện và sửa trong lúc làm: validator bắt chân nhân vật lún dưới pivot và gậy/nắm đấm chạm mép ô (`frame_clipped`), đã sửa hình học và kích thước ô. Ảnh native đầu tiên có phông sáng làm chữ HUD xám, nhãn "Boss" và nền gạch khó đọc; palette phông đã được làm tối và giảm tương phản.

## Phát hiện còn mở

1. **Nhân vật nhỏ trên màn điện thoại ngang:** camera giữ cả arena 1.680 px thiết kế nên Long Lưu khoảng 40 px CSS ở 844×390. Cần quyết định zoom/camera theo người chơi trước art slice; không đổi scale asset để bù.
2. **Nhãn "P1" xanh trên cột nước xanh** tương phản thấp; cần viền/nền cho nhãn tên.
3. **Nút cảm ứng phía phải đè lên boss** khi boss đứng sát mép phải (layout có sẵn từ #2, art làm dễ thấy hơn).
4. **Chưa có mipmap:** sprite thu khoảng 2,6 lần ở 1280×720 DPR 1 với lọc tuyến tính. Cần thử mipmap/tier texture và đo bộ nhớ.
5. **Fixture tải trọn một manifest:** bundle mới là đơn vị kế toán, chưa là đơn vị tải. Nối planner D05-B với `AssetServer` là bước tiếp theo.
6. **Một clip `attack` cho mọi đòn**, boss bị hạ chỉ tô tối; clip theo đòn và trạng thái ngã thuộc art slice.

## Ranh giới

Không đổi `myva-sim`, `myva-gameplay`, `myva-web-shell`, `myva-economy`, schema lệnh/sự kiện, replay hay bridge giao thức. Telemetry host (không phải giao thức mạng) thêm khối `art` chỉ đọc. Art là cosmetic trong graybox; với slice, asset thiết yếu chặn ở điểm an toàn theo [assets-performance §6](../technical/assets-performance.md#6-cache-và-lỗi-tải).
