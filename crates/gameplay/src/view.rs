//! Đọc lại component mirror thành khung nhìn snapshot của lõi, cho presentation.
//!
//! Client vẽ và hiện HUD từ [`FighterView`]/[`ProjectileView`] lấy từ entity, không đọc `World`;
//! cùng hàm vẽ nhận được snapshot dựng trong test hoặc gửi từ server. Kết quả trùng
//! `Session::snapshot()` sau mỗi lần mirror.

use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use myva_sim::EntityRef;
use myva_sim::session::Role;
use myva_sim::snapshot::{FighterView, ProjectileView};

use crate::components::*;

#[derive(QueryData)]
pub struct FighterMirror {
    id: &'static SimId,
    kit: &'static KitRef,
    faction: &'static Faction,
    position: &'static Position,
    facing: &'static Facing,
    health: &'static Health,
    stamina: &'static Stamina,
    energy: &'static Energy,
    mach: &'static Mach,
    stance: &'static Stance,
    action: &'static CurrentAction,
    cooldowns: &'static Cooldowns,
    statuses: &'static StatusEffects,
    hurtbox: &'static Hurtbox,
    attack_box: &'static AttackBox,
    invulnerable: &'static Invulnerable,
    grounded: &'static Grounded,
    guard: &'static GuardWindow,
    acked: &'static AckedSeq,
    player: Has<Player>,
    ai: Option<&'static Ai>,
}

impl FighterMirrorItem<'_, '_> {
    pub fn view(&self) -> FighterView {
        let EntityRef::Fighter(id) = self.id.0 else {
            unreachable!("entity nhân vật luôn mang ID nhân vật");
        };
        FighterView {
            id,
            role: if self.player {
                Role::Player
            } else {
                Role::Monster
            },
            kit: self.kit.0,
            team: self.faction.0,
            x: self.position.x,
            y: self.position.y,
            facing: self.facing.0,
            hp: self.health.hp,
            max_hp: self.health.max,
            stamina: self.stamina.0,
            energy: self.energy.0,
            mach: self.mach.0,
            state: self.stance.0,
            action: self.action.0,
            cooldowns: self.cooldowns.0,
            statuses: self.statuses.0.clone(),
            hurtbox: self.hurtbox.0,
            attack_box: self.attack_box.0,
            invulnerable: self.invulnerable.0,
            grounded: self.grounded.0,
            guard: self.guard.0,
            last_seq: self.acked.0,
            ai: self.ai.map(|ai| ai.0),
        }
    }
}

#[derive(QueryData)]
pub struct ProjectileMirror {
    id: &'static SimId,
    owner: &'static Owner,
    action: &'static SourceAction,
    volume: &'static Volume,
    heading: &'static Heading,
}

impl ProjectileMirrorItem<'_, '_> {
    pub fn view(&self) -> ProjectileView {
        let EntityRef::Projectile(id) = self.id.0 else {
            unreachable!("entity đạn luôn mang ID đạn");
        };
        ProjectileView {
            id,
            owner: self.owner.0,
            action: self.action.0,
            rect: self.volume.0,
            dir: self.heading.0,
        }
    }
}

/// Mọi nhân vật theo `FighterId` tăng dần, như `Snapshot::fighters`.
pub fn fighters(query: &Query<FighterMirror>) -> Vec<FighterView> {
    let mut views: Vec<_> = query.iter().map(|item| item.view()).collect();
    views.sort_by_key(|view| view.id);
    views
}

/// Mọi viên đạn theo `ProjectileId` tăng dần, như `Snapshot::projectiles`.
pub fn projectiles(query: &Query<ProjectileMirror, With<Projectile>>) -> Vec<ProjectileView> {
    let mut views: Vec<_> = query.iter().map(|item| item.view()).collect();
    views.sort_by_key(|view| view.id);
    views
}
