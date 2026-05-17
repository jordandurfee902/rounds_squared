use bevy::prelude::*;
use crate::player::Player;
use crate::physics::components::{Grounded, Velocity, WallContact};
use crate::player::Health;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};

// --- COMPONENTS & ENUMS ---

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct PlayerAim {
    pub direction: Vec2, // Normalized aim direction vector
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FootState {
    Planted { position: Vec2 },
    Stepping {
        start: Vec2,
        target: Vec2,
        progress: f32, // 0.0 to 1.0 lerp progress
    },
    Airborne { current: Vec2 },
}

#[derive(Component, Debug)]
pub struct ProceduralLimbs {
    pub left_foot: FootState,
    pub right_foot: FootState,
    pub step_cooldown: f32, // alternating step timer
}

impl Default for ProceduralLimbs {
    fn default() -> Self {
        Self {
            left_foot: FootState::Airborne { current: Vec2::ZERO },
            right_foot: FootState::Airborne { current: Vec2::ZERO },
            step_cooldown: 0.0,
        }
    }
}

// --- SYSTEMS ---

/// Converts screen mouse coordinates (P1) or IJKL keys / movement velocity (P2) into a normalized aim vector.
pub fn update_aim(
    windows: Query<&Window>,
    mut query: Query<(&Player, &Transform, &Velocity, &mut PlayerAim)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    
    // Physical cursor position in pixel coordinates (origin is top-left)
    let cursor_pos = window.cursor_position();
    
    let window_width = window.width();
    let window_height = window.height();
    let window_aspect = window_width / window_height;
    
    let target_aspect = TARGET_WIDTH / TARGET_HEIGHT;
    
    // Solve exact letterboxing or pillarboxing offsets to translate to virtual 1920x1080 bounds
    let (vp_width, vp_height, vp_x, vp_y) = if window_aspect > target_aspect {
        let height = window_height;
        let width = height * target_aspect;
        let x = (window_width - width) / 2.0;
        (width, height, x, 0.0)
    } else {
        let width = window_width;
        let height = width / target_aspect;
        let y = (window_height - height) / 2.0;
        (width, height, 0.0, y)
    };

    for (player, transform, velocity, mut aim) in query.iter_mut() {
        match player {
            Player::P1 => {
                if let Some(cursor) = cursor_pos {
                    // Coordinates relative to the scaled viewport
                    let rx = cursor.x - vp_x;
                    let ry = cursor.y - vp_y;
                    
                    // Normalize to [0.0, 1.0] range
                    let u = rx / vp_width;
                    let v = ry / vp_height;
                    
                    // Scale into our camera's virtual space (-TARGET_WIDTH/2 to TARGET_WIDTH/2)
                    let world_x = (u - 0.5) * TARGET_WIDTH;
                    let world_y = (0.5 - v) * TARGET_HEIGHT;
                    
                    let player_pos = transform.translation.xy() + Vec2::new(0.0, 25.0);
                    let target_dir = Vec2::new(world_x, world_y) - player_pos;
                    if target_dir.length_squared() > 1.0 {
                        aim.direction = target_dir.normalize();
                    }
                }
            }
            Player::P2 => {
                // Shared keyboard IJKL aiming mapping
                let mut key_dir = Vec2::ZERO;
                if keys.pressed(KeyCode::KeyI) { key_dir.y += 1.0; }
                if keys.pressed(KeyCode::KeyK) { key_dir.y -= 1.0; }
                if keys.pressed(KeyCode::KeyJ) { key_dir.x -= 1.0; }
                if keys.pressed(KeyCode::KeyL) { key_dir.x += 1.0; }
                
                if key_dir.length_squared() > 0.1 {
                    aim.direction = key_dir.normalize();
                } else {
                    // Fall back to moving velocity direction, or face right (X) if stationary
                    let vel = velocity.0;
                    if vel.length_squared() > 10.0 {
                        aim.direction = vel.normalize();
                    } else if aim.direction == Vec2::ZERO {
                        aim.direction = Vec2::X;
                    }
                }
            }
        }
    }
}

/// Dynamic walk cycle stepping solver for circular characters, drawing thick noodle leg Bezier segments.
pub fn update_and_draw_legs(
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut query: Query<(
        &Transform,
        &Velocity,
        &Grounded,
        &Player,
        &mut ProceduralLimbs,
    )>,
) {
    let dt = time.delta_secs().min(0.05); // cap delta time to safeguard against frame lag spikes
    
    for (transform, velocity, grounded, player, mut limbs) in query.iter_mut() {
        let body_pos = transform.translation.xy();
        let vel = velocity.0;
        
        let visual_center = body_pos + Vec2::new(0.0, 25.0);
        
        // Offset hips relative to player's visual center (lifted higher so legs are extended upright!)
        let left_hip = visual_center + Vec2::new(-16.0, -15.0);
        let right_hip = visual_center + Vec2::new(16.0, -15.0);
        
        let color = match player {
            Player::P1 => Color::srgb(0.1, 0.35, 0.8), // Deep P1 Blue
            Player::P2 => Color::srgb(0.8, 0.35, 0.1), // Deep P2 Orange
        };
        
        if limbs.step_cooldown > 0.0 {
            limbs.step_cooldown -= dt;
        }
        
        let ground_y = body_pos.y - 40.0;
        
        // 1. Process left foot stride
        limbs.left_foot = process_foot(
            dt,
            left_hip,
            vel,
            grounded.0,
            limbs.left_foot,
            limbs.right_foot,
            &mut limbs.step_cooldown,
            -16.0,
            ground_y,
        );
        
        // 2. Process right foot stride
        limbs.right_foot = process_foot(
            dt,
            right_hip,
            vel,
            grounded.0,
            limbs.right_foot,
            limbs.left_foot,
            &mut limbs.step_cooldown,
            16.0,
            ground_y,
        );
        
        // Segment lengths: 32px upper thigh, 28px lower shin
        let l1 = 32.0;
        let l2 = 28.0;
        
        // Draw Left Noodle Leg
        let left_current = get_foot_pos(limbs.left_foot);
        let left_knee = solve_ik_2d(left_hip, left_current, l1, l2, false);
        draw_connected_noodle(&mut gizmos, left_hip, left_knee, left_current, color);
        
        // Draw Right Noodle Leg
        let right_current = get_foot_pos(limbs.right_foot);
        let right_knee = solve_ik_2d(right_hip, right_current, l1, l2, true);
        draw_connected_noodle(&mut gizmos, right_hip, right_knee, right_current, color);
    }
}

/// Solves the weapon arms, aiming a gun with the dominant hand, and holding a floating white shield/orb with the supporting hand.
pub fn draw_procedural_arms(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &PlayerAim, &Player)>,
) {
    for (transform, aim, player) in query.iter() {
        let body_pos = transform.translation.xy();
        let visual_center = body_pos + Vec2::new(0.0, 25.0);
        let aim_dir = aim.direction;
        
        // Arm colors
        let color = match player {
            Player::P1 => Color::srgb(0.2, 0.5, 1.0),
            Player::P2 => Color::srgb(1.0, 0.5, 0.2),
        };
        
        // --- 1. WEAPON & DOMINANT ARM (AIMING SIDE) ---
        let gun_center = visual_center + aim_dir * 66.0;
        let normal = Vec2::new(-aim_dir.y, aim_dir.x); // perpendicular normal vector
        
        // Draw elegant minimal weapon shapes
        let barrel_start = visual_center + aim_dir * 52.0;
        let barrel_end = visual_center + aim_dir * 84.0;
        gizmos.line_2d(barrel_start, barrel_end, Color::srgb(0.75, 0.75, 0.75));
        
        // Flip the gun stock handle dynamically so it always points downwards (gravity-upright)
        let grip_sign = if aim_dir.x >= 0.0 { -1.0 } else { 1.0 };
        let stock_end = barrel_start + normal * (grip_sign * 12.0);
        gizmos.line_2d(barrel_start, stock_end, Color::srgb(0.55, 0.55, 0.55));
        
        // Dominant hand anchor holding the gun stock
        let dominant_hand = gun_center + normal * (grip_sign * 2.0);
        
        // --- 2. FLOATING WHITE ORB & SUPPORTING ARM (OPPOSITE SIDE) ---
        // Make the orb stay horizontally fixed on the opposite side instead of rotating with the aim vector!
        let shield_offset_x = if aim_dir.x >= 0.0 { -38.0 } else { 38.0 };
        let shield_center = visual_center + Vec2::new(shield_offset_x, 0.0);
        
        // Draw solid floating white circle opposite to aim direction
        for r in 0..=10 {
            gizmos.circle_2d(shield_center, r as f32, Color::srgb(1.0, 1.0, 1.0));
        }
        
        // Supporting hand anchor holding the floating white circle
        let supporting_hand = shield_center;
        
        // --- 3. SHOULDER & INVERSE KINEMATICS ---
        let left_shoulder = visual_center + Vec2::new(-20.0, 0.0);
        let right_shoulder = visual_center + Vec2::new(20.0, 0.0);
        
        let arm_l1 = 34.0;
        let arm_l2 = 30.0;
        
        // Draw Dominant Hand Arm (Aim Side - swaps between left/right automatically)
        let dom_shoulder = if aim_dir.x >= 0.0 { right_shoulder } else { left_shoulder };
        let dom_elbow = solve_ik_2d(dom_shoulder, dominant_hand, arm_l1, arm_l2, aim_dir.x < 0.0);
        draw_connected_noodle(&mut gizmos, dom_shoulder, dom_elbow, dominant_hand, color);
        
        // Draw Supporting Hand Arm (Opposite Floating Orb Side - swaps between left/right automatically)
        let sup_shoulder = if aim_dir.x >= 0.0 { left_shoulder } else { right_shoulder };
        let sup_elbow = solve_ik_2d(sup_shoulder, supporting_hand, arm_l1, arm_l2, aim_dir.x >= 0.0);
        draw_connected_noodle(&mut gizmos, sup_shoulder, sup_elbow, supporting_hand, color);
    }
}

