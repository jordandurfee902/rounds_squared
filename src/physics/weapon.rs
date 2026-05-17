use bevy::prelude::*;
use crate::player::{Player, Health};
use crate::physics::components::{Velocity, Collider, Platform};
use crate::physics::anim::PlayerAim;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};
use crate::settings::{PhysicsSettings, PersistentPlayerStats};
use super::particles::{spawn_spark_burst, spawn_trail_particle, spawn_damage_explosion};

// --- WEAPON CONFIGURATION COMPONENT ---
#[derive(Component, Debug, Clone)]
pub struct Weapon {
    pub max_ammo: u32,
    pub current_ammo: u32,
    pub fire_cooldown: f32,          // remaining seconds between consecutive shots
    pub fire_rate: f32,              // duration between consecutive shots (e.g., 0.3s)
    pub reload_timer: f32,           // remaining active reload time
    pub reload_time: f32,            // total active/passive reload duration (e.g., 1.2s)
    pub time_since_last_shot: f32,   // tracks passive reloading trigger
}

// --- PROJECTILE COMPONENT ---
#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub owner: Player,
    pub velocity: Vec2,
    pub base_damage: f32,
    pub damage: f32,
    pub gravity: f32,
    pub size_multiplier: f32,
    pub growth: f32,
    pub time_in_air: f32,
    pub lifetime: f32,
    pub special_effects: Vec<String>,
}

// --- REAL-TIME COOLDOWNS & RELOADS UPDATE SYSTEM ---
pub fn weapon_update_system(
    time: Res<Time>,
    mut query: Query<&mut Weapon>,
) {
    let dt = time.delta_secs().min(0.05);

    for mut weapon in query.iter_mut() {
        // 1. Progress active reload timer
        if weapon.reload_timer > 0.0 {
            weapon.reload_timer -= dt;
            if weapon.reload_timer <= 0.0 {
                // Reload finished!
                weapon.current_ammo = weapon.max_ammo;
            }
        }
        // 2. Otherwise progress passive background reload
        else if weapon.current_ammo < weapon.max_ammo {
            weapon.time_since_last_shot += dt;
            
            // Force active reload if completely empty
            if weapon.current_ammo == 0 {
                weapon.reload_timer = weapon.reload_time;
            }
            // Passive reload triggers if they hold fire for reload_time
            else if weapon.time_since_last_shot >= weapon.reload_time {
                weapon.current_ammo = weapon.max_ammo;
                weapon.time_since_last_shot = 0.0;
            }
        }

        // 3. Progress rate of fire cooldown
        if weapon.fire_cooldown > 0.0 {
            weapon.fire_cooldown -= dt;
        }
    }
}

// --- SHOOTING INPUTS & BARREL-END SPAWNING SYSTEM ---
pub fn weapon_fire_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    persistent_stats: Res<PersistentPlayerStats>,
    mut query: Query<(&Player, &Transform, &PlayerAim, &mut Velocity, &mut Weapon)>,
) {
    let mut seed_idx = 0u32;
    for (player, transform, aim, mut velocity, mut weapon) in query.iter_mut() {
        seed_idx += 1;
        
        let p_stats = match player {
            Player::P1 => &persistent_stats.p1,
            Player::P2 => &persistent_stats.p2,
        };
        
        // 1. Process manual reload keys (R for P1, P for P2)
        let manual_reload_pressed = match player {
            Player::P1 => keys.just_pressed(KeyCode::KeyR),
            Player::P2 => keys.just_pressed(KeyCode::KeyP),
        };

        if manual_reload_pressed && weapon.current_ammo < weapon.max_ammo && weapon.reload_timer <= 0.0 {
            weapon.reload_timer = weapon.reload_time;
            continue;
        }

        // 2. Process active firing inputs
        let is_firing = match player {
            Player::P1 => mouse.pressed(MouseButton::Left),
            Player::P2 => keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::KeyO),
        };

        if is_firing && weapon.current_ammo > 0 && weapon.fire_cooldown <= 0.0 && weapon.reload_timer <= 0.0 {
            // Deduct ammo and update timers
            weapon.current_ammo -= 1;
            weapon.fire_cooldown = weapon.fire_rate;
            weapon.time_since_last_shot = 0.0;

            // Spawning positions (relative to player's floating visual center)
            let scale = p_stats.player_scale;
            let body_pos = transform.translation.xy();
            let visual_center = body_pos + Vec2::new(0.0, 25.0 * scale);
            let aim_dir = aim.direction;
            let barrel_end = visual_center + aim_dir * (98.0 * scale);

            // Spawn Projectile Entity with dynamic settings
            commands.spawn((
                Projectile {
                    owner: *player,
                    velocity: aim_dir * p_stats.bullet_speed,
                    base_damage: p_stats.bullet_damage,
                    damage: p_stats.bullet_damage,
                    gravity: p_stats.bullet_gravity,
                    size_multiplier: p_stats.bullet_size_mult,
                    growth: p_stats.bullet_growth,
                    time_in_air: 0.0,
                    lifetime: p_stats.bullet_range,
                    special_effects: p_stats.special_effects.clone(),
                },
                Transform::from_xyz(barrel_end.x, barrel_end.y, 11.0),
            ));

            // Apply satisfying physical recoil kick-back scaled by bullet damage!
            let recoil_kick = p_stats.bullet_damage * 12.0;
            velocity.0 -= aim_dir * recoil_kick;

            // Trigger beautiful muzzle flash spark burst scaled by bullet damage!
            let spark_color = match player {
                Player::P1 => Color::srgb(0.4, 0.7, 1.0), // Bright Blue sparks
                Player::P2 => Color::srgb(1.0, 0.7, 0.3), // Bright Orange sparks
            };
            let spark_count = (p_stats.bullet_damage * 0.5 + 4.0).round() as usize;
            spawn_spark_burst(&mut commands, barrel_end, spark_color, spark_count, seed_idx * 100);
        }
    }
}

