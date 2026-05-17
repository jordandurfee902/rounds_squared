use bevy::prelude::*;
use bevy::camera::Viewport;

pub const TARGET_WIDTH: f32 = 3840.0;
pub const TARGET_HEIGHT: f32 = 2160.0;
pub const ASPECT_RATIO: f32 = TARGET_WIDTH / TARGET_HEIGHT;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (fit_viewport, draw_border));
    }
}

fn fit_viewport(
    windows: Query<&Window>,
    mut query: Query<&mut Camera, With<Camera2d>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let window_width = window.width();
    let window_height = window.height();
    let window_aspect = window_width / window_height;

    let Some(mut camera) = query.iter_mut().next() else {
        return;
    };

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

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(x as u32, y as u32),
        physical_size: UVec2::new(width as u32, height as u32),
        depth: 0.0..1.0,
    });
}

fn draw_border(mut gizmos: Gizmos) {
    gizmos.rect_2d(
        Vec2::ZERO,
        Vec2::new(TARGET_WIDTH - 2.0, TARGET_HEIGHT - 2.0),
        Color::srgb(1.0, 0.0, 0.0),
    );
}
