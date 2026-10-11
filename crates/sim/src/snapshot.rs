//! Snapshot tối thiểu cho presentation, mirror ECS và hòa giải mạng (architecture.md §5).
//!
//! Snapshot chỉ là khung nhìn: không khôi phục được `World` từ nó (thiếu buffer, tập mục tiêu đã
//! trúng, bộ đếm hồi tài nguyên). Lưu/khôi phục trong process dùng `World: Clone`; định dạng lưu
//! bền vững và snapshot mạng đầy đủ là việc của prototype online.

use crate::fighter::{Fighter, FighterId, State, Team};
use crate::kit::{ActionKind, Kit, Phase, Rect};
use crate::meter::Meter;
use crate::monster::AiState;
use crate::npc::{NpcId, NpcSpec};
use crate::projectile::ProjectileId;
use crate::protocol::PROTOCOL_VERSION;
use crate::session::Role;
use crate::status::StatusEffect;
use crate::tick::Tick;
use crate::world::World;

/// Thanh tài nguyên theo đơn vị con (`SUB_PER_POINT` mỗi điểm).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Gauge {
    pub sub: u32,
    pub max_sub: u32,
}

impl Gauge {
    pub const fn of(meter: &Meter) -> Self {
        Self {
            sub: meter.sub(),
            max_sub: meter.max_sub(),
        }
    }

    pub const fn points(&self) -> u32 {
        self.sub / crate::meter::SUB_PER_POINT
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionView {
    pub action: ActionKind,
    pub phase: Phase,
    pub elapsed: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FighterView {
    pub id: FighterId,
    pub role: Role,
    pub kit: &'static Kit,
    pub team: Option<Team>,
    pub x: i32,
    pub y: i32,
    pub facing: i8,
    pub hp: u32,
    pub max_hp: u32,
    pub stamina: Gauge,
    pub energy: Gauge,
    pub mach: Gauge,
    pub state: State,
    pub action: Option<ActionView>,
    pub cooldowns: [u32; 3],
    pub statuses: Vec<StatusEffect>,
    pub hurtbox: Rect,
    pub attack_box: Option<Rect>,
    pub invulnerable: bool,
    /// `seq` lớn nhất đã áp dụng, để client bỏ input đã được xác nhận khi hòa giải.
    pub last_seq: Option<u32>,
    pub ai: Option<AiState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileView {
    pub id: ProjectileId,
    pub owner: FighterId,
    pub action: ActionKind,
    pub rect: Rect,
    pub dir: i8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpcView {
    pub id: NpcId,
    pub spec: &'static NpcSpec,
    pub x: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub protocol: u16,
    /// Số tick đã chạy.
    pub tick: Tick,
    pub hash: u64,
    /// Theo `FighterId` tăng dần.
    pub fighters: Vec<FighterView>,
    /// Theo `ProjectileId` tăng dần.
    pub projectiles: Vec<ProjectileView>,
    pub npcs: Vec<NpcView>,
}

impl Snapshot {
    /// `meta` trả vai trò và trạng thái AI của mỗi nhân vật; `World` không biết hai điều này.
    pub fn capture(world: &World, meta: impl Fn(FighterId) -> (Role, Option<AiState>)) -> Self {
        let mut projectiles: Vec<_> = world
            .projectiles()
            .iter()
            .map(|p| ProjectileView {
                id: p.id,
                owner: p.owner,
                action: p.action,
                rect: p.rect,
                dir: p.dir,
            })
            .collect();
        projectiles.sort_by_key(|p| p.id);
        Self {
            protocol: PROTOCOL_VERSION,
            tick: world.tick(),
            hash: world.state_hash(),
            fighters: world
                .fighters()
                .iter()
                .map(|f| {
                    let (role, ai) = meta(f.id);
                    view(f, role, ai)
                })
                .collect(),
            projectiles,
            npcs: world
                .npcs()
                .iter()
                .map(|n| NpcView {
                    id: n.id,
                    spec: n.spec,
                    x: n.x,
                })
                .collect(),
        }
    }
}

fn view(f: &Fighter, role: Role, ai: Option<AiState>) -> FighterView {
    FighterView {
        id: f.id,
        role,
        kit: f.kit,
        team: f.team,
        x: f.x,
        y: f.y,
        facing: f.facing,
        hp: f.hp,
        max_hp: f.kit.body.max_hp,
        stamina: Gauge::of(&f.stamina),
        energy: Gauge::of(&f.energy),
        mach: Gauge::of(&f.mach),
        state: f.state,
        action: match (f.state, f.action()) {
            (State::Attack { elapsed, .. }, Some((action, _, phase))) => Some(ActionView {
                action,
                phase,
                elapsed,
            }),
            _ => None,
        },
        cooldowns: f.cooldowns,
        statuses: f.statuses.as_slice().to_vec(),
        hurtbox: f.hurtbox(),
        attack_box: f.attack_box(),
        invulnerable: f.is_invulnerable(),
        last_seq: f.last_seq(),
        ai,
    }
}
