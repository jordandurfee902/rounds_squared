use bevy::prelude::*;
use bevy_ggrs::{ggrs, PlayerInputs};
use matchbox_socket::WebRtcSocket;
use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};
use crate::physics::components::ControllerInput;

// --- PACKED DETERMINISTIC NETWORK INPUTS ---
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, Serialize, Deserialize)]
#[repr(C)]
pub struct ControllerInputNet {
    pub move_dir_packed: i8,    // f32 scaled from -100 to 100
    pub buttons: u8,            // Bit 0 = Jump, Bit 1 = Fast Fall, Bit 2 = Fire, Bit 3 = Reload, Bit 4 = Block
    pub aim_dir_packed_x: i8,   // f32 aiming x scaled from -100 to 100
    pub aim_dir_packed_y: i8,   // f32 aiming y scaled from -100 to 100
}

// --- GGRS CONFIGURATION DEFINITION ---
pub type GgrsConfig = bevy_ggrs::GgrsConfig<ControllerInputNet, matchbox_socket::PeerId>;

// --- DETERMINISTIC ROLLBACKABLE RNG RESOURCE ---
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackRng {
    pub seed: u32,
}

impl RollbackRng {
    pub fn new(seed: u32) -> Self {
        Self { seed: seed.wrapping_add(54321) }
    }

    /// Generates a float in the range [0.0, 1.0) deterministically
    pub fn next_f32(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed & 0x7FFFFFFF) as f32 / 2147483648.0
    }

    /// Generates a float in the range [min, max) deterministically
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

// --- SESSION BINDINGS & STATUS RESOURCES ---
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayerIndex(pub usize);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsNetworked(pub bool);

// --- MATCHMAKING & SIGNALING SYSTEMS ---
#[derive(Resource, Deref, DerefMut)]
pub struct MatchboxSocketResource(pub WebRtcSocket);

pub fn start_matchmaking(mut commands: Commands) {
    info!("MATCHMAKING: Connecting to local Matchbox signaling server at ws://192.168.0.29:3536");
    // Connect to matchmaking lobby with a room capacity of exactly 2 players
    let room_url = "ws://192.168.0.29:3536/rounds_lobby?next=2";
    
    let (socket, message_loop) = WebRtcSocket::builder(room_url)
        .add_channel(matchbox_socket::ChannelConfig::unreliable())
        .build();
    
    // Spawn the async message loop using Bevy's IoTaskPool to process WebRTC signaling
    bevy::tasks::IoTaskPool::get().spawn(message_loop).detach();
    
    commands.insert_resource(MatchboxSocketResource(socket));
}

pub fn lobby_system(
    mut commands: Commands,
    socket_res: Option<ResMut<MatchboxSocketResource>>,
    mut next_state: ResMut<NextState<crate::settings::GameState>>,
) {
    let Some(mut socket) = socket_res else {
        return;
    };

    // Update connection status
    socket.update_peers();

    let connected_peers = socket.connected_peers().count();
    if connected_peers < 1 {
        // Wait until at least 1 peer connects (2 players total in the WebRTC room)
        return;
    }

    info!("MATCHMAKING: Peer connected! Initializing GGRS Session...");

    let local_id = socket.id().expect("Socket should have an ID");

    // 1. Retrieve all players and sort deterministically by actual Peer ID to guarantee consistent P1/P2 index assignment
    let mut players = socket.players();
    players.sort_by_key(|p| match p {
        ggrs::PlayerType::Remote(id) => id.to_string(),
        ggrs::PlayerType::Local => local_id.to_string(),
        ggrs::PlayerType::Spectator(_) => "spectator".to_string(),
    });

    // 2. Build GGRS P2P Session with 8-frame prediction capacity and 2-frame input delay
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(2)
        .with_max_prediction_window(8)
        .with_input_delay(2);

    let mut local_player_idx = 0;
    for (i, player) in players.iter().enumerate() {
        if let ggrs::PlayerType::Local = player {
            local_player_idx = i;
        }
        session_builder = session_builder.add_player(*player, i).unwrap();
    }

    // 3. Take WebRTC transmission channel and spawn GGRS P2P connection
    let channel = socket.take_channel(0).unwrap();
    let session = session_builder.start_p2p_session(channel).unwrap();

    // 4. Insert resources and proceed to Gameplay state
    commands.insert_resource(bevy_ggrs::Session::P2P(session));
    commands.insert_resource(LocalPlayerIndex(local_player_idx));
    commands.insert_resource(IsNetworked(true));
    commands.insert_resource(RollbackRng::new(98765));

    // Remove socket resource as connection is fully established
    commands.remove_resource::<MatchboxSocketResource>();

    info!("MATCHMAKING: Connection verified! Local Player Index: {}", local_player_idx);
    next_state.set(crate::settings::GameState::Lobby);
}

