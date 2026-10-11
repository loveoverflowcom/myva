//! Thể hiện arena bằng hình khối. [`shapes`] là hàm thuần từ [`ArenaView`] (khung nhìn đọc từ
//! component mirror) ra danh sách hình, nên test được không cần GPU bằng snapshot của lõi; Bevy
//! chỉ chép danh sách vào một pool sprite mỗi khung hình. Popup trúng đòn đọc `SimEvent`.
//!
//! Đơn vị thế giới là pixel thiết kế (mili-pixel của lõi chia 1.000), trục y hướng lên, nền ở
//! y = 0. Vùng đòn của boss luôn hiện trong lúc báo và khi đánh: không có hitbox ẩn (combat.md
//! §13). Thanh tiến trình trên vùng báo cho biết còn bao lâu tới lúc đòn có hiệu lực.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use myva_gameplay::SimEvent;
use myva_sim::fighter::ARENA_WIDTH;
use myva_sim::kit::Rect as SimRect;
use myva_sim::snapshot::FighterView;
use myva_sim::{Event, FighterId, PX, Phase, State};

use crate::fight::{ArenaView, Match};

pub const BACKGROUND: Color = Color::srgb(0.11, 0.12, 0.15);
const FLOOR: Color = Color::srgb(0.17, 0.18, 0.22);
const WALL: Color = Color::srgb(0.07, 0.08, 0.1);
const GROUND_LINE: Color = Color::srgb(0.55, 0.57, 0.62);
const PLAYER: Color = Color::srgb(0.30, 0.56, 0.92);
const RIVAL: Color = Color::srgb(0.62, 0.64, 0.70);
const BOSS: Color = Color::srgb(0.58, 0.45, 0.34);
const STARTUP: Color = Color::srgb(1.0, 0.85, 0.2);
const ACTIVE: Color = Color::srgb(0.95, 0.25, 0.2);
const RECOVERY: Color = Color::srgb(0.95, 0.55, 0.15);
const GUARD: Color = Color::srgb(0.53, 0.81, 0.92);
const HITSTUN: Color = Color::srgb(0.85, 0.3, 0.85);
const GUARD_BREAK: Color = Color::srgb(0.55, 0.35, 0.85);
const DOWNED: Color = Color::srgb(0.3, 0.3, 0.33);
const HURTBOX: Color = Color::srgb(0.3, 0.9, 0.4);

/// Khung nhìn tối thiểu (px thiết kế): cả arena cùng lề, chỗ cho HUD phía trên và nút cảm ứng
/// phía dưới.
pub const VIEW_WIDTH: f32 = 1_680.0;
pub const VIEW_HEIGHT: f32 = 760.0;
const VIEW_CENTER_Y: f32 = 150.0;
const FLOOR_DEPTH: f32 = 400.0;
const OUTLINE: f32 = 3.0;
const POPUP_SECONDS: f32 = 0.9;

/// Hình chữ nhật đặc trong tọa độ thế giới.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub min: Vec2,
    pub max: Vec2,
    pub color: Color,
    pub z: f32,
}

/// Vai trò của hình, để test kiểm tra mà không phụ thuộc màu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Arena,
    Telegraph,
    TelegraphProgress,
    Danger,
    Projectile,
    Body,
    Detail,
    Debug,
}

impl Layer {
    const fn z(self) -> f32 {
        match self {
            Layer::Arena => 0.0,
            Layer::Telegraph => 1.0,
            Layer::TelegraphProgress => 1.1,
            Layer::Danger => 1.2,
            Layer::Projectile => 3.0,
            Layer::Body => 2.0,
            Layer::Detail => 2.5,
            Layer::Debug => 4.0,
        }
    }
}

#[derive(Default)]
pub struct Canvas {
    pub shapes: Vec<(Layer, Shape)>,
}

impl Canvas {
    fn fill(&mut self, layer: Layer, min: Vec2, max: Vec2, color: Color) {
        self.shapes.push((
            layer,
            Shape {
                min,
                max,
                color,
                z: layer.z(),
            },
        ));
    }

