use bevy::prelude::*;
use crate::player::{Player, Health, PlayerStatsComponent, BlockComponent};
use crate::physics::components::{Grounded, Velocity, WallContact};
use crate::physics::weapon::Weapon;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};
use crate::settings::{LobbySlots, InputDevice};
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

impl FootState {
    pub fn is_zero(&self) -> bool {
        match self {
            FootState::Airborne { current } => current.length_squared() < 0.0001,
            _ => false,
        }
    }
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
    mut query: Query<(&Player, &Transform, &Velocity, &mut PlayerAim, &PlayerStatsComponent)>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    lobby_slots: Res<LobbySlots>,
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

    for (player, transform, velocity, mut aim, stats) in query.iter_mut() {
        let slot = match player {
            Player::P1 => &lobby_slots.p1,
            Player::P2 => &lobby_slots.p2,
        };

        if let Some(device) = slot {
            match device {
                InputDevice::KeyboardMouse => {
                    if let Some(cursor) = cursor_pos {
                        let rx = cursor.x - vp_x;
                        let ry = cursor.y - vp_y;
                        let u = rx / vp_width;
                        let v = ry / vp_height;
                        let world_x = (u - 0.5) * TARGET_WIDTH;
                        let world_y = (0.5 - v) * TARGET_HEIGHT;
                        
                        let scale = stats.player_scale;
                        let player_pos = transform.translation.xy() + Vec2::new(0.0, 25.0 * scale);
                        let target_dir = Vec2::new(world_x, world_y) - player_pos;
                        if target_dir.length_squared() > 1.0 {
                            aim.direction = target_dir.normalize();
                        }
                    }
                }
                InputDevice::Gamepad(gp_entity) => {
                    let mut got_aim = false;
                    if let Ok(gp) = gamepads.get(*gp_entity) {
                        // Retrieve raw unclamped axis values to bypass snapping axis deadzones
                        let rx = gp.get_unclamped(GamepadAxis::RightStickX).unwrap_or(0.0);
                        let ry = gp.get_unclamped(GamepadAxis::RightStickY).unwrap_or(0.0);
                        let stick = Vec2::new(rx, ry);
                        // Apply a smooth, fluid 360-degree circular deadzone
                        if stick.length_squared() > 0.04 {
                            aim.direction = stick.normalize();
                            got_aim = true;
                        }
                    }
                    if !got_aim {
                        let vel = velocity.0;
                        if vel.length_squared() > 10.0 {
                            aim.direction = vel.normalize();
                        } else if aim.direction == Vec2::ZERO {
                            aim.direction = Vec2::X;
                        }
                    }
                }
            }
        } else {
            // FALLBACK / DEFAULTS
            match player {
                Player::P1 => {
                    if let Some(cursor) = cursor_pos {
                        let rx = cursor.x - vp_x;
                        let ry = cursor.y - vp_y;
                        let u = rx / vp_width;
                        let v = ry / vp_height;
                        let world_x = (u - 0.5) * TARGET_WIDTH;
                        let world_y = (0.5 - v) * TARGET_HEIGHT;
                        
                        let scale = stats.player_scale;
                        let player_pos = transform.translation.xy() + Vec2::new(0.0, 25.0 * scale);
                        let target_dir = Vec2::new(world_x, world_y) - player_pos;
                        if target_dir.length_squared() > 1.0 {
                            aim.direction = target_dir.normalize();
                        }
                    }
                }
                Player::P2 => {
                    let mut got_aim = false;
                    let first_gamepad = gamepads.iter().next();
                    if let Some(gp) = first_gamepad {
                        // Retrieve raw unclamped axis values to bypass snapping axis deadzones
                        let rx = gp.get_unclamped(GamepadAxis::RightStickX).unwrap_or(0.0);
                        let ry = gp.get_unclamped(GamepadAxis::RightStickY).unwrap_or(0.0);
                        let stick = Vec2::new(rx, ry);
                        // Apply a smooth, fluid 360-degree circular deadzone
                        if stick.length_squared() > 0.04 {
                            aim.direction = stick.normalize();
                            got_aim = true;
                        }
                    }
                    
                    if !got_aim {
                        let mut key_dir = Vec2::ZERO;
                        if keys.pressed(KeyCode::KeyI) { key_dir.y += 1.0; }
                        if keys.pressed(KeyCode::KeyK) { key_dir.y -= 1.0; }
                        if keys.pressed(KeyCode::KeyJ) { key_dir.x -= 1.0; }
                        if keys.pressed(KeyCode::KeyL) { key_dir.x += 1.0; }
                        
                        if key_dir.length_squared() > 0.1 {
                            aim.direction = key_dir.normalize();
                        } else {
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
        &PlayerStatsComponent,
    )>,
) {
    let dt = time.delta_secs().min(0.05); // cap delta time to safeguard against frame lag spikes
    
    for (transform, velocity, grounded, player, mut limbs, stats) in query.iter_mut() {
        let scale = stats.player_scale;
        let body_pos = transform.translation.xy();
        let vel = velocity.0;
        
        let visual_center = body_pos + Vec2::new(0.0, 15.0 * scale);
        
        // Offset hips relative to player's visual center (lifted higher so legs are extended upright!)
        let left_hip = visual_center + Vec2::new(-16.0, -15.0) * scale;
        let right_hip = visual_center + Vec2::new(16.0, -15.0) * scale;
        
        let color = match player {
            Player::P1 => Color::srgb(0.1, 0.35, 0.8), // Deep P1 Blue
            Player::P2 => Color::srgb(0.8, 0.35, 0.1), // Deep P2 Orange
        };
        
        if limbs.step_cooldown > 0.0 {
            limbs.step_cooldown -= dt;
        }
        
        let ground_y = body_pos.y - 40.0 * scale;
        
        // Initialize foot positions if they are at the center of the world (Vec2::ZERO) at startup
        if limbs.left_foot.is_zero() {
            limbs.left_foot = FootState::Planted {
                position: Vec2::new(left_hip.x, ground_y),
            };
        }
        if limbs.right_foot.is_zero() {
            limbs.right_foot = FootState::Planted {
                position: Vec2::new(right_hip.x, ground_y),
            };
        }
        
        // 1. Process left foot stride
        limbs.left_foot = process_foot(
            dt,
            left_hip,
            vel,
            grounded.0,
            limbs.left_foot,
            limbs.right_foot,
            &mut limbs.step_cooldown,
            -16.0 * scale,
            ground_y,
            scale,
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
            16.0 * scale,
            ground_y,
            scale,
        );
        
        // Segment lengths: 26px upper thigh, 22px lower shin
        let l1 = 26.0 * scale;
        let l2 = 22.0 * scale;
        
        // Draw Left Noodle Leg
        let left_current = get_foot_pos(limbs.left_foot, scale);
        let left_knee = solve_ik_2d(left_hip, left_current, l1, l2, false);
        draw_connected_noodle(&mut gizmos, left_hip, left_knee, left_current, color, scale);
        
        // Draw Right Noodle Leg
        let right_current = get_foot_pos(limbs.right_foot, scale);
        let right_knee = solve_ik_2d(right_hip, right_current, l1, l2, true);
        draw_connected_noodle(&mut gizmos, right_hip, right_knee, right_current, color, scale);
    }
}

// --- SEVEN SEGMENT PROCEDURAL DIGITAL INDICATORS ---
fn draw_digital_digit(
    gizmos: &mut Gizmos,
    pos: Vec2,
    aim_dir: Vec2,
    up_normal: Vec2,
    digit: u32,
    color: Color,
    scale: f32,
) {
    let w = 6.0 * scale;
    let h = 10.0 * scale;
    let right_vec = aim_dir * w;
    let up_vec = up_normal * h;

    let bottom_left = pos - right_vec * 0.5;
    let bottom_right = pos + right_vec * 0.5;
    let mid_left = bottom_left + up_vec * 0.5;
    let mid_right = bottom_right + up_vec * 0.5;
    let top_left = bottom_left + up_vec;
    let top_right = bottom_right + up_vec;

    let draw_seg = |gizmos: &mut Gizmos, p1: Vec2, p2: Vec2| {
        gizmos.line_2d(p1, p2, color);
        gizmos.line_2d(p1 + up_normal * (0.5 * scale), p2 + up_normal * (0.5 * scale), color);
    };

    let segs = match digit {
        0 => [true, true, true, true, true, true, false],
        1 => [false, true, true, false, false, false, false],
        2 => [true, true, false, true, true, false, true],
        3 => [true, true, true, true, false, false, true],
        4 => [false, true, true, false, false, true, true],
        5 => [true, false, true, true, false, true, true],
        6 => [true, false, true, true, true, true, true],
        7 => [true, true, true, false, false, false, false],
        8 => [true, true, true, true, true, true, true],
        9 => [true, true, true, true, false, true, true],
        _ => [false; 7],
    };

    if segs[0] { draw_seg(gizmos, top_left, top_right); }
    if segs[1] { draw_seg(gizmos, top_right, mid_right); }
    if segs[2] { draw_seg(gizmos, mid_right, bottom_right); }
    if segs[3] { draw_seg(gizmos, bottom_right, bottom_left); }
    if segs[4] { draw_seg(gizmos, bottom_left, mid_left); }
    if segs[5] { draw_seg(gizmos, mid_left, top_left); }
    if segs[6] { draw_seg(gizmos, mid_left, mid_right); }
}

fn draw_digital_number(
    gizmos: &mut Gizmos,
    pos: Vec2,
    aim_dir: Vec2,
    up_normal: Vec2,
    number: u32,
    color: Color,
    scale: f32,
) {
    let s = number.to_string();
    let char_width = 8.0 * scale;
    let char_spacing = 3.0 * scale;
    let total_chars = s.len() as f32;
    let total_width = total_chars * char_width + (total_chars - 1.0) * char_spacing;
    
    let mut current_pos = pos - aim_dir * (total_width * 0.5 - char_width * 0.5);

    for c in s.chars() {
        if let Some(digit) = c.to_digit(10) {
            draw_digital_digit(gizmos, current_pos, aim_dir, up_normal, digit, color, scale);
        }
        current_pos += aim_dir * (char_width + char_spacing);
    }
}

/// Solves the weapon arms, aiming a gun with the dominant hand, and holding a floating white shield/orb with the supporting hand.
pub fn draw_procedural_arms(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &PlayerAim, &Player, &Weapon, &PlayerStatsComponent, &BlockComponent)>,
) {
    for (transform, aim, player, weapon, stats, block) in query.iter() {
        let scale = stats.player_scale;
        let body_pos = transform.translation.xy();
        let visual_center = body_pos + Vec2::new(0.0, 15.0 * scale);
        let aim_dir = aim.direction;
        
        // Arm colors
        let color = match player {
            Player::P1 => Color::srgb(0.2, 0.5, 1.0),
            Player::P2 => Color::srgb(1.0, 0.5, 0.2),
        };
        
        // --- 1. WEAPON & DOMINANT ARM (AIMING SIDE) ---
        let gun_center = visual_center + aim_dir * (78.0 * scale);
        let normal = Vec2::new(-aim_dir.y, aim_dir.x); // perpendicular normal vector
        
        // Draw elegant minimal weapon shapes
        let barrel_start = visual_center + aim_dir * (62.0 * scale);
        let barrel_end = visual_center + aim_dir * (98.0 * scale);
        gizmos.line_2d(barrel_start, barrel_end, Color::srgb(0.75, 0.75, 0.75));
        
        // Flip the gun stock handle dynamically so it always points downwards (gravity-upright)
        let grip_sign = if aim_dir.x >= 0.0 { -1.0 } else { 1.0 };
        let stock_end = barrel_start + normal * (grip_sign * 12.0 * scale);
        gizmos.line_2d(barrel_start, stock_end, Color::srgb(0.55, 0.55, 0.55));
        
        // Dominant hand anchor holding the gun stock
        let dominant_hand = gun_center + normal * (grip_sign * 2.0 * scale);
        
        // --- 1.5 AMMO INDICATORS & RELOAD GAUGES ABOVE WEAPON BARREL ---
        let up_normal = -normal * grip_sign;
        let ammo_base = barrel_start.lerp(barrel_end, 0.4) + up_normal * (12.0 * scale);

        if weapon.reload_timer > 0.0 {
            // Spinning reload indicator
            let reload_center = barrel_start.lerp(barrel_end, 0.4) + up_normal * (14.0 * scale);
            let ring_radius = 6.0 * scale;
            gizmos.circle_2d(reload_center, ring_radius, Color::srgb(0.6, 0.6, 0.6));
            
            let reload_pct = weapon.reload_timer / weapon.reload_time;
            let angle = reload_pct * std::f32::consts::TAU * 3.0; // Spin rapidly
            let dot_pos = reload_center + Vec2::new(angle.cos(), angle.sin()) * ring_radius;
            gizmos.circle_2d(dot_pos, 2.0 * scale, Color::srgb(1.0, 1.0, 1.0));
        } else if weapon.current_ammo > 18 {
            // Large ammo count bold dynamic digital 7-segment indicator
            // Flip the text horizontally when aiming left so it reads left-to-right, but keep up_normal pointing UP so it is upright!
            let text_dir = if aim_dir.x >= 0.0 { aim_dir } else { -aim_dir };
            draw_digital_number(&mut gizmos, ammo_base, text_dir, up_normal, weapon.current_ammo, Color::srgb(1.0, 1.0, 1.0), scale);
        } else if weapon.current_ammo > 0 {
            // Stacked rows of 3 ammo indicators - perfectly balanced size and spacing!
            let dot_spacing_col = 10.5 * scale;
            let dot_spacing_row = 9.5 * scale;
            let dot_radius = 2.4 * scale;
            
            for i in 0..weapon.current_ammo {
                let row = i / 3;
                let col = i % 3;
                
                let col_offset = (col as f32 - 1.0) * dot_spacing_col;
                let row_offset = (row as f32) * dot_spacing_row;
                
                let dot_pos = ammo_base 
                    + aim_dir * col_offset 
                    + up_normal * row_offset;
                
                gizmos.circle_2d(dot_pos, dot_radius, Color::srgb(1.0, 1.0, 1.0));
                gizmos.circle_2d(dot_pos, dot_radius - 0.9 * scale, Color::srgb(1.0, 1.0, 1.0));
            }
        }
        
        // --- 2. FLOATING WHITE ORB & SUPPORTING ARM (OPPOSITE SIDE) ---
        // Make the orb stay horizontally fixed on the opposite side instead of rotating with the aim vector!
        let shield_offset_x = if aim_dir.x >= 0.0 { -52.0 * scale } else { 52.0 * scale };
        let shield_center = visual_center + Vec2::new(shield_offset_x, 0.0);
        
        let radius = 10.0 * scale;
        
        if block.cooldown_timer <= 0.0 {
            // Fully charged: Draw completely filled solid white circle
            let max_r = radius.round() as i32;
            for r in 0..=max_r {
                gizmos.circle_2d(shield_center, r as f32, Color::srgb(1.0, 1.0, 1.0));
            }
        } else {
            // Reloading/cooldown: Draw circular border outline, and radial fill spinning around unit circle
            gizmos.circle_2d(shield_center, radius, Color::srgb(1.0, 1.0, 1.0));
            
            // Refill percentage (goes from 0.0 to 1.0 as cooldown decreases)
            let fill_pct = 1.0 - (block.cooldown_timer / block.block_cooldown).clamp(0.0, 1.0);
            
            // Draw radial fill lines sweeping around the unit circle
            let steps = 36;
            let max_step = (fill_pct * steps as f32).round() as i32;
            for i in 0..max_step {
                let pct = i as f32 / steps as f32;
                // Clockwise sweep starting from the top (-PI/2)
                let angle = -std::f32::consts::FRAC_PI_2 + pct * 2.0 * std::f32::consts::PI;
                let end_point = shield_center + Vec2::new(angle.cos() * radius, angle.sin() * radius);
                gizmos.line_2d(shield_center, end_point, Color::srgb(1.0, 1.0, 1.0));
            }
        }
        
        // If blocking is active (invincible duration), draw a beautiful, soft, blurry glowing shield bubble around the main body!
        if block.active_timer > 0.0 {
            // Pulsing glow factor
            let pulse = (block.active_timer * 15.0).sin() * 0.05 + 0.95;
            let base_radius = (40.0 * scale + 15.0) * pulse;
            
            // Draw 15 concentric circles with soft opacity scaling to simulate a blur filter!
            for i in -7..=7 {
                let offset = i as f32 * 1.5;
                let dist_pct = (i as f32 / 7.0).abs();
                let alpha = (1.0 - dist_pct) * 0.12; // soft drop-off gradient
                
                let ring_radius = base_radius + offset;
                let color = match player {
                    Player::P1 => Color::srgba(0.0, 0.85, 1.0, alpha), // Soft glowing cyan/turquoise (similar but not exact to P1 blue)
                    Player::P2 => Color::srgba(1.0, 0.55, 0.1, alpha), // Soft glowing amber/deep-orange (similar but not exact to P2 orange)
                };
                gizmos.circle_2d(visual_center, ring_radius, color);
            }
        }
        
        // Supporting hand anchor holding the floating white circle
        let supporting_hand = shield_center;
        
        // --- 3. SHOULDER & INVERSE KINEMATICS ---
        let left_shoulder = visual_center + Vec2::new(-20.0 * scale, 0.0);
        let right_shoulder = visual_center + Vec2::new(20.0 * scale, 0.0);
        
        let arm_l1 = 40.0 * scale;
        let arm_l2 = 36.0 * scale;
        
        // Draw Dominant Hand Arm (Aim Side - swaps between left/right automatically)
        let dom_shoulder = if aim_dir.x >= 0.0 { right_shoulder } else { left_shoulder };
        let dom_elbow = solve_ik_2d(dom_shoulder, dominant_hand, arm_l1, arm_l2, aim_dir.x < 0.0);
        draw_connected_noodle(&mut gizmos, dom_shoulder, dom_elbow, dominant_hand, color, scale);
        
        // Draw Supporting Hand Arm (Opposite Floating Orb Side - swaps between left/right automatically)
        let sup_shoulder = if aim_dir.x >= 0.0 { left_shoulder } else { right_shoulder };
        let sup_elbow = solve_ik_2d(sup_shoulder, supporting_hand, arm_l1, arm_l2, aim_dir.x >= 0.0);
        draw_connected_noodle(&mut gizmos, sup_shoulder, sup_elbow, supporting_hand, color, scale);
    }
}

/// Dynamically renders black pixel cartoon eyes and tilting eyebrows that express emotions based on stats.
pub fn draw_expressive_faces(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &PlayerAim, &Health, &Velocity, &Grounded, &WallContact, &PlayerStatsComponent)>,
) {
    for (transform, aim, health, velocity, grounded, wall, stats) in query.iter() {
        let scale = stats.player_scale;
        let body_pos = transform.translation.xy();
        let visual_center = body_pos + Vec2::new(0.0, 25.0 * scale);
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
fn get_foot_pos(state: FootState, scale: f32) -> Vec2 {
    match state {
        FootState::Planted { position } => position,
        FootState::Stepping { start, target, progress } => {
            let t = progress.clamp(0.0, 1.0);
            let horizontal = start.lerp(target, t);
            let arc_height = 20.0 * scale * (4.0 * t * (1.0 - t)); // parabolic lift arc
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
    scale: f32,
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
            side_offset - velocity.x * 0.0005, // reduced flailing: stay much more directly under player when moving horizontally
            -30.0 * scale,                     // no elasticity: do not stretch or shrink when jumping or falling
        ).clamp_length_max(34.0 * scale);             // keep target well within physical reach to preserve organic bend
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
            let max_stride = 50.0 * scale;
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
    scale: f32,
) {
    let segments = 6;
    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;
        
        let p1 = (1.0 - t1).powi(2) * start + 2.0 * (1.0 - t1) * t1 * joint + t1.powi(2) * target;
        let p2 = (1.0 - t2).powi(2) * start + 2.0 * (1.0 - t2) * t2 * joint + t2.powi(2) * target;
        
        // Draw 3 closely-spaced lines parallel to achieve thickness!
        gizmos.line_2d(p1, p2, color);
        gizmos.line_2d(p1 + Vec2::new(1.0, 0.0) * scale, p2 + Vec2::new(1.0, 0.0) * scale, color);
        gizmos.line_2d(p1 + Vec2::new(-1.0, 0.0) * scale, p2 + Vec2::new(-1.0, 0.0) * scale, color);
    }
}

/// Pill-shaped black eye renderer.
fn draw_eye(
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
fn draw_eyebrow(
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

/// Renders a UI score overlay in the top left corner with rows of blue and orange circles.
pub fn draw_score_overlay(
    mut gizmos: Gizmos,
    score: Res<crate::settings::ScoreTracker>,
    state: Res<State<crate::settings::GameState>>,
) {
    if *state.get() == crate::settings::GameState::MainMenu || *state.get() == crate::settings::GameState::Lobby {
        return;
    }
    let half_width = TARGET_WIDTH / 2.0;
    let half_height = TARGET_HEIGHT / 2.0;
    let radius = 18.0; // Larger size to look very premium
    let spacing = 48.0; // Larger spacing to comfortably fit the larger size
    
    // Player 1 (Blue) Row
    let p1_color = Color::srgb(0.2, 0.5, 1.0);
    let p1_y = half_height - 50.0;
    for i in 0..score.p1_wins {
        let pos = Vec2::new(-half_width + 45.0 + i as f32 * spacing, p1_y);
        // Draw beautiful solid filled blue circle
        let max_r = radius as i32;
        for r in 0..=max_r {
            gizmos.circle_2d(pos, r as f32, p1_color);
        }
    }
    
    // Player 2 (Orange) Row
    let p2_color = Color::srgb(1.0, 0.6, 0.2);
    let p2_y = half_height - 100.0;
    for i in 0..score.p2_wins {
        let pos = Vec2::new(-half_width + 45.0 + i as f32 * spacing, p2_y);
        // Draw beautiful solid filled orange circle
        let max_r = radius as i32;
        for r in 0..=max_r {
            gizmos.circle_2d(pos, r as f32, p2_color);
        }
    }
}

