use bevy::prelude::*;
use bevy::camera::Viewport;

pub const TARGET_WIDTH: f32 = 3840.0;
pub const TARGET_HEIGHT: f32 = 2160.0;
pub const ASPECT_RATIO: f32 = TARGET_WIDTH / TARGET_HEIGHT;

#[derive(Resource, Default, Debug, Clone)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
}

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (fit_viewport, draw_border, apply_screen_shake));
    }
}

pub fn apply_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let Some(mut transform) = query.iter_mut().next() else {
        return;
    };
    if shake.duration > 0.0 {
        shake.duration -= time.delta_secs();
        if shake.duration <= 0.0 {
            shake.intensity = 0.0;
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        } else {
            let t = time.elapsed_secs() * 65.0;
            let dx = (t.sin() * 0.7 + (t * 1.7).cos() * 0.3) * shake.intensity;
            let dy = ((t * 1.3).cos() * 0.7 + (t * 0.9).sin() * 0.3) * shake.intensity;
            transform.translation.x = dx;
            transform.translation.y = dy;
        }
    } else {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

fn fit_viewport(
    windows: Query<&Window>,
    mut query: Query<&mut Camera, With<Camera2d>>,
    state: Res<State<crate::settings::GameState>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let Some(mut camera) = query.iter_mut().next() else {
        return;
    };

    if *state.get() == crate::settings::GameState::MainMenu || *state.get() == crate::settings::GameState::Lobby {
        camera.viewport = None;
        return;
    }

    let window_width = window.width();
    let window_height = window.height();
    let window_aspect = window_width / window_height;

    let (width, height, x, y) = if window_aspect > ASPECT_RATIO {
        // Pillarboxing (window is wider than 16:9)
        let height = window_height;
        let width = height * ASPECT_RATIO;
        let x = (window_width - width) / 2.0;
        (width, height, x, 0.0)
    } else {
        // Letterboxing (window is taller than 16:9)
        let width = window_width;
        let height = width / ASPECT_RATIO;
        let y = (window_height - height) / 2.0;
        (width, height, 0.0, y)
    };

    let scale_factor = window.scale_factor();
    let phys_width = (width * scale_factor) as u32;
    let phys_height = (height * scale_factor) as u32;
    let phys_x = (x * scale_factor) as u32;
    let phys_y = (y * scale_factor) as u32;

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(phys_x, phys_y),
        physical_size: UVec2::new(phys_width, phys_height),
        depth: 0.0..1.0,
    });
}

fn draw_border(
    mut gizmos: Gizmos,
    state: Res<State<crate::settings::GameState>>,
) {
    if *state.get() == crate::settings::GameState::MainMenu || *state.get() == crate::settings::GameState::Lobby {
        return;
    }
    // Only draw red borders on the left and right sides
    let half_w = TARGET_WIDTH / 2.0 - 1.0;
    let half_h = TARGET_HEIGHT / 2.0;
    
    // Left border line
    gizmos.line_2d(
        Vec2::new(-half_w, -half_h),
        Vec2::new(-half_w, half_h),
        Color::srgb(1.0, 0.0, 0.0),
    );
    
    // Right border line
    gizmos.line_2d(
        Vec2::new(half_w, -half_h),
        Vec2::new(half_w, half_h),
        Color::srgb(1.0, 0.0, 0.0),
    );
}
