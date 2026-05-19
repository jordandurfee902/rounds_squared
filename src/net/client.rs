use bevy::prelude::*;
use crate::physics::components::{ControllerInput, Velocity, Grounded, WallContact, JumpAllowance};
use crate::physics::weapon::{Weapon, Projectile};
use crate::player::{Player, Health, PlayerStatsComponent, BlockComponent};
use crate::settings::{LobbySlots, GameState, ScoreTracker, PersistentPlayerStats, InputDevice};
use crate::physics::anim::PlayerAim;
use crate::maps::ActiveMap;
use super::{MatchboxSocketResource, LocalInput, ClientInputPacket, HostStatePacket, matchmaking::parse_game_state, matchmaking::parse_active_map};

pub fn client_network_system(
    mut commands: Commands,
    socket_res: Option<ResMut<MatchboxSocketResource>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut lobby_slots: ResMut<LobbySlots>,
    mut score_tracker: ResMut<ScoreTracker>,
    mut active_map: ResMut<ActiveMap>,
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
    client_projectiles: Query<Entity, With<Projectile>>,
    client_gravity_wells: Query<Entity, With<crate::physics::card_selection::cards::gravity_vortex::GravityWell>>,
    mut persistent_stats: ResMut<PersistentPlayerStats>,
    card_state: Option<ResMut<crate::physics::card_selection::CardSelectionState>>,
    local_input: LocalInput,
    time: Res<Time>,
    mut gamepad_cooldown: Local<f32>,
    local_player_idx: Option<Res<crate::net::LocalPlayerIndex>>,
) {
    let Some(mut socket) = socket_res else {
        return;
    };
    socket.update_peers();

    let peer_ids: Vec<_> = socket.connected_peers().collect();
    if peer_ids.is_empty() {
        return;
    }
    let host_peer = peer_ids[0];

    let local_idx = local_player_idx.map(|idx| idx.0).unwrap_or(0);

    // 1. Gather client inputs
    let mut move_dir = 0.0;
    let mut jump = false;
    let mut fast_fall = false;
    let mut fire = false;
    let mut reload = false;
    let mut block = false;

    let is_card_selection = *state.get() == GameState::CardSelection;

    let mut card_left = false;
    let mut card_right = false;
    let mut card_confirm = false;

    let selecting_device = if local_idx < lobby_slots.slots.len() {
        &lobby_slots.slots[local_idx]
    } else {
        &None
    };

    if is_card_selection {
        if *gamepad_cooldown > 0.0 {
            *gamepad_cooldown = (*gamepad_cooldown - time.delta_secs()).max(0.0);
        }

        let mut poll_keyboard = false;
        let mut poll_gamepad = None;
        match selecting_device {
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
                card_left = true;
            }
            if local_input.keys.just_pressed(KeyCode::KeyD) || local_input.keys.just_pressed(KeyCode::ArrowRight) {
                card_right = true;
            }
            if local_input.keys.just_pressed(KeyCode::Space) || local_input.keys.just_pressed(KeyCode::Enter) {
                card_confirm = true;
            }
        }

        let mut gp = None;
        if let Some(gp_ent) = poll_gamepad {
            if let Ok(g) = local_input.gamepads.get(gp_ent) {
                gp = Some(g);
            } else if let Some(g) = local_input.gamepads.iter().next() {
                gp = Some(g);
            }
        } else if selecting_device.is_none() {
            if let Some(g) = local_input.gamepads.iter().next() {
                gp = Some(g);
            }
        }

        if let Some(g) = gp {
            if g.just_pressed(GamepadButton::DPadLeft) {
                card_left = true;
            }
            if g.just_pressed(GamepadButton::DPadRight) {
                card_right = true;
            }
            if g.just_pressed(GamepadButton::South) {
                card_confirm = true;
            }

            if *gamepad_cooldown <= 0.0 {
                let stick_x = g.left_stick().x;
                if stick_x < -0.5 {
                    card_left = true;
                    *gamepad_cooldown = 0.25;
                } else if stick_x > 0.5 {
                    card_right = true;
                    *gamepad_cooldown = 0.25;
                }
            }
        }
    } else {
        let kb_default = crate::settings::KeyboardControls::default();
        let kb = local_input.kb_controls.as_deref().unwrap_or(&kb_default);
        let ctrl_default = crate::settings::ControllerControls::default();
        let ctrl = local_input.ctrl_controls.as_deref().unwrap_or(&ctrl_default);

        if let Some(device) = selecting_device {
            match device {
                InputDevice::KeyboardMouse => {
                    let left_key = crate::settings::parse_key_code(&kb.move_left).unwrap_or(KeyCode::KeyA);
                    let right_key = crate::settings::parse_key_code(&kb.move_right).unwrap_or(KeyCode::KeyD);
                    let jump_key = crate::settings::parse_key_code(&kb.jump).unwrap_or(KeyCode::KeyW);
                    let fast_fall_key = crate::settings::parse_key_code(&kb.fast_fall).unwrap_or(KeyCode::KeyS);
                    let reload_key = crate::settings::parse_key_code(&kb.reload).unwrap_or(KeyCode::KeyR);
                    let block_key = crate::settings::parse_key_code(&kb.block).unwrap_or(KeyCode::KeyX);

                    if local_input.keys.pressed(left_key) { move_dir -= 1.0; }
                    if local_input.keys.pressed(right_key) { move_dir += 1.0; }
                    if local_input.keys.just_pressed(jump_key) { jump = true; }
                    if local_input.keys.pressed(fast_fall_key) { fast_fall = true; }
                    if local_input.keys.just_pressed(reload_key) { reload = true; }
                    
                    if let Some(mb) = crate::settings::parse_mouse_button(&kb.block) {
                        if local_input.mouse.just_pressed(mb) { block = true; }
                    } else if local_input.keys.just_pressed(block_key) {
                        block = true;
                    }

                    if let Some(mb) = crate::settings::parse_mouse_button(&kb.shoot) {
                        if local_input.mouse.pressed(mb) { fire = true; }
                    } else if let Some(kc) = crate::settings::parse_key_code(&kb.shoot) {
                        if local_input.keys.pressed(kc) { fire = true; }
                    } else {
                        if local_input.mouse.pressed(MouseButton::Left) { fire = true; }
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
                        move_dir = stick.x;
                        let jump_btn = crate::settings::parse_gamepad_button(&ctrl.jump).unwrap_or(GamepadButton::South);
                        if gp.just_pressed(jump_btn) { jump = true; }
                        if stick.y < -0.5 { fast_fall = true; }
                        let reload_btn = crate::settings::parse_gamepad_button(&ctrl.reload).unwrap_or(GamepadButton::West);
                        if gp.just_pressed(reload_btn) { reload = true; }
                        let shoot_btn = crate::settings::parse_gamepad_button(&ctrl.shoot).unwrap_or(GamepadButton::RightTrigger2);
                        if gp.pressed(shoot_btn) { fire = true; }
                        let block_btn = crate::settings::parse_gamepad_button(&ctrl.block).unwrap_or(GamepadButton::LeftTrigger2);
                        if gp.just_pressed(block_btn) { block = true; }
                    }
                }
            }
        }
    }

    let mut aim_direction = Vec2::X;
    let local_player = Player::from_index(local_idx);
    for (player, _, _, _, _, _, _, aim, _, _, _, _) in players.iter() {
        if *player == local_player {
            aim_direction = aim.direction;
        }
    }

    let input_pkt = ClientInputPacket {
        move_dir,
        jump,
        fast_fall,
        fire,
        reload,
        block,
        aim_dir_x: aim_direction.x,
        aim_dir_y: aim_direction.y,
        lobby_joined: selecting_device.is_some(),
        is_gamepad: selecting_device.as_ref().map(|d| matches!(d, InputDevice::Gamepad(_))).unwrap_or(false),
        card_left,
        card_right,
        card_confirm,
    };

    if let Ok(bytes) = serde_json::to_vec(&input_pkt) {
        socket.channel_mut(0).send(bytes.into(), host_peer);
    }

    // 2. Receive authoritative Host state packet
    let mut host_packet: Option<HostStatePacket> = None;
    let packets = socket.channel_mut(0).receive();
    for (peer, data) in packets {
        if peer == host_peer {
            if let Ok(pkt) = serde_json::from_slice::<HostStatePacket>(&data) {
                host_packet = Some(pkt);
            }
        }
    }

    let Some(pkt) = host_packet else {
        return;
    };

    // Sync persistent stats and apply cards if changed
    let mut stats_changed = false;
    for i in 0..8 {
        if persistent_stats.players[i].cards != pkt.player_cards[i] {
            persistent_stats.players[i].cards = pkt.player_cards[i].clone();
            stats_changed = true;
        }
    }
    if stats_changed {
        let settings = crate::settings::load_settings();
        rebuild_player_stats(&mut persistent_stats, &settings);
    }

    // 3. Overwrite local players status
    for (player, mut transform, mut vel, mut health, mut _stats, mut block, mut weapon, mut aim, mut _input, mut grounded, mut _wall, mut _jump_allow) in players.iter_mut() {
        let p_idx = player.index();
        if p_idx < pkt.players.len() {
            let p_state = &pkt.players[p_idx];
            transform.translation = p_state.pos.extend(transform.translation.z);
            vel.0 = p_state.vel;
            health.current = p_state.health;
            health.max = p_state.max_health;
            block.active_timer = p_state.block_active_timer;
            block.cooldown_timer = p_state.block_cooldown_timer;
            block.control_lockout_timer = p_state.block_lockout_timer;
            aim.direction = p_state.aim_dir;
            weapon.max_ammo = p_state.ammo_max;
            weapon.current_ammo = p_state.ammo_current;
            weapon.reload_timer = p_state.reload_timer;
            grounded.0 = p_state.grounded;
        }
    }

    // Overwrite Lobby slots
    for i in 0..8 {
        if pkt.active_players[i] {
            if lobby_slots.slots[i].is_none() {
                lobby_slots.slots[i] = Some(if pkt.is_gamepad[i] { InputDevice::Gamepad(Entity::PLACEHOLDER) } else { InputDevice::KeyboardMouse });
            }
        } else {
            lobby_slots.slots[i] = None;
        }
    }

    // Overwrite Score
    score_tracker.wins = pkt.wins;

    // Overwrite GameState
    let target_state = parse_game_state(&pkt.game_state);
    if *state.get() != target_state {
        next_state.set(target_state);
    }

    // Overwrite Map
    let target_map = parse_active_map(&pkt.active_map);
    if *active_map != target_map {
        *active_map = target_map;
    }

    // Overwrite Card selection state
    if target_state == GameState::CardSelection {
        let sel_player = if let Some(idx) = pkt.selecting_player {
            Player::from_index(idx)
        } else {
            Player::P1
        };
        if let Some(mut cs) = card_state {
            cs.selecting_player = sel_player;
            cs.selected_idx = pkt.card_selected_idx;
            cs.drawn_cards = pkt.drawn_cards;
        } else {
            commands.insert_resource(crate::physics::card_selection::CardSelectionState {
                selected_idx: pkt.card_selected_idx,
                selecting_player: sel_player,
                drawn_cards: pkt.drawn_cards,
                queue: Vec::new(),
            });
        }
    } else {
        commands.remove_resource::<crate::physics::card_selection::CardSelectionState>();
    }

    // 4. Overwrite projectiles
    for ent in client_projectiles.iter() {
        commands.entity(ent).despawn();
    }

    for b in pkt.bullets {
        commands.spawn((
            Projectile {
                owner: match b.owner.as_str() {
                    "P1" => Player::P1,
                    "P2" => Player::P2,
                    "P3" => Player::P3,
                    "P4" => Player::P4,
                    "P5" => Player::P5,
                    "P6" => Player::P6,
                    "P7" => Player::P7,
                    "P8" => Player::P8,
                    _ => Player::P1,
                },
                velocity: b.vel,
                base_damage: b.damage,
                damage: b.damage,
                gravity: b.gravity,
                size_multiplier: b.size_multiplier,
                growth: b.growth,
                time_in_air: b.time_in_air,
                lifetime: b.lifetime,
                special_effects: b.special_effects,
                player_scale: b.player_scale,
                bounces: b.bounces,
                bounce_speed_multiplier: b.bounce_speed_multiplier,
            },
            Transform::from_xyz(b.pos.x, b.pos.y, 11.0),
        ));
    }

    // 5. Trigger explosions
    for exp in pkt.explosion_events {
        let color = Color::srgb(exp.color_r, exp.color_g, exp.color_b);
        if exp.damage < 0.0 {
            crate::physics::particles::spawn_spark_burst(&mut commands, exp.pos, color, 8, 42);
        } else {
            crate::physics::particles::spawn_damage_explosion(&mut commands, exp.pos, color, exp.damage, 42);
        }
    }

    // 6. Overwrite gravity wells
    for ent in client_gravity_wells.iter() {
        commands.entity(ent).despawn();
    }

    for gw in pkt.gravity_wells {
        commands.spawn((
            crate::physics::card_selection::cards::gravity_vortex::GravityWell {
                strength: 320.0,
                radius: gw.radius,
                lifetime: gw.lifetime,
            },
            Transform::from_xyz(gw.pos.x, gw.pos.y, 5.0),
        ));
    }
}