// --- GGRS INPUT PACK SYSTEM ---
pub fn ggrs_input_system(
    mut commands: Commands,
    local_players: Res<bevy_ggrs::LocalPlayers>,
    lobby_slots: Res<crate::settings::LobbySlots>,
    kb_controls: Res<crate::settings::KeyboardControls>,
    ctrl_controls: Res<crate::settings::ControllerControls>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    aim_query: Query<(&crate::player::Player, &crate::physics::PlayerAim)>,
    state: Res<State<crate::settings::GameState>>,
    time: Res<Time>,
    mut stick_cooldowns: Local<[f32; 2]>,
) {
    let mut local_inputs = bevy::platform::collections::HashMap::default();

    for &handle in &local_players.0 {
        let mut move_dir = 0.0;
        let mut jump = false;
        let mut fast_fall = false;
        let mut fire = false;
        let mut reload = false;
        let mut block = false;

        // handle == 0 is P1, handle == 1 is P2
        let slot = match handle {
            0 => &lobby_slots.p1,
            1 => &lobby_slots.p2,
            _ => &lobby_slots.p1,
        };

        // Decrement joystick navigation cooldown for this handle
        let h_idx = (handle as usize).min(1);
        stick_cooldowns[h_idx] = (stick_cooldowns[h_idx] - time.delta_secs()).max(0.0);

        let mut left_nav = false;
        let mut right_nav = false;
        let mut confirm_nav = false;

        let is_card_selection = *state.get() == crate::settings::GameState::CardSelection;

        if is_card_selection {
            if let Some(device) = slot {
                match device {
                    crate::settings::InputDevice::KeyboardMouse => {
                        if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) {
                            left_nav = true;
                        }
                        if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) {
                            right_nav = true;
                        }
                        if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
                            confirm_nav = true;
                        }
                    }
                    crate::settings::InputDevice::Gamepad(gp_entity) => {
                        if let Ok(gp) = gamepads.get(*gp_entity) {
                            let left_stick = gp.left_stick();

                            // Reset cooldown instantly if the stick is returned to neutral
                            if left_stick.x.abs() < 0.2 {
                                stick_cooldowns[h_idx] = 0.0;
                            }

                            if gp.just_pressed(GamepadButton::DPadLeft) {
                                left_nav = true;
                            } else if left_stick.x < -0.5 && stick_cooldowns[h_idx] <= 0.0 {
                                left_nav = true;
                                stick_cooldowns[h_idx] = 0.25;
                            }

                            if gp.just_pressed(GamepadButton::DPadRight) {
                                right_nav = true;
                            } else if left_stick.x > 0.5 && stick_cooldowns[h_idx] <= 0.0 {
                                right_nav = true;
                                stick_cooldowns[h_idx] = 0.25;
                            }

                            if gp.just_pressed(GamepadButton::South) {
                                confirm_nav = true;
                            }
                        }
                    }
                }
            } else {
                if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) {
                    left_nav = true;
                }
                if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) {
                    right_nav = true;
                }
                if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
                    confirm_nav = true;
                }
            }
        } else {
            if let Some(device) = slot {
                match device {
                    crate::settings::InputDevice::KeyboardMouse => {
                        let left_key = crate::settings::parse_key_code(&kb_controls.move_left).unwrap_or(KeyCode::KeyA);
                        let right_key = crate::settings::parse_key_code(&kb_controls.move_right).unwrap_or(KeyCode::KeyD);
                        let jump_key = crate::settings::parse_key_code(&kb_controls.jump).unwrap_or(KeyCode::KeyW);
                        let fast_fall_key = crate::settings::parse_key_code(&kb_controls.fast_fall).unwrap_or(KeyCode::KeyS);
                        let reload_key = crate::settings::parse_key_code(&kb_controls.reload).unwrap_or(KeyCode::KeyR);
                        let block_key = crate::settings::parse_key_code(&kb_controls.block).unwrap_or(KeyCode::KeyX);

                        if keys.pressed(left_key) { move_dir -= 1.0; }
                        if keys.pressed(right_key) { move_dir += 1.0; }
                        if keys.just_pressed(jump_key) { jump = true; }
                        if keys.pressed(fast_fall_key) { fast_fall = true; }
                        if keys.just_pressed(reload_key) { reload = true; }
                        
                        if let Some(mb) = crate::settings::parse_mouse_button(&kb_controls.block) {
                            if mouse.just_pressed(mb) { block = true; }
                        } else if keys.just_pressed(block_key) {
                            block = true;
                        }

                        if let Some(mb) = crate::settings::parse_mouse_button(&kb_controls.shoot) {
                            if mouse.pressed(mb) { fire = true; }
                        } else if let Some(kc) = crate::settings::parse_key_code(&kb_controls.shoot) {
                            if keys.pressed(kc) { fire = true; }
                        } else {
                            if mouse.pressed(MouseButton::Left) { fire = true; }
                        }
                    }
                    crate::settings::InputDevice::Gamepad(gp_entity) => {
                        if let Ok(gp) = gamepads.get(*gp_entity) {
                            let stick = gp.left_stick();
                            move_dir = stick.x;
                            let jump_btn = crate::settings::parse_gamepad_button(&ctrl_controls.jump).unwrap_or(GamepadButton::South);
                            if gp.just_pressed(jump_btn) { jump = true; }
                            if stick.y < -0.5 { fast_fall = true; }
                            let reload_btn = crate::settings::parse_gamepad_button(&ctrl_controls.reload).unwrap_or(GamepadButton::West);
                            if gp.just_pressed(reload_btn) { reload = true; }
                            let shoot_btn = crate::settings::parse_gamepad_button(&ctrl_controls.shoot).unwrap_or(GamepadButton::RightTrigger2);
                            if gp.pressed(shoot_btn) { fire = true; }
                            let block_btn = crate::settings::parse_gamepad_button(&ctrl_controls.block).unwrap_or(GamepadButton::LeftTrigger2);
                            if gp.just_pressed(block_btn) { block = true; }
                        }
                    }
                }
            } else {
                // Fallback reading P1 controls
                if keys.pressed(KeyCode::KeyA) { move_dir -= 1.0; }
                if keys.pressed(KeyCode::KeyD) { move_dir += 1.0; }
                if keys.just_pressed(KeyCode::KeyW) { jump = true; }
                if keys.pressed(KeyCode::KeyS) { fast_fall = true; }
                if keys.just_pressed(KeyCode::KeyR) { reload = true; }
                if mouse.just_pressed(MouseButton::Right) { block = true; }
                if mouse.pressed(MouseButton::Left) { fire = true; }
            }
        }

        // Get aim direction for the player
        let target_player = match handle {
            0 => crate::player::Player::P1,
            _ => crate::player::Player::P2,
        };
        let mut aim_direction = Vec2::X;
        for (player, aim) in aim_query.iter() {
            if *player == target_player {
                aim_direction = aim.direction;
            }
        }

        let mut buttons = 0u8;
        if is_card_selection {
            if left_nav { buttons |= 1 << 5; }
            if right_nav { buttons |= 1 << 6; }
            if confirm_nav { buttons |= 1 << 7; }
        } else {
            if jump { buttons |= 1 << 0; }
            if fast_fall { buttons |= 1 << 1; }
            if fire { buttons |= 1 << 2; }
            if reload { buttons |= 1 << 3; }
            if block { buttons |= 1 << 4; }

            if slot.is_some() {
                buttons |= 1 << 5; // Joined/Ready
                if let Some(crate::settings::InputDevice::Gamepad(_)) = slot {
                    buttons |= 1 << 6; // Gamepad
                }
            }
        }

        let input = ControllerInputNet {
            move_dir_packed: (move_dir * 100.0) as i8,
            buttons,
            aim_dir_packed_x: (aim_direction.x * 100.0) as i8,
            aim_dir_packed_y: (aim_direction.y * 100.0) as i8,
        };

        local_inputs.insert(handle, input);
    }

    commands.insert_resource(bevy_ggrs::LocalInputs::<GgrsConfig>(local_inputs));
}

