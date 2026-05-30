use bevy::prelude::*;
use crate::physics::{Velocity, Mass, Collider, Grounded, StandingOn, Platform};
use crate::physics::components::{MovingPlatform, RopeSwing, PhysicsObject};
use crate::settings::PhysicsSettings;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};
use crate::player::Player;

pub fn update_moving_platforms(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MovingPlatform)>,
    mut players_q: Query<(&mut Transform, &StandingOn), (With<Player>, Without<MovingPlatform>)>,
) {
    let elapsed = time.elapsed_secs();
    let dt = time.delta_secs();

    for (plat_ent, mut plat_trans, mut mp) in query.iter_mut() {
        let p_start = plat_trans.translation.xy();
        let r_start = mp.current_rotation;

        // Linear oscillation
        let offset_x = (elapsed * mp.frequency.x).sin() * mp.amplitude.x;
        let offset_y = (elapsed * mp.frequency.y).sin() * mp.amplitude.y;
        let p_new = mp.initial_pos + Vec2::new(offset_x, offset_y);

        let delta_theta = mp.spin_speed * dt;
        let r_new = r_start + delta_theta;

        // Push player standing on this platform
        for (mut player_trans, standing_on) in players_q.iter_mut() {
            if standing_on.0 == Some(plat_ent) {
                let player_pos = player_trans.translation.xy();
                let rel_pos = player_pos - p_start;
                let cos_t = delta_theta.cos();
                let sin_t = delta_theta.sin();
                let rotated_rel = Vec2::new(
                    rel_pos.x * cos_t - rel_pos.y * sin_t,
                    rel_pos.x * sin_t + rel_pos.y * cos_t,
                );
                let new_player_pos = p_new + rotated_rel;
                player_trans.translation = new_player_pos.extend(player_trans.translation.z);
            }
        }

        plat_trans.translation = p_new.extend(plat_trans.translation.z);
        mp.current_rotation = r_new;
        plat_trans.rotation = Quat::from_rotation_z(r_new);
    }
}

pub fn apply_physics_object_physics(
    time: Res<Time>,
    settings: Res<PhysicsSettings>,
    mut query: Query<(&mut Transform, &mut Velocity, &Mass), With<PhysicsObject>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity, mass) in query.iter_mut() {
        // Apply gravity (multiplied by object mass to scale weight)
        velocity.0.y += settings.gravity * mass.0 * dt;

        // Apply simple air friction/drag
        velocity.0 *= 0.99;

        // Apply velocity to translation
        transform.translation += velocity.0.extend(0.0) * dt;
    }
}

pub fn update_rope_swing(
    mut query: Query<(&mut Transform, &mut Velocity, &RopeSwing)>,
) {
    for (mut transform, mut velocity, rope) in query.iter_mut() {
        let pos = transform.translation.xy();
        let diff = pos - rope.anchor;
        let dist = diff.length();
        if dist > rope.length && dist > 0.0 {
            let dir = diff / dist;
            let constrained_pos = rope.anchor + dir * rope.length;
            transform.translation.x = constrained_pos.x;
            transform.translation.y = constrained_pos.y;

            // Kill velocity pointing away from anchor
            let vel_dot = velocity.0.dot(dir);
            if vel_dot > 0.0 {
                velocity.0 -= vel_dot * dir;
            }
        }
    }
}

