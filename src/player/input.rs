use bevy::prelude::*;
use crate::physics::components::ControllerInput;
use crate::settings::{
    LobbySlots, InputDevice, KeyboardControls, ControllerControls,
    parse_key_code, parse_gamepad_button, parse_mouse_button, PhysicsSettings
};
use super::components::*;

pub fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&Player, &mut ControllerInput)>,
    gamepads: Query<&Gamepad>,
    lobby_slots: Res<LobbySlots>,
    kb_controls: Res<KeyboardControls>,
    ctrl_controls: Res<ControllerControls>,
    physics_settings: Res<PhysicsSettings>,
) {
    for (player, mut input) in query.iter_mut() {
        let mut move_dir = 0.0;
        let mut jump = false;
        let mut fast_fall = false;
        let mut fire = false;
        let mut reload = false;
        let mut block = false;

        let slot = match player {
            Player::P1 => &lobby_slots.p1,
            Player::P2 => &lobby_slots.p2,
        };

        if let Some(device) = slot {
            match device {
                InputDevice::KeyboardMouse => {
                    let left_key = parse_key_code(&kb_controls.move_left).unwrap_or(KeyCode::KeyA);
                    let right_key = parse_key_code(&kb_controls.move_right).unwrap_or(KeyCode::KeyD);
                    let jump_key = parse_key_code(&kb_controls.jump).unwrap_or(KeyCode::KeyW);
                    let fast_fall_key = parse_key_code(&kb_controls.fast_fall).unwrap_or(KeyCode::KeyS);
                    let reload_key = parse_key_code(&kb_controls.reload).unwrap_or(KeyCode::KeyR);
                    let block_key = parse_key_code(&kb_controls.block).unwrap_or(KeyCode::KeyX);

                    if keys.pressed(left_key) { move_dir -= 1.0; }
                    if keys.pressed(right_key) { move_dir += 1.0; }
                    if keys.pressed(jump_key) { jump = true; }
                    if keys.pressed(fast_fall_key) { fast_fall = true; }
                    if keys.just_pressed(reload_key) { reload = true; }
                    
                    if let Some(mb) = parse_mouse_button(&kb_controls.block) {
                        if mouse.just_pressed(mb) { block = true; }
                    } else if keys.just_pressed(block_key) {
                        block = true;
                    }

                    if let Some(mb) = parse_mouse_button(&kb_controls.shoot) {
                        if mouse.pressed(mb) { fire = true; }
                    } else if let Some(kc) = parse_key_code(&kb_controls.shoot) {
                        if keys.pressed(kc) { fire = true; }
                    } else {
                        if mouse.pressed(MouseButton::Left) { fire = true; }
                    }
                }
                InputDevice::Gamepad(gp_entity) => {
                    if let Ok(gp) = gamepads.get(*gp_entity) {
                        let stick = gp.left_stick();
                        move_dir = stick.x;
                        let jump_btn = parse_gamepad_button(&ctrl_controls.jump).unwrap_or(GamepadButton::South);
                        if gp.pressed(jump_btn) { jump = true; }
                        if stick.y < physics_settings.fast_fall_stick_threshold { fast_fall = true; }
                        let reload_btn = parse_gamepad_button(&ctrl_controls.reload).unwrap_or(GamepadButton::West);
                        if gp.just_pressed(reload_btn) { reload = true; }
                        let shoot_btn = parse_gamepad_button(&ctrl_controls.shoot).unwrap_or(GamepadButton::RightTrigger2);
                        if gp.pressed(shoot_btn) { fire = true; }
                        let block_btn = parse_gamepad_button(&ctrl_controls.block).unwrap_or(GamepadButton::LeftTrigger2);
                        if gp.just_pressed(block_btn) { block = true; }
                    }
                }
            }
        } else {
            // FALLBACK / DEFAULTS
            match player {
                Player::P1 => {
                    if keys.pressed(KeyCode::KeyA) { move_dir -= 1.0; }
                    if keys.pressed(KeyCode::KeyD) { move_dir += 1.0; }
                    if keys.pressed(KeyCode::KeyW) { jump = true; }
                    if keys.pressed(KeyCode::KeyS) { fast_fall = true; }
                    if keys.just_pressed(KeyCode::KeyR) { reload = true; }
                    if mouse.just_pressed(MouseButton::Right) { block = true; }
                    if mouse.pressed(MouseButton::Left) { fire = true; }
                }
                Player::P2 => {
                    let first_gamepad = gamepads.iter().next();
                    if let Some(gp) = first_gamepad {
                        let stick = gp.left_stick();
                        move_dir = stick.x;
                        if gp.pressed(GamepadButton::South) { jump = true; }
                        if stick.y < physics_settings.fast_fall_stick_threshold { fast_fall = true; }
                        if gp.just_pressed(GamepadButton::West) { reload = true; }
                        if gp.just_pressed(GamepadButton::LeftTrigger2) { block = true; }
                        if gp.pressed(GamepadButton::RightTrigger2) { fire = true; }
                    } else {
                        if keys.pressed(KeyCode::ArrowLeft) { move_dir -= 1.0; }
                        if keys.pressed(KeyCode::ArrowRight) { move_dir += 1.0; }
                        if keys.pressed(KeyCode::ArrowUp) { jump = true; }
                        if keys.pressed(KeyCode::ArrowDown) { fast_fall = true; }
                        if keys.just_pressed(KeyCode::KeyU) { block = true; }
                        if keys.just_pressed(KeyCode::KeyI) { reload = true; }
                        if keys.pressed(KeyCode::Space) { fire = true; }
                    }
                }
            }
        }

        input.move_dir = move_dir;
        input.jump = jump;
        input.fast_fall = fast_fall;
        input.fire = fire;
        input.reload = reload;
        input.block = block;
    }
}

pub fn player_block_system(
    time: Res<Time>,
    mut query: Query<(&ControllerInput, &mut BlockComponent)>,
) {
    let dt = time.delta_secs().min(0.05);

    for (input, mut block) in query.iter_mut() {
        if block.active_timer > 0.0 {
            block.active_timer = (block.active_timer - dt).max(0.0);
        }
        if block.cooldown_timer > 0.0 {
            block.cooldown_timer = (block.cooldown_timer - dt).max(0.0);
        }
        if block.control_lockout_timer > 0.0 {
            block.control_lockout_timer = (block.control_lockout_timer - dt).max(0.0);
        }

        let block_pressed = input.block;

        if block_pressed && block.cooldown_timer <= 0.0 {
            block.active_timer = block.block_duration;
            block.cooldown_timer = block.block_cooldown;
        }
    }
}
