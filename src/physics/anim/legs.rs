use bevy::prelude::*;
use crate::player::{Player, PlayerStatsComponent};
use crate::physics::components::{Velocity, Grounded, Collider, Platform};
use super::components::{ProceduralLimbs, FootState};
use super::ik_math::{solve_ik_2d, draw_connected_noodle};

fn find_ground_y(
    body_pos: Vec2,
    scale: f32,
    platforms: &Query<(&Transform, &Collider), With<Platform>>,
) -> f32 {
    let mut ground_y = -1080.0;
    let player_radius = 40.0 * scale;
    for (plat_trans, plat_coll) in platforms.iter() {
        if let Collider::Rect { size } = plat_coll {
            let plat_pos = plat_trans.translation.xy();
            let half_size = *size / 2.0;
            // Check horizontal overlap between player body radius and platform
            let overlaps_x = body_pos.x + player_radius >= plat_pos.x - half_size.x &&
                             body_pos.x - player_radius <= plat_pos.x + half_size.x;
            if overlaps_x {
                let top_y = plat_pos.y + half_size.y;
                if top_y <= body_pos.y + 5.0 {
                    if top_y > ground_y {
                        ground_y = top_y;
                    }
                }
            }
        }
    }
    ground_y
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
    platforms: Query<(&Transform, &Collider), With<Platform>>,
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
        
        let ground_y = find_ground_y(body_pos, scale, &platforms);
        
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

/// Translates foot states into real-time horizontal and vertical positions.
pub fn get_foot_pos(state: FootState, scale: f32) -> Vec2 {
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
pub fn process_foot(
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
            FootState::Planted { position: Vec2::new(current.x, ground_y) }
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
                FootState::Planted { position: Vec2::new(position.x, ground_y) }
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