pub fn resolve_physics_object_collisions(
    time: Res<Time>,
    settings: Res<PhysicsSettings>,
    mut objects: Query<(Entity, &mut Transform, &mut Velocity, &Collider, &Mass, &PhysicsObject), (Without<Platform>, Without<Player>)>,
    platforms: Query<(&Transform, &Collider), (With<Platform>, Without<PhysicsObject>, Without<Player>)>,
    mut players: Query<(&mut Transform, &mut Velocity, &Collider, &Mass, &mut Grounded), (With<Player>, Without<PhysicsObject>, Without<Platform>)>,
) {
    let restitution = settings.boundary_restitution;
    let push_factor = settings.overlapping_push_factor;
    let dt = time.delta_secs();

    // 1. Resolve against static and moving platforms
    for (_, mut o_trans, mut o_vel, o_coll, o_mass, _) in objects.iter_mut() {
        let mut pos = o_trans.translation.xy();
        let mut vel = o_vel.0;
        let mut grounded = false;

        for (plat_trans, plat_coll) in platforms.iter() {
            if let Collider::Rect { size } = plat_coll {
                if resolve_object_platform_collision(
                    &mut pos,
                    &mut vel,
                    o_coll,
                    o_mass.0,
                    plat_trans.translation.xy(),
                    plat_trans.rotation,
                    *size,
                    settings.collision_penetration_skin_buffer,
                    restitution,
                ) {
                    grounded = true;
                }
            }
        }

        // 2. Resolve against screen boundaries
        if resolve_object_boundary_collision(
            &mut pos,
            &mut vel,
            o_coll,
            TARGET_WIDTH,
            TARGET_HEIGHT,
            restitution,
        ) {
            grounded = true;
        }

        // Apply friction to the horizontal velocity if grounded
        if grounded {
            vel.x = vel.x / (1.0 + 8.0 * dt);
        }

        o_trans.translation = pos.extend(o_trans.translation.z);
        o_vel.0 = vel;
    }

    // 3. Resolve Box vs Box and Swing vs Box collisions (Physics Object vs Physics Object)
    // We use a combination loop to avoid duplicate checks
    let mut combinations = objects.iter_combinations_mut();
    while let Some([(_e1, mut t1, mut v1, c1, m1, _), (_e2, mut t2, mut v2, c2, m2, _)]) = combinations.fetch_next() {
        let mut pos1 = t1.translation.xy();
        let mut pos2 = t2.translation.xy();
        let mut vel1 = v1.0;
        let mut vel2 = v2.0;

        match (c1, c2) {
            (Collider::Rect { size: size1 }, Collider::Rect { size: size2 }) => {
                resolve_rect_rect_collision(
                    &mut pos1, &mut vel1, *size1, m1.0,
                    &mut pos2, &mut vel2, *size2, m2.0,
                    push_factor, restitution,
                );
            }
            (Collider::Circle { radius: r1 }, Collider::Rect { size: size2 }) => {
                resolve_circle_rect_collision(
                    &mut pos1, &mut vel1, *r1, m1.0,
                    &mut pos2, &mut vel2, *size2, m2.0,
                    push_factor, restitution,
                );
            }
            (Collider::Rect { size: size1 }, Collider::Circle { radius: r2 }) => {
                resolve_circle_rect_collision(
                    &mut pos2, &mut vel2, *r2, m2.0,
                    &mut pos1, &mut vel1, *size1, m1.0,
                    push_factor, restitution,
                );
            }
            (Collider::Circle { radius: r1 }, Collider::Circle { radius: r2 }) => {
                resolve_circle_circle_collision(
                    &mut pos1, &mut vel1, *r1, m1.0,
                    &mut pos2, &mut vel2, *r2, m2.0,
                    push_factor, restitution,
                );
            }
        }

        t1.translation = pos1.extend(t1.translation.z);
        t2.translation = pos2.extend(t2.translation.z);
        v1.0 = vel1;
        v2.0 = vel2;
    }

    // 4. Resolve Player vs Physics Object collisions
    for (mut p_trans, mut p_vel, p_coll, p_mass, mut grounded) in players.iter_mut() {
        let mut p_pos = p_trans.translation.xy();
        let mut p_v = p_vel.0;

        for (_, mut o_trans, mut o_vel, o_coll, o_mass, _) in objects.iter_mut() {
            let mut o_pos = o_trans.translation.xy();
            let mut o_v = o_vel.0;

            if let Collider::Circle { radius: p_r } = p_coll {
                match o_coll {
                    Collider::Circle { radius: o_r } => {
                        resolve_circle_circle_collision(
                            &mut p_pos, &mut p_v, *p_r, p_mass.0,
                            &mut o_pos, &mut o_v, *o_r, o_mass.0,
                            push_factor, restitution,
                        );
                    }
                    Collider::Rect { size: o_size } => {
                        let prev_p_y = p_pos.y;
                        resolve_circle_rect_collision(
                            &mut p_pos, &mut p_v, *p_r, p_mass.0,
                            &mut o_pos, &mut o_v, *o_size, o_mass.0,
                            push_factor, restitution,
                        );
                        // If player was pushed UP and landed on the box, count them as grounded!
                        if p_pos.y > prev_p_y && (p_pos.y - o_pos.y) > (o_size.y / 2.0 + p_r - 5.0) {
                            grounded.0 = true;
                        }
                    }
                }
            }

            o_trans.translation = o_pos.extend(o_trans.translation.z);
            o_vel.0 = o_v;
        }

        p_trans.translation = p_pos.extend(p_trans.translation.z);
        p_vel.0 = p_v;
    }
}

