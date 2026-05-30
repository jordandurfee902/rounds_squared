use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInputPacket {
    pub move_dir: f32,
    pub jump: bool,
    pub fast_fall: bool,
    pub fire: bool,
    pub reload: bool,
    pub block: bool,
    pub aim_dir_x: f32,
    pub aim_dir_y: f32,
    pub lobby_joined: bool,
    pub is_gamepad: bool,
    pub card_left: bool,
    pub card_right: bool,
    pub card_confirm: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerNetState {
    pub pos: Vec2,
    pub vel: Vec2,
    pub health: f32,
    pub max_health: f32,
    pub block_active_timer: f32,
    pub block_cooldown_timer: f32,
    pub block_lockout_timer: f32,
    pub aim_dir: Vec2,
    pub ammo_max: u32,
    pub ammo_current: u32,
    pub reload_timer: f32,
    pub grounded: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BulletNetState {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: String, // "P1" or "P2"
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PoisonCloudNetState {
    pub pos: Vec2,
    pub size: f32,
    pub life: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExplosionEvent {
    pub pos: Vec2,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub damage: f32, // -1.0 represents muzzle flash
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GravityWellNetState {
    pub pos: Vec2,
    pub radius: f32,
    pub lifetime: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovingPlatformNetState {
    pub id: u32,
    pub pos: Vec2,
    pub rotation: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhysicsObjectNetState {
    pub id: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub rotation: f32,
    pub health: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostStatePacket {
    pub players: Vec<PlayerNetState>, // length matches number of spawned players, or fixed length 8
    pub bullets: Vec<BulletNetState>,
    pub poison_clouds: Vec<PoisonCloudNetState>,
    pub explosion_events: Vec<ExplosionEvent>,
    pub gravity_wells: Vec<GravityWellNetState>,
    pub moving_platforms: Vec<MovingPlatformNetState>,
    pub physics_objects: Vec<PhysicsObjectNetState>,
    pub wins: [u32; 8],
    pub active_players: [bool; 8],
    pub is_gamepad: [bool; 8],
    pub active_map: String,
    pub game_state: String,
    pub selecting_player: Option<usize>, // Player index currently selecting a card
    pub card_selected_idx: usize,
    pub drawn_cards: [usize; 5],
    pub player_cards: [Vec<usize>; 8],
}
