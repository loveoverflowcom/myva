# Đóng góp cho MyVa — Thần Mạch

## Nhánh và review

- Nhánh tích hợp và mặc định: `develop`.
- Tạo nhánh công việc từ `develop`, ví dụ `docs/combat-playtest` hoặc `prototype/native-renderer`.
- PR lấy `develop` làm base; mô tả kết quả người chơi, thay đổi luật và bằng chứng kiểm chứng.
- Không đổi tên Thần Mạch, tạo nhánh `main` hoặc thay stack mà không nêu quyết định rõ ràng.

## Tài liệu

- Mỗi tài liệu chuyên môn có trạng thái draft/đã kiểm chứng, điều kiện thử và giới hạn phạm vi.
- Thay một thông số dùng chung phải sửa tất cả tài liệu phụ thuộc trong cùng PR.
- Giữ Markdown UTF-8, tên file ASCII, link tương đối cho tài liệu nội bộ.
- Tuyên bố kỹ thuật có nguồn chính thức; kết quả đo có device/build/config và ngày thực hiện.
- Tên truyền thuyết có thật cần nguồn theo đúng cộng đồng; tên do dự án tạo phải ghi là hư cấu.
- Không tự chọn license mã nguồn hoặc nhập asset chưa rõ quyền sử dụng. Ghi nguồn, author, license và điều kiện tái phân phối trước commit asset.

## Khi bắt đầu code

Triển khai công việc ưu tiên trong [work-plan](docs/work-plan/README.md). Chọn ranh giới module từ prototype thực tế; giữ gameplay simulation không phụ thuộc renderer và client không sở hữu kinh tế online. Không mở rộng roster hoặc service ngoài consumer đang được kiểm chứng.

Theo [cấu trúc repository](docs/technical/repository-layout.md), đặt entry platform trong `apps`, luật và thư viện dùng chung trong `packages`. `sim` giữ combat/session/protocol/replay; `gameplay` chỉ adapter ECS gọi core; `economy` giữ ledger/budget, reward online do server xác nhận. Shell web sở hữu DOM/navigation/IME, Bevy sở hữu canvas/render/game input. Giữ `prototypes/native` đến gate Android/iOS thật; chỉ tách desktop app, contract hoặc asset package khi có consumer và bằng chứng cần thiết trong [backlog boundaries](docs/work-plan/backlog/repository-boundaries.md).

Các test nên bảo vệ luật hoặc lỗi có thể gây mất state, nhân đôi reward, sai budget và trải nghiệm điều khiển. Không chạy benchmark trên placeholder rồi gọi đó là năng lực của game thành phẩm.

## Kiểm tra tài liệu

Developer và agent dùng [`cargo xtask`](xtask/README.md) theo [build contract](docs/architecture/xtask-build-contract.md), nguồn duy nhất về Single Surface Rule, command mapping, process ownership và verification selection. Repository chưa có agent instructions/skills riêng; không tạo thêm bản sao quy tắc. Scripts là implementation primitives, npm `test:web` và CI forward về xtask.

```bash
cargo xtask fmt
cargo xtask verify xtask             # khi sửa tooling
cargo xtask preflight --changed-only # base mặc định origin/develop
cargo xtask preflight                # cùng full contract với cargo xtask ci
```

`verify repository` kiểm inline links, ảnh và heading fragments trong README, CONTRIBUTING, `docs/`, README prototype/xtask, cùng Rust/asset/report contracts. Checker không truy cập mạng hoặc xác nhận nội dung nguồn bên ngoài. Thiếu tool/browser/target phải báo BLOCKED và nonzero; không coi dry-run hoặc test bị bỏ qua là PASS. Xem [bằng chứng xtask](docs/reports/xtask-verification.md) cho môi trường đã chạy.

Khi sửa Mermaid, chạy parser/render Mermaid riêng; link checker không xác nhận cú pháp diagram. Ví dụ dùng Mermaid CLI đã cài: trích nội dung một fence `mermaid` vào file `.mmd`, rồi chạy `mmdc -i diagram.mmd -o diagram.svg` và xem kết quả. ADR 0003 và sơ đồ ownership architecture đã được parse bằng Mermaid `12.1.0`; việc parse không thay thế review ownership trong sơ đồ.
