use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

pub mod types;
pub mod input_parse;

pub use types::*;
pub use input_parse::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub physics: PhysicsSettings,
    pub p1_character: CharacterSettings,
    pub p2_character: CharacterSettings,
    pub keyboard_controls: KeyboardControls,
    pub controller_controls: ControllerControls,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            physics: PhysicsSettings::default(),
            p1_character: CharacterSettings::default(),
            p2_character: CharacterSettings::default(),
            keyboard_controls: KeyboardControls::default(),
            controller_controls: ControllerControls::default(),
        }
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = load_settings();
        app.insert_resource(settings.physics.clone());
        app.insert_resource(P1WeaponSettings(settings.p1_character.clone()));
        app.insert_resource(P2WeaponSettings(settings.p2_character.clone()));
        app.insert_resource(settings.keyboard_controls.clone());
        app.insert_resource(settings.controller_controls.clone());
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
            cards: Vec::new(),
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
            cards: Vec::new(),
        };
        app.insert_resource(PersistentPlayerStats { p1: p1_stats, p2: p2_stats });
        app.init_state::<GameState>();
        app.init_resource::<LobbySlots>();
    }
}

pub fn load_settings() -> AppSettings {
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

pub fn save_settings(settings: &AppSettings) {
    if let Ok(file) = std::fs::File::create("settings.json") {
        let _ = serde_json::to_writer_pretty(file, settings);
    }
}