fn rebuild_player_stats(
    persistent_stats: &mut PersistentPlayerStats,
    settings: &crate::settings::AppSettings,
) {
    for i in 0..8 {
        let char_settings = settings.get_character(i);
        let p = &mut persistent_stats.players[i];
        
        p.movement_speed = char_settings.speed;
        p.jump_force = settings.physics.player_jump_force;
        p.player_scale = char_settings.size;
        p.health_max = char_settings.health;
        p.bullet_range = char_settings.bullet_range;
        p.bullet_speed = char_settings.bullet_speed;
        p.bullet_gravity = char_settings.bullet_gravity;
        p.bullet_damage = char_settings.damage;
        p.bullet_size_mult = char_settings.bullet_size_mult;
        p.bullet_growth = char_settings.bullet_growth;
        p.max_ammo = char_settings.max_ammo;
        p.reload_time = char_settings.reload_time;
        p.fire_rate = char_settings.fire_rate;
        p.bounces = char_settings.bounces;
        p.bounce_speed_multiplier = char_settings.bounce_speed_multiplier;
        p.block_duration = char_settings.block_duration;
        p.block_cooldown = char_settings.block_cooldown;
        p.block_border_boost = char_settings.block_border_boost;
        p.special_effects = char_settings.special_effects.clone();

        let p_cards = p.cards.clone();
        for &card_idx in &p_cards {
            if let Some(card) = crate::physics::card_selection::cards::get_card(card_idx) {
                card.apply(p);
            }
        }
    }
}
