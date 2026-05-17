mod physics;
mod player;
mod graphics;
mod settings;
mod map;
pub mod maps;
mod net;

use bevy::prelude::*;
use bevy::camera::*;
use physics::*;
use player::*;
use graphics::*;
use settings::*;
use map::*;
use bevy_ggrs::RollbackApp;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SETS".into(),
                canvas: Some("#bevy-canvas".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((SettingsPlugin, PhysicsPlugin, PlayerPlugin, GraphicsPlugin, MapPlugin))
        .add_plugins(bevy_ggrs::GgrsPlugin::<net::GgrsConfig>::default())
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .insert_resource(net::IsNetworked(false))
        .insert_resource(net::LocalPlayerIndex(0))
        .insert_resource(net::RollbackRng::new(12345))
        .add_systems(Startup, (setup_camera, maximize_window))
        .add_systems(OnEnter(GameState::Matchmaking), net::start_matchmaking)
        .add_systems(Update, net::lobby_system.run_if(in_state(GameState::Matchmaking)))
        .rollback_component_with_copy::<Transform>()
        .rollback_component_with_copy::<Velocity>()
        .rollback_component_with_copy::<Acceleration>()
        .rollback_component_with_copy::<Mass>()
        .rollback_component_with_copy::<Friction>()
        .rollback_component_with_copy::<Restitution>()
        .rollback_component_with_copy::<Grounded>()
        .rollback_component_with_copy::<WallContact>()
        .rollback_component_with_copy::<ControllerInput>()
        .rollback_component_with_copy::<JumpAllowance>()
        .rollback_component_with_copy::<Health>()
        .rollback_component_with_copy::<PlayerStatsComponent>()
        .rollback_component_with_copy::<BlockComponent>()
        .rollback_component_with_clone::<crate::physics::weapon::Projectile>()
        .rollback_resource_with_copy::<net::RollbackRng>()
        .rollback_resource_with_copy::<crate::settings::ScoreTracker>()
        .rollback_resource_with_copy::<crate::maps::ActiveMap>()
        .rollback_resource_with_clone::<crate::physics::card_selection::CardSelectionState>()
        .rollback_resource_with_clone::<crate::settings::PersistentPlayerStats>()
        .run();
}

use bevy::window::PrimaryWindow;

fn maximize_window(mut q: Query<&mut Window, With<PrimaryWindow>>) {
    if let Some(mut window) = q.iter_mut().next() {
        window.set_maximized(true);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: TARGET_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
