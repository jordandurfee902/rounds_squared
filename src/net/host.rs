use bevy::prelude::*;
use crate::physics::components::{ControllerInput, Velocity, Grounded, WallContact, JumpAllowance};
use crate::physics::weapon::{Weapon, Projectile};
use crate::physics::particles::ExplosionRecordComponent;
use crate::player::{Player, Health, PlayerStatsComponent, BlockComponent};
use crate::settings::{LobbySlots, GameState, ScoreTracker, PersistentPlayerStats, InputDevice};
use crate::physics::anim::PlayerAim;
use crate::maps::ActiveMap;
use super::{MatchboxSocketResource, ClientInputPacket, HostStatePacket, PlayerNetState, BulletNetState, ExplosionEvent};

pub fn apply_card_selection_effect(p_stats: &mut crate::settings::PlayerStats, card_idx: usize) {
    p_stats.cards.push(card_idx);
    if let Some(card) = crate::physics::card_selection::cards::get_card(card_idx) {
        card.apply(p_stats);
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct LocalInput<'w, 's> {
    pub keys: Res<'w, ButtonInput<KeyCode>>,
    pub mouse: Res<'w, ButtonInput<MouseButton>>,
    pub gamepads: Query<'w, 's, &'static Gamepad>,
    pub kb_controls: Option<Res<'w, crate::settings::KeyboardControls>>,
    pub ctrl_controls: Option<Res<'w, crate::settings::ControllerControls>>,
}

pub fn host_network_system(
    mut commands: Commands,
    socket_res: Option<ResMut<MatchboxSocketResource>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut lobby_slots: ResMut<LobbySlots>,
    score_tracker: Res<ScoreTracker>,
    active_map: Res<ActiveMap>,
    state: Res<State<GameState>>,
    mut players: Query<(
        &Player,
        &mut Transform,
        &mut Velocity,
        &mut Health,
        &mut PlayerStatsComponent,
        &mut BlockComponent,
        &mut Weapon,
        &mut PlayerAim,
        &mut ControllerInput,
        &mut Grounded,
        &mut WallContact,
        &mut JumpAllowance,
    ), Without<Projectile>>,
    projectiles: Query<(&Transform, &Projectile), Without<Player>>,
    explosions: Query<(Entity, &ExplosionRecordComponent)>,
    mut persistent_stats: ResMut<PersistentPlayerStats>,
    mut card_state: Option<ResMut<crate::physics::card_selection::CardSelectionState>>,
    local_input: LocalInput,
    time: Res<Time>,
    mut gamepad_cooldown: Local<f32>,
    gravity_wells_query: Query<(&Transform, &crate::physics::card_selection::cards::gravity_vortex::GravityWell), Without<Player>>,
) {
    let Some(mut socket) = socket_res else {
        return;
    };
    socket.update_peers();

    let peer_ids: Vec<_> = socket.connected_peers().collect();
    if peer_ids.is_empty() {
        return;
    }
    let client_peer = peer_ids[0];

    // 1. Receive client inputs
    let mut client_packet: Option<ClientInputPacket> = None;
    let packets = socket.channel_mut(0).receive();
    for (peer, data) in packets {
        if peer == client_peer {
            if let Ok(pkt) = serde_json::from_slice::<ClientInputPacket>(&data) {
                client_packet = Some(pkt);
            }
        }
    }

    // 1.5 Gather host inputs (Player 1 controls)
    let mut p1_move_dir = 0.0;
    let mut p1_jump = false;
    let mut p1_fast_fall = false;
    let mut p1_fire = false;
    let mut p1_reload = false;
    let mut p1_block = false;

    let is_card_selection = *state.get() == GameState::CardSelection;

    let mut p1_card_left = false;
    let mut p1_card_right = false;
    let mut p1_card_confirm = false;

    if is_card_selection {
        if *gamepad_cooldown > 0.0 {
            *gamepad_cooldown = (*gamepad_cooldown - time.delta_secs()).max(0.0);
        }

        let mut poll_keyboard = false;
        let mut poll_gamepad = None;
        match &lobby_slots.p1 {
            Some(InputDevice::KeyboardMouse) => {
                poll_keyboard = true;
            }
            Some(InputDevice::Gamepad(gp_entity)) => {
                poll_gamepad = Some(*gp_entity);
            }
            None => {
                poll_keyboard = true;
            }
        }

        if poll_keyboard {
            if local_input.keys.just_pressed(KeyCode::KeyA) || local_input.keys.just_pressed(KeyCode::ArrowLeft) {
                p1_card_left = true;
            }
            if local_input.keys.just_pressed(KeyCode::KeyD) || local_input.keys.just_pressed(KeyCode::ArrowRight) {
                p1_card_right = true;
            }
            if local_input.keys.just_pressed(KeyCode::Space) || local_input.keys.just_pressed(KeyCode::Enter) {
                p1_card_confirm = true;
            }
        }

        let mut gp = None;
        if let Some(gp_ent) = poll_gamepad {
            if let Ok(g) = local_input.gamepads.get(gp_ent) {
                gp = Some(g);
            } else if let Some(g) = local_input.gamepads.iter().next() {
                gp = Some(g);
            }
        } else if lobby_slots.p1.is_none() {
            if let Some(g) = local_input.gamepads.iter().next() {
                gp = Some(g);
            }
        }

        if let Some(g) = gp {
            if g.just_pressed(GamepadButton::DPadLeft) {
                p1_card_left = true;
            }
            if g.just_pressed(GamepadButton::DPadRight) {
                p1_card_right = true;
            }
            if g.just_pressed(GamepadButton::South) {
                p1_card_confirm = true;
            }

            if *gamepad_cooldown <= 0.0 {
                let stick_x = g.left_stick().x;
                if stick_x < -0.5 {
                    p1_card_left = true;
                    *gamepad_cooldown = 0.25;
                } else if stick_x > 0.5 {
                    p1_card_right = true;
                    *gamepad_cooldown = 0.25;
                }
            }
        }
    } else {
        let kb_default = crate::settings::KeyboardControls::default();
        let kb = local_input.kb_controls.as_deref().unwrap_or(&kb_default);
        let ctrl_default = crate::settings::ControllerControls::default();
        let ctrl = local_input.ctrl_controls.as_deref().unwrap_or(&ctrl_default);

        if let Some(device) = &lobby_slots.p1 {
            match device {
                InputDevice::KeyboardMouse => {
                    let left_key = crate::settings::parse_key_code(&kb.move_left).unwrap_or(KeyCode::KeyA);
                    let right_key = crate::settings::parse_key_code(&kb.move_right).unwrap_or(KeyCode::KeyD);
                    let jump_key = crate::settings::parse_key_code(&kb.jump).unwrap_or(KeyCode::KeyW);
                    let fast_fall_key = crate::settings::parse_key_code(&kb.fast_fall).unwrap_or(KeyCode::KeyS);
                    let reload_key = crate::settings::parse_key_code(&kb.reload).unwrap_or(KeyCode::KeyR);
                    let block_key = crate::settings::parse_key_code(&kb.block).unwrap_or(KeyCode::KeyX);

                    if local_input.keys.pressed(left_key) { p1_move_dir -= 1.0; }
                    if local_input.keys.pressed(right_key) { p1_move_dir += 1.0; }
                    if local_input.keys.just_pressed(jump_key) { p1_jump = true; }
                    if local_input.keys.pressed(fast_fall_key) { p1_fast_fall = true; }
                    if local_input.keys.just_pressed(reload_key) { p1_reload = true; }
                    
                    if let Some(mb) = crate::settings::parse_mouse_button(&kb.block) {
                        if local_input.mouse.just_pressed(mb) { p1_block = true; }
                    } else if local_input.keys.just_pressed(block_key) {
                        p1_block = true;
                    }

                    if let Some(mb) = crate::settings::parse_mouse_button(&kb.shoot) {
                        if local_input.mouse.pressed(mb) { p1_fire = true; }
                    } else if let Some(kc) = crate::settings::parse_key_code(&kb.shoot) {
                        if local_input.keys.pressed(kc) { p1_fire = true; }
                    } else {
                        if local_input.mouse.pressed(MouseButton::Left) { p1_fire = true; }
                    }
                }
                InputDevice::Gamepad(gp_entity) => {
                    let mut gamepad = None;
                    if let Ok(gp) = local_input.gamepads.get(*gp_entity) {
                        gamepad = Some(gp);
                    } else if let Some(gp) = local_input.gamepads.iter().next() {
                        gamepad = Some(gp);
                    }

                    if let Some(gp) = gamepad {
                        let stick = gp.left_stick();
                        p1_move_dir = stick.x;
                        let jump_btn = crate::settings::parse_gamepad_button(&ctrl.jump).unwrap_or(GamepadButton::South);
                        if gp.just_pressed(jump_btn) { p1_jump = true; }
                        if stick.y < -0.5 { p1_fast_fall = true; }
                        let reload_btn = crate::settings::parse_gamepad_button(&ctrl.reload).unwrap_or(GamepadButton::West);
                        if gp.just_pressed(reload_btn) { p1_reload = true; }
                        let shoot_btn = crate::settings::parse_gamepad_button(&ctrl.shoot).unwrap_or(GamepadButton::RightTrigger2);
                        if gp.pressed(shoot_btn) { p1_fire = true; }
                        let block_btn = crate::settings::parse_gamepad_button(&ctrl.block).unwrap_or(GamepadButton::LeftTrigger2);
                        if gp.just_pressed(block_btn) { p1_block = true; }
                    }
                }
            }
        }
    }

    // Apply local inputs to P1
    for (player, _, _, _, _, _, _, _, mut input, _, _, _) in players.iter_mut() {
        if *player == Player::P1 {
            input.move_dir = p1_move_dir;
            input.jump = p1_jump;
            input.fast_fall = p1_fast_fall;
            input.fire = p1_fire;
            input.reload = p1_reload;
            input.block = p1_block;
        }
    }

    // Apply local P1 Card Selection navigation
    if *state.get() == GameState::CardSelection {
        if let Some(ref mut cs) = card_state {
            if cs.selecting_player == Player::P1 {
                if p1_card_left {
                    cs.selected_idx = if cs.selected_idx == 0 { 4 } else { cs.selected_idx - 1 };
                } else if p1_card_right {
                    cs.selected_idx = (cs.selected_idx + 1) % 5;
                }

                if p1_card_confirm {
                    let p_stats = &mut persistent_stats.p1;
                    apply_card_selection_effect(p_stats, cs.drawn_cards[cs.selected_idx]);
                    next_state.set(GameState::Gameplay);
                }
            }
        }
    } else {
        commands.remove_resource::<crate::physics::card_selection::CardSelectionState>();
    }

    // 2. Process Client Inputs
    if let Some(pkt) = client_packet {
        // Sync P2 lobby ready
        if pkt.lobby_joined {
            if lobby_slots.p2.is_none() {
                lobby_slots.p2 = Some(if pkt.is_gamepad { InputDevice::Gamepad(Entity::PLACEHOLDER) } else { InputDevice::KeyboardMouse });
            }
        } else {
            lobby_slots.p2 = None;
        }

        // Apply inputs to P2
        for (player, _, _, _, _, _, _, mut aim, mut input, _, _, _) in players.iter_mut() {
            if *player == Player::P2 {
                input.move_dir = pkt.move_dir;
                input.jump = pkt.jump;
                input.fast_fall = pkt.fast_fall;
                input.fire = pkt.fire;
                input.reload = pkt.reload;
                input.block = pkt.block;
                if pkt.aim_dir_x.abs() > 0.01 || pkt.aim_dir_y.abs() > 0.01 {
                    aim.direction = Vec2::new(pkt.aim_dir_x, pkt.aim_dir_y).normalize();
                }
            }
        }

        // Card Selection navigation
        if *state.get() == GameState::CardSelection {
            if let Some(ref mut cs) = card_state {
                if cs.selecting_player == Player::P2 {
                    if pkt.card_left {
                        cs.selected_idx = if cs.selected_idx == 0 { 4 } else { cs.selected_idx - 1 };
                    } else if pkt.card_right {
                        cs.selected_idx = (cs.selected_idx + 1) % 5;
                    }

                    if pkt.card_confirm {
                        let p_stats = &mut persistent_stats.p2;
                        apply_card_selection_effect(p_stats, cs.drawn_cards[cs.selected_idx]);
                        next_state.set(GameState::Gameplay);
                    }
                }
            }
        }
    }

    // Auto gameplay trigger in Lobby
    if *state.get() == GameState::Lobby && lobby_slots.p1.is_some() && lobby_slots.p2.is_some() {
        next_state.set(GameState::Gameplay);
    }

    // 3. Serialize World State
    let mut p1_state = None;
    let mut p2_state = None;

    for (player, transform, vel, health, _, block, weapon, aim, _, grounded, _, _) in players.iter() {
        let p_state = PlayerNetState {
            pos: transform.translation.xy(),
            vel: vel.0,
            health: health.current,
            max_health: health.max,
            block_active_timer: block.active_timer,
            block_cooldown_timer: block.cooldown_timer,
            block_lockout_timer: block.control_lockout_timer,
            aim_dir: aim.direction,
            ammo_max: weapon.max_ammo,
            ammo_current: weapon.current_ammo,
            reload_timer: weapon.reload_timer,
            grounded: grounded.0,
        };
        match player {
            Player::P1 => p1_state = Some(p_state),
            Player::P2 => p2_state = Some(p_state),
        }
    }

    let p1 = p1_state.unwrap_or(PlayerNetState {
        pos: Vec2::ZERO, vel: Vec2::ZERO, health: 100.0, max_health: 100.0,
        block_active_timer: 0.0, block_cooldown_timer: 0.0, block_lockout_timer: 0.0,
        aim_dir: Vec2::X, ammo_max: 5, ammo_current: 5, reload_timer: 0.0,
        grounded: true,
    });
    let p2 = p2_state.unwrap_or(PlayerNetState {
        pos: Vec2::ZERO, vel: Vec2::ZERO, health: 100.0, max_health: 100.0,
        block_active_timer: 0.0, block_cooldown_timer: 0.0, block_lockout_timer: 0.0,
        aim_dir: Vec2::X, ammo_max: 5, ammo_current: 5, reload_timer: 0.0,
        grounded: true,
    });

    let mut bullets = Vec::new();
    for (trans, proj) in projectiles.iter() {
        bullets.push(BulletNetState {
            pos: trans.translation.xy(),
            vel: proj.velocity,
            owner: match proj.owner {
                Player::P1 => "P1".to_string(),
                Player::P2 => "P2".to_string(),
            },
            damage: proj.damage,
            gravity: proj.gravity,
            size_multiplier: proj.size_multiplier,
            growth: proj.growth,
            time_in_air: proj.time_in_air,
            lifetime: proj.lifetime,
            special_effects: proj.special_effects.clone(),
            player_scale: proj.player_scale,
            bounces: proj.bounces,
            bounce_speed_multiplier: proj.bounce_speed_multiplier,
        });
    }

    let mut explosion_events = Vec::new();
    for (ent, exp) in explosions.iter() {
        explosion_events.push(ExplosionEvent {
            pos: exp.pos,
            color_r: exp.color.to_srgba().red,
            color_g: exp.color.to_srgba().green,
            color_b: exp.color.to_srgba().blue,
            damage: exp.damage,
        });
        commands.entity(ent).despawn();
    }

    let selecting_player = if let Some(cs) = card_state.as_ref() {
        match cs.selecting_player {
            Player::P1 => "P1".to_string(),
            Player::P2 => "P2".to_string(),
        }
    } else {
        "P1".to_string()
    };
    let card_selected_idx = if let Some(cs) = card_state.as_ref() { cs.selected_idx } else { 0 };
    let drawn_cards = if let Some(cs) = card_state.as_ref() { cs.drawn_cards } else { [0; 5] };

    let mut gravity_wells = Vec::new();
    for (trans, gw) in gravity_wells_query.iter() {
        gravity_wells.push(super::GravityWellNetState {
            pos: trans.translation.xy(),
            radius: gw.radius,
            lifetime: gw.lifetime,
        });
    }

    let packet = HostStatePacket {
        p1,
        p2,
        bullets,
        poison_clouds: Vec::new(),
        explosion_events,
        gravity_wells,
        p1_wins: score_tracker.p1_wins,
        p2_wins: score_tracker.p2_wins,
        p1_joined: lobby_slots.p1.is_some(),
        p2_joined: lobby_slots.p2.is_some(),
        p1_is_gamepad: lobby_slots.p1.as_ref().map(|d| matches!(d, InputDevice::Gamepad(_))).unwrap_or(false),
        p2_is_gamepad: lobby_slots.p2.as_ref().map(|d| matches!(d, InputDevice::Gamepad(_))).unwrap_or(false),
        active_map: format!("{:?}", *active_map),
        game_state: format!("{:?}", *state.get()),
        selecting_player,
        card_selected_idx,
        drawn_cards,
        p1_cards: persistent_stats.p1.cards.clone(),
        p2_cards: persistent_stats.p2.cards.clone(),
    };

    if let Ok(bytes) = serde_json::to_vec(&packet) {
        socket.channel_mut(0).send(bytes.into(), client_peer);
    }
}
