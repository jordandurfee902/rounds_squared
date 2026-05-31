use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Player {
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
    P8,
}

impl Player {
    pub fn index(self) -> usize {
        match self {
            Player::P1 => 0,
            Player::P2 => 1,
            Player::P3 => 2,
            Player::P4 => 3,
            Player::P5 => 4,
            Player::P6 => 5,
            Player::P7 => 6,
            Player::P8 => 7,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Player::P1,
            1 => Player::P2,
            2 => Player::P3,
            3 => Player::P4,
            4 => Player::P5,
            5 => Player::P6,
            6 => Player::P7,
            7 => Player::P8,
            _ => panic!("Invalid player index: {}", index),
        }
    }

    pub fn color(self) -> Color {
        match self {
            Player::P1 => Color::srgb(0.2, 0.5, 1.0),
            Player::P2 => Color::srgb(1.0, 0.5, 0.2),
            Player::P3 => Color::srgb(0.2, 0.8, 0.4),
            Player::P4 => Color::srgb(0.8, 0.2, 0.8),
            Player::P5 => Color::srgb(0.9, 0.8, 0.1),
            Player::P6 => Color::srgb(0.1, 0.8, 0.9),
            Player::P7 => Color::srgb(0.9, 0.2, 0.2),
            Player::P8 => Color::srgb(0.5, 0.2, 0.9),
        }
    }
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
pub struct PlayerBody;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct BlockComponent {
    pub active_timer: f32,
    pub cooldown_timer: f32,
    pub block_duration: f32,
    pub block_cooldown: f32,
    pub control_lockout_timer: f32,
}
