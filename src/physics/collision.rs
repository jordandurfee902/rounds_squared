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
                    
                    let ring_color = match player {
                        Player::P1 => Color::srgb(0.0, 0.85, 1.0),
                        Player::P2 => Color::srgb(1.0, 0.55, 0.1),
                    };
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
                    
                    let ring_color = match player {
                        Player::P1 => Color::srgb(0.0, 0.85, 1.0),
                        Player::P2 => Color::srgb(1.0, 0.55, 0.1),
                    };
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
                    
                    let ring_color = match player {
                        Player::P1 => Color::srgb(0.0, 0.85, 1.0),
                        Player::P2 => Color::srgb(1.0, 0.55, 0.1),
                    };
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
                    
                    let ring_color = match player {
                        Player::P1 => Color::srgb(0.0, 0.85, 1.0),
                        Player::P2 => Color::srgb(1.0, 0.55, 0.1),
                    };
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
    mut players: Query<(&mut Transform, &mut Velocity, &Collider, &mut Grounded, &mut WallContact), Without<Platform>>,
    platforms: Query<(&Transform, &Collider), With<Platform>>,
) {
    let restitution = settings.boundary_restitution;

    for (mut p_trans, mut p_vel, p_coll, mut grounded, mut wall) in players.iter_mut() {
        if let Collider::Circle { radius } = p_coll {
            for (plat_trans, plat_coll) in platforms.iter() {
                if let Collider::Rect { size } = plat_coll {
                    let player_pos = p_trans.translation.xy();
                    let plat_pos = plat_trans.translation.xy();
                    let half_size = *size / 2.0;

                    let closest = Vec2::new(
                        player_pos.x.clamp(plat_pos.x - half_size.x, plat_pos.x + half_size.x),
                        player_pos.y.clamp(plat_pos.y - half_size.y, plat_pos.y + half_size.y),
                    );

                    let dist_sq = player_pos.distance_squared(closest);
                    let dist = dist_sq.sqrt();

                    // Perform check using contact skin/buffer from settings
                    if dist <= *radius + settings.collision_penetration_skin_buffer {
                        let normal = if dist > 0.0 {
                            (player_pos - closest) / dist
                        } else {
                            Vec2::Y
                        };

                        // Only resolve physical overlap if they actually penetrate
                        if dist < *radius {
                            let overlap = *radius - dist;
                            p_trans.translation += (normal * overlap).extend(0.0);
                        }

                        // Stable ground contact: normal points mostly up
                        if normal.y > settings.grounded_slope_threshold {
                            grounded.0 = true;
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
