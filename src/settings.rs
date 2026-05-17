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
    pub special_effects: Vec<String>,
}

#[derive(Resource, Debug, Clone)]
pub struct PersistentPlayerStats {
    pub p1: PlayerStats,
    pub p2: PlayerStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeaponSettings {
    pub bullet_range: f32,       // base range in seconds
    pub bullet_speed: f32,       // projectile velocity (px/sec)
    pub bullet_gravity: f32,     // downward gravitational acceleration on the projectile (px/sec^2)
    pub bullet_damage: f32,      // base impact damage
    pub bullet_size_mult: f32,   // base size scaling multiplier
    pub bullet_growth: f32,      // rate of exponential damage growth per second in the air (e.g. 0.10 for 10%)
    pub max_ammo: u32,           // number of bullets in a full magazine
    pub reload_time: f32,        // time taken to reload in seconds
    pub fire_rate: f32,          // minimum time between shots in seconds
    pub special_effects: Vec<String>, // list of active special effects (e.g. "PoisonCloud")
}

impl Default for WeaponSettings {
    fn default() -> Self {
        Self {
            bullet_range: 2.0,
            bullet_speed: 1250.0,
            bullet_gravity: 0.0,
            bullet_damage: 15.0,
            bullet_size_mult: 1.0,
            bullet_growth: 0.0,
            max_ammo: 6,
            reload_time: 1.2,
            fire_rate: 0.35,
            special_effects: Vec::new(),
        }
    }
}

#[derive(Resource, Debug, Serialize, Deserialize, Clone)]
pub struct PhysicsSettings {
    pub gravity: f32,
    pub player_speed: f32,
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
    pub player_scale: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: -1600.0,
            player_speed: 300.0,
            player_accel: 10.0,
            player_jump_force: 500.0,
            boundary_restitution: 0.25,
            player_restitution: 0.5,
            air_friction: 3.0,
            ground_friction: 0.50,
            movement_stop_friction: 10.0,
            wall_slide_speed: 150.0,
            wall_jump_push_force: 1000.0,
            fast_fall_acceleration: 4000.0,
            air_accel: 25.0,
            player_scale: 1.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct P1WeaponSettings(pub WeaponSettings);

#[derive(Resource, Debug, Clone)]
pub struct P2WeaponSettings(pub WeaponSettings);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub physics: PhysicsSettings,
    pub p1_weapon: WeaponSettings,
    pub p2_weapon: WeaponSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            physics: PhysicsSettings::default(),
            p1_weapon: WeaponSettings::default(),
            p2_weapon: WeaponSettings::default(),
        }
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = load_settings();
        app.insert_resource(settings.physics.clone());
        app.insert_resource(P1WeaponSettings(settings.p1_weapon.clone()));
        app.insert_resource(P2WeaponSettings(settings.p2_weapon.clone()));

        // Populate initial PersistentPlayerStats from settings
        let p1_stats = PlayerStats {
            movement_speed: settings.physics.player_speed,
            jump_force: settings.physics.player_jump_force,
            player_scale: settings.physics.player_scale,
            health_max: 100.0,
            bullet_range: settings.p1_weapon.bullet_range,
            bullet_speed: settings.p1_weapon.bullet_speed,
            bullet_gravity: settings.p1_weapon.bullet_gravity,
            bullet_damage: settings.p1_weapon.bullet_damage,
            bullet_size_mult: settings.p1_weapon.bullet_size_mult,
            bullet_growth: settings.p1_weapon.bullet_growth,
            max_ammo: settings.p1_weapon.max_ammo,
            reload_time: settings.p1_weapon.reload_time,
            fire_rate: settings.p1_weapon.fire_rate,
            special_effects: settings.p1_weapon.special_effects.clone(),
        };
        let p2_stats = PlayerStats {
            movement_speed: settings.physics.player_speed,
            jump_force: settings.physics.player_jump_force,
            player_scale: settings.physics.player_scale,
            health_max: 100.0,
            bullet_range: settings.p2_weapon.bullet_range,
            bullet_speed: settings.p2_weapon.bullet_speed,
            bullet_gravity: settings.p2_weapon.bullet_gravity,
            bullet_damage: settings.p2_weapon.bullet_damage,
            bullet_size_mult: settings.p2_weapon.bullet_size_mult,
            bullet_growth: settings.p2_weapon.bullet_growth,
            max_ammo: settings.p2_weapon.max_ammo,
            reload_time: settings.p2_weapon.reload_time,
            fire_rate: settings.p2_weapon.fire_rate,
            special_effects: settings.p2_weapon.special_effects.clone(),
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

    match serde_json::from_str(&contents) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to parse settings JSON: {}, using defaults.", e);
            AppSettings::default()
        }
    }
}