fn resolve_object_platform_collision(
    pos: &mut Vec2,
    vel: &mut Vec2,
    collider: &Collider,
    _mass: f32,
    plat_pos: Vec2,
    plat_rot: Quat,
    plat_size: Vec2,
    skin: f32,
    restitution: f32,
) -> bool {
    let mut grounded = false;
    let (_, _, theta) = plat_rot.to_euler(EulerRot::XYZ);
    match collider {
        Collider::Circle { radius } => {
            let diff = *pos - plat_pos;
            let cos_t = (-theta).cos();
            let sin_t = (-theta).sin();
            let local_pos = Vec2::new(
                diff.x * cos_t - diff.y * sin_t,
                diff.x * sin_t + diff.y * cos_t,
            );
            let half_size = plat_size / 2.0;
            let closest_local = Vec2::new(
                local_pos.x.clamp(-half_size.x, half_size.x),
                local_pos.y.clamp(-half_size.y, half_size.y),
            );
            let dist = local_pos.distance(closest_local);
            if dist <= *radius + skin {
                let normal_local = if dist > 0.0 { (local_pos - closest_local) / dist } else { Vec2::Y };
                let cos_w = theta.cos();
                let sin_w = theta.sin();
                let normal = Vec2::new(
                    normal_local.x * cos_w - normal_local.y * sin_w,
                    normal_local.x * sin_w + normal_local.y * cos_w,
                );
                if dist < *radius {
                    *pos += normal * (*radius - dist);
                }
                let vel_along_n = vel.dot(normal);
                if vel_along_n < 0.0 {
                    *vel -= (1.0 + restitution) * vel_along_n * normal;
                }
                if normal.y > 0.5 {
                    grounded = true;
                }
            }
        }
        Collider::Rect { size } => {
            let diff = *pos - plat_pos;
            let cos_t = (-theta).cos();
            let sin_t = (-theta).sin();
            let local_pos = Vec2::new(
                diff.x * cos_t - diff.y * sin_t,
                diff.x * sin_t + diff.y * cos_t,
            );
            let half_plat = plat_size / 2.0;
            let half_box = *size / 2.0;
            let overlap_x = (half_plat.x + half_box.x) - local_pos.x.abs();
            let overlap_y = (half_plat.y + half_box.y) - local_pos.y.abs();
            if overlap_x > 0.0 && overlap_y > 0.0 {
                let normal_local = if overlap_x < overlap_y {
                    Vec2::new(local_pos.x.signum(), 0.0)
                } else {
                    Vec2::new(0.0, local_pos.y.signum())
                };
                let overlap = if overlap_x < overlap_y { overlap_x } else { overlap_y };

                let cos_w = theta.cos();
                let sin_w = theta.sin();
                let normal = Vec2::new(
                    normal_local.x * cos_w - normal_local.y * sin_w,
                    normal_local.x * sin_w + normal_local.y * cos_w,
                );
                *pos += normal * overlap;
                let vel_along_n = vel.dot(normal);
                if vel_along_n < 0.0 {
                    *vel -= (1.0 + restitution) * vel_along_n * normal;
                }
                if normal.y > 0.5 {
                    grounded = true;
                }
            }
        }
    }
    grounded
}

