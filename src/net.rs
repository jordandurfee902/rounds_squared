use bevy::prelude::*;
use matchbox_socket::WebRtcSocket;
use serde::{Serialize, Deserialize};

use crate::physics::components::{ControllerInput, Velocity, Grounded, WallContact, JumpAllowance};
use crate::physics::weapon::{Weapon, Projectile};
use crate::physics::particles::ExplosionRecordComponent;
use crate::player::{Player, Health, PlayerStatsComponent, BlockComponent};
use crate::settings::{LobbySlots, GameState, ScoreTracker, PersistentPlayerStats, InputDevice};
use crate::physics::anim::PlayerAim;
use crate::maps::ActiveMap;

// --- DETERMINISTIC RNG FOR OTHER SYSTEMS ---

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

// --- AUTHORITATIVE REPLICATION PROTOCOL STRUCTS ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInputPacket {
    pub move_dir: f32,
    pub jump: bool,
    pub fast_fall: bool,
    pub fire: bool,
    pub reload: bool,
    pub block: bool,
    pub aim_dir_x: f32,
    pub aim_dir_y: f32,
    pub lobby_joined: bool,
    pub is_gamepad: bool,
    pub card_left: bool,
    pub card_right: bool,
    pub card_confirm: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerNetState {
    pub pos: Vec2,
    pub vel: Vec2,
    pub health: f32,
    pub max_health: f32,
    pub block_active_timer: f32,
    pub block_cooldown_timer: f32,
    pub block_lockout_timer: f32,
    pub aim_dir: Vec2,
    pub ammo_max: u32,
    pub ammo_current: u32,
    pub reload_timer: f32,
    pub grounded: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BulletNetState {
    pub pos: Vec2,
    pub vel: Vec2,
    pub owner: String, // "P1" or "P2"
    pub damage: f32,
    pub gravity: f32,
    pub size_multiplier: f32,
    pub growth: f32,
    pub time_in_air: f32,
    pub lifetime: f32,
    pub special_effects: Vec<String>,
    pub player_scale: f32,
    pub bounces: u32,
    pub bounce_speed_multiplier: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PoisonCloudNetState {
    pub pos: Vec2,
    pub size: f32,
    pub life: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExplosionEvent {
    pub pos: Vec2,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub damage: f32, // -1.0 represents muzzle flash
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostStatePacket {
    pub p1: PlayerNetState,
    pub p2: PlayerNetState,
    pub bullets: Vec<BulletNetState>,
    pub poison_clouds: Vec<PoisonCloudNetState>,
    pub explosion_events: Vec<ExplosionEvent>,
    pub p1_wins: u32,
    pub p2_wins: u32,
    pub p1_joined: bool,
    pub p2_joined: bool,
    pub p1_is_gamepad: bool,
    pub p2_is_gamepad: bool,
    pub active_map: String,
    pub game_state: String,
    pub selecting_player: String, // "P1" or "P2"
    pub card_selected_idx: usize,
}

// --- SESSION BINDINGS & STATUS RESOURCES ---

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayerIndex(pub usize);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsNetworked(pub bool);

#[derive(Resource, Deref, DerefMut)]
pub struct MatchboxSocketResource(pub WebRtcSocket);

#[derive(Resource, Default, Debug, Clone)]
pub struct OnlineCodeResource {
    pub code: String, // Empty String means public matchmaking
    pub is_host: bool,
}

// --- MATCHMAKING & SIGNALING SYSTEMS ---

pub fn start_matchmaking(
    mut commands: Commands,
    code_res: Option<Res<OnlineCodeResource>>,
) {
    let room_url = if let Some(res) = code_res {
        if res.code.is_empty() {
            info!("MATCHMAKING: Connecting to public matchmaking lobby...");
            "wss://durfdog-sets.hf.space/rounds_lobby?next=2".to_string()
        } else {
            let formatted_code = res.code.replace(" ", "");
            info!("MATCHMAKING: Connecting to private room code: {} (is_host: {})", formatted_code, res.is_host);
            format!("wss://durfdog-sets.hf.space/room_{}?next=2", formatted_code)
        }
    } else {
        info!("MATCHMAKING: Connecting to public matchmaking lobby (fallback)...");
        "wss://durfdog-sets.hf.space/rounds_lobby?next=2".to_string()
    };
    
    let (socket, message_loop) = WebRtcSocket::builder(&room_url)
        .add_channel(matchbox_socket::ChannelConfig::unreliable())
        .build();
    
    bevy::tasks::IoTaskPool::get().spawn(message_loop).detach();
    commands.insert_resource(MatchboxSocketResource(socket));
}

pub fn lobby_system(
    mut commands: Commands,
    socket_res: Option<ResMut<MatchboxSocketResource>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(mut socket) = socket_res else {
        return;
    };

    socket.update_peers();

    let connected_peers = socket.connected_peers().count();
    if connected_peers < 1 {
        return;
    }

    info!("MATCHMAKING: Peer connected! Establishing Host-Client relationship...");

    let local_id = socket.id().expect("Socket should have an ID");
    let mut all_ids = vec![local_id];
    for peer in socket.connected_peers() {
        all_ids.push(peer);
    }
    all_ids.sort_by_key(|id| id.to_string());

    let mut local_player_idx = 0;
    for (i, id) in all_ids.iter().enumerate() {
        if *id == local_id {
            local_player_idx = i;
        }
    }

    commands.insert_resource(LocalPlayerIndex(local_player_idx));
    commands.insert_resource(IsNetworked(true));
    commands.insert_resource(RollbackRng::new(98765));

    info!("MATCHMAKING: Connection verified! Local Player Index: {}", local_player_idx);
    next_state.set(GameState::Lobby);
}

// --- HELPER PARSING FUNCTIONS ---

pub fn parse_game_state(name: &str) -> GameState {
    match name {
        "MainMenu" => GameState::MainMenu,
        "Lobby" => GameState::Lobby,
        "OnlineMenu" => GameState::OnlineMenu,
        "Matchmaking" => GameState::Matchmaking,
        "Gameplay" => GameState::Gameplay,
        "CardSelection" => GameState::CardSelection,
        _ => GameState::Gameplay,
    }
}

pub fn parse_active_map(name: &str) -> ActiveMap {
    match name {
        "DefaultMap" => ActiveMap::DefaultMap,
        "PillarsMap" => ActiveMap::PillarsMap,
        "StadiumMap" => ActiveMap::StadiumMap,
        "ChasmBridge" => ActiveMap::ChasmBridge,
        "Gridlock" => ActiveMap::Gridlock,
        "Hourglass" => ActiveMap::Hourglass,
        "IceTemple" => ActiveMap::IceTemple,
        "IndustrialFoundry" => ActiveMap::IndustrialFoundry,
        "VerticalHelix" => ActiveMap::VerticalHelix,
        "TectonicFissure" => ActiveMap::TectonicFissure,
        "ZenGarden" => ActiveMap::ZenGarden,
        "SpaceStation" => ActiveMap::SpaceStation,
        "AncientColiseum" => ActiveMap::AncientColiseum,
        _ => ActiveMap::DefaultMap,
    }
}

pub fn apply_card_selection_effect(p_stats: &mut crate::settings::PlayerStats, card_idx: usize) {
    p_stats.cards.push(card_idx);
    match card_idx {
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
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct LocalInput<'w, 's> {
    pub keys: Res<'w, ButtonInput<KeyCode>>,
    pub mouse: Res<'w, ButtonInput<MouseButton>>,
    pub gamepads: Query<'w, 's, &'static Gamepad>,
    pub kb_controls: Option<Res<'w, crate::settings::KeyboardControls>>,
    pub ctrl_controls: Option<Res<'w, crate::settings::ControllerControls>>,
}

// --- HOST EXECUTOR NETWORK SYSTEM ---

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
                    apply_card_selection_effect(p_stats, cs.selected_idx);
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
                        apply_card_selection_effect(p_stats, cs.selected_idx);
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

    let packet = HostStatePacket {
        p1,
        p2,
        bullets,
        poison_clouds: Vec::new(),
        explosion_events,
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
    };

    if let Ok(bytes) = serde_json::to_vec(&packet) {
        socket.channel_mut(0).send(bytes.into(), client_peer);
    }
}

// --- CLIENT EXECUTOR NETWORK SYSTEM ---

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
    _persistent_stats: ResMut<PersistentPlayerStats>,
    card_state: Option<ResMut<crate::physics::card_selection::CardSelectionState>>,
    local_input: LocalInput,
    time: Res<Time>,
    mut gamepad_cooldown: Local<f32>,
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

    // 1. Gather client inputs (Player 2 controls)
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

    if is_card_selection {
        if *gamepad_cooldown > 0.0 {
            *gamepad_cooldown = (*gamepad_cooldown - time.delta_secs()).max(0.0);
        }

        let mut poll_keyboard = false;
        let mut poll_gamepad = None;
        match &lobby_slots.p2 {
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
        } else if lobby_slots.p2.is_none() {
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

        if let Some(device) = &lobby_slots.p2 {
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
    for (player, _, _, _, _, _, _, aim, _, _, _, _) in players.iter() {
        if *player == Player::P2 {
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
        lobby_joined: lobby_slots.p2.is_some(),
        is_gamepad: lobby_slots.p2.as_ref().map(|d| matches!(d, InputDevice::Gamepad(_))).unwrap_or(false),
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

    // 3. Overwrite local players status
    for (player, mut transform, mut vel, mut health, mut _stats, mut block, mut weapon, mut aim, mut _input, mut grounded, mut _wall, mut _jump_allow) in players.iter_mut() {
        let p_state = match player {
            Player::P1 => &pkt.p1,
            Player::P2 => &pkt.p2,
        };

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

    // Overwrite Lobby slots
    if pkt.p1_joined {
        if lobby_slots.p1.is_none() {
            lobby_slots.p1 = Some(if pkt.p1_is_gamepad { InputDevice::Gamepad(Entity::PLACEHOLDER) } else { InputDevice::KeyboardMouse });
        }
    } else {
        lobby_slots.p1 = None;
    }

    // Overwrite Score
    score_tracker.p1_wins = pkt.p1_wins;
    score_tracker.p2_wins = pkt.p2_wins;

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
        let sel_player = match pkt.selecting_player.as_str() {
            "P1" => Player::P1,
            _ => Player::P2,
        };
        if let Some(mut cs) = card_state {
            cs.selecting_player = sel_player;
            cs.selected_idx = pkt.card_selected_idx;
        } else {
            commands.insert_resource(crate::physics::card_selection::CardSelectionState {
                selected_idx: pkt.card_selected_idx,
                selecting_player: sel_player,
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
                    _ => Player::P2,
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
}
