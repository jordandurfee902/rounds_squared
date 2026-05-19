use bevy::prelude::*;
use crate::player::{Player, Health};
use crate::physics::components::{Collider, Platform, Velocity};
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};
use crate::settings::PhysicsSettings;
use crate::physics::particles::spawn_damage_explosion;
use super::components::*;

fn trigger_impact_shake(damage: f32, shake: &mut crate::graphics::ScreenShake) {
    if damage > 100.0 {
        let base_intensity = 6.0;
        let extra_intensity = (damage - 100.0) * 0.05;
        shake.intensity = (base_intensity + extra_intensity).min(35.0);
        shake.duration = (0.15 + (damage - 100.0) * 0.001).min(0.45);
    }
}

// --- PROJECTILE MOVEMENT, DENSE COLLISION & DAMAGE SYSTEM ---
pub fn projectile_physics_system(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile), (Without<Platform>, Without<Player>)>,
    platforms: Query<(&Transform, &Collider), (With<Platform>, Without<Projectile>, Without<Player>)>,
    mut players: Query<(Entity, &Transform, &Collider, &Player, &mut Health, &crate::player::PlayerStatsComponent, &crate::player::BlockComponent, &mut Velocity), (Without<Projectile>, Without<Platform>)>,
    settings: Res<PhysicsSettings>,
    mut shake: ResMut<crate::graphics::ScreenShake>,
    persistent_stats: Option<Res<crate::settings::PersistentPlayerStats>>,
) {
    let dt = time.delta_secs().min(0.05);
    let half_width = TARGET_WIDTH / 2.0;
    let half_height = TARGET_HEIGHT / 2.0;
    let mut rng_seed = 0u32;

    for (proj_entity, mut proj_transform, mut proj) in projectiles.iter_mut() {
        let owner_cards = if let Some(p_stats) = persistent_stats.as_ref() {
            p_stats.players[proj.owner.index()].cards.clone()
        } else {
            Vec::new()
        };

        rng_seed += 1;
        proj.lifetime -= dt;
        proj.time_in_air += dt;

        if proj.lifetime <= 0.0 {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // Apply projectile specific gravity
        proj.velocity.y += proj.gravity * dt;

        // Exponential damage growth (e.g. damage = base * (1 + growth)^time)
        let current_damage = proj.base_damage * (1.0 + proj.growth).powf(proj.time_in_air);
        proj.damage = current_damage;

        // Solve dynamic visual and collision radius: sqrt(damage) * size_multiplier * player_scale
        let scale = proj.player_scale;
        let bullet_radius = current_damage.sqrt() * proj.size_multiplier * scale;

        // Print real-time dynamic growth stats so the user can verify them instantly in the console!
        if proj.growth > 0.0 && proj.time_in_air > 0.0 {
            info!("BULLET GROWTH: time={:.2}s, damage={:.1}, radius={:.1}px", proj.time_in_air, current_damage, bullet_radius);
        }

        // 1. Move Projectile coordinates
        proj_transform.translation += (proj.velocity * dt).extend(0.0);
        let curr_pos = proj_transform.translation.xy();

        // 2. Check Playfield Boundaries
        let mut out_of_bounds = false;
        let spark_color = if proj.special_effects.contains(&"PoisonCloud".to_string()) {
            Color::srgb(0.2, 0.9, 0.2)
        } else {
            Color::srgb(0.8, 0.8, 0.8)
        };

        // Horizontal bounds check
        if curr_pos.x.abs() > half_width {
            let sign = curr_pos.x.signum();
            if proj.bounces > 0 {
                proj.velocity.x = -proj.velocity.x * proj.bounce_speed_multiplier;
                proj.bounces -= 1;
                proj_transform.translation.x = sign * (half_width - bullet_radius - 1.0);
                spawn_damage_explosion(&mut commands, Vec2::new(sign * half_width, curr_pos.y), spark_color, proj.damage, rng_seed);
            } else {
                // Despawn and trigger particle explosion!
                spawn_damage_explosion(&mut commands, Vec2::new(sign * half_width, curr_pos.y), spark_color, proj.damage, rng_seed);
                out_of_bounds = true;
            }
        }

        // Vertical bounds check (only run if not already marked out_of_bounds)
        if !out_of_bounds && curr_pos.y.abs() > half_height {
            let sign = curr_pos.y.signum();
            if sign > 0.0 {
                // Top border!
                if proj.bounces > 0 {
                    // Bounce off the top border
                    proj.velocity.y = -proj.velocity.y * proj.bounce_speed_multiplier;
                    proj.bounces -= 1;
                    proj_transform.translation.y = half_height - bullet_radius - 1.0;
                    spawn_damage_explosion(&mut commands, Vec2::new(curr_pos.x, half_height), spark_color, proj.damage, rng_seed);
                } else {
                    // 0 bounces: Allow to pass through the top border! Do NOT bounce, do NOT despawn, do NOT trigger particles!
                }
            } else {
                // Bottom border!
                if proj.bounces > 0 {
                    // Bounce off bottom border
                    proj.velocity.y = -proj.velocity.y * proj.bounce_speed_multiplier;
                    proj.bounces -= 1;
                    proj_transform.translation.y = -half_height + bullet_radius + 1.0;
                    spawn_damage_explosion(&mut commands, Vec2::new(curr_pos.x, -half_height), spark_color, proj.damage, rng_seed);
                } else {
                    // Despawn and trigger particle explosion!
                    spawn_damage_explosion(&mut commands, Vec2::new(curr_pos.x, -half_height), spark_color, proj.damage, rng_seed);
                    out_of_bounds = true;
                }
            }
        }

        if out_of_bounds {
            trigger_impact_shake(proj.damage, &mut shake);
            for &card_idx in owner_cards.iter() {
                if let Some(card) = crate::physics::card_selection::cards::get_card(card_idx) {
                    card.on_bullet_land(&mut commands, &proj, curr_pos);
                }
            }
            commands.entity(proj_entity).despawn();
            continue;
        }

        // 3. Collision Check: Map Platforms (using clean circle-to-AABB intersection)
        let mut hit_detected = false;
        let mut land_pos = curr_pos;
        for (plat_trans, plat_coll) in platforms.iter() {
            if let Collider::Rect { size } = plat_coll {
                let plat_pos = plat_trans.translation.xy();
                let half_size = *size / 2.0;

                // Standard circle-AABB overlap check
                let clamped_x = curr_pos.x.clamp(plat_pos.x - half_size.x, plat_pos.x + half_size.x);
                let clamped_y = curr_pos.y.clamp(plat_pos.y - half_size.y, plat_pos.y + half_size.y);
                let closest_point = Vec2::new(clamped_x, clamped_y);
                
                if curr_pos.distance_squared(closest_point) <= bullet_radius * bullet_radius {
                    // Spawn platform landing explosion particles scaled by sqrt(damage)
                    let explosion_color = if proj.special_effects.contains(&"PoisonCloud".to_string()) {
                        Color::srgb(0.2, 0.9, 0.2) // Green poison platform dust!
                    } else {
                        Color::srgb(0.8, 0.8, 0.8) // Standard gray platform dust!
                    };
                    spawn_damage_explosion(&mut commands, closest_point, explosion_color, proj.damage, rng_seed);

                    if proj.bounces > 0 {
                        let diff = curr_pos - closest_point;
                        let normal = if diff.length_squared() > 1e-4 {
                            diff.normalize()
                        } else {
                            -proj.velocity.normalize_or_zero()
                        };
                        
                        let reflected = proj.velocity - 2.0 * proj.velocity.dot(normal) * normal;
                        proj.velocity = reflected * proj.bounce_speed_multiplier;
                        proj.bounces -= 1;

                        // Reposition projectile to prevent double-collision sticking
                        let new_pos = closest_point + normal * (bullet_radius + 1.0);
                        proj_transform.translation.x = new_pos.x;
                        proj_transform.translation.y = new_pos.y;
                    } else {
                        hit_detected = true;
                        land_pos = closest_point;
                    }
                    break;
                }
            }
        }

        if hit_detected {
            trigger_impact_shake(proj.damage, &mut shake);
            for &card_idx in owner_cards.iter() {
                if let Some(card) = crate::physics::card_selection::cards::get_card(card_idx) {
                    card.on_bullet_land(&mut commands, &proj, land_pos);
                }
            }
            commands.entity(proj_entity).despawn();
            continue;
        }

        // 4. Collision Check: Enemy Players (using circle-to-circle combined radius)
        for (_, player_trans, _, player_id, mut health, p_stats, block, mut player_velocity) in players.iter_mut() {
            if *player_id == proj.owner {
                continue; // Cannot hit self
            }

            let scale = p_stats.player_scale;
            let target_center = player_trans.translation.xy() + Vec2::new(0.0, settings.player_aim_offset_y * scale);
            let dist_sq = curr_pos.distance_squared(target_center);
            let combined_radius = settings.player_base_radius * scale + bullet_radius;

            if dist_sq <= combined_radius * combined_radius {
                // If player is actively blocking (invincible), bounce/ricochet the bullet!
                if block.active_timer > 0.0 {
                    let diff = curr_pos - target_center;
                    let normal = if diff.length_squared() > 1e-4 {
                        diff.normalize()
                    } else {
                        -proj.velocity.normalize_or_zero()
                    };

                    let reflected = proj.velocity - 2.0 * proj.velocity.dot(normal) * normal;
                    let speed_mult = if proj.bounce_speed_multiplier > 0.0 { proj.bounce_speed_multiplier } else { 1.0 };
                    proj.velocity = reflected * speed_mult;
                    
                    if proj.bounces > 0 {
                        proj.bounces -= 1;
                    }

                    // Ownership shifts to the blocking player! Excellent custom deflecting mechanic.
                    proj.owner = *player_id;

                    // Reposition projectile to prevent double-collision sticking
                    let new_pos = target_center + normal * (combined_radius + 1.0);
                    proj_transform.translation.x = new_pos.x;
                    proj_transform.translation.y = new_pos.y;

                    // Spawn impact shield splash effect
                    spawn_damage_explosion(&mut commands, curr_pos, Color::srgb(0.5, 0.8, 1.0), proj.damage, rng_seed + 100);
                } else {
                    hit_detected = true;
                    land_pos = curr_pos;
                    
                    // Deal dynamic damage and cap at 0
                    health.current = (health.current - proj.damage).max(0.0);

                    // Apply bullet knockback: direction of travel * (sqrt(damage) * settings.bullet_knockback_constant)
                    let knockback_dir = proj.velocity.normalize_or_zero();
                    let knockback_mag = proj.damage.sqrt() * settings.bullet_knockback_constant;
                    player_velocity.0 += knockback_dir * knockback_mag;

                    // Spawn landing particle explosion scaled by sqrt(damage)
                    let splatter_color = if proj.special_effects.contains(&"PoisonCloud".to_string()) {
                        Color::srgb(0.2, 0.9, 0.2) // Neon green poison splash explosion!
                    } else {
                        Color::srgb(1.0, 0.2, 0.2) // Dynamic red impact explosion!
                    };
                    spawn_damage_explosion(&mut commands, curr_pos, splatter_color, proj.damage, rng_seed + 50);
                }
                break;
            }
        }

        if hit_detected {
            trigger_impact_shake(proj.damage, &mut shake);
            for &card_idx in owner_cards.iter() {
                if let Some(card) = crate::physics::card_selection::cards::get_card(card_idx) {
                    card.on_bullet_land(&mut commands, &proj, land_pos);
                }
            }
            commands.entity(proj_entity).despawn();
        }
    }
}
