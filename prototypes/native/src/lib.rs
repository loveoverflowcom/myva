//! Standalone native baseline for issue #10. This is not a CMP adapter.
use bevy::{input::touch::TouchPhase, prelude::*, window::AppLifecycle, winit::WinitSettings};

#[derive(Component)]
struct Player;

#[derive(Resource, Default)]
struct Drag {
    pointer: Option<u64>,
    target: Option<Vec2>,
}

/// GameActivity owns the Android native app; Bevy owns its single event loop.
#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MyVa native standalone probe — drag to move".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.025, 0.045, 0.075)))
        .insert_resource(WinitSettings::mobile())
        .init_resource::<Drag>()
        .add_systems(Startup, setup)
        .add_systems(Update, (read_touch, read_lifecycle, move_player).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Msaa::Off));
    commands.spawn((
        Player,
        Sprite::from_color(Color::srgb(0.2, 0.9, 0.7), Vec2::splat(64.0)),
        Transform::default(),
    ));
    info!("MYVA_NATIVE protocol=1 standalone=true ready");
}

fn read_touch(
    mut touches: MessageReader<TouchInput>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut drag: ResMut<Drag>,
) {
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    for touch in touches.read() {
        match touch.phase {
            TouchPhase::Started if drag.pointer.is_none() => drag.pointer = Some(touch.id),
            TouchPhase::Ended | TouchPhase::Canceled if drag.pointer == Some(touch.id) => {
                drag.pointer = None;
                drag.target = None;
                info!("MYVA_NATIVE pointer_released phase={:?}", touch.phase);
            }
            _ => {}
        }
        if drag.pointer == Some(touch.id) {
            drag.target = camera
                .viewport_to_world_2d(camera_transform, touch.position)
                .ok();
        }
    }
}

fn read_lifecycle(mut events: MessageReader<AppLifecycle>, mut drag: ResMut<Drag>) {
    for event in events.read() {
        info!("MYVA_NATIVE lifecycle={event:?}");
        if matches!(event, AppLifecycle::WillSuspend | AppLifecycle::Suspended) {
            drag.pointer = None;
            drag.target = None;
        }
    }
}

fn move_player(time: Res<Time>, drag: Res<Drag>, mut player: Query<&mut Transform, With<Player>>) {
    let (Some(target), Ok(mut transform)) = (drag.target, player.single_mut()) else {
        return;
    };
    let delta = target - transform.translation.truncate();
    let step = delta.clamp_length_max(320.0 * time.delta_secs());
    transform.translation += step.extend(0.0);
}
