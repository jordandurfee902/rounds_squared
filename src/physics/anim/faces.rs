use bevy::prelude::*;
use crate::player::{Health, PlayerStatsComponent};
use crate::physics::components::{Velocity, Grounded, WallContact};
use crate::settings::PhysicsSettings;
use super::components::PlayerAim;

/// Dynamically renders black pixel cartoon eyes and tilting eyebrows that express emotions based on stats.
pub fn draw_expressive_faces(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &PlayerAim, &Health, &Velocity, &Grounded, &WallContact, &PlayerStatsComponent)>,
    settings: Res<PhysicsSettings>,
) {
    for (transform, aim, health, velocity, grounded, wall, stats) in query.iter() {
        let scale = stats.player_scale;
        let body_pos = transform.translation.xy();
        let visual_center = body_pos + Vec2::new(0.0, settings.player_aim_offset_y * scale);
        let aim_dir = aim.direction;
        let hp_pct = health.current / health.max;
        let speed = velocity.0.length();
        
        // Offset face forward in direction of aim, and offset vertically slightly
        let face_center = visual_center + aim_dir * (18.0 * scale) + Vec2::new(0.0, 3.0 * scale);
        
        // Force facial features to stay horizontal and upright (side-by-side) regardless of aiming angles!
        let eye_spacing = 11.0 * scale;
        let left_eye = face_center - Vec2::new(eye_spacing, 0.0);
        let right_eye = face_center + Vec2::new(eye_spacing, 0.0);
        
        // Dynamic expression triggers
        let is_panicked = hp_pct < 0.4 || (!grounded.0 && velocity.0.y < -900.0);
        let is_angry = speed > 450.0 || wall.left || wall.right;
        
        let color = Color::srgb(0.05, 0.05, 0.05); // Pure black eyes
        
        // Render Eyes
        draw_eye(&mut gizmos, left_eye, is_panicked, is_angry, color, scale);
        draw_eye(&mut gizmos, right_eye, is_panicked, is_angry, color, scale);
        
        // Render Eyebrows
        draw_eyebrow(&mut gizmos, left_eye, true, is_panicked, is_angry, color, scale);
        draw_eyebrow(&mut gizmos, right_eye, false, is_panicked, is_angry, color, scale);
        
        // Render Cyan Tears if player has critically low health (< 40%)
        if hp_pct < 0.4 {
            let tear_color = Color::srgb(0.2, 0.65, 0.95);
            gizmos.line_2d(left_eye - Vec2::new(0.0, 4.0 * scale), left_eye - Vec2::new(0.0, 20.0 * scale), tear_color);
            gizmos.line_2d(right_eye - Vec2::new(0.0, 4.0 * scale), right_eye - Vec2::new(0.0, 20.0 * scale), tear_color);
        }
    }
}

/// Pill-shaped black eye renderer.
pub fn draw_eye(
    gizmos: &mut Gizmos,
    center: Vec2,
    panicked: bool,
    angry: bool,
    color: Color,
    scale: f32,
) {
    let mut height = 8.0 * scale;
    let mut width = 3.0 * scale;
    
    if panicked {
        height = 14.0 * scale;
        width = 4.0 * scale;
    } else if angry {
        height = 4.0 * scale;
        width = 4.0 * scale;
    }
    
    let steps = (width.round() as i32).max(1);
    for w in 0..steps {
        let offset_x = if steps > 1 {
            (w as f32 / (steps - 1) as f32 - 0.5) * width
        } else {
            0.0
        };
        gizmos.line_2d(
            center + Vec2::new(offset_x, -height / 2.0),
            center + Vec2::new(offset_x, height / 2.0),
            color,
        );
    }
}

/// Dynamic, responsive cartoon eyebrow renderer.
pub fn draw_eyebrow(
    gizmos: &mut Gizmos,
    eye_center: Vec2,
    is_left: bool,
    panicked: bool,
    angry: bool,
    color: Color,
    scale: f32,
) {
    let brow_center = eye_center + Vec2::new(0.0, 8.0 * scale);
    let half_width = 5.0 * scale;
    
    let mut tilt = 0.0;
    if angry {
        tilt = if is_left { -3.5 * scale } else { 3.5 * scale };
    } else if panicked {
        tilt = if is_left { 3.5 * scale } else { -3.5 * scale };
    }
    
    let start = brow_center + Vec2::new(-half_width, -tilt);
    let end = brow_center + Vec2::new(half_width, tilt);
    
    gizmos.line_2d(start, end, color);
    gizmos.line_2d(start + Vec2::new(0.0, 1.0 * scale), end + Vec2::new(0.0, 1.0 * scale), color);
}
