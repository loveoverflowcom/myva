use std::cell::{Cell, RefCell};

// Bevy 0.20 derives inspect top-level dependencies only. Our target-specific
// dependency needs this documented alias for generated `bevy_ecs` paths.
use bevy::{camera::ScalingMode, ecs as bevy_ecs, prelude::*, winit::WinitSettings};
use wasm_bindgen::prelude::*;

use crate::state::{ARENA_HEIGHT, ARENA_WIDTH, GameState};

thread_local! {
    static STATE: RefCell<GameState> = RefCell::new(GameState::default());
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

/// Start exactly once per iframe. The iframe owns teardown of the WASM runtime.
#[wasm_bindgen]
pub fn start_game() {
    if STARTED.with(|started| started.replace(true)) {
        return;
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(10, 17, 32)))
        .insert_resource(WinitSettings::continuous())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MyVa · Bevy WASM spike".into(),
                canvas: Some("#game-canvas".into()),
                resolution: (640, 360).into(),
                fit_canvas_to_parent: true,
                // JS prevents only movement keys; Tab and browser shortcuts stay usable.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

/// Direction uses game coordinates: positive x is right, positive y is up.
#[wasm_bindgen]
pub fn set_input(x: f32, y: f32) {
    STATE.with_borrow_mut(|state| state.set_input(x, y));
}

#[wasm_bindgen]
pub fn set_paused(paused: bool) {
    STATE.with_borrow_mut(|state| state.set_paused(paused));
}

#[wasm_bindgen]
pub fn reset_game() {
    STATE.with_borrow_mut(GameState::reset);
}

/// A primitive JSON bridge keeps the prototype independent of shell internals.
#[wasm_bindgen]
pub fn telemetry() -> String {
    STATE.with_borrow(GameState::telemetry)
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Target;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: ARENA_WIDTH,
                min_height: ARENA_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
        // Tonemapping is disabled because this sprite-only build needs no LUTs.
        bevy::core_pipeline::tonemapping::Tonemapping::None,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb_u8(63, 89, 120), Vec2::new(640.0, 360.0)),
        Transform::from_xyz(0.0, 0.0, -3.0),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(17, 31, 48), Vec2::new(624.0, 344.0)),
        Transform::from_xyz(0.0, 0.0, -2.0),
    ));
    for x in (-280..=280).step_by(40) {
        commands.spawn((
            Sprite::from_color(Color::srgb_u8(25, 43, 61), Vec2::new(1.0, 344.0)),
            Transform::from_xyz(x as f32, 0.0, -1.0),
        ));
    }
    for y in (-160..=160).step_by(40) {
        commands.spawn((
            Sprite::from_color(Color::srgb_u8(25, 43, 61), Vec2::new(624.0, 1.0)),
            Transform::from_xyz(0.0, y as f32, -1.0),
        ));
    }

    commands
        .spawn((
            Player,
            Sprite::from_color(Color::srgb_u8(65, 224, 188), Vec2::new(24.0, 28.0)),
            Transform::from_xyz(-220.0, 0.0, 2.0),
        ))
        .with_children(|player| {
            for x in [-5.0, 5.0] {
                player.spawn((
                    Sprite::from_color(Color::srgb_u8(7, 39, 47), Vec2::new(4.0, 5.0)),
                    Transform::from_xyz(x, 5.0, 0.1),
                ));
                player.spawn((
                    Sprite::from_color(Color::srgb_u8(27, 128, 139), Vec2::new(8.0, 6.0)),
                    Transform::from_xyz(x, -11.0, 0.1),
                ));
            }
        });

    commands
        .spawn((
            Target,
            Sprite::from_color(Color::srgb_u8(251, 184, 65), Vec2::splat(22.0)),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .with_children(|target| {
            for size in [Vec2::new(8.0, 32.0), Vec2::new(32.0, 8.0)] {
                target.spawn((
                    Sprite::from_color(Color::srgb_u8(255, 220, 104), size),
                    Transform::from_xyz(0.0, 0.0, 0.1),
                ));
            }
        });
}

fn update(
    time: Res<Time>,
    mut player: Single<&mut Transform, (With<Player>, Without<Target>)>,
    mut target: Single<&mut Transform, (With<Target>, Without<Player>)>,
) {
    let state = STATE.with_borrow_mut(|state| {
        state.step(time.delta_secs());
        *state
    });
    player.translation.x = state.x;
    player.translation.y = state.y;
    let position = state.target();
    target.translation.x = position.0;
    target.translation.y = position.1;
}
