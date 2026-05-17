use bevy::prelude::*;
use crate::settings::PhysicsSettings;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};
use super::components::*;

pub fn boundary_collision(
    settings: Res<PhysicsSettings>,
    mut query: Query<(&mut Transform, &mut Velocity, &Collider, &mut Grounded, &mut WallContact)>,
) {
    let half_width = TARGET_WIDTH / 2.0;
    let half_height = TARGET_HEIGHT / 2.0;
    let restitution = settings.boundary_restitution;

    for (mut transform, mut velocity, collider, mut grounded, mut wall) in query.iter_mut() {
        if let Collider::Circle { radius } = collider {
            let mut pos = transform.translation.xy();
            let mut vel = velocity.0;

            // Horizontal bounds (with 1.0-pixel contact skin/buffer)
            if pos.x - radius <= -half_width + 1.0 {
                if pos.x - radius < -half_width {
                    pos.x = -half_width + radius;
                }
                if vel.x < 0.0 {
                    vel.x = vel.x.abs() * restitution;
                }
                wall.left = true;
            } else if pos.x + radius >= half_width - 1.0 {
                if pos.x + radius > half_width {
                    pos.x = half_width - radius;
                }
                if vel.x > 0.0 {
                    vel.x = -vel.x.abs() * restitution;
                }
                wall.right = true;
            }

            // Vertical bounds (with 1.0-pixel contact skin/buffer)
            if pos.y - radius <= -half_height + 1.0 {
                if pos.y - radius < -half_height {
                    pos.y = -half_height + radius;
                }
                if vel.y < 0.0 {
                    vel.y = vel.y.abs() * restitution;
                }
                grounded.0 = true;
            } else if pos.y + radius >= half_height - 1.0 {
                if pos.y + radius > half_height {
                    pos.y = half_height - radius;
                }
                if vel.y > 0.0 {
                    vel.y = -vel.y.abs() * restitution;
                }
            }

            transform.translation = pos.extend(transform.translation.z);
            velocity.0 = vel;
        }
    }
}

pub fn player_collision(
    settings: Res<PhysicsSettings>,
    mut query: Query<(&mut Transform, &mut Velocity, &Collider)>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(mut t1, mut v1, c1), (mut t2, mut v2, c2)]) = combinations.fetch_next() {
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

                // Resolve overlap (push apart equally)
                t1.translation += (normal * overlap * 0.5).extend(0.0);
                t2.translation -= (normal * overlap * 0.5).extend(0.0);

                // Resolve velocity (elastic collision)
                let relative_velocity = v1.0 - v2.0;
                let velocity_along_normal = relative_velocity.dot(normal);

                // Only resolve if velocities are towards each other
                if velocity_along_normal < 0.0 {
                    let restitution = settings.player_restitution;
                    let impulse_scalar = -(1.0 + restitution) * velocity_along_normal;
                    let impulse = normal * impulse_scalar * 0.5; // Equal mass-scale resolution

                    v1.0 += impulse;
                    v2.0 -= impulse;
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

                    // Perform check using a 1.0-pixel contact skin/buffer
                    if dist <= *radius + 1.0 {
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
                        if normal.y > 0.5 {
                            grounded.0 = true;
                        }

                        // Stable wall contacts:
                        if normal.x > 0.5 {
                            wall.left = true;
                        } else if normal.x < -0.5 {
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
