use bevy::prelude::*;
use crate::settings::PhysicsSettings;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};
use super::components::*;
use crate::player::{Player, Health, BlockComponent, PlayerStatsComponent};
use crate::physics::particles::spawn_damage_explosion;

pub fn boundary_collision(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<PhysicsSettings>,
    mut query: Query<(
        &Player,
        &mut Transform,
        &mut Velocity,
        &Collider,
        &mut Grounded,
        &mut WallContact,
        &mut Health,
        &mut BlockComponent,
        &PlayerStatsComponent,
    )>,
) {
    let half_width = TARGET_WIDTH / 2.0;
    let half_height = TARGET_HEIGHT / 2.0;
    let _elapsed = time.delta_secs(); // Use delta_secs elapsed helper or total elapsed
    let total_elapsed = time.elapsed_secs();
    // grace period on level load to prevent spawn damage
    let grace_period = total_elapsed < settings.spawn_invincibility_grace_period;

    let mut seed = 0u32;
    for (player, mut transform, mut velocity, collider, mut grounded, mut wall, mut health, mut block, stats) in query.iter_mut() {
        seed += 1;
        if let Collider::Circle { radius } = collider {
            let mut pos = transform.translation.xy();
            let mut vel = velocity.0;

            // Horizontal bounds (with contact skin/buffer from settings)
            if pos.x - radius <= -half_width + settings.collision_penetration_skin_buffer {
                if block.active_timer > 0.0 {
                    // Border block deflect!
                    vel.x = stats.block_border_boost;
                    pos.x = -half_width + radius + 15.0; // push inside safely
                    block.control_lockout_timer = settings.boundary_deflect_lockout; // Disable input control to carry launch momentum
                    
                    let ring_color = player.color();
                    spawn_damage_explosion(&mut commands, Vec2::new(-half_width, pos.y), ring_color, 25.0, seed);
                } else if !grace_period && vel.x < -10.0 {
                    // Take boundary damage!
                    health.current = (health.current - settings.boundary_hazard_damage).max(0.0);
                    vel.x = settings.boundary_knockback_speed;
                    pos.x = -half_width + radius + 5.0;
                    block.control_lockout_timer = settings.boundary_damage_lockout; // Disable controls during knockback
                    spawn_damage_explosion(&mut commands, Vec2::new(-half_width, pos.y), Color::srgb(1.0, 0.2, 0.2), settings.boundary_hazard_damage, seed + 10);
                } else {
                    if pos.x - radius < -half_width {
                        pos.x = -half_width + radius;
                    }
                    wall.left = true;
                }
            } else if pos.x + radius >= half_width - settings.collision_penetration_skin_buffer {
                if block.active_timer > 0.0 {
                    // Border block deflect!
                    vel.x = -stats.block_border_boost;
                    pos.x = half_width - radius - 15.0; // push inside safely
                    block.control_lockout_timer = settings.boundary_deflect_lockout; // Disable input control to carry launch momentum
                    
                    let ring_color = player.color();
                    spawn_damage_explosion(&mut commands, Vec2::new(half_width, pos.y), ring_color, 25.0, seed + 1);
                } else if !grace_period && vel.x > 10.0 {
                    // Take boundary damage!
                    health.current = (health.current - settings.boundary_hazard_damage).max(0.0);
                    vel.x = -settings.boundary_knockback_speed;
                    pos.x = half_width - radius - 5.0;
                    block.control_lockout_timer = settings.boundary_damage_lockout; // Disable controls during knockback
                    spawn_damage_explosion(&mut commands, Vec2::new(half_width, pos.y), Color::srgb(1.0, 0.2, 0.2), settings.boundary_hazard_damage, seed + 11);
                } else {
                    if pos.x + radius > half_width {
                        pos.x = half_width - radius;
                    }
                    wall.right = true;
                }
            }

            // Vertical bounds (with contact skin/buffer from settings)
            if pos.y - radius <= -half_height + settings.collision_penetration_skin_buffer {
                if block.active_timer > 0.0 {
                    // Border block deflect!
                    vel.y = stats.block_border_boost;
                    pos.y = -half_height + radius + 15.0; // push inside safely
                    block.control_lockout_timer = settings.boundary_deflect_lockout; // Disable input control to carry launch momentum
                    
                    let ring_color = player.color();
                    spawn_damage_explosion(&mut commands, Vec2::new(pos.x, -half_height), ring_color, 25.0, seed + 2);
                } else if !grace_period && vel.y < -10.0 {
                    // Take boundary damage!
                    health.current = (health.current - settings.boundary_hazard_damage).max(0.0);
                    vel.y = settings.boundary_knockback_speed;
                    pos.y = -half_height + radius + 5.0;
                    block.control_lockout_timer = settings.boundary_damage_lockout; // Disable controls during knockback
                    spawn_damage_explosion(&mut commands, Vec2::new(pos.x, -half_height), Color::srgb(1.0, 0.2, 0.2), settings.boundary_hazard_damage, seed + 12);
                    grounded.0 = true;
                } else {
                    if pos.y - radius < -half_height {
                        pos.y = -half_height + radius;
                    }
                    grounded.0 = true;
                }
            } else if pos.y + radius >= half_height - settings.collision_penetration_skin_buffer {
                if block.active_timer > 0.0 {
                    // Border block deflect!
                    vel.y = -stats.block_border_boost;
                    pos.y = half_height - radius - 15.0; // push inside safely
                    block.control_lockout_timer = settings.boundary_deflect_lockout; // Disable input control to carry launch momentum
                    
                    let ring_color = player.color();
                    spawn_damage_explosion(&mut commands, Vec2::new(pos.x, half_height), ring_color, 25.0, seed + 3);
                } else if !grace_period && vel.y > 10.0 {
                    // Take boundary damage!
                    health.current = (health.current - settings.boundary_hazard_damage).max(0.0);
                    vel.y = -settings.boundary_knockback_speed;
                    pos.y = half_height - radius - 5.0;
                    block.control_lockout_timer = settings.boundary_damage_lockout; // Disable controls during knockback
                    spawn_damage_explosion(&mut commands, Vec2::new(pos.x, half_height), Color::srgb(1.0, 0.2, 0.2), settings.boundary_hazard_damage, seed + 13);
                } else {
                    if pos.y + radius > half_height {
                        pos.y = half_height - radius;
                    }
                }
            }

            transform.translation = pos.extend(transform.translation.z);
            velocity.0 = vel;
        }
    }
}