    fn rect(&mut self, layer: Layer, rect: SimRect, color: Color) {
        let (min, max) = world(rect);
        self.fill(layer, min, max, color);
    }

    fn outline(&mut self, layer: Layer, rect: SimRect, color: Color) {
        let (min, max) = world(rect);
        let t = OUTLINE;
        self.fill(layer, min, Vec2::new(max.x, min.y + t), color);
        self.fill(layer, Vec2::new(min.x, max.y - t), max, color);
        self.fill(layer, min, Vec2::new(min.x + t, max.y), color);
        self.fill(layer, Vec2::new(max.x - t, min.y), max, color);
    }
}

pub fn px(value: i32) -> f32 {
    value as f32 / PX as f32
}

fn world(rect: SimRect) -> (Vec2, Vec2) {
    (
        Vec2::new(px(rect.x0), px(rect.y0)),
        Vec2::new(px(rect.x1), px(rect.y1)),
    )
}

/// Vùng một đòn đang báo hoặc đang đánh: hộp đòn cận chiến, hoặc với đòn bắn đạn là chỗ đạn sinh
/// ra kéo dài hết tầm bay. `None` khi đòn không gây damage (ví dụ tư thế phản công).
pub fn threat_zone(fighter: &FighterView) -> Option<(SimRect, Phase, f32)> {
    let (action, spec) = fighter.action.zip(fighter.action_spec())?;
    let phase = action.phase;
    let progress = (action.elapsed as f32 / spec.startup.max(1) as f32).min(1.0);
    match spec.projectile {
        Some(projectile) if phase == Phase::Startup => {
            let spawn = projectile
                .hitbox
                .place(fighter.x, fighter.y, fighter.facing);
            let reach = projectile.speed * projectile.lifetime as i32;
            let lane = if fighter.facing > 0 {
                SimRect {
                    x1: (spawn.x1 + reach).min(ARENA_WIDTH),
                    ..spawn
                }
            } else {
                SimRect {
                    x0: (spawn.x0 - reach).max(0),
                    ..spawn
                }
            };
            Some((lane, phase, progress))
        }
        Some(_) => None,
        None if spec.damage > 0 && phase != Phase::Recovery => Some((
            spec.hitbox.place(fighter.x, fighter.y, fighter.facing),
            phase,
            progress,
        )),
        None => None,
    }
}

/// Toàn bộ hình của một khung hình; `player` là nhân vật người chơi điều khiển.
pub fn shapes(arena: &ArenaView, player: FighterId, show_boxes: bool, canvas: &mut Canvas) {
    let width = px(ARENA_WIDTH);
    canvas.fill(
        Layer::Arena,
        Vec2::new(-VIEW_WIDTH, -FLOOR_DEPTH),
        Vec2::new(0.0, VIEW_HEIGHT * 2.0),
        WALL,
    );
    canvas.fill(
        Layer::Arena,
        Vec2::new(width, -FLOOR_DEPTH),
        Vec2::new(width + VIEW_WIDTH, VIEW_HEIGHT * 2.0),
        WALL,
    );
    canvas.fill(
        Layer::Arena,
        Vec2::new(0.0, -FLOOR_DEPTH),
        Vec2::new(width, 0.0),
        FLOOR,
    );
    canvas.fill(
        Layer::Arena,
        Vec2::new(0.0, -2.0),
        Vec2::new(width, 0.0),
        GROUND_LINE,
    );

    for fighter in &arena.fighters {
        // Boss luôn báo vùng đòn; đòn của người chơi chỉ hiện khi bật hitbox.
        if fighter.kit.body.armored || show_boxes {
            telegraph(canvas, fighter);
        }
    }
    for projectile in &arena.projectiles {
        let color = if projectile.owner == player {
            PLAYER
        } else {
            STARTUP
        };
        canvas.rect(Layer::Projectile, projectile.rect, color);
        canvas.outline(Layer::Projectile, projectile.rect, Color::WHITE);
    }
    for fighter in &arena.fighters {
        body(canvas, fighter, base_color(player, fighter), show_boxes);
    }
}

