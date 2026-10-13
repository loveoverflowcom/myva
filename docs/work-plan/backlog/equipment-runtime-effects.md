# Equipment runtime effects và replay contract

**Trạng thái:** deferred; phụ thuộc [equipment data/headless rules](../080-equipment-registry.md) và Rust verification thật. Issue #20 vẫn mở, #19 sở hữu luật combat.

## Why và outcome

Build hiện lock/validate được nhưng combat từ chối trước spawn. Muốn item có tác dụng thật cần authoritative action/hit identity và tích hợp cost/guard/proc phase; pure fixture functions chưa thay thế core.

## Review boundary

- Chính core gọi bounded rules, một source action không refund nhiều lần, proc không kích proc; cost dương, cooldown không reset, guard break/back/unblockable giữ ý nghĩa.
- Chốt replay version mới khi bổ sung item/proc state; giữ v3/v4 playback/golden hoặc explicit historical path, không fallback.
- Một luật duy nhất giữa World, Session, ECS/headless; resource/action/cooldown state khóa theo round, item changes chỉ khi safe state có luật.
- Rust/native/WASM parity và exploit invariants trước matchup/human playtest. Không tuyên bố balance từ formulas hoặc tổng win rate.

**Non-goals:** shop/inventory/production economy, online PvP/ranked, server entitlement, roster năm thế lực hoặc item khác để lấp catalog. Không thêm những việc này vào current PR.
