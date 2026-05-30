use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Grounded(pub bool);

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Velocity(pub Vec2);

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Acceleration(pub Vec2);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct Friction(pub f32);

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct Restitution(pub f32);

#[derive(Component, Debug, Clone, PartialEq)]
pub enum Collider {
    Circle { radius: f32 },
    Rect { size: Vec2 },
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct WallContact {
    pub left: bool,
    pub right: bool,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Platform;

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct ControllerInput {
    pub move_dir: f32,
    pub jump: bool,
    pub fast_fall: bool,
    pub fire: bool,
    pub reload: bool,
    pub block: bool,
}

#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct JumpAllowance {
    pub value: u32,
}

#[derive(Component, Debug, Clone)]
pub struct MovingPlatform {
    pub id: u32,
    pub initial_pos: Vec2,
    pub amplitude: Vec2,
    pub frequency: Vec2,
    pub spin_speed: f32,
    pub current_rotation: f32,
}

#[derive(Component, Debug, Clone)]
pub struct RopeSwing {
    pub anchor: Vec2,
    pub length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsObjectType {
    SwingWeight,
    StackableBox,
    HollowSquare,
}

#[derive(Component, Debug, Clone)]
pub struct PhysicsObject {
    pub id: u32,
    pub obj_type: PhysicsObjectType,
    pub health: f32,
    pub max_health: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandingOn(pub Option<Entity>);

