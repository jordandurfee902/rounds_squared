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
    #[serde(default)]
    pub p3_character: CharacterSettings,
    #[serde(default)]
    pub p4_character: CharacterSettings,
    #[serde(default)]
    pub p5_character: CharacterSettings,
    #[serde(default)]
    pub p6_character: CharacterSettings,
    #[serde(default)]
    pub p7_character: CharacterSettings,
    #[serde(default)]
    pub p8_character: CharacterSettings,
    pub keyboard_controls: KeyboardControls,
    pub controller_controls: ControllerControls,
}

impl AppSettings {
    pub fn get_character(&self, index: usize) -> &CharacterSettings {
        match index {
            0 => &self.p1_character,
            1 => &self.p2_character,
            2 => &self.p3_character,
            3 => &self.p4_character,
            4 => &self.p5_character,
            5 => &self.p6_character,
            6 => &self.p7_character,
            7 => &self.p8_character,
            _ => &self.p1_character,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            physics: PhysicsSettings::default(),
            p1_character: CharacterSettings::default(),
            p2_character: CharacterSettings::default(),
            p3_character: CharacterSettings::default(),
            p4_character: CharacterSettings::default(),
            p5_character: CharacterSettings::default(),
            p6_character: CharacterSettings::default(),
            p7_character: CharacterSettings::default(),
            p8_character: CharacterSettings::default(),
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
        app.insert_resource(PlayerWeaponSettings([
            settings.p1_character.clone(),
            settings.p2_character.clone(),
            settings.p3_character.clone(),
            settings.p4_character.clone(),
            settings.p5_character.clone(),
            settings.p6_character.clone(),
            settings.p7_character.clone(),
            settings.p8_character.clone(),
        ]));
        app.insert_resource(settings.keyboard_controls.clone());
        app.insert_resource(settings.controller_controls.clone());
        app.insert_resource(ScoreTracker::default());

        let make_stats = |char_settings: &CharacterSettings| PlayerStats {
            movement_speed: char_settings.speed,
            jump_force: settings.physics.player_jump_force,
            player_scale: char_settings.size,
            health_max: char_settings.health,
            bullet_range: char_settings.bullet_range,
            bullet_speed: char_settings.bullet_speed,
            bullet_gravity: char_settings.bullet_gravity,
            bullet_damage: char_settings.damage,
            bullet_size_mult: char_settings.bullet_size_mult,
            bullet_growth: char_settings.bullet_growth,
            max_ammo: char_settings.max_ammo,
            reload_time: char_settings.reload_time,
            fire_rate: char_settings.fire_rate,
            bounces: char_settings.bounces,
            bounce_speed_multiplier: char_settings.bounce_speed_multiplier,
            block_duration: char_settings.block_duration,
            block_cooldown: char_settings.block_cooldown,
            block_border_boost: char_settings.block_border_boost,
            special_effects: char_settings.special_effects.clone(),
            cards: Vec::new(),
        };

        app.insert_resource(PersistentPlayerStats {
            players: [
                make_stats(&settings.p1_character),
                make_stats(&settings.p2_character),
                make_stats(&settings.p3_character),
                make_stats(&settings.p4_character),
                make_stats(&settings.p5_character),
                make_stats(&settings.p6_character),
                make_stats(&settings.p7_character),
                make_stats(&settings.p8_character),
            ]
        });
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