fn telegraph(canvas: &mut Canvas, fighter: &FighterView) {
    let Some((zone, phase, progress)) = threat_zone(fighter) else {
        return;
    };
    match phase {
        Phase::Startup => {
            canvas.rect(Layer::Telegraph, zone, STARTUP.with_alpha(0.16));
            canvas.outline(Layer::Telegraph, zone, STARTUP.with_alpha(0.9));
            // Thanh đầy dần phía trên vùng: đầy là lúc đòn có hiệu lực.
            let (min, max) = self::world(zone);
            let filled = min.x + (max.x - min.x) * progress;
            canvas.fill(
                Layer::TelegraphProgress,
                Vec2::new(min.x, max.y + 6.0),
                Vec2::new(filled, max.y + 14.0),
                STARTUP,
            );
        }
        Phase::Active => {
            canvas.rect(Layer::Danger, zone, ACTIVE.with_alpha(0.35));
            canvas.outline(Layer::Danger, zone, ACTIVE);
        }
        Phase::Recovery => {}
    }
}

fn base_color(player: FighterId, fighter: &FighterView) -> Color {
    match (fighter.id == player, fighter.kit.body.armored) {
        (true, _) => PLAYER,
        (false, true) => BOSS,
        (false, false) => RIVAL,
    }
}

fn body(canvas: &mut Canvas, fighter: &FighterView, base: Color, show_boxes: bool) {
    let mut color = match (fighter.state, fighter.action.map(|a| a.phase)) {
        (_, Some(Phase::Startup)) => STARTUP,
        (_, Some(Phase::Active)) => ACTIVE,
        (_, Some(Phase::Recovery)) => RECOVERY,
        (State::Guard { .. }, _) => GUARD,
        (State::Hitstun { .. }, _) => HITSTUN,
        (State::GuardBreak { .. }, _) => GUARD_BREAK,
        (State::Downed, _) => DOWNED,
        _ => base,
    };
    if fighter.invulnerable && fighter.state != State::Downed {
        color = color.with_alpha(0.35);
    }
    let hurtbox = fighter.hurtbox;
    canvas.rect(Layer::Body, hurtbox, color);
    // Dải màu riêng ở chân giữ nhận diện hai bên khi cùng ra đòn.
    let (min, max) = world(hurtbox);
    canvas.fill(Layer::Detail, min, Vec2::new(max.x, min.y + 8.0), base);
    if let Some(perfect) = fighter.guard {
        let outline = if perfect { Color::WHITE } else { GUARD };
        let pad = 4 * PX;
        canvas.outline(
            Layer::Detail,
            SimRect {
                x0: hurtbox.x0 - pad,
                x1: hurtbox.x1 + pad,
                y0: hurtbox.y0 - pad,
                y1: hurtbox.y1 + pad,
            },
            outline,
        );
    }
    // Mắt chỉ hướng mặt.
    let eye_x = if fighter.facing > 0 {
        max.x - 12.0
    } else {
        min.x + 4.0
    };
    canvas.fill(
        Layer::Detail,
        Vec2::new(eye_x, max.y - 18.0),
        Vec2::new(eye_x + 8.0, max.y - 10.0),
        BACKGROUND,
    );
    if show_boxes {
        canvas.outline(Layer::Debug, hurtbox, HURTBOX);
        if let Some(attack) = fighter.attack_box {
            canvas.rect(Layer::Debug, attack, ACTIVE.with_alpha(0.35));
            canvas.outline(Layer::Debug, attack, ACTIVE);
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ShapePool>()
        .add_systems(Startup, spawn_camera)
        .add_systems(
            Update,
            (draw_shapes, name_tags, spawn_popups, animate_popups),
        );
}

#[derive(Component)]
pub struct ArenaCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        ArenaCamera,
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIEW_WIDTH,
                min_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(px(ARENA_WIDTH) / 2.0, VIEW_CENTER_Y, 0.0),
        // Cảnh chỉ có sprite màu phẳng, không cần LUT tonemapping.
        bevy::core_pipeline::tonemapping::Tonemapping::None,
    ));
    // Trận graybox luôn có hai bên: người chơi và đối thủ.
    for id in 0..2 {
        commands.spawn((
            NameTag(FighterId(id)),
            Text2d::new(""),
            TextFont::from_font_size(22.0),
            TextColor(Color::WHITE),
            bevy::sprite::Anchor::BOTTOM_CENTER,
            Transform::from_xyz(0.0, 0.0, 5.0),
        ));
    }
}