fn resolve_object_boundary_collision(
    pos: &mut Vec2,
    vel: &mut Vec2,
    collider: &Collider,
    width: f32,
    height: f32,
    restitution: f32,
) -> bool {
    let mut grounded = false;
    let half_w = width / 2.0;
    let half_h = height / 2.0;
    match collider {
        Collider::Circle { radius } => {
            if pos.x - radius < -half_w {
                pos.x = -half_w + radius;
                vel.x = -vel.x * restitution;
            } else if pos.x + radius > half_w {
                pos.x = half_w - radius;
                vel.x = -vel.x * restitution;
            }
            if pos.y - radius < -half_h {
                pos.y = -half_h + radius;
                vel.y = -vel.y * restitution;
                grounded = true;
            } else if pos.y + radius > half_h {
                pos.y = half_h - radius;
                vel.y = -vel.y * restitution;
            }
        }
        Collider::Rect { size } => {
            let half_sz = *size / 2.0;
            if pos.x - half_sz.x < -half_w {
                pos.x = -half_w + half_sz.x;
                vel.x = -vel.x * restitution;
            } else if pos.x + half_sz.x > half_w {
                pos.x = half_w - half_sz.x;
                vel.x = -vel.x * restitution;
            }
            if pos.y - half_sz.y < -half_h {
                pos.y = -half_h + half_sz.y;
                vel.y = -vel.y * restitution;
                grounded = true;
            } else if pos.y + half_sz.y > half_h {
                pos.y = half_h - half_sz.y;
                vel.y = -vel.y * restitution;
            }
        }
    }
    grounded
}

fn resolve_circle_circle_collision(
    pos_a: &mut Vec2,
    vel_a: &mut Vec2,
    rad_a: f32,
    mass_a: f32,
    pos_b: &mut Vec2,
    vel_b: &mut Vec2,
    rad_b: f32,
    mass_b: f32,
    push_factor: f32,
    restitution: f32,
) {
    let dist_sq = pos_a.distance_squared(*pos_b);
    let combined_radius = rad_a + rad_b;
    if dist_sq < combined_radius * combined_radius {
        let dist = dist_sq.sqrt();
        let normal = if dist > 0.0 { (*pos_a - *pos_b) / dist } else { Vec2::Y };
        let overlap = combined_radius - dist;

        let total_inv_mass = (1.0 / mass_a) + (1.0 / mass_b);
        if total_inv_mass > 0.0 {
            let push_a = (1.0 / mass_a) / total_inv_mass * overlap * push_factor;
            let push_b = (1.0 / mass_b) / total_inv_mass * overlap * push_factor;
            *pos_a += normal * push_a;
            *pos_b -= normal * push_b;
        }

        let relative_vel = *vel_a - *vel_b;
        let vel_along_n = relative_vel.dot(normal);
        if vel_along_n < 0.0 {
            let impulse_scalar = -(1.0 + restitution) * vel_along_n / total_inv_mass;
            *vel_a += normal * (impulse_scalar / mass_a);
            *vel_b -= normal * (impulse_scalar / mass_b);
        }
    }
}