// --- GGRS INPUT UNPACK SYSTEM ---
pub fn unpack_network_inputs(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut query: Query<(&crate::player::Player, &mut ControllerInput, &mut crate::physics::PlayerAim)>,
) {
    for (player, mut input, mut aim) in query.iter_mut() {
        let handle = match player {
            crate::player::Player::P1 => 0,
            crate::player::Player::P2 => 1,
        };

        let ggrs_input = if let Some((inp, _)) = inputs.get(handle) {
            inp
        } else {
            continue;
        };

        input.move_dir = ggrs_input.move_dir_packed as f32 / 100.0;
        input.jump = (ggrs_input.buttons & (1 << 0)) != 0;
        input.fast_fall = (ggrs_input.buttons & (1 << 1)) != 0;
        input.fire = (ggrs_input.buttons & (1 << 2)) != 0;
        input.reload = (ggrs_input.buttons & (1 << 3)) != 0;
        input.block = (ggrs_input.buttons & (1 << 4)) != 0;

        let aim_x = ggrs_input.aim_dir_packed_x as f32 / 100.0;
        let aim_y = ggrs_input.aim_dir_packed_y as f32 / 100.0;
        let unpacked_aim = Vec2::new(aim_x, aim_y);
        if unpacked_aim.length_squared() > 0.01 {
            aim.direction = unpacked_aim.normalize();
        }
    }
}

