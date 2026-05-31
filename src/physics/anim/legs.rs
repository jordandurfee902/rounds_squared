use bevy::prelude::*;
use crate::player::{Player, PlayerStatsComponent};
use crate::physics::components::{Velocity, Grounded, Collider, Platform, RopeSwing};
use super::components::{ProceduralLimbs, FootState};
use super::ik_math::{solve_ik_2d, draw_connected_noodle};

fn find_ground_y(
    body_pos: Vec2,
    scale: f32,
    platforms: &Query<(&Transform, &Collider), With<Platform>>,
) -> f32 {
    let mut ground_y = -2160.0;
    let player_radius = 40.0 * scale;
    for (plat_trans, plat_coll) in platforms.iter() {
        if let Collider::Rect { size } = plat_coll {
            let plat_pos = plat_trans.translation.xy();
            let half_size = *size / 2.0;
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

/// Returns the foot's world position, clamped to always be within IK reach
/// of the hip so the Bezier-curve leg never visually stretches.
fn get_foot_pos(state: FootState, hip_pos: Vec2, foot_reach: f32, scale: f32) -> Vec2 {
    let pos = match state {
        FootState::Planted { position } => position,
        FootState::Stepping { start, target, progress } => {
            let t = progress.clamp(0.0, 1.0);
            let horizontal = start.lerp(target, t);
            let arc = 20.0 * scale * (4.0 * t * (1.0 - t));
            horizontal + Vec2::new(0.0, arc)
        }
        FootState::Airborne => hip_pos + Vec2::new(0.0, -foot_reach * 0.7),
    };

    let to_hip = pos - hip_pos;
    if to_hip.length_squared() > foot_reach * foot_reach {
        hip_pos + to_hip.normalize() * foot_reach * 0.95
    } else {
        pos
    }
}

/// Advances one foot's state for the current frame.
/// When airborne, returns Airborne immediately — feet snap to a fixed
/// dangling position at draw time (computed in get_foot_pos).
/// When grounded, alternates stepping with a stride that anticipates
/// where the hip will be when the step finishes.
fn process_foot(
    dt: f32,
    hip_pos: Vec2,
    grounded: bool,
    current: FootState,
    other_is_stepping: bool,
    cooldown: &mut f32,
    ground_y: f32,
    foot_reach: f32,
    step_speed: f32,
) -> FootState {
    if !grounded {
        return FootState::Airborne;
    }

    let max_stride = foot_reach * 0.45;

    match current {
        FootState::Airborne => {
            FootState::Planted { position: Vec2::new(hip_pos.x, ground_y) }
        }
        FootState::Planted { position } => {
            let dist = hip_pos.x - position.x;
            let trigger_threshold = max_stride * 1.1;

            if dist.abs() > trigger_threshold && !other_is_stepping && *cooldown <= 0.0 {
                let step_dir = dist.signum();
                let target_x = hip_pos.x + step_dir * max_stride;
                *cooldown = 0.06;
                FootState::Stepping {
                    start: position,
                    target: Vec2::new(target_x, ground_y),
                    progress: 0.0,
                }
            } else {
                FootState::Planted { position: Vec2::new(position.x, ground_y) }
            }
        }
        FootState::Stepping { start, target, mut progress } => {
            progress += dt * step_speed;
            if progress >= 1.0 {
                let step_dir = (target.x - start.x).signum();
                FootState::Planted { position: Vec2::new(hip_pos.x + step_dir * max_stride, ground_y) }
            } else {
                FootState::Stepping { start, target, progress }
            }
        }
    }
}

/// Solves and draws procedural walking-leg IK noodles for each player.
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
    let dt = time.delta_secs().min(0.05);

    for (transform, velocity, grounded, player, mut limbs, stats) in query.iter_mut() {
        let scale = stats.player_scale;
        let body_pos = transform.translation.xy();
        let vel = velocity.0;

        let visual_center = body_pos + Vec2::new(0.0, 15.0 * scale);
        let left_hip = visual_center + Vec2::new(-16.0, -15.0) * scale;
        let right_hip = visual_center + Vec2::new(16.0, -15.0) * scale;

        let color = player.color();

        if limbs.step_cooldown > 0.0 {
            limbs.step_cooldown -= dt;
        }

        let ground_y = find_ground_y(body_pos, scale, &platforms);

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

        let l1 = 26.0 * scale;
        let l2 = 22.0 * scale;
        let foot_reach = l1 + l2;
        let step_speed = 6.0 + vel.length() * 0.008;

        limbs.left_foot = process_foot(
            dt,
            left_hip,
            grounded.0,
            limbs.left_foot,
            matches!(limbs.right_foot, FootState::Stepping { .. }),
            &mut limbs.step_cooldown,
            ground_y,
            foot_reach,
            step_speed,
        );
        limbs.right_foot = process_foot(
            dt,
            right_hip,
            grounded.0,
            limbs.right_foot,
            matches!(limbs.left_foot, FootState::Stepping { .. }),
            &mut limbs.step_cooldown,
            ground_y,
            foot_reach,
            step_speed,
        );

        let left_foot_pos = get_foot_pos(limbs.left_foot, left_hip, foot_reach, scale);
        let left_knee = solve_ik_2d(left_hip, left_foot_pos, l1, l2, false);
        draw_connected_noodle(&mut gizmos, left_hip, left_knee, left_foot_pos, color, scale);

        let right_foot_pos = get_foot_pos(limbs.right_foot, right_hip, foot_reach, scale);
        let right_knee = solve_ik_2d(right_hip, right_foot_pos, l1, l2, true);
        draw_connected_noodle(&mut gizmos, right_hip, right_knee, right_foot_pos, color, scale);
    }
}

pub fn draw_rope_lines(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &RopeSwing)>,
) {
    for (transform, rope) in query.iter() {
        let start = rope.anchor;
        let end = transform.translation.xy();
        gizmos.line_2d(start, end, Color::srgb(0.5, 0.4, 0.3));
    }
}