/// Dynamically renders black pixel cartoon eyes and tilting eyebrows that express emotions based on stats.
pub fn draw_expressive_faces(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &PlayerAim, &Health, &Velocity, &Grounded, &WallContact)>,
) {
    for (transform, aim, health, velocity, grounded, wall) in query.iter() {
        let body_pos = transform.translation.xy();
        let visual_center = body_pos + Vec2::new(0.0, 25.0);
        let aim_dir = aim.direction;
        let hp_pct = health.current / health.max;
        let speed = velocity.0.length();
        
        // Offset face forward in direction of aim, and offset vertically slightly
        let face_center = visual_center + aim_dir * 18.0 + Vec2::new(0.0, 3.0);
        
        // Force facial features to stay horizontal and upright (side-by-side) regardless of aiming angles!
        let eye_spacing = 11.0;
        let left_eye = face_center - Vec2::new(eye_spacing, 0.0);
        let right_eye = face_center + Vec2::new(eye_spacing, 0.0);
        
        // Dynamic expression triggers
        let is_panicked = hp_pct < 0.4 || (!grounded.0 && velocity.0.y < -900.0);
        let is_angry = speed > 450.0 || wall.left || wall.right;
        
        let color = Color::srgb(0.05, 0.05, 0.05); // Pure black eyes
        
        // Render Eyes
        draw_eye(&mut gizmos, left_eye, is_panicked, is_angry, color);
        draw_eye(&mut gizmos, right_eye, is_panicked, is_angry, color);
        
        // Render Eyebrows
        draw_eyebrow(&mut gizmos, left_eye, true, is_panicked, is_angry, color);
        draw_eyebrow(&mut gizmos, right_eye, false, is_panicked, is_angry, color);
        
        // Render Cyan Tears if player has critically low health (< 40%)
        if hp_pct < 0.4 {
            let tear_color = Color::srgb(0.2, 0.65, 0.95);
            gizmos.line_2d(left_eye - Vec2::new(0.0, 4.0), left_eye - Vec2::new(0.0, 20.0), tear_color);
            gizmos.line_2d(right_eye - Vec2::new(0.0, 4.0), right_eye - Vec2::new(0.0, 20.0), tear_color);
        }
    }
}