// --- GGRS LOBBY SYNC SYSTEM ---
pub fn lobby_sync_network_system(
    inputs: Res<bevy_ggrs::PlayerInputs<GgrsConfig>>,
    mut lobby_slots: ResMut<crate::settings::LobbySlots>,
    local_idx_opt: Option<Res<LocalPlayerIndex>>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut state: ResMut<NextState<crate::settings::GameState>>,
) {
    let local_idx = local_idx_opt.map(|idx| idx.0).unwrap_or(0);

    let p1_input = if let Some((inp, _)) = inputs.get(0) {
        inp
    } else {
        return;
    };

    let p2_input = if let Some((inp, _)) = inputs.get(1) {
        inp
    } else {
        return;
    };

    // Synchronize P1 (handle 0)
    if local_idx != 0 {
        let joined = (p1_input.buttons & (1 << 5)) != 0;
        if joined {
            let is_gamepad = (p1_input.buttons & (1 << 6)) != 0;
            if is_gamepad {
                let gp_entity = gamepads.iter().next().map(|(e, _)| e).unwrap_or(Entity::PLACEHOLDER);
                lobby_slots.p1 = Some(crate::settings::InputDevice::Gamepad(gp_entity));
            } else {
                lobby_slots.p1 = Some(crate::settings::InputDevice::KeyboardMouse);
            }
        } else {
            lobby_slots.p1 = None;
        }
    }

    // Synchronize P2 (handle 1)
    if local_idx != 1 {
        let joined = (p2_input.buttons & (1 << 5)) != 0;
        if joined {
            let is_gamepad = (p2_input.buttons & (1 << 6)) != 0;
            if is_gamepad {
                let gp_entity = gamepads.iter().next().map(|(e, _)| e).unwrap_or(Entity::PLACEHOLDER);
                lobby_slots.p2 = Some(crate::settings::InputDevice::Gamepad(gp_entity));
            } else {
                lobby_slots.p2 = Some(crate::settings::InputDevice::KeyboardMouse);
            }
        } else {
            lobby_slots.p2 = None;
        }
    }

    // Transition to Gameplay if BOTH players are ready!
    let p1_ready = (p1_input.buttons & (1 << 5)) != 0;
    let p2_ready = (p2_input.buttons & (1 << 5)) != 0;

    if p1_ready && p2_ready {
        info!("ONLINE LOBBY: Both players joined! Starting match...");
        state.set(crate::settings::GameState::Gameplay);
    }
}

