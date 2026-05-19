use bevy::prelude::*;
use crate::player::Player;

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
    pub player_scale: f32,
    pub bounces: u32,
    pub bounce_speed_multiplier: f32,
}
