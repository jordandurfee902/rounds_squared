mod physics;
mod player;
mod graphics;
mod settings;
mod map;
pub mod maps;

use bevy::prelude::*;
use bevy::camera::*;
use physics::*;
use player::*;
use graphics::*;
use settings::*;
use map::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SETS".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((SettingsPlugin, PhysicsPlugin, PlayerPlugin, GraphicsPlugin, MapPlugin))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .add_systems(Startup, (setup_camera, maximize_window))
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
