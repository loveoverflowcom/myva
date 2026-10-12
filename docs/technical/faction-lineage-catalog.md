# Catalog thế lực và trực hệ

**Trạng thái:** PR1 domain metadata, 2026-10-12; phạm vi theo [ADR 0004](../decisions/0004-five-factions-lineage-hierarchy.md), [lineages](../design/lineages.md), [#18](https://github.com/loveoverflowcom/myva/issues/18) và [#20](https://github.com/loveoverflowcom/myva/issues/20). Không duyệt roster hoặc balance.

`packages/sim/src/catalog.rs` là module Rust thuần trong `myva-sim`, không có dependency mới. Consumer hiện tại là kiểm tra identity của `Kit` legacy và các fixture validator. PR1 không có parser/registry/loadout; [PR2](kit-registry-loadout.md) bổ sung registry thực thi, quyền trang bị, loadout và pins replay, dùng validator này.

## Identity và phân tầng

| Tầng | Identity | Cạnh tới tầng cha |
| --- | --- | --- |
| Nền tảng thế lực | `FactionId`: `christianity`, `islam`, `hinduism`, `buddhism`, `taoism` | Năm nền tảng được chọn; không phải tên tổ chức trong game |
| Nguồn | `SourceId`, `SourceRole` | `Source.faction` |
| Trực hệ | `LineageId`, `Transmission` | `Lineage.source`; `via` tùy chọn cho nguồn trung gian cùng nền tảng |
| Nhánh chiến đấu | `BranchId` | `Branch.lineage` |
| Kit metadata | `KitId` | `KitPlacement::Branch`, hoặc `UnmappedFixture` cho gameplay legacy |

Các ID ngoài `FactionId` là slug ASCII lowercase/digit, nối bằng một dấu `-`, không rỗng, tối đa 64 byte, namespace riêng theo kiểu. Không chuẩn hóa nhãn thành ID, không dùng thứ tự array/discriminant enum làm identity. Catalog PR1 được author bằng dữ liệu tĩnh; constructors nhận `&'static str`. Parser/owned data nếu consumer PR2 cần phải được thiết kế riêng, không leak chuỗi vào static lifetime.

`BASELINE_CATALOG` chỉ chứa năm nền tảng đã chốt và kit `long-luu` ở trạng thái fixture chưa ánh xạ. Sources/lineages/branches đều rỗng. Mười ứng viên trong tài liệu chưa được nhập làm runtime roster. `Kit::catalog_id()` đọc ID hiện hữu, không thêm field hoặc sửa hash của kit. Nó không xác nhận kit có trong catalog hoặc đã có implementation.

Bản đồ/linh vực/quê hương/chính trị không nằm trong hierarchy. `myva_sim::Team` và component immutable `myva_gameplay::components::CombatTeam` chỉ biểu diễn đồng minh/địch trong trận; không lấy giá trị từ `FactionId`. Component cũ tên `Faction` đã được đổi tên ở mirror/query cùng consumer, không có alias gây nhầm hai khái niệm.

## Metadata và validator

Mỗi record có display label, `ReviewStatus` riêng và provenance DK (`SourceFact`), ST (`GameFiction`) hoặc GT (`GameplayHypothesis`). Duyệt phải ghi decision, người quyết định và ngày; duyệt nền tảng không duyệt nguồn/trực hệ/nhánh/kit. Validator kiểm cấu trúc và bằng chứng được điền, không xác minh quyết định thật hay nội dung nguồn ngoài mạng.

`Catalog::validate()` trả tất cả `ValidationError` theo thứ tự duyệt ổn định:

- ID trùng ở từng tầng, thiếu một trong năm nền tảng, cạnh tham chiếu không tồn tại.
- Metadata/provenance/approval bị bỏ trống; nguồn thiếu DK, quan hệ trực hệ thiếu ST.
- Child được duyệt trong khi parent hoặc nguồn trung gian chưa duyệt.
- Nguồn trung gian trỏ chính nguồn hoặc sang nền tảng khác.
- Quan hệ không có trong allow-list của nguồn/nguồn trung gian; danh sách quan hệ rỗng.
- Huyết thống với nguồn thần độc thần, thiên thần, Phật, Bồ-tát hoặc pháp hệ; truyền pháp với role không thuộc Phật/Bồ-tát/pháp hệ; role thần độc thần gắn ngoài hai nền tảng tương ứng.

Allow-list phải được hồ sơ từng nguồn xác định. Các guard trên không thay review DK/ST theo truyền thống; validator không đọc tên để suy ra thần học. Các fixture synthetic trong tests không phải đề xuất roster.

`Catalog::hierarchy(KitId)` validate trước lookup và trả `Result`. ID lạ trả `UnknownKit`; Long Lưu mặc định trả `UnmappedFixture`; dữ liệu sai trả `InvalidCatalog`. Không fallback sang tôn giáo/kit khác và không panic. Kết quả metadata của một kit proposed không phải quyền trang bị, duyệt roster hay xác nhận implementation; [Registry/loadout PR2](kit-registry-loadout.md) có gate riêng cho implementation, quyền trang bị và version/hash.

## Compatibility và bàn giao

Không đưa metadata vào `Kit`, `Fighter`, `World`, snapshot, protocol, replay hoặc hash. Chuỗi `Kit.lineage` và `ActionSpec.name` legacy vẫn tham gia hash; consumer presentation không sửa chúng để đổi nhãn. Luật damage/skill/tick 60 Hz/hitbox/guard/Mạch, economy và replay semantics giữ nguyên.

Regression `packages/sim/tests/catalog_compatibility.rs` khóa trận boss B1 round 1 từ develop `e4bdba46a5750ab623c6fac1be120e5835fc093a`: 3.763 tick, 63 checkpoint, hash cuối `227d609f34dcd9e4`; digest toàn bộ replay text `5c27a4f922c21eb2` (179.328 byte). Bản ghi baseline có SHA-256 `b9a13a910849b28879d7d92ac4068ac1b6c8239234c7950837889da7d76112dd`. Test thay label/provenance catalog và đối chiếu replay từng byte. Test adapter bảo vệ `CombatTeam`, `None`, metadata khác thế lực/cùng đội và cùng thế lực/khác đội; parity hiện có tiếp tục so từng tick và mirror snapshot.

Gates và giới hạn của lần chạy này ở [báo cáo PR1](../reports/faction-lineage-domain.md). [PR2](../work-plan/060-kit-registry-loadout.md) sở hữu data-driven registry/loadout/version; [PR3](../work-plan/070-lineage-presentation.md) sở hữu metadata UI/asset mapping.
