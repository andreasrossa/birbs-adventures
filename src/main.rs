pub mod birb;
pub mod physics;
pub mod util;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use birb::*;
use physics::Acceleration;
use physics::Velocity;
use physics::*;
use rand::prelude::*;
use util::get_pipe_from_position_and_size;

#[derive(SystemLabel)]
enum SystemOrder {
    Physics,
    Input,
    Camera,
}

fn main() {
    App::new()
        .init_resource::<Game>()
        .init_resource::<PipeSpawnConfig>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin {
            enabled: true,
            always_on_top: true,
            ..Default::default()
        })
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .label(SystemOrder::Physics)
                .after(SystemOrder::Input)
                .with_system(gravity_system)
                .with_system(calculate_position_from_velocity),
        )
        .add_system_set(
            SystemSet::new()
                .label(SystemOrder::Input)
                .with_system(player_input),
        )
        .add_system(spawn_pipes)
        .add_system(despawn_pipes)
        .add_system(game_over.at_end())
        .add_system(rotate_birb.after(SystemOrder::Physics))
        .add_system(follow_cam.label(SystemOrder::Camera))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
    let camera = commands.spawn(Camera2dBundle::default()).id();

    let birb_transform = Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.2));
    let birb = commands
        .spawn(Birb)
        .insert(SpriteBundle {
            texture: asset_server.load("birb/blue/birb.png"),
            transform: birb_transform,
            ..default()
        })
        .insert(Velocity(Vec3::new(300.0, 0.0, 0.0)))
        .insert(Acceleration(Vec3::new(0.0, -300.0, 0.0)))
        .insert(Collider::cuboid(110.0, 100.0))
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::STATIC_STATIC)
        .insert(ActiveEvents::COLLISION_EVENTS)
        // .insert(Sensor)
        .id();

    game.player.entity = Some(birb);
    game.camera = Some(camera);
}

#[derive(Default)]
struct Player {
    entity: Option<Entity>,
}

#[derive(Resource, Default)]
struct Game {
    player: Player,
    camera: Option<Entity>,
}

#[derive(Default)]
pub enum PipePosition {
    #[default]
    Top,
    Bottom,
}

#[derive(Component, Default)]
struct Pipe {
    position: PipePosition,
}

impl Pipe {
    fn new(position: PipePosition) -> Self {
        Pipe { position }
    }
}

#[derive(Bundle, Default)]
pub struct PipeBundle {
    #[bundle]
    sprite: SpriteBundle,
    collider: Collider,
    _s: Sensor,
    pipe: Pipe,
}

fn game_over(mut events: EventReader<CollisionEvent>) {
    // if !events.is_empty() {
    //     println!("Collided!");
    // }
    for _event in events.iter() {
        panic!();
    }
}

fn player_input(mut query: Query<&mut Velocity>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        for mut velocity in query.iter_mut() {
            velocity.0.y = 400.0;
        }
    }
}

fn follow_cam(
    game: Res<Game>,
    mut transforms: ParamSet<(
        Query<&mut Transform, With<Camera>>,
        Query<&Transform, With<Birb>>,
    )>,
) {
    let mut player_pos: Option<Transform> = None;
    let player_transform_query = transforms.p1();
    if let Some(player_entity) = game.player.entity {
        if let Ok(player_transform) = player_transform_query.get(player_entity) {
            player_pos = Some(*player_transform);
        }
    }

    if let Some(player_pos_result) = player_pos {
        for mut camera_transform in transforms.p0().iter_mut() {
            camera_transform.translation.x = player_pos_result.translation.x;
        }
    }
}

#[derive(Resource)]
struct PipeSpawnConfig {
    timer: Timer,
}

impl Default for PipeSpawnConfig {
    fn default() -> Self {
        PipeSpawnConfig {
            timer: Timer::from_seconds(1.6, TimerMode::Repeating),
        }
    }
}

pub const PIPE_SPACING: f32 = 180.0;
pub const HALF_PIPE_SPACING: f32 = PIPE_SPACING / 2.0;
pub const HOLE_OFFSET: f32 = MIN_PIPE_HEIGHT + HALF_PIPE_SPACING;
pub const MIN_PIPE_HEIGHT: f32 = 50.0;
pub const PIPE_WIDTH: f32 = 100.0;
pub const HALF_PIPE_WIDTH: f32 = PIPE_WIDTH / 2.0;

/**
 * Spawn pipes shortly before they're about to come in view
 */
fn spawn_pipes(
    mut commands: Commands,
    query: Query<&Transform, With<Camera>>,
    time: Res<Time>,
    game: Res<Game>,
    windows: Res<Windows>,
    mut spawn_config: ResMut<PipeSpawnConfig>,
) {
    spawn_config.timer.tick(time.delta());

    if spawn_config.timer.just_finished() {
        if let Some(camera_entity) = game.camera {
            if let Ok(camera_transform) = query.get(camera_entity) {
                let window = windows.get_primary().unwrap();
                // Calculate center of hole:
                let consumable_space = window.height() - HOLE_OFFSET * 2.0;

                let mut rng = rand::thread_rng();
                let hole_center: f32 = HOLE_OFFSET + rng.gen::<f32>() * consumable_space;

                commands.spawn(get_pipe_from_position_and_size(
                    window.height() - (hole_center + HALF_PIPE_SPACING),
                    PipePosition::Top,
                    camera_transform.translation,
                    window.height(),
                    window.width(),
                ));

                commands.spawn(get_pipe_from_position_and_size(
                    hole_center - HALF_PIPE_SPACING,
                    PipePosition::Bottom,
                    camera_transform.translation,
                    window.height(),
                    window.width(),
                ));
            }
        }
    }
}

fn despawn_pipes(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    pipe_query: Query<(Entity, &Transform), With<Pipe>>,
    windows: Res<Windows>,
    game: Res<Game>,
) {
    for (entity, transform) in pipe_query.iter() {
        if let Some(window) = windows.get_primary() {
            if let Ok(camera_transform) = camera_query.get(game.camera.unwrap()) {
                // check if pipe is out of view
                // substract half of window width to get left side of camera (x of camera is in the middle of the window)
                if transform.translation.x
                    < camera_transform.translation.x - window.width() / 2.0 - HALF_PIPE_WIDTH
                {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