// --- UTILITY MATH FUNCTIONS ---

/// Performs a standard 2-joint 2D IK calculation, returning the intermediate elbow/knee coordinate.
pub fn solve_ik_2d(start: Vec2, target: Vec2, l1: f32, l2: f32, flip: bool) -> Vec2 {
    let to_target = target - start;
    let dist = to_target.length();
    
    // Avoid numerical collapse when targets exceed standard bounds
    let max_reach = l1 + l2;
    let clamped_dist = dist.clamp(0.01, max_reach - 0.1);
    let clamped_target = if dist > 0.0 {
        start + to_target.normalize() * clamped_dist
    } else {
        start + Vec2::new(0.0, -clamped_dist)
    };
    
    let to_clamped = clamped_target - start;
    let d = to_clamped.length();
    
    let cos_a = (l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d);
    let angle_a = cos_a.clamp(-1.0, 1.0).acos();
    
    let base_angle = to_clamped.y.atan2(to_clamped.x);
    let angle = if flip {
        base_angle + angle_a
    } else {
        base_angle - angle_a
    };
    
    start + Vec2::new(angle.cos(), angle.sin()) * l1
}

/// Translates foot states into real-time horizontal and vertical positions.
fn get_foot_pos(state: FootState) -> Vec2 {
    match state {
        FootState::Planted { position } => position,
        FootState::Stepping { start, target, progress } => {
            let t = progress.clamp(0.0, 1.0);
            let horizontal = start.lerp(target, t);
            let arc_height = 20.0 * (4.0 * t * (1.0 - t)); // parabolic lift arc
            horizontal + Vec2::new(0.0, arc_height)
        }
        FootState::Airborne { current } => current,
    }
}

