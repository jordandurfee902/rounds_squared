use bevy::prelude::*;
use crate::player::Player;
use crate::physics::components::ControllerInput;
use crate::physics::anim::PlayerAim;
use crate::settings::{PersistentPlayerStats, PhysicsSettings};
use crate::physics::particles::spawn_spark_burst;
use super::components::*;

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
    persistent_stats: Res<PersistentPlayerStats>,
    mut query: Query<(&Player, &Transform, &PlayerAim, &ControllerInput, &mut Weapon)>,
    physics_settings: Res<PhysicsSettings>,
) {
    let mut seed_idx = 0u32;
    for (player, transform, aim, input, mut weapon) in query.iter_mut() {
        seed_idx += 1;
        
        let p_stats = match player {
            Player::P1 => &persistent_stats.p1,
            Player::P2 => &persistent_stats.p2,
        };

        // 1. Process manual reload inputs
        let manual_reload_pressed = input.reload;

        if manual_reload_pressed && weapon.current_ammo < weapon.max_ammo && weapon.reload_timer <= 0.0 {
            weapon.reload_timer = weapon.reload_time;
            continue;
        }

        // 2. Process active firing inputs
        let is_firing = input.fire;

        if is_firing && weapon.current_ammo > 0 && weapon.fire_cooldown <= 0.0 && weapon.reload_timer <= 0.0 {
            // Deduct ammo and update timers
            weapon.current_ammo -= 1;
            weapon.fire_cooldown = weapon.fire_rate;
            weapon.time_since_last_shot = 0.0;

            // Spawning positions (relative to player's floating visual center)
            let scale = p_stats.player_scale;
            let body_pos = transform.translation.xy();
            let visual_center = body_pos + Vec2::new(0.0, physics_settings.player_aim_offset_y * scale);
            let aim_dir = aim.direction;
            let barrel_end = visual_center + aim_dir * (98.0 * scale);

            // Spawn Projectile Entity with dynamic settings
            commands.spawn((
                Projectile {
                    owner: player.clone(),
                    velocity: aim_dir * p_stats.bullet_speed,
                    base_damage: p_stats.bullet_damage,
                    damage: p_stats.bullet_damage,
                    gravity: p_stats.bullet_gravity,
                    size_multiplier: p_stats.bullet_size_mult,
                    growth: p_stats.bullet_growth,
                    time_in_air: 0.0,
                    lifetime: p_stats.bullet_range,
                    special_effects: p_stats.special_effects.clone(),
                    player_scale: p_stats.player_scale,
                    bounces: p_stats.bounces,
                    bounce_speed_multiplier: p_stats.bounce_speed_multiplier,
                },
                Transform::from_xyz(barrel_end.x, barrel_end.y, 11.0),
            ));

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
