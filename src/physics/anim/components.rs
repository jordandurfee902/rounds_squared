use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct PlayerAim {
    pub direction: Vec2, // Normalized aim direction vector
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FootState {
    Planted { position: Vec2 },
    Stepping {
        start: Vec2,
        target: Vec2,
        progress: f32,
    },
    Airborne,
}

impl FootState {
    pub fn is_zero(&self) -> bool {
        matches!(self, FootState::Airborne)
    }
}

#[derive(Component, Debug)]
pub struct ProceduralLimbs {
    pub left_foot: FootState,
    pub right_foot: FootState,
    pub step_cooldown: f32, // alternating step timer
}

impl Default for ProceduralLimbs {
    fn default() -> Self {
        Self {
            left_foot: FootState::Airborne,
            right_foot: FootState::Airborne,
            step_cooldown: 0.0,
        }
    }
}
