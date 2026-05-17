use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

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
        }
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = load_settings();
        app.insert_resource(settings);
    }
}

fn load_settings() -> PhysicsSettings {
    let mut file = match File::open("settings.json") {
        Ok(f) => f,
        Err(_) => {
            println!("Settings file not found, using defaults.");
            return PhysicsSettings::default();
        }
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        println!("Failed to read settings file, using defaults.");
        return PhysicsSettings::default();
    }

    match serde_json::from_str(&contents) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to parse settings JSON: {}, using defaults.", e);
            PhysicsSettings::default()
        }
    }
}
