# PR1 — Domain thế lực và trực hệ

**Trạng thái:** đã triển khai trên `feat/faction-lineage-domain`, chưa merge; xem [boundary](../technical/faction-lineage-catalog.md) và [evidence](../reports/faction-lineage-domain.md). Liên quan #18/#20, base develop sau PR26.

## Kết quả và lý do

Catalog Rust thuần với identity, provenance, trạng thái duyệt và validator giúp consumer tiếp theo phân biệt thế lực, nguồn, trực hệ, nhánh và kit. Team combat được mirror thành `CombatTeam`. Long Lưu vẫn fixture chưa ánh xạ; không nhập mười trực hệ đề xuất.

## Phạm vi review

Validator/identity, metadata không vào combat hash, replay legacy giữ nguyên, mirror/query dùng Team độc lập. Không thêm crate, parser, skill/damage rules, kit mới, economy hoặc UI. Gate browser/WASM chưa đủ prerequisite không được coi PASS.

## Bàn giao

Giữ nguyên branch/worktree cho phiên tiếp theo. [PR2](060-kit-registry-loadout.md) tạo nhánh từ head PR1 khi chưa merge; không merge PR1 để tạo dependency. Không tự chạy phiên2, không đóng issue.