fn resolve_circle_rect_collision(
    circle_pos: &mut Vec2,
    circle_vel: &mut Vec2,
    circle_radius: f32,
    circle_mass: f32,
    rect_pos: &mut Vec2,
    rect_vel: &mut Vec2,
    rect_size: Vec2,
    rect_mass: f32,
    push_factor: f32,
    restitution: f32,
) {
    let half_size = rect_size / 2.0;
    let closest = Vec2::new(
        circle_pos.x.clamp(rect_pos.x - half_size.x, rect_pos.x + half_size.x),
        circle_pos.y.clamp(rect_pos.y - half_size.y, rect_pos.y + half_size.y),
    );
    let dist = circle_pos.distance(closest);
    if dist <= circle_radius {
        let normal = if dist > 0.0 { (*circle_pos - closest) / dist } else { Vec2::Y };
        let overlap = circle_radius - dist;

        let total_inv_mass = (1.0 / circle_mass) + (1.0 / rect_mass);
        if total_inv_mass > 0.0 {
            let circle_push = (1.0 / circle_mass) / total_inv_mass * overlap * push_factor;
            let rect_push = (1.0 / rect_mass) / total_inv_mass * overlap * push_factor;
            *circle_pos += normal * circle_push;
            *rect_pos -= normal * rect_push;
        }

        let relative_vel = *circle_vel - *rect_vel;
        let vel_along_n = relative_vel.dot(normal);
        if vel_along_n < 0.0 {
            let impulse_scalar = -(1.0 + restitution) * vel_along_n / total_inv_mass;
            *circle_vel += normal * (impulse_scalar / circle_mass);
            *rect_vel -= normal * (impulse_scalar / rect_mass);
        }
    }
}

fn resolve_rect_rect_collision(
    pos_a: &mut Vec2,
    vel_a: &mut Vec2,
    size_a: Vec2,
    mass_a: f32,
    pos_b: &mut Vec2,
    vel_b: &mut Vec2,
    size_b: Vec2,
    mass_b: f32,
    push_factor: f32,
    restitution: f32,
) {
    let half_a = size_a / 2.0;
    let half_b = size_b / 2.0;
    let overlap_x = (half_a.x + half_b.x) - (pos_a.x - pos_b.x).abs();
    let overlap_y = (half_a.y + half_b.y) - (pos_a.y - pos_b.y).abs();
    if overlap_x > 0.0 && overlap_y > 0.0 {
        let normal = if overlap_x < overlap_y {
            Vec2::new((pos_a.x - pos_b.x).signum(), 0.0)
        } else {
            Vec2::new(0.0, (pos_a.y - pos_b.y).signum())
        };
        let overlap = if overlap_x < overlap_y { overlap_x } else { overlap_y };

        let total_inv_mass = (1.0 / mass_a) + (1.0 / mass_b);
        if total_inv_mass > 0.0 {
            let push_a = (1.0 / mass_a) / total_inv_mass * overlap * push_factor;
            let push_b = (1.0 / mass_b) / total_inv_mass * overlap * push_factor;
            *pos_a += normal * push_a;
            *pos_b -= normal * push_b;
        }

        let relative_vel = *vel_a - *vel_b;
        let vel_along_n = relative_vel.dot(normal);
        if vel_along_n < 0.0 {
            let impulse_scalar = -(1.0 + restitution) * vel_along_n / total_inv_mass;
            *vel_a += normal * (impulse_scalar / mass_a);
            *vel_b -= normal * (impulse_scalar / mass_b);
        }
    }
}

pub fn draw_physics_object_gizmos(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Collider, &PhysicsObject)>,
) {
    use crate::physics::components::PhysicsObjectType;
    use crate::physics::card_selection::systems::draw_rotated_rect;

    for (transform, collider, obj) in query.iter() {
        if obj.obj_type == PhysicsObjectType::HollowSquare {
            if let Collider::Rect { size } = collider {
                let center = transform.translation.xy();
                let (_, _, angle) = transform.rotation.to_euler(EulerRot::XYZ);
                let color = Color::srgb(1.0, 0.5, 0.2); // Neon Orange
                
                // Draw outer outline
                draw_rotated_rect(&mut gizmos, center, *size, angle, color);
                // Draw inner outline (inset by 8px or so to make it look hollow/framed)
                let inner_size = *size - Vec2::new(8.0, 8.0);
                if inner_size.x > 0.0 && inner_size.y > 0.0 {
                    draw_rotated_rect(&mut gizmos, center, inner_size, angle, color);
                }
            }
        }
    }
}