/// Sprite dùng lại giữa các khung hình; hình thừa bị ẩn thay vì despawn.
#[derive(Resource, Default)]
struct ShapePool {
    sprites: Vec<Entity>,
}

fn draw_shapes(
    mut commands: Commands,
    arena: Res<ArenaView>,
    game: Res<Match>,
    mut pool: ResMut<ShapePool>,
    mut sprites: Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
    mut canvas: Local<Canvas>,
) {
    canvas.shapes.clear();
    shapes(&arena, game.bout.player(), game.show_boxes, &mut canvas);
    while pool.sprites.len() < canvas.shapes.len() {
        let entity = commands
            .spawn((Sprite::default(), Transform::default(), Visibility::Hidden))
            .id();
        pool.sprites.push(entity);
    }
    for (index, &entity) in pool.sprites.iter().enumerate() {
        let Ok((mut sprite, mut transform, mut visibility)) = sprites.get_mut(entity) else {
            // Vừa spawn ở khung này; khung sau mới có component.
            continue;
        };
        match canvas.shapes.get(index) {
            Some((_, shape)) => {
                let size = shape.max - shape.min;
                let center = (shape.min + shape.max) / 2.0;
                sprite.color = shape.color;
                sprite.custom_size = Some(size);
                transform.translation = center.extend(shape.z + index as f32 * 1e-4);
                *visibility = Visibility::Inherited;
            }
            None => *visibility = Visibility::Hidden,
        }
    }
}

#[derive(Component)]
struct NameTag(FighterId);

fn name_tags(
    arena: Res<ArenaView>,
    game: Res<Match>,
    mut tags: Query<(&NameTag, &mut Text2d, &mut Transform, &mut TextColor)>,
) {
    for (tag, mut text, mut transform, mut color) in &mut tags {
        let Some(fighter) = arena.fighter(tag.0) else {
            text.0.clear();
            continue;
        };
        // Tên ngắn để hai tag không chồng nhau khi đứng sát; HUD ghi đủ truyền thừa.
        let label = game.name(fighter.id);
        if text.0 != label {
            text.0 = label.to_owned();
        }
        transform.translation = Vec3::new(px(fighter.x), px(fighter.hurtbox.y1) + 10.0, 5.0);
        color.0 = base_color(game.bout.player(), fighter);
    }
}

#[derive(Component)]
struct Popup {
    age: f32,
}

fn spawn_popups(
    mut commands: Commands,
    arena: Res<ArenaView>,
    mut events: MessageReader<SimEvent>,
) {
    for SimEvent(record) in events.read() {
        let (target, label, color) = match record.event {
            Event::Hit { target, damage, .. } => (target, format!("-{damage}"), Color::WHITE),
            Event::Blocked {
                target, perfect, ..
            } => (
                target,
                if perfect { "DO HOAN HAO" } else { "DO" }.to_owned(),
                GUARD,
            ),
            Event::Countered {
                attacker, damage, ..
            } => (attacker, format!("PHAN CONG -{damage}"), STARTUP),
            Event::GuardBroken { fighter } => (fighter, "VO THE DO".to_owned(), GUARD_BREAK),
            Event::Downed { fighter } => (fighter, "BI HA".to_owned(), ACTIVE),
            Event::ActionStarted { .. }
            | Event::InputRejected { .. }
            | Event::StatusApplied { .. }
            | Event::StatusEnded { .. }
            | Event::Interacted { .. } => continue,
        };
        let Some(fighter) = arena.fighter(target) else {
            continue;
        };
        commands.spawn((
            Popup { age: 0.0 },
            Text2d::new(label),
            TextFont::from_font_size(28.0),
            TextColor(color),
            Transform::from_xyz(px(fighter.x), px(fighter.hurtbox.y1) + 40.0, 6.0),
        ));
    }
}

