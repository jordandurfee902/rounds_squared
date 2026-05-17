use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Gameplay,
    CardSelection,
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub movement_speed: f32,
    pub jump_force: f32,
    pub player_scale: f32,
    pub health_max: f32,
    // Weapon stats
    pub bullet_range: f32,
    pub bullet_speed: f32,
    pub bullet_gravity: f32,
    pub bullet_damage: f32,
    pub bullet_size_mult: f32,
    pub bullet_growth: f32,
    pub max_ammo: u32,
    pub reload_time: f32,
    pub fire_rate: f32,
    pub bounces: u32,
    pub bounce_speed_multiplier: f32,
    pub block_duration: f32,
    pub block_cooldown: f32,
    pub block_border_boost: f32,
    pub special_effects: Vec<String>,
}

#[derive(Resource, Debug, Clone)]
pub struct PersistentPlayerStats {
    pub p1: PlayerStats,
    pub p2: PlayerStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterSettings {
    /// Maximum HP of the character (e.g. 100.0 HP)
    pub health: f32,
    /// Maximum movement speed of the character in pixels per second (e.g. 600.0 px/sec)
    pub speed: f32,
    /// Size/scale multiplier of the character (e.g. 1.5 multiplier)
    pub size: f32,
    /// Base projectile damage on impact (e.g. 12.0 damage points)
    pub damage: f32,
    /// Maximum duration of bullet flight in seconds (e.g. 2.2 seconds)
    pub bullet_range: f32,
    /// Projectile firing velocity in pixels per second (e.g. 1000.0 px/sec)
    pub bullet_speed: f32,
    /// Gravity applied to the bullet in pixels per second squared (e.g. -600.0 px/sec^2)
    pub bullet_gravity: f32,
    /// Projectile size scaling multiplier (e.g. 1.1 multiplier)
    pub bullet_size_mult: f32,
    /// Rate of exponential damage growth per second in the air (e.g. 10.0 for 10% increase per second)
    pub bullet_growth: f32,
    /// Magazine size (number of bullets in a full clip)
    pub max_ammo: u32,
    /// Reload time in seconds (e.g. 1.2 seconds)
    pub reload_time: f32,
    /// Fire rate - minimum time between shots in seconds (e.g. 0.35 seconds)
    pub fire_rate: f32,
    /// Number of bullet bounces/ricochets off walls and platforms
    pub bounces: u32,
    /// Velocity retention factor after each bounce (e.g. 0.75 for 75%)
    pub bounce_speed_multiplier: f32,
    /// Invincibility duration on block (seconds)
    pub block_duration: f32,
    /// Wait time between block activations (seconds)
    pub block_cooldown: f32,
    /// Velocity boost when blocking off world boundaries (pixels/second)
    pub block_border_boost: f32,
    /// List of active special effects on shots (e.g. ["PoisonCloud"])
    pub special_effects: Vec<String>,
}

impl Default for CharacterSettings {
    fn default() -> Self {
        Self {
            health: 100.0,
            speed: 600.0,
            size: 1.5,
            damage: 15.0,
            bullet_range: 2.0,
            bullet_speed: 1250.0,
            bullet_gravity: 0.0,
            bullet_size_mult: 1.0,
            bullet_growth: 0.0,
            max_ammo: 6,
            reload_time: 1.2,
            fire_rate: 0.35,
            bounces: 0,
            bounce_speed_multiplier: 0.0,
            block_duration: 0.20,
            block_cooldown: 4.0,
            block_border_boost: 1800.0,
            special_effects: Vec::new(),
        }
    }
}

#[derive(Resource, Debug, Serialize, Deserialize, Clone)]
pub struct PhysicsSettings {
    pub gravity: f32,
    pub player_accel: f32,
    pub player_jump_force: f32,
    pub boundary_restitution: f32,
    pub player_restitution: f32,
    pub air_friction: f32,
    pub ground_friction: f32,
    pub movement_stop_friction: f32,
    pub wall_slide_speed: f32,
    pub wall_jump_push_force: f32,
    pub fast_fall_acceleration: f32,
    pub air_accel: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: -1600.0,
            player_accel: 10.0,
            player_jump_force: 800.0,
            boundary_restitution: 0.0,
            player_restitution: 0.5,
            air_friction: 1.0,
            ground_friction: 1.0,
            movement_stop_friction: 10.0,
            wall_slide_speed: 150.0,
            wall_jump_push_force: 800.0,
            fast_fall_acceleration: 4000.0,
            air_accel: 25.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct P1WeaponSettings(pub CharacterSettings);

#[derive(Resource, Debug, Clone)]
pub struct P2WeaponSettings(pub CharacterSettings);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub physics: PhysicsSettings,
    pub p1_character: CharacterSettings,
    pub p2_character: CharacterSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            physics: PhysicsSettings::default(),
            p1_character: CharacterSettings::default(),
            p2_character: CharacterSettings::default(),
        }
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ScoreTracker {
    pub p1_wins: u32,
    pub p2_wins: u32,
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = load_settings();
        app.insert_resource(settings.physics.clone());
        app.insert_resource(P1WeaponSettings(settings.p1_character.clone()));
        app.insert_resource(P2WeaponSettings(settings.p2_character.clone()));
        app.insert_resource(ScoreTracker::default());

        let p1_stats = PlayerStats {
            movement_speed: settings.p1_character.speed,
            jump_force: settings.physics.player_jump_force,
            player_scale: settings.p1_character.size,
            health_max: settings.p1_character.health,
            bullet_range: settings.p1_character.bullet_range,
            bullet_speed: settings.p1_character.bullet_speed,
            bullet_gravity: settings.p1_character.bullet_gravity,
            bullet_damage: settings.p1_character.damage,
            bullet_size_mult: settings.p1_character.bullet_size_mult,
            bullet_growth: settings.p1_character.bullet_growth,
            max_ammo: settings.p1_character.max_ammo,
            reload_time: settings.p1_character.reload_time,
            fire_rate: settings.p1_character.fire_rate,
            bounces: settings.p1_character.bounces,
            bounce_speed_multiplier: settings.p1_character.bounce_speed_multiplier,
            block_duration: settings.p1_character.block_duration,
            block_cooldown: settings.p1_character.block_cooldown,
            block_border_boost: settings.p1_character.block_border_boost,
            special_effects: settings.p1_character.special_effects.clone(),
        };
        let p2_stats = PlayerStats {
            movement_speed: settings.p2_character.speed,
            jump_force: settings.physics.player_jump_force,
            player_scale: settings.p2_character.size,
            health_max: settings.p2_character.health,
            bullet_range: settings.p2_character.bullet_range,
            bullet_speed: settings.p2_character.bullet_speed,
            bullet_gravity: settings.p2_character.bullet_gravity,
            bullet_damage: settings.p2_character.damage,
            bullet_size_mult: settings.p2_character.bullet_size_mult,
            bullet_growth: settings.p2_character.bullet_growth,
            max_ammo: settings.p2_character.max_ammo,
            reload_time: settings.p2_character.reload_time,
            fire_rate: settings.p2_character.fire_rate,
            bounces: settings.p2_character.bounces,
            bounce_speed_multiplier: settings.p2_character.bounce_speed_multiplier,
            block_duration: settings.p2_character.block_duration,
            block_cooldown: settings.p2_character.block_cooldown,
            block_border_boost: settings.p2_character.block_border_boost,
            special_effects: settings.p2_character.special_effects.clone(),
        };
        app.insert_resource(PersistentPlayerStats { p1: p1_stats, p2: p2_stats });
        app.init_state::<GameState>();
    }
}

fn load_settings() -> AppSettings {
    let mut file = match File::open("settings.json") {
        Ok(f) => f,
        Err(_) => {
            println!("Settings file not found, using defaults.");
            return AppSettings::default();
        }
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        println!("Failed to read settings file, using defaults.");
        return AppSettings::default();
    }

    // Strip inline line comments (// ...) to allow documentation in settings.json
    let cleaned_contents: String = contents
        .lines()
        .map(|line| {
            if let Some(idx) = line.find("//") {
                &line[..idx]
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");

    match serde_json::from_str(&cleaned_contents) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to parse settings JSON: {}, using defaults.", e);
            AppSettings::default()
        }
    }
}
