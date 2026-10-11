//! NPC không chiến đấu: chỉ có vị trí và tầm tương tác.
//!
//! Mô phỏng chỉ xác nhận "nhân vật này tương tác với NPC kia ở tick này"; hội thoại, nhiệm vụ và
//! phần thưởng thuộc server nghiệp vụ (architecture.md §2, §7), không do client quyết định.

use crate::kit::PX;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NpcId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NpcSpec {
    /// Mã ASCII ổn định dùng trong replay và cấu hình.
    pub id: &'static str,
    pub name: &'static str,
    /// Khoảng cách ngang tối đa giữa tâm nhân vật và tâm NPC, mili-pixel.
    pub reach: i32,
}

/// NPC hướng dẫn ở phòng thử (hư cấu, GT).
pub const GUIDE: NpcSpec = NpcSpec {
    id: "huong-dan",
    name: "Người hướng dẫn",
    reach: 80 * PX,
};

pub const NPCS: [&NpcSpec; 1] = [&GUIDE];

pub fn npc_by_id(id: &str) -> Option<&'static NpcSpec> {
    NPCS.into_iter().find(|spec| spec.id == id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Npc {
    pub id: NpcId,
    pub spec: &'static NpcSpec,
    pub x: i32,
}
