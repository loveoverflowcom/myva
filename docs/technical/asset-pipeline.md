# Pipeline asset 2D — D05-A

**Trạng thái:** đã triển khai cho fixture, ngày 2026-10-11. Issue [#13](https://github.com/loveoverflowcom/myva/issues/13). Phạm vi: một bộ asset mẫu có provenance, định dạng atlas/manifest, validator và loader Bevy `0.20.0`. Không phải art thành phẩm, CDN, cache bền vững, streaming theo bundle (D05-B) hay skeletal runtime. Kết quả chạy thật nằm trong [báo cáo D05-A](../reports/asset-pipeline.md).

Pipeline hiện thực hóa đường `nguồn có provenance → export → validate → pack → hash → manifest` của [assets-performance §4](assets-performance.md#4-pipeline-nội-dung-đề-xuất). Asset chỉ thể hiện trạng thái của lõi: hitbox, thời gian đòn, replay và giao thức không đọc file asset nào.

## 1. Thành phần

| Phần | Vị trí | Vai trò |
| --- | --- | --- |
| Bộ sinh fixture | [`scripts/generate-asset-fixture.py`](../../scripts/generate-asset-fixture.py) | Vẽ placeholder bằng SDF (chỉ thư viện chuẩn Python), đóng gói atlas, ghi PNG, sheet/layout JSON và manifest có hash. `--check` vẽ lại và so pixel với file đã commit |
| Fixture | [`assets/fixture/`](../../assets/fixture/README.md) | Long Lưu, boss Kẻ Giữ Đập, bản đồ Đập Cổ, VFX combat cơ bản; 4 bundle, 11 asset |
| Schema + validator | `crates/assets` (`myva-assets`) | Kiểu dữ liệu format v1, luật không cần file (dùng chung với client) và kiểm thư mục (feature `fs`). Không phụ thuộc Bevy |
| CLI | `myva-assets validate <manifest>` | In budget theo bundle và từng lỗi; mã thoát 1 khi có lỗi |
| Loader Bevy | `crates/graybox/src/art.rs` | `AssetLoader` đọc manifest và từng file qua `AssetServer`, kiểm size + SHA-256 rồi mới parse/giải mã, chạy lại luật tài liệu; fallback hình khối khi lỗi |

```bash
python3 scripts/generate-asset-fixture.py           # ghi lại assets/fixture
python3 scripts/generate-asset-fixture.py --check   # tái lập pixel và metadata
cargo run -p myva-assets -- validate assets/fixture/manifest.json
cargo run -p myva-graybox                            # G bật/tắt art; --no-art giữ hình khối
```

Bản native đọc `assets/` của repo (đổi bằng `MYVA_ASSET_DIR`). `scripts/build-web-shell.sh` chạy validator rồi chép `assets/fixture` vào `game/assets/` cạnh game document; Bevy WASM tải bằng `fetch` cùng origin. Cả hai host đặt `AssetMetaCheck::Never` vì fixture không có file `.meta`.

## 2. Quy ước tọa độ, pivot, padding và scale

- **Texel:** gốc trên-trái, y hướng xuống. **Px thiết kế:** đơn vị thế giới của graybox (mili-pixel lõi / 1.000), y hướng lên, nền ở y = 0.
- **Scale:** manifest khai báo `texels_per_unit` theo mặt phẳng: `gameplay` = 2 (nhân vật, boss, VFX, tile nền), `backdrop` = 1 (phông xa). Sheet phải khớp mặt phẳng của role. Kích thước vẽ = `rect / texels_per_unit`; Long Lưu 160 texel cao ≈ hộp va chạm 80 px của lõi. Mật độ 2 dành cho màn điện thoại DPR cao; màn desktop nhỏ phải thu nhỏ texture (xem rủi ro mipmap).
- **Pivot:** nhân vật và boss neo ở **đế chân chạm nền**; mọi frame trong sheet cùng kích thước ô và cùng pivot, nên đổi frame không làm nhân vật rung. Pivot không cần ở giữa ô: Long Lưu có thêm chỗ phía trước cho gậy. VFX neo giữa ô; tile neo trên-trái (mặt nền ở y = 0); phông neo dưới-trái.
- **Anchor Bevy:** `(pivot.x / w − 0,5, 0,5 − pivot.y / h)`. Khi `flip_x`, Bevy lật UV nhưng giữ quad, nên loader đổi dấu `anchor.x` để pivot đứng yên.
- **Padding:** khoảng trống ≥ `padding` texel giữa mọi frame và tới mép atlas (fixture dùng 4). **Extrude:** tile/phông lặp lại `extrude` texel biên ra ngoài frame (fixture dùng 2, phải ≤ padding / 2) để lọc tuyến tính không lấy màu ô bên cạnh. Sprite có viền trong suốt ≥ 1 texel trong ô; texel trong suốt mang màu nét viền để lọc tuyến tính không tạo quầng sáng.
- **Kích thước texture:** cạnh tối đa 2.048 (limit của manifest; trần chính sách 4.096). Atlas chia theo bundle, không gộp cả thế giới.

## 3. Animation và mốc combat

Sheet chứa `frames` (`name`, `rect` `[x, y, w, h]`, `pivot` `[x, y]`) và `clips` (`frames`, `frame_ms`, `loop`, `events`).

| Role | Clip bắt buộc | Mốc |
| --- | --- | --- |
| `character` | `idle`, `run`, `jump`, `attack` | `jump` có `apex`; `attack` có `hit` ở frame ≥ 1 |
| `boss` | `idle`, `run`, `attack` | `attack` có `hit` ở frame ≥ 1 |
| `vfx` | ít nhất một clip | — |
| `tileset`, `backdrop` | không | — |

`idle`/`run` và VFX chạy theo `frame_ms`. `attack` **không** chạy theo thời gian mà theo pha của lõi: frame trước `hit` chia đều startup, frame `hit` hiện đúng lúc đòn vào `Active`, frame sau `hit` chia đều recovery. Vì vậy FPS thấp không kéo dài hay rút ngắn cửa sổ đòn trên màn hình, và vùng báo hình khối vẫn là tín hiệu chính xác. `jump` chọn frame trước/ở/sau `apex` theo hướng y giữa hai khung hình. Trong fixture, mọi đòn của một kit dùng chung clip `attack`; clip theo từng đòn là việc của art slice.

## 4. Manifest (format v1)

```json
{
  "format": "myva.asset-manifest", "format_version": 1, "content_version": "d05a-fixture-1",
  "texels_per_unit": { "gameplay": 2.0, "backdrop": 1.0 },
  "limits": { "max_texture_side": 2048, "total_texture_bytes": 33554432, "total_file_bytes": 4194304 },
  "assets": [{
    "id": "long_luu.atlas", "kind": "texture", "path": "long_luu/atlas.png", "version": 1,
    "bytes": 156992, "sha256": "…", "dependencies": [],
    "texture": { "width": 1852, "height": 1044, "format": "rgba8_srgb" },
    "provenance": { "origin": "procedural", "tool": "scripts/generate-asset-fixture.py",
      "author": "…", "license": "LicenseRef-MyVa-Unreleased", "redistribution": "…", "created": "2026-10-11" }
  }],
  "bundles": [{ "id": "character.long_luu", "kind": "character", "assets": ["long_luu.sheet", "long_luu.atlas"],
    "dependencies": [], "budget": { "texture_bytes": 8388608, "file_bytes": 1048576 } }]
}
```

- `kind`: `texture` (PNG RGBA8), `sprite_sheet` (phụ thuộc đúng một texture), `map_layout` (chỉ phụ thuộc sheet). Bundle `kind` trùng `BundleKind` của planner D05-B: `character`, `boss`, `map`, `vfx`.
- Mỗi asset thuộc đúng một bundle. Dependency sang bundle khác chỉ hợp lệ khi bundle khai báo bundle đó trong `dependencies`.
- `texture_bytes` = tổng `width × height × 4` (RGBA8 sau decode, chưa mipmap). Đây là ước lượng base level GPU, không phải RAM tiến trình hay byte truyền.
- Mọi file JSON từ chối trường lạ; đổi ý nghĩa trường phải tăng `format_version`.
- Provenance: `origin` `procedural` cần `tool`, `third_party` cần `source`; license phải nằm trong danh sách đã duyệt (`LicenseRef-MyVa-Unreleased` cho nội dung dự án tự tạo khi chưa chọn license, `CC0-1.0`). Không nhập asset bên thứ ba khi chưa ghi nguồn và điều kiện phân phối (CONTRIBUTING).

Hash xác nhận toàn vẹn, không xác nhận nguồn: fixture được tải cùng origin. Chữ ký/root tin cậy khi tách kho phân phối thuộc D05-B và online.

## 5. Luật validator

Mã lỗi ổn định (`myva_assets::Code`). Test asset lỗi trong `crates/assets/tests/validator.rs` phủ phần lớn mã; `bad_id`, `texture_format`, `sheet_texture`, `bundle_unknown_asset` và `bundle_duplicate_asset` chưa có test riêng.

| Nhóm | Mã | Kiểm |
| --- | --- | --- |
| Định dạng | `json`, `format` | Parse được, trường lạ, format/version |
| ID, path | `bad_id`, `duplicate_id`, `bad_path` | ID ASCII thường; path tương đối, không `..`/tuyệt đối/`\`, đúng đuôi theo kind |
| File | `missing_file`, `unlisted_file`, `size_mismatch`, `hash_mismatch` | Có mặt, kích thước, SHA-256; file trong thư mục không khai báo |
| Dependency | `unknown_dependency`, `dependency_cycle`, `dependency_kind` | Tồn tại, không chu trình, đúng loại |
| PNG | `png_invalid`, `texture_format`, `texture_dims`, `texture_too_large` | Giải mã đủ tới IEND với CRC, RGBA8 không interlace, kích thước khớp manifest, cạnh ≤ limit |
| Frame | `sheet_texture`, `frame_bounds`, `frame_overlap`, `padding`, `frame_clipped`, `extrude` | Trong texture, không chồng, đủ padding; sprite có viền trong suốt (hình chạm mép ô là bị cắt); vòng extrude lặp đúng biên |
| Pivot, scale | `pivot`, `pivot_inconsistent`, `scale` | Pivot trong frame; nhân vật/boss cùng kích thước ô và pivot; `texels_per_unit` khớp mặt phẳng |
| Clip | `clip_frame`, `clip_missing`, `clip_event`, `clip_timing` | Frame tồn tại; clip bắt buộc theo role; mốc `hit`/`apex` hợp lệ, `hit` không ở frame 0; 16 ≤ `frame_ms` ≤ 1.000 |
| Bản đồ | `layout_ref` | Sheet là dependency, đúng role, frame tồn tại; lớp phông có `z < 0` để không che vùng báo |
| Bundle | `bundle_unknown_asset`, `bundle_duplicate_asset`, `bundle_orphan_asset`, `bundle_cross_reference`, `budget` | Sở hữu duy nhất, tham chiếu chéo đã khai báo, budget bundle và tổng |
| Nguồn | `provenance`, `license` | Trường bắt buộc và license đã duyệt |

Client chỉ chạy nhóm luật trên tài liệu (`check_documents`) và kiểm size/hash từng file; giải mã pixel để kiểm viền/extrude là việc của CLI và CI.

## 6. Loader runtime và fallback

- Loader đăng ký đuôi `json`; host tải `fixture/manifest.json`. Mỗi file đọc qua `LoadContext::read_asset_bytes`, nên hoạt động giống nhau trên native (file) và web (`fetch`).
- Texture giải mã bằng `Image::from_buffer`, sampler linear, sRGB, `RenderAssetUsages::RENDER_WORLD`: bản CPU bị bỏ sau upload. Texture là sub-asset gắn nhãn của fixture nên `LoadState::Loaded` đồng nghĩa mọi texture đã có trong `Assets<Image>`; chưa đồng nghĩa GPU upload xong.
- Bất kỳ lỗi nào (manifest thiếu, hash sai, PNG hỏng, luật tài liệu) làm cả fixture `Failed`. Client ghi `warn`, giữ cảnh hình khối và telemetry trả `art.status = "failed"` cùng mã (`io`, `hash_mismatch`, `size_mismatch`, `json`, `image`, hoặc mã validator). Trận không bị chặn vì art là cosmetic trong graybox; với asset thiết yếu ở slice, chính sách chặn ở điểm an toàn thuộc D05-B.
- Art chỉ thay khối thân, nền và đạn của người chơi. Vùng báo/vùng nguy hiểm của boss, thanh tiến trình báo đòn, viền đỡ, tường và hitbox debug luôn vẽ bằng hình khối phía trên phông.

## 7. Giới hạn đã biết

- Placeholder sinh bằng mã: đủ kiểm chứng pivot, padding, scale, clip và fallback; không chứng minh chất lượng art hay chi phí sản xuất bằng tay.
- Chưa có mipmap; ở màn 1280×720 DPR 1 sprite bị thu khoảng 2,6 lần với lọc tuyến tính. Cần thử mipmap/tier texture trước art slice.
- Một manifest tải trọn: bundle là đơn vị kế toán và ownership, chưa là đơn vị tải riêng. Planner D05-B nối với `AssetServer` sau.
- PNG thêm decoder vào WASM; số đo trong báo cáo D05-A.