/// Solves continuous foot walking strides and airborne dangling trailing physics.
fn process_foot(
    dt: f32,
    hip_pos: Vec2,
    velocity: Vec2,
    grounded: bool,
    current_state: FootState,
    other_foot_state: FootState,
    cooldown: &mut f32,
    side_offset: f32,
    ground_y: f32,
) -> FootState {
    if !grounded {
        // Airborne trailing dangling leg physics
        let current_pos = match current_state {
            FootState::Planted { position } => position,
            FootState::Stepping { start, target, progress } => {
                let t = progress.clamp(0.0, 1.0);
                start.lerp(target, t)
            }
            FootState::Airborne { current } => current,
        };
        
        let target_offset = Vec2::new(
            side_offset - velocity.x * 0.015, // gravity effect: stay more directly under player when moving horizontally
            -45.0 - velocity.y * 0.01,        // reduced elasticity: stretch less when jumping or falling
        ).clamp_length_max(48.0);             // keep target well within physical reach to preserve organic bend
        let target_pos = hip_pos + target_offset;
        
        let new_pos = current_pos + (target_pos - current_pos) * dt * 10.0;
        return FootState::Airborne { current: new_pos };
    }
    
    // Grounded stepping cycles
    match current_state {
        FootState::Airborne { current } => {
            FootState::Planted { position: current }
        }
        FootState::Planted { position } => {
            let dist_h = hip_pos.x - position.x;
            let max_stride = 50.0;
            let other_is_stepping = matches!(other_foot_state, FootState::Stepping { .. });
            
            if dist_h.abs() > max_stride && !other_is_stepping && *cooldown <= 0.0 {
                let step_lead = 0.16;
                let step_target_x = hip_pos.x + velocity.x * step_lead + side_offset * 0.5;
                let step_target = Vec2::new(step_target_x, ground_y); // Lands perfectly on the floor
                
                *cooldown = 0.08; // alternate strides
                
                FootState::Stepping {
                    start: position,
                    target: step_target,
                    progress: 0.0,
                }
            } else {
                FootState::Planted { position }
            }
        }
        FootState::Stepping { start, target, mut progress } => {
            progress += dt * 8.0; // steps take ~120ms
            if progress >= 1.0 {
                FootState::Planted { position: target }
            } else {
                FootState::Stepping { start, target, progress }
            }
        }
    }
}

/// Renders a thick, satisfying 3D-esque noodle segment by sweeping connected Bezier lines.
fn draw_connected_noodle(
    gizmos: &mut Gizmos,
    start: Vec2,
    joint: Vec2,
    target: Vec2,
    color: Color,
) {
    let segments = 6;
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;
        
        let p1 = (1.0 - t1).powi(2) * start + 2.0 * (1.0 - t1) * t1 * joint + t1.powi(2) * target;
        let p2 = (1.0 - t2).powi(2) * start + 2.0 * (1.0 - t2) * t2 * joint + t2.powi(2) * target;
        
        // Draw 3 closely-spaced lines parallel to achieve thickness!
        gizmos.line_2d(p1, p2, color);
        gizmos.line_2d(p1 + Vec2::new(1.0, 0.0), p2 + Vec2::new(1.0, 0.0), color);
        gizmos.line_2d(p1 + Vec2::new(-1.0, 0.0), p2 + Vec2::new(-1.0, 0.0), color);
    }
}

/// Pill-shaped black eye renderer.
fn draw_eye(
    gizmos: &mut Gizmos,
    center: Vec2,
    panicked: bool,
    angry: bool,
    color: Color,
) {
    let mut height = 8.0;
    let mut width = 3.0;
    
    if panicked {
        height = 14.0;
        width = 4.0;
    } else if angry {
        height = 4.0;
        width = 4.0;
    }
    
    for w in 0..(width as i32) {
        let offset_x = (w as f32) - (width / 2.0);
        gizmos.line_2d(
            center + Vec2::new(offset_x, -height / 2.0),
            center + Vec2::new(offset_x, height / 2.0),
            color,
        );
    }
}

/// Dynamic, responsive cartoon eyebrow renderer.
fn draw_eyebrow(
    gizmos: &mut Gizmos,
    eye_center: Vec2,
    is_left: bool,
    panicked: bool,
    angry: bool,
    color: Color,
) {
    let brow_center = eye_center + Vec2::new(0.0, 8.0);
    let half_width = 5.0;
    
    let mut tilt = 0.0;
    if angry {
        tilt = if is_left { -3.5 } else { 3.5 };
    } else if panicked {
        tilt = if is_left { 3.5 } else { -3.5 };
    }
    
    let start = brow_center + Vec2::new(-half_width, -tilt);
    let end = brow_center + Vec2::new(half_width, tilt);
    
    gizmos.line_2d(start, end, color);
    gizmos.line_2d(start + Vec2::new(0.0, 1.0), end + Vec2::new(0.0, 1.0), color);
}