pub fn player_collision(
    settings: Res<PhysicsSettings>,
    mut query: Query<(&mut Transform, &mut Velocity, &Collider, Option<&Mass>)>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(mut t1, mut v1, c1, m1), (mut t2, mut v2, c2, m2)]) = combinations.fetch_next() {
        if let (Collider::Circle { radius: r1 }, Collider::Circle { radius: r2 }) = (c1, c2) {
            let pos1 = t1.translation.xy();
            let pos2 = t2.translation.xy();
            let dist_sq = pos1.distance_squared(pos2);
            let combined_radius = r1 + r2;

            if dist_sq < combined_radius * combined_radius {
                let dist = dist_sq.sqrt();
                let normal = if dist > 0.0 {
                    (pos1 - pos2) / dist
                } else {
                    Vec2::Y
                };
                let overlap = combined_radius - dist;

                let mass1 = m1.map(|m| m.0).unwrap_or(1.0);
                let mass2 = m2.map(|m| m.0).unwrap_or(1.0);
                let total_inverse_mass = (1.0 / mass1) + (1.0 / mass2);

                // Resolve overlap (push apart inversely proportional to mass)
                if total_inverse_mass > 0.0 {
                    let ratio1 = (1.0 / mass1) / total_inverse_mass;
                    let ratio2 = (1.0 / mass2) / total_inverse_mass;
                    t1.translation += (normal * overlap * settings.overlapping_push_factor * ratio1).extend(0.0);
                    t2.translation -= (normal * overlap * settings.overlapping_push_factor * ratio2).extend(0.0);
                } else {
                    t1.translation += (normal * overlap * settings.overlapping_push_factor * 0.5).extend(0.0);
                    t2.translation -= (normal * overlap * settings.overlapping_push_factor * 0.5).extend(0.0);
                }

                // Resolve velocity (elastic collision using Mass)
                let relative_velocity = v1.0 - v2.0;
                let velocity_along_normal = relative_velocity.dot(normal);

                // Only resolve if velocities are towards each other
                if velocity_along_normal < 0.0 {
                    let restitution = settings.player_restitution;
                    if total_inverse_mass > 0.0 {
                        let impulse_scalar = -(1.0 + restitution) * velocity_along_normal / total_inverse_mass;
                        v1.0 += normal * (impulse_scalar / mass1);
                        v2.0 -= normal * (impulse_scalar / mass2);
                    } else {
                        let impulse_scalar = -(1.0 + restitution) * velocity_along_normal;
                        let impulse = normal * impulse_scalar * 0.5;
                        v1.0 += impulse;
                        v2.0 -= impulse;
                    }
                }
            }
        }
    }
}

