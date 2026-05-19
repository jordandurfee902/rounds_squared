use bevy::prelude::*;
use crate::player::{Player, PlayerStatsComponent};
use crate::physics::components::Velocity;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};
use crate::settings::{LobbySlots, InputDevice, PhysicsSettings};
use super::components::PlayerAim;

/// Converts screen mouse coordinates (P1) or IJKL keys / movement velocity (P2) into a normalized aim vector.
pub fn update_aim(
    windows: Query<&Window>,
    mut query: Query<(&Player, &Transform, &Velocity, &mut PlayerAim, &PlayerStatsComponent)>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    lobby_slots: Res<LobbySlots>,
    is_networked: Option<Res<crate::net::IsNetworked>>,
    local_player_idx: Option<Res<crate::net::LocalPlayerIndex>>,
    settings: Res<PhysicsSettings>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    
    let is_net = is_networked.map(|n| n.0).unwrap_or(false);
    let local_idx = local_player_idx.map(|idx| idx.0).unwrap_or(0);
    
    // Physical cursor position in pixel coordinates (origin is top-left)
    let cursor_pos = window.cursor_position();
    
    let window_width = window.width();
    let window_height = window.height();
    let window_aspect = window_width / window_height;
    
    let target_aspect = TARGET_WIDTH / TARGET_HEIGHT;
    
    // Solve exact letterboxing or pillarboxing offsets to translate to virtual 1920x1080 bounds
    let (vp_width, vp_height, vp_x, vp_y) = if window_aspect > target_aspect {
        let height = window_height;
        let width = height * target_aspect;
        let x = (window_width - width) / 2.0;
        (width, height, x, 0.0)
    } else {
        let width = window_width;
        let height = width / target_aspect;
        let y = (window_height - height) / 2.0;
        (width, height, 0.0, y)
    };

    for (player, transform, velocity, mut aim, stats) in query.iter_mut() {
        if is_net {
            let local_player = match local_idx {
                0 => Player::P1,
                _ => Player::P2,
            };
            if *player != local_player {
                continue; // Skip updating remote player's aim locally!
            }
        }

        let slot = match player {
            Player::P1 => &lobby_slots.p1,
            Player::P2 => &lobby_slots.p2,
        };

        if let Some(device) = slot {
            match device {
                InputDevice::KeyboardMouse => {
                    if let Some(cursor) = cursor_pos {
                        let rx = cursor.x - vp_x;
                        let ry = cursor.y - vp_y;
                        let u = rx / vp_width;
                        let v = ry / vp_height;
                        let world_x = (u - 0.5) * TARGET_WIDTH;
                        let world_y = (0.5 - v) * TARGET_HEIGHT;
                        
                        let scale = stats.player_scale;
                        let player_pos = transform.translation.xy() + Vec2::new(0.0, settings.player_aim_offset_y * scale);
                        let target_dir = Vec2::new(world_x, world_y) - player_pos;
                        if target_dir.length_squared() > 1.0 {
                            aim.direction = target_dir.normalize();
                        }
                    }
                }
                InputDevice::Gamepad(gp_entity) => {
                    let mut got_aim = false;
                    if let Ok(gp) = gamepads.get(*gp_entity) {
                        // Retrieve raw unclamped axis values to bypass snapping axis deadzones
                        let rx = gp.get_unclamped(GamepadAxis::RightStickX).unwrap_or(0.0);
                        let ry = gp.get_unclamped(GamepadAxis::RightStickY).unwrap_or(0.0);
                        let stick = Vec2::new(rx, ry);
                        // Apply a smooth, fluid 360-degree circular deadzone
                        if stick.length_squared() > 0.04 {
                            aim.direction = stick.normalize();
                            got_aim = true;
                        }
                    }
                    if !got_aim {
                        let vel = velocity.0;
                        if vel.length_squared() > 10.0 {
                            aim.direction = vel.normalize();
                        } else if aim.direction == Vec2::ZERO {
                            aim.direction = Vec2::X;
                        }
                    }
                }
            }
        } else {
            // FALLBACK / DEFAULTS
            match player {
                Player::P1 => {
                    if let Some(cursor) = cursor_pos {
                        let rx = cursor.x - vp_x;
                        let ry = cursor.y - vp_y;
                        let u = rx / vp_width;
                        let v = ry / vp_height;
                        let world_x = (u - 0.5) * TARGET_WIDTH;
                        let world_y = (0.5 - v) * TARGET_HEIGHT;
                        
                        let scale = stats.player_scale;
                        let player_pos = transform.translation.xy() + Vec2::new(0.0, settings.player_aim_offset_y * scale);
                        let target_dir = Vec2::new(world_x, world_y) - player_pos;
                        if target_dir.length_squared() > 1.0 {
                            aim.direction = target_dir.normalize();
                        }
                    }
                }
                Player::P2 => {
                    let mut got_aim = false;
                    let first_gamepad = gamepads.iter().next();
                    if let Some(gp) = first_gamepad {
                        // Retrieve raw unclamped axis values to bypass snapping axis deadzones
                        let rx = gp.get_unclamped(GamepadAxis::RightStickX).unwrap_or(0.0);
                        let ry = gp.get_unclamped(GamepadAxis::RightStickY).unwrap_or(0.0);
                        let stick = Vec2::new(rx, ry);
                        // Apply a smooth, fluid 360-degree circular deadzone
                        if stick.length_squared() > 0.04 {
                            aim.direction = stick.normalize();
                            got_aim = true;
                        }
                    }
                    
                    if !got_aim {
                        let mut key_dir = Vec2::ZERO;
                        if keys.pressed(KeyCode::KeyI) { key_dir.y += 1.0; }
                        if keys.pressed(KeyCode::KeyK) { key_dir.y -= 1.0; }
                        if keys.pressed(KeyCode::KeyJ) { key_dir.x -= 1.0; }
                        if keys.pressed(KeyCode::KeyL) { key_dir.x += 1.0; }
                        
                        if key_dir.length_squared() > 0.1 {
                            aim.direction = key_dir.normalize();
                        } else {
                            let vel = velocity.0;
                            if vel.length_squared() > 10.0 {
                                aim.direction = vel.normalize();
                            } else if aim.direction == Vec2::ZERO {
                                aim.direction = Vec2::X;
                            }
                        }
                    }
                }
            }
        }
    }
}
