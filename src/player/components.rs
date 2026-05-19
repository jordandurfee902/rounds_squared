use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    P1,
    P2,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlayerStatsComponent {
    pub movement_speed: f32,
    pub jump_force: f32,
    pub player_scale: f32,
    pub health_max: f32,
    pub block_duration: f32,
    pub block_cooldown: f32,
    pub block_border_boost: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct BlockComponent {
    pub active_timer: f32,
    pub cooldown_timer: f32,
    pub block_duration: f32,
    pub block_cooldown: f32,
    pub control_lockout_timer: f32,
}