pub fn player_platform_collision(
    settings: Res<PhysicsSettings>,
    mut players: Query<(&mut Transform, &mut Velocity, &Collider, &mut Grounded, &mut WallContact, &mut StandingOn), Without<Platform>>,
    platforms: Query<(Entity, &Transform, &Collider), With<Platform>>,
) {
    let restitution = settings.boundary_restitution;

    for (mut p_trans, mut p_vel, p_coll, mut grounded, mut wall, mut standing_on) in players.iter_mut() {
        if let Collider::Circle { radius } = p_coll {
            for (plat_ent, plat_trans, plat_coll) in platforms.iter() {
                if let Collider::Rect { size } = plat_coll {
                    let player_pos = p_trans.translation.xy();
                    let plat_pos = plat_trans.translation.xy();
                    let plat_rot = plat_trans.rotation;
                    let (_, _, theta) = plat_rot.to_euler(EulerRot::XYZ);

                    let diff = player_pos - plat_pos;
                    let cos_t = (-theta).cos();
                    let sin_t = (-theta).sin();
                    let player_local = Vec2::new(
                        diff.x * cos_t - diff.y * sin_t,
                        diff.x * sin_t + diff.y * cos_t,
                    );

                    let half_size = *size / 2.0;
                    let closest_local = Vec2::new(
                        player_local.x.clamp(-half_size.x, half_size.x),
                        player_local.y.clamp(-half_size.y, half_size.y),
                    );

                    let dist = player_local.distance(closest_local);

                    // Perform check using contact skin/buffer from settings
                    if dist <= *radius + settings.collision_penetration_skin_buffer {
                        let normal_local = if dist > 0.0 {
                            (player_local - closest_local) / dist
                        } else {
                            Vec2::Y
                        };

                        let cos_w = theta.cos();
                        let sin_w = theta.sin();
                        let normal = Vec2::new(
                            normal_local.x * cos_w - normal_local.y * sin_w,
                            normal_local.x * sin_w + normal_local.y * cos_w,
                        );

                        // Only resolve physical overlap if they actually penetrate
                        if dist < *radius {
                            let overlap = *radius - dist;
                            p_trans.translation += (normal * overlap).extend(0.0);
                        }

                        // Stable ground contact: normal points mostly up
                        if normal.y > settings.grounded_slope_threshold {
                            grounded.0 = true;
                            standing_on.0 = Some(plat_ent);
                        }

                        // Stable wall contacts:
                        if normal.x > settings.wall_contact_slope_threshold {
                            wall.left = true;
                        } else if normal.x < -settings.wall_contact_slope_threshold {
                            wall.right = true;
                        }

                        let velocity_along_normal = p_vel.0.dot(normal);
                        if velocity_along_normal < 0.0 {
                            p_vel.0 -= (1.0 + restitution) * velocity_along_normal * normal;
                        }
                    }
                }
            }
        }
    }
}
