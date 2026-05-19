use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Lobby,
    OnlineMenu,
    Matchmaking,
    Gameplay,
    CardSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDevice {
    KeyboardMouse,
    Gamepad(Entity),
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LobbySlots {
    pub slots: [Option<InputDevice>; 8],
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
    pub cards: Vec<usize>,
}

#[derive(Resource, Debug, Clone)]
pub struct PersistentPlayerStats {
    pub players: [PlayerStats; 8],
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
#[serde(default)]
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
    pub player_base_radius: f32,
    pub player_base_mass: f32,
    pub player_visual_offset: f32,
    pub player_aim_offset_y: f32,
    pub boundary_knockback_speed: f32,
    pub boundary_damage_lockout: f32,
    pub boundary_deflect_lockout: f32,
    pub spawn_invincibility_grace_period: f32,
    pub boundary_hazard_damage: f32,
    pub fast_fall_stick_threshold: f32,
    pub fast_fall_velocity_limit: f32,
    pub wall_cling_stick_threshold: f32,
    pub max_jump_allowance: u32,
    pub collision_penetration_skin_buffer: f32,
    pub overlapping_push_factor: f32,
    pub grounded_slope_threshold: f32,
    pub wall_contact_slope_threshold: f32,
    pub bullet_knockback_constant: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: -2800.0,
            player_accel: 10.0,
            player_jump_force: 1060.0,
            boundary_restitution: 0.0,
            player_restitution: 0.5,
            air_friction: 1.0,
            ground_friction: 1.0,
            movement_stop_friction: 10.0,
            wall_slide_speed: 150.0,
            wall_jump_push_force: 800.0,
            fast_fall_acceleration: 4000.0,
            air_accel: 25.0,
            player_base_radius: 40.0,
            player_base_mass: 1.0,
            player_visual_offset: 15.0,
            player_aim_offset_y: 25.0,
            boundary_knockback_speed: 1200.0,
            boundary_damage_lockout: 0.20,
            boundary_deflect_lockout: 0.25,
            spawn_invincibility_grace_period: 0.5,
            boundary_hazard_damage: 34.0,
            fast_fall_stick_threshold: -0.5,
            fast_fall_velocity_limit: 50.0,
            wall_cling_stick_threshold: 0.1,
            max_jump_allowance: 1,
            collision_penetration_skin_buffer: 1.0,
            overlapping_push_factor: 0.5,
            grounded_slope_threshold: 0.5,
            wall_contact_slope_threshold: 0.5,
            bullet_knockback_constant: 100.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct PlayerWeaponSettings(pub [CharacterSettings; 8]);

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreTracker {
    pub wins: [u32; 8],
}
