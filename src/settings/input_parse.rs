use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Serialize, Deserialize, Clone)]
pub struct KeyboardControls {
    pub move_left: String,
    pub move_right: String,
    pub jump: String,
    pub fast_fall: String,
    pub block: String,
    pub shoot: String,
    pub reload: String,
}

impl Default for KeyboardControls {
    fn default() -> Self {
        Self {
            move_left: "A".to_string(),
            move_right: "D".to_string(),
            jump: "W".to_string(),
            fast_fall: "S".to_string(),
            block: "MouseRight".to_string(),
            shoot: "MouseLeft".to_string(),
            reload: "R".to_string(),
        }
    }
}

#[derive(Resource, Debug, Serialize, Deserialize, Clone)]
pub struct ControllerControls {
    pub jump: String,
    pub block: String,
    pub shoot: String,
    pub reload: String,
}

impl Default for ControllerControls {
    fn default() -> Self {
        Self {
            jump: "South".to_string(),
            block: "LeftTrigger2".to_string(),
            shoot: "RightTrigger2".to_string(),
            reload: "West".to_string(),
        }
    }
}

pub fn parse_key_code(s: &str) -> Option<KeyCode> {
    match s.trim().to_uppercase().as_str() {
        "A" => Some(KeyCode::KeyA),
        "B" => Some(KeyCode::KeyB),
        "C" => Some(KeyCode::KeyC),
        "D" => Some(KeyCode::KeyD),
        "E" => Some(KeyCode::KeyE),
        "F" => Some(KeyCode::KeyF),
        "G" => Some(KeyCode::KeyG),
        "H" => Some(KeyCode::KeyH),
        "I" => Some(KeyCode::KeyI),
        "J" => Some(KeyCode::KeyJ),
        "K" => Some(KeyCode::KeyK),
        "L" => Some(KeyCode::KeyL),
        "M" => Some(KeyCode::KeyM),
        "N" => Some(KeyCode::KeyN),
        "O" => Some(KeyCode::KeyO),
        "P" => Some(KeyCode::KeyP),
        "Q" => Some(KeyCode::KeyQ),
        "R" => Some(KeyCode::KeyR),
        "S" => Some(KeyCode::KeyS),
        "T" => Some(KeyCode::KeyT),
        "U" => Some(KeyCode::KeyU),
        "V" => Some(KeyCode::KeyV),
        "W" => Some(KeyCode::KeyW),
        "X" => Some(KeyCode::KeyX),
        "Y" => Some(KeyCode::KeyY),
        "Z" => Some(KeyCode::KeyZ),
        "SPACE" => Some(KeyCode::Space),
        "ENTER" => Some(KeyCode::Enter),
        "ESCAPE" => Some(KeyCode::Escape),
        "BACKSPACE" => Some(KeyCode::Backspace),
        "UP" => Some(KeyCode::ArrowUp),
        "DOWN" => Some(KeyCode::ArrowDown),
        "LEFT" => Some(KeyCode::ArrowLeft),
        "RIGHT" => Some(KeyCode::ArrowRight),
        _ => None,
    }
}

pub fn parse_gamepad_button(s: &str) -> Option<GamepadButton> {
    match s.trim().to_uppercase().as_str() {
        "SOUTH" | "A" => Some(GamepadButton::South),
        "EAST" | "B" => Some(GamepadButton::East),
        "WEST" | "X" => Some(GamepadButton::West),
        "NORTH" | "Y" => Some(GamepadButton::North),
        "LEFTTRIGGER" | "LB" => Some(GamepadButton::LeftTrigger),
        "RIGHTTRIGGER" | "RB" => Some(GamepadButton::RightTrigger),
        "LEFTTRIGGER2" | "LT" => Some(GamepadButton::LeftTrigger2),
        "RIGHTTRIGGER2" | "RT" => Some(GamepadButton::RightTrigger2),
        _ => None,
    }
}

pub fn parse_mouse_button(s: &str) -> Option<MouseButton> {
    match s.trim().to_uppercase().as_str() {
        "MOUSELEFT" | "LEFTCLICK" => Some(MouseButton::Left),
        "MOUSERIGHT" | "RIGHTCLICK" => Some(MouseButton::Right),
        "MOUSEMIDDLE" | "MIDDLECLICK" => Some(MouseButton::Middle),
        _ => None,
    }
}
