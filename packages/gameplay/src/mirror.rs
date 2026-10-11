//! Ánh xạ snapshot của lõi sang entity/component Bevy.
//!
//! Mirror đọc [`Snapshot`], không đọc `World` trực tiếp, nên cùng hệ này sau này áp được
//! snapshot server gửi xuống client. Entity được tạo khi ID miền xuất hiện lần đầu và xóa khi ID
//! biến mất (đạn); ID miền không bao giờ tái sử dụng, còn handle `Entity` có thể được Bevy tái
//! cấp sau khi xóa, nên mọi tham chiếu qua mạng/replay dùng [`SimId`].

use std::collections::BTreeMap;

use bevy_ecs::prelude::*;
use myva_sim::EntityRef;
use myva_sim::projectile::ProjectileId;
use myva_sim::session::Role;
use myva_sim::snapshot::{FighterView, ProjectileView, Snapshot};

use crate::SimSession;
use crate::components::*;

/// Bảng hai chiều ID miền ↔ `Entity` của process này. Chỉ hệ mirror ghi.
#[derive(Resource, Debug, Default)]
pub struct EntityIndex {
    map: BTreeMap<EntityRef, Entity>,
}

impl EntityIndex {
    pub fn get(&self, id: EntityRef) -> Option<Entity> {
        self.map.get(&id).copied()
    }

    /// Theo thứ tự ID miền.
    pub fn iter(&self) -> impl Iterator<Item = (EntityRef, Entity)> + '_ {
        self.map.iter().map(|(&id, &entity)| (id, entity))
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Snapshot đã mirror lần trước, để chỉ thay component thật sự đổi.
#[derive(Resource, Debug, Default)]
pub(crate) struct MirrorCache {
    fighters: Vec<FighterView>,
    projectiles: BTreeMap<ProjectileId, ProjectileView>,
}

/// Xóa mọi entity mirror và bộ nhớ đệm, trước khi nạp phiên khác.
pub(crate) fn reset(world: &mut World) {
    let entities: Vec<Entity> = std::mem::take(&mut world.resource_mut::<EntityIndex>().map)
        .into_values()
        .collect();
    for entity in entities {
        world.despawn(entity);
    }
    *world.resource_mut::<MirrorCache>() = MirrorCache::default();
}

pub(crate) fn mirror_world(
    mut commands: Commands,
    session: Res<SimSession>,
    mut index: ResMut<EntityIndex>,
    mut cache: ResMut<MirrorCache>,
) {
    apply(
        &mut commands,
        &session.get().snapshot(),
        &mut index,
        &mut cache,
    );
}

fn apply(
    commands: &mut Commands,
    snapshot: &Snapshot,
    index: &mut EntityIndex,
    cache: &mut MirrorCache,
) {
    for view in &snapshot.fighters {
        let id = EntityRef::Fighter(view.id);
        match (index.get(id), cache.fighters.get(usize::from(view.id.0))) {
            (Some(entity), Some(old)) => update_fighter(&mut commands.entity(entity), old, view),
            _ => {
                let entity = spawn_fighter(commands, view);
                index.map.insert(id, entity);
            }
        }
    }
    cache.fighters.clone_from(&snapshot.fighters);

    for npc in &snapshot.npcs {
        let id = EntityRef::Npc(npc.id);
        if index.get(id).is_none() {
            let entity = commands
                .spawn((
                    SimId(id),
                    Npc,
                    NpcKind(npc.spec),
                    Position { x: npc.x, y: 0 },
                ))
                .id();
            index.map.insert(id, entity);
        }
    }

    let mut alive = BTreeMap::new();
    for view in &snapshot.projectiles {
        let id = EntityRef::Projectile(view.id);
        match (index.get(id), cache.projectiles.get(&view.id)) {
            (Some(entity), Some(old)) if old.rect != view.rect => {
                commands
                    .entity(entity)
                    .insert((projectile_position(view), Volume(view.rect)));
            }
            (Some(_), _) => {}
            (None, _) => {
                let entity = commands
                    .spawn((
                        SimId(id),
                        Projectile,
                        Owner(view.owner),
                        SourceAction(view.action),
                        Heading(view.dir),
                        projectile_position(view),
                        Volume(view.rect),
                    ))
                    .id();
                index.map.insert(id, entity);
            }
        }
        alive.insert(view.id, *view);
    }
    for gone in cache
        .projectiles
        .keys()
        .filter(|id| !alive.contains_key(id))
    {
        if let Some(entity) = index.map.remove(&EntityRef::Projectile(*gone)) {
            commands.entity(entity).despawn();
        }
    }
    cache.projectiles = alive;
}

fn projectile_position(view: &ProjectileView) -> Position {
    Position {
        x: view.rect.center_x(),
        y: view.rect.y0,
    }
}

fn spawn_fighter(commands: &mut Commands, view: &FighterView) -> Entity {
    let mut entity = commands.spawn((
        SimId(EntityRef::Fighter(view.id)),
        KitRef(view.kit),
        Faction(view.team),
        (
            Position {
                x: view.x,
                y: view.y,
            },
            Facing(view.facing),
            Health {
                hp: view.hp,
                max: view.max_hp,
            },
            Stamina(view.stamina),
            Energy(view.energy),
            Mach(view.mach),
        ),
        (
            Stance(view.state),
            CurrentAction(view.action),
            Cooldowns(view.cooldowns),
            StatusEffects(view.statuses.clone()),
            Hurtbox(view.hurtbox),
            AttackBox(view.attack_box),
            Invulnerable(view.invulnerable),
            Grounded(view.grounded),
            GuardWindow(view.guard),
            AckedSeq(view.last_seq),
        ),
    ));
    match view.role {
        Role::Player => entity.insert(Player),
        Role::Monster => entity.insert(Monster),
    };
    if let Some(ai) = view.ai {
        entity.insert(Ai(ai));
    }
    entity.id()
}

/// Thay component nào có giá trị khác lần mirror trước.
fn update_fighter(entity: &mut EntityCommands, old: &FighterView, new: &FighterView) {
    macro_rules! sync {
        ($($changed:expr => $component:expr),* $(,)?) => {
            $(if $changed { entity.insert($component); })*
        };
    }
    sync! {
        (old.x, old.y) != (new.x, new.y) => Position { x: new.x, y: new.y },
        old.facing != new.facing => Facing(new.facing),
        old.hp != new.hp => Health { hp: new.hp, max: new.max_hp },
        old.stamina != new.stamina => Stamina(new.stamina),
        old.energy != new.energy => Energy(new.energy),
        old.mach != new.mach => Mach(new.mach),
        old.state != new.state => Stance(new.state),
        old.action != new.action => CurrentAction(new.action),
        old.cooldowns != new.cooldowns => Cooldowns(new.cooldowns),
        old.statuses != new.statuses => StatusEffects(new.statuses.clone()),
        old.hurtbox != new.hurtbox => Hurtbox(new.hurtbox),
        old.attack_box != new.attack_box => AttackBox(new.attack_box),
        old.invulnerable != new.invulnerable => Invulnerable(new.invulnerable),
        old.grounded != new.grounded => Grounded(new.grounded),
        old.guard != new.guard => GuardWindow(new.guard),
        old.last_seq != new.last_seq => AckedSeq(new.last_seq),
    }
    if old.ai != new.ai
        && let Some(ai) = new.ai
    {
        entity.insert(Ai(ai));
    }
}
