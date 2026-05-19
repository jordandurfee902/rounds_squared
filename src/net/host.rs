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

#[derive(bevy::ecs::system::SystemParam)]
pub struct HostSystemResources<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub socket_res: Option<ResMut<'w, MatchboxSocketResource>>,
    pub next_state: ResMut<'w, NextState<GameState>>,
    pub lobby_slots: ResMut<'w, LobbySlots>,
    pub score_tracker: Res<'w, ScoreTracker>,
    pub active_map: Res<'w, ActiveMap>,
    pub state: Res<'w, State<GameState>>,
    pub persistent_stats: ResMut<'w, PersistentPlayerStats>,
    pub card_state: Option<ResMut<'w, crate::physics::card_selection::CardSelectionState>>,
    pub time: Res<'w, Time>,
    pub rollback_rng: Option<ResMut<'w, crate::net::RollbackRng>>,
}

pub fn host_network_system(
    mut resources: HostSystemResources,
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
    local_input: LocalInput,
    mut gamepad_cooldown: Local<f32>,
    gravity_wells_query: Query<(&Transform, &crate::physics::card_selection::cards::gravity_vortex::GravityWell), Without<Player>>,
) {
    let HostSystemResources {
        mut commands,
        socket_res,
        mut next_state,
        mut lobby_slots,
        score_tracker,
        active_map,
        state,
        mut persistent_stats,
        mut card_state,
        time,
        mut rollback_rng,
    } = resources;

    let Some(mut socket) = socket_res else {
        return;
    };
    socket.update_peers();

    let local_id = socket.id().expect("Socket should have an ID");
    let mut all_ids = vec![local_id];
    for peer in socket.connected_peers() {
        all_ids.push(peer);
    }
    all_ids.sort_by_key(|id| id.to_string());

    // 1. Receive client inputs from all client peers
    let mut client_packets: [Option<ClientInputPacket>; 8] = [None, None, None, None, None, None, None, None];
    let packets = socket.channel_mut(0).receive();
    for (peer, data) in packets {
        if let Some(p_idx) = all_ids.iter().position(|&id| id == peer) {
            if p_idx > 0 && p_idx < 8 {
                if let Ok(pkt) = serde_json::from_slice::<ClientInputPacket>(&data) {
                    client_packets[p_idx] = Some(pkt);
                }
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

    let host_device = &lobby_slots.slots[0];

    if is_card_selection {
        if *gamepad_cooldown > 0.0 {
            *gamepad_cooldown = (*gamepad_cooldown - time.delta_secs()).max(0.0);
        }

        let mut poll_keyboard = false;
        let mut poll_gamepad = None;
        match host_device {
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
        } else if host_device.is_none() {
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

        if let Some(device) = host_device {
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

    // Apply inputs to players
    for (player, mut transform, mut vel, mut health, mut _stats, mut block, mut weapon, mut aim, mut input, mut grounded, mut _wall, mut _jump_allow) in players.iter_mut() {
        let p_idx = player.index();
        if p_idx == 0 {
            input.move_dir = p1_move_dir;
            input.jump = p1_jump;
            input.fast_fall = p1_fast_fall;
            input.fire = p1_fire;
            input.reload = p1_reload;
            input.block = p1_block;
        } else if p_idx < 8 {
            if let Some(pkt) = &client_packets[p_idx] {
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
    }

    // Apply local P1 Card Selection navigation
    if *state.get() == GameState::CardSelection {
        if let Some(ref mut cs) = card_state {
            let sel_idx = cs.selecting_player.index();
            if sel_idx == 0 {
                if p1_card_left {
                    cs.selected_idx = if cs.selected_idx == 0 { 4 } else { cs.selected_idx - 1 };
                } else if p1_card_right {
                    cs.selected_idx = (cs.selected_idx + 1) % 5;
                }

                if p1_card_confirm {
                    let p_stats = &mut persistent_stats.players[0];
                    apply_card_selection_effect(p_stats, cs.drawn_cards[cs.selected_idx]);
                    
                    if let Some(ref mut rng) = rollback_rng {
                        if !cs.queue.is_empty() {
                            let next_player = cs.queue.remove(0);
                            cs.selecting_player = next_player;
                            cs.selected_idx = 2;

                            let mut drawn = [0; 5];
                            let mut available: Vec<usize> = (0..crate::physics::card_selection::cards::TOTAL_CARDS_COUNT).collect();
                            for i in 0..5 {
                                if available.is_empty() {
                                    break;
                                }
                                let idx = (rng.next_f32() * available.len() as f32) as usize;
                                drawn[i] = available.remove(idx);
                            }
                            cs.drawn_cards = drawn;
                        } else {
                            next_state.set(GameState::Gameplay);
                        }
                    }
                }
            }
        }
    }

    // Process Client Inputs for Lobby Slots & Card Selection
    for i in 1..8 {
        if i < all_ids.len() {
            if let Some(pkt) = &client_packets[i] {
                // Sync lobby slots
                if pkt.lobby_joined {
                    if lobby_slots.slots[i].is_none() {
                        lobby_slots.slots[i] = Some(if pkt.is_gamepad { InputDevice::Gamepad(Entity::PLACEHOLDER) } else { InputDevice::KeyboardMouse });
                    }
                } else {
                    lobby_slots.slots[i] = None;
                }

                // Card Selection navigation for clients
                if *state.get() == GameState::CardSelection {
                    if let Some(ref mut cs) = card_state {
                        if cs.selecting_player.index() == i {
                            if pkt.card_left {
                                cs.selected_idx = if cs.selected_idx == 0 { 4 } else { cs.selected_idx - 1 };
                            } else if pkt.card_right {
                                cs.selected_idx = (cs.selected_idx + 1) % 5;
                            }

                            if pkt.card_confirm {
                                let p_stats = &mut persistent_stats.players[i];
                                apply_card_selection_effect(p_stats, cs.drawn_cards[cs.selected_idx]);

                                if let Some(ref mut rng) = rollback_rng {
                                    if !cs.queue.is_empty() {
                                        let next_player = cs.queue.remove(0);
                                        cs.selecting_player = next_player;
                                        cs.selected_idx = 2;

                                        let mut drawn = [0; 5];
                                        let mut available: Vec<usize> = (0..crate::physics::card_selection::cards::TOTAL_CARDS_COUNT).collect();
                                        for i in 0..5 {
                                            if available.is_empty() {
                                                break;
                                            }
                                            let idx = (rng.next_f32() * available.len() as f32) as usize;
                                            drawn[i] = available.remove(idx);
                                        }
                                        cs.drawn_cards = drawn;
                                    } else {
                                        next_state.set(GameState::Gameplay);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            lobby_slots.slots[i] = None;
        }
    }

    // 3. Serialize World State
    let mut players_net = vec![PlayerNetState {
        pos: Vec2::ZERO, vel: Vec2::ZERO, health: 100.0, max_health: 100.0,
        block_active_timer: 0.0, block_cooldown_timer: 0.0, block_lockout_timer: 0.0,
        aim_dir: Vec2::X, ammo_max: 5, ammo_current: 5, reload_timer: 0.0,
        grounded: true,
    }; 8];

    for (player, transform, vel, health, _, block, weapon, aim, _, grounded, _, _) in players.iter() {
        let p_idx = player.index();
        if p_idx < 8 {
            players_net[p_idx] = PlayerNetState {
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
        }
    }

    let mut bullets = Vec::new();
    for (trans, proj) in projectiles.iter() {
        bullets.push(BulletNetState {
            pos: trans.translation.xy(),
            vel: proj.velocity,
            owner: format!("{:?}", proj.owner),
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

    let selecting_player = card_state.as_ref().map(|cs| cs.selecting_player.index());
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

    let mut active_players = [false; 8];
    let mut is_gamepad = [false; 8];
    for i in 0..8 {
        if lobby_slots.slots[i].is_some() {
            active_players[i] = true;
            is_gamepad[i] = lobby_slots.slots[i].as_ref().map(|d| matches!(d, InputDevice::Gamepad(_))).unwrap_or(false);
        }
    }

    let mut player_cards = [
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new()
    ];
    for i in 0..8 {
        player_cards[i] = persistent_stats.players[i].cards.clone();
    }

    let packet = HostStatePacket {
        players: players_net,
        bullets,
        poison_clouds: Vec::new(),
        explosion_events,
        gravity_wells,
        wins: score_tracker.wins,
        active_players,
        is_gamepad,
        active_map: format!("{:?}", *active_map),
        game_state: format!("{:?}", *state.get()),
        selecting_player,
        card_selected_idx,
        drawn_cards,
        player_cards,
    };

    let peers: Vec<_> = socket.connected_peers().collect();
    for peer in peers {
        if let Ok(bytes) = serde_json::to_vec(&packet) {
            socket.channel_mut(0).send(bytes.into(), peer);
        }
    }
}