// --- GGRS CARD SELECTION SYNC SYSTEM ---
pub fn card_selection_sync_network_system(
    inputs: Res<bevy_ggrs::PlayerInputs<GgrsConfig>>,
    mut card_state: ResMut<crate::physics::card_selection::CardSelectionState>,
    mut persistent_stats: ResMut<crate::settings::PersistentPlayerStats>,
    mut next_state: ResMut<NextState<crate::settings::GameState>>,
) {
    let selecting_player = card_state.selecting_player;
    let selecting_handle = match selecting_player {
        crate::player::Player::P1 => 0,
        crate::player::Player::P2 => 1,
    };

    let input = if let Some((inp, _)) = inputs.get(selecting_handle) {
        inp
    } else {
        return;
    };

    let left_nav = (input.buttons & (1 << 5)) != 0;
    let right_nav = (input.buttons & (1 << 6)) != 0;
    let confirm = (input.buttons & (1 << 7)) != 0;

    if left_nav {
        card_state.selected_idx = if card_state.selected_idx == 0 { 4 } else { card_state.selected_idx - 1 };
    } else if right_nav {
        card_state.selected_idx = (card_state.selected_idx + 1) % 5;
    }

    if confirm {
        let p_stats = match selecting_player {
            crate::player::Player::P1 => &mut persistent_stats.p1,
            crate::player::Player::P2 => &mut persistent_stats.p2,
        };

        p_stats.cards.push(card_state.selected_idx);

        match card_state.selected_idx {
            0 => { // Fast & Light
                p_stats.movement_speed *= 1.30;
                p_stats.health_max *= 0.80;
            }
            1 => { // Tanky Giant
                p_stats.health_max *= 1.40;
                p_stats.player_scale *= 1.30;
                p_stats.jump_force *= 0.85;
            }
            2 => { // Hyper-Shot
                p_stats.bullet_speed *= 1.35;
                p_stats.bullet_damage *= 1.20;
                p_stats.max_ammo = p_stats.max_ammo.saturating_sub(1).max(1);
            }
            3 => { // Toxic Spray
                if !p_stats.special_effects.contains(&"PoisonCloud".to_string()) {
                    p_stats.special_effects.push("PoisonCloud".to_string());
                }
                p_stats.bullet_growth += 0.15;
                p_stats.max_ammo += 2;
            }
            4 => { // Heavy Artillery
                p_stats.bullet_damage *= 1.80;
                p_stats.bullet_size_mult *= 1.30;
                p_stats.bullet_gravity += 200.0;
                p_stats.reload_time += 1.0;
            }
            _ => {}
        }

        // Return to gameplay round deterministically on both clients!
        next_state.set(crate::settings::GameState::Gameplay);
    }
}
