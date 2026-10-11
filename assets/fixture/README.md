# Fixture asset D05-A

Placeholder hoạt hình viền đậm, màu phẳng để kiểm chứng pipeline 2D; **không phải art thành phẩm**. Đặc tả: [asset-pipeline](../../docs/technical/asset-pipeline.md).

| Bundle | Nội dung |
| --- | --- |
| `character.long_luu` | Long Lưu: `idle` 4, `run` 6, `jump` 3 (mốc `apex`), `attack` 5 (mốc `hit`) |
| `boss.ke_giu_dap` | Kẻ Giữ Đập: `idle` 4, `run` 4, `attack` 5 (mốc `hit`) |
| `map.dap_co` | Đập Cổ: tile mặt nền/thân nền, phông trời, đồi, thân đập, sậy và bố cục |
| `vfx.combat` | `hit`, `guard`, `slash`, `dust`, `bolt` (đạn Lưu Tiễn) |

**Nguồn gốc và quyền:** mọi file được sinh bởi [`scripts/generate-asset-fixture.py`](../../scripts/generate-asset-fixture.py) từ mã nguồn trong repo; không dùng ảnh, font hay dữ liệu bên thứ ba. Tác giả: những người đóng góp MyVa — Thần Mạch. License `LicenseRef-MyVa-Unreleased`: chủ dự án chưa chọn license phát hành, chỉ dùng trong repo này. Từng file có provenance, kích thước và SHA-256 trong `manifest.json`.

Không sửa tay PNG/JSON: sửa script, chạy lại rồi chạy `cargo run -p myva-assets -- validate assets/fixture/manifest.json`.
