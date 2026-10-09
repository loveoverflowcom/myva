# Kiểm chứng bể tài nguyên và luật phát hành

## Why

Giới hạn tồn lượng không tự bảo đảm giá ổn định hay phân phối công bằng. Simulator phải tìm starvation, dao động và loophole trước người chơi có inventory thật.

## Scope

Simulator độc lập cho 2 tài nguyên thường, 1 hiếm và một tiền tệ. Kiểm tra bảo toàn, ngân sách giờ/ngày, cấp độ hiệu dụng, tái chế, onboarding, alt/bot, tích trữ và số channel.

## Non-goals

Không làm auction UI, chống bot production, real-money economy hoặc kết luận thị trường từ một seed.

## Dependencies

[Economy](../design/economy.md), [QA](../production/qa.md). Chốt định nghĩa hữu hạn tồn lượng so với trần tổng lượt tái phát hành trước chạy kịch bản.

## Suggested sequence

1. Mô hình ledger và chuyển trạng thái, đối soát mỗi bước.
2. Mô hình cohort solo/new/guild/alt và nhu cầu tiêu dùng; chạy nhiều seed với shocks.
3. Báo cáo invariant, khả năng tiếp cận, thời gian kiếm/craft và độ nhạy tham số; sửa thiết kế.

## Review boundary

Không đơn vị nào bị tạo ngoài bể hoặc ghi hai lần; thêm shard không tăng budget. Báo cáo cả trường hợp thất bại và những giả thuyết không được bảo đảm.

## Risks / unknowns

Kích thước bể có thể thiếu cho onboarding; hiếm không tái chế sẽ cạn; controller trễ có thể starvation; hành vi mô phỏng chưa phản ánh người thật.

## Follow-ups

Prototype transaction/crash và telemetry trước trade online; hiệu chỉnh từ pilot, giữ invariant bất biến.