// --- PROJECTILE MOVEMENT, DENSE COLLISION & DAMAGE SYSTEM ---
pub fn projectile_physics_system(
    mut commands: Commands,
    time: Res<Time>,
    mut gizmos: Gizmos,
    settings: Res<PhysicsSettings>,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile), (Without<Platform>, Without<Player>)>,
    platforms: Query<(&Transform, &Collider), (With<Platform>, Without<Projectile>, Without<Player>)>,
    mut players: Query<(Entity, &Transform, &Collider, &Player, &mut Health, &crate::player::PlayerStatsComponent), (Without<Projectile>, Without<Platform>)>,
) {
    let dt = time.delta_secs().min(0.05);
    let half_width = TARGET_WIDTH / 2.0;
    let half_height = TARGET_HEIGHT / 2.0;
    let mut rng_seed = 0u32;

    for (proj_entity, mut proj_transform, mut proj) in projectiles.iter_mut() {
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
        let scale = settings.player_scale;
        let bullet_radius = current_damage.sqrt() * proj.size_multiplier * scale;

        // Print real-time dynamic growth stats so the user can verify them instantly in the console!
        if proj.growth > 0.0 && proj.time_in_air > 0.0 {
            info!("BULLET GROWTH: time={:.2}s, damage={:.1}, radius={:.1}px", proj.time_in_air, current_damage, bullet_radius);
        }

        // 1. Move Projectile coordinates
        proj_transform.translation += (proj.velocity * dt).extend(0.0);
        let curr_pos = proj_transform.translation.xy();

        // Color theme based on bullet owner (Green for Poison, Blue for P1, Orange for P2)
        let bullet_color = if proj.special_effects.contains(&"PoisonCloud".to_string()) {
            Color::srgb(0.2, 0.9, 0.2)
        } else {
            match proj.owner {
                Player::P1 => Color::srgb(0.3, 0.8, 1.0),
                Player::P2 => Color::srgb(1.0, 0.6, 0.2),
            }
        };

        // Render beautiful laser circle using the dynamic bullet_radius
        gizmos.circle_2d(curr_pos, bullet_radius, bullet_color);
        let trail_len = proj.velocity * 0.015;
        gizmos.line_2d(curr_pos - trail_len, curr_pos, bullet_color);

        // Spawn beautiful physical trail particles scaled by sqrt(damage)
        spawn_trail_particle(
            &mut commands,
            curr_pos,
            bullet_color,
            proj.damage,
            proj.velocity,
            rng_seed * 200,
        );

        // Visual Special Effects: Poison Cloud neon green outer halo
        if proj.special_effects.contains(&"PoisonCloud".to_string()) {
            // Neon green outer glowing halo
            gizmos.circle_2d(curr_pos, bullet_radius + 4.0, Color::srgb(0.2, 0.9, 0.2));
        }

        // 2. Check Playfield Boundaries
        if curr_pos.x.abs() > half_width || curr_pos.y.abs() > half_height {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // 3. Collision Check: Map Platforms (using clean circle-to-AABB intersection)
        let mut hit_detected = false;
        for (plat_trans, plat_coll) in platforms.iter() {
            if let Collider::Rect { size } = plat_coll {
                let plat_pos = plat_trans.translation.xy();
                let half_size = *size / 2.0;

                // Standard circle-AABB overlap check
                let clamped_x = curr_pos.x.clamp(plat_pos.x - half_size.x, plat_pos.x + half_size.x);
                let clamped_y = curr_pos.y.clamp(plat_pos.y - half_size.y, plat_pos.y + half_size.y);
                let closest_point = Vec2::new(clamped_x, clamped_y);
                
                if curr_pos.distance_squared(closest_point) <= bullet_radius * bullet_radius {
                    hit_detected = true;
                    // Spawn platform landing explosion particles scaled by sqrt(damage)
                    let explosion_color = if proj.special_effects.contains(&"PoisonCloud".to_string()) {
                        Color::srgb(0.2, 0.9, 0.2) // Green poison platform dust!
                    } else {
                        Color::srgb(0.8, 0.8, 0.8) // Standard gray platform dust!
                    };
                    spawn_damage_explosion(&mut commands, closest_point, explosion_color, proj.damage, rng_seed);
                    break;
                }
            }
        }

        if hit_detected {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // 4. Collision Check: Enemy Players (using circle-to-circle combined radius)
        for (_, player_trans, _, player_id, mut health, p_stats) in players.iter_mut() {
            if *player_id == proj.owner {
                continue; // Cannot hit self
            }

            let scale = p_stats.player_scale;
            let target_center = player_trans.translation.xy() + Vec2::new(0.0, 25.0 * scale);
            let dist_sq = curr_pos.distance_squared(target_center);
            let combined_radius = 40.0 * scale + bullet_radius;

            if dist_sq <= combined_radius * combined_radius {
                hit_detected = true;
                
                // Deal dynamic damage and cap at 0
                health.current = (health.current - proj.damage).max(0.0);

                // Spawn landing particle explosion scaled by sqrt(damage)
                let splatter_color = if proj.special_effects.contains(&"PoisonCloud".to_string()) {
                    Color::srgb(0.2, 0.9, 0.2) // Neon green poison splash explosion!
                } else {
                    Color::srgb(1.0, 0.2, 0.2) // Dynamic red impact explosion!
                };
                spawn_damage_explosion(&mut commands, curr_pos, splatter_color, proj.damage, rng_seed + 50);
                break;
            }
        }

        if hit_detected {
            commands.entity(proj_entity).despawn();
        }
    }
}
