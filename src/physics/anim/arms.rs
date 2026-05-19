use bevy::prelude::*;
use crate::player::{Player, PlayerStatsComponent, BlockComponent};
use crate::physics::weapon::Weapon;
use super::components::PlayerAim;
use super::ik_math::{solve_ik_2d, draw_connected_noodle};

// --- SEVEN SEGMENT PROCEDURAL DIGITAL INDICATORS ---
pub fn draw_digital_digit(
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

pub fn draw_digital_number(
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
        let color = player.color();
        
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
                let color = player.color().with_alpha(alpha);
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
