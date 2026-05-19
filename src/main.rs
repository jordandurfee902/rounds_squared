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
fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        std::env::set_var("WGPU_BACKEND", "dx12");
    }

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
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .insert_resource(net::IsNetworked(false))
        .insert_resource(net::LocalPlayerIndex(0))
        .insert_resource(net::RollbackRng::new(98765))
        .insert_resource(graphics::ScreenShake::default())
        .add_systems(Startup, (setup_camera, maximize_window))
        .add_systems(OnEnter(GameState::Matchmaking), net::start_matchmaking)
        .add_systems(Update, net::lobby_system.run_if(in_state(GameState::Matchmaking)))
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