fn animate_popups(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut popups: Query<(Entity, &mut Popup, &mut Transform, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut popup, mut transform, mut color) in &mut popups {
        popup.age += dt;
        if popup.age >= POPUP_SECONDS {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y += 60.0 * dt;
        color.0 = color.0.with_alpha(1.0 - popup.age / POPUP_SECONDS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myva_sim::battle::{Battle, Mode};
    use myva_sim::protocol::{Action, Attack, CommandFrame};

    fn layers(battle: &Battle, show_boxes: bool) -> Vec<Layer> {
        let mut canvas = Canvas::default();
        shapes(
            &ArenaView::of(battle),
            battle.bout().player(),
            show_boxes,
            &mut canvas,
        );
        canvas.shapes.iter().map(|(layer, _)| *layer).collect()
    }

    fn rival(battle: &Battle) -> FighterView {
        ArenaView::of(battle)
            .fighter(battle.bout().rival())
            .unwrap()
            .clone()
    }

    fn rival_phase(battle: &Battle) -> Option<Phase> {
        rival(battle).action.map(|a| a.phase)
    }

    fn press(action: Action) -> CommandFrame {
        CommandFrame {
            action: Some(action),
            ..CommandFrame::IDLE
        }
    }

    /// Chạy tới khi boss bắt đầu đòn đầu tiên.
    fn boss_winding_up() -> Battle {
        let mut battle = Battle::new(Mode::Boss, 1);
        battle.bout_mut().set_autopilot(true);
        while rival_phase(&battle) != Some(Phase::Startup) {
            battle.step(CommandFrame::IDLE);
        }
        battle
    }

    #[test]
    fn boss_startup_is_always_telegraphed_with_progress() {
        let battle = boss_winding_up();
        let drawn = layers(&battle, false);
        assert!(drawn.contains(&Layer::Telegraph));
        assert!(drawn.contains(&Layer::TelegraphProgress));
        assert!(!drawn.contains(&Layer::Debug), "hitbox debug tắt mặc định");
        let (zone, phase, progress) = threat_zone(&rival(&battle)).unwrap();
        assert_eq!(phase, Phase::Startup);
        assert!(progress < 0.1);
        assert!(zone.x1 > zone.x0);
    }

    #[test]
    fn telegraph_zone_matches_the_hitbox_that_lands() {
        let mut battle = boss_winding_up();
        let (zone, ..) = threat_zone(&rival(&battle)).unwrap();
        while rival_phase(&battle) != Some(Phase::Active) {
            battle.step(CommandFrame::IDLE);
        }
        assert_eq!(
            rival(&battle).attack_box,
            Some(zone),
            "vùng báo trùng hộp đòn thật"
        );
        assert!(layers(&battle, false).contains(&Layer::Danger));
    }

    #[test]
    fn player_attack_boxes_only_show_with_debug() {
        let mut battle = Battle::new(Mode::Duel, 1);
        battle.toggle_sparring_bot();
        battle.step(press(Action::Attack(Attack::Light)));
        assert!(!layers(&battle, false).contains(&Layer::Telegraph));
        let debug = layers(&battle, true);
        assert!(debug.contains(&Layer::Telegraph));
        assert!(debug.contains(&Layer::Debug));
    }

    #[test]
    fn projectile_telegraph_covers_its_flight_lane() {
        let mut battle = Battle::new(Mode::Duel, 1);
        battle.toggle_sparring_bot();
        battle.step(press(Action::Skill(0)));
        let arena = ArenaView::of(&battle);
        let player = arena.fighter(battle.bout().player()).unwrap();
        let (lane, phase, _) = threat_zone(player).unwrap();
        assert_eq!(phase, Phase::Startup);
        let spec = player.kit.skills[0].projectile.unwrap();
        assert_eq!(
            lane.x1 - lane.x0,
            spec.hitbox.width + spec.speed * spec.lifetime as i32
        );
    }
}
