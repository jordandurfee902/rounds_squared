use bevy::prelude::*;
use crate::physics::*;
use crate::physics::weapon::Weapon;
use crate::settings::{PersistentPlayerStats, GameState, LobbySlots, InputDevice, KeyboardControls, ControllerControls, parse_key_code, parse_gamepad_button, parse_mouse_button};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Gameplay), spawn_players.after(crate::map::spawn_platforms))
           .add_systems(OnExit(GameState::Gameplay), despawn_gameplay_entities)
           .add_systems(Update, (
               player_input.before(crate::physics::forces::apply_gravity_and_movement),
               player_block_system,
           ).chain().run_if(in_state(GameState::Gameplay).and(crate::physics::is_not_paused).and(resource_equals(crate::net::IsNetworked(false)))))
           .add_systems(Update, (
               draw_health_bars,
           ).run_if(in_state(GameState::Gameplay)));
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    P1,
    P2,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PlayerStatsComponent {
    pub movement_speed: f32,
    pub jump_force: f32,
    pub player_scale: f32,
    pub health_max: f32,
    pub block_duration: f32,
    pub block_cooldown: f32,
    pub block_border_boost: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct BlockComponent {
    pub active_timer: f32,
    pub cooldown_timer: f32,
    pub block_duration: f32,
    pub block_cooldown: f32,
    pub control_lockout_timer: f32,
}

pub fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    persistent_stats: Res<PersistentPlayerStats>,
    active_map: Res<crate::maps::ActiveMap>,
) {
    let p1_stats = &persistent_stats.p1;
    let p2_stats = &persistent_stats.p2;

    let p1_scale = p1_stats.player_scale;
    let p2_scale = p2_stats.player_scale;

    let (p1_spawn, p2_spawn) = match *active_map {
        crate::maps::ActiveMap::DefaultMap => {
            // Default map: Spawn above left/right outer ledge wings
            (Vec3::new(-1350.0, 100.0, 10.0), Vec3::new(1350.0, 100.0, 10.0))
        }
        crate::maps::ActiveMap::PillarsMap => {
            // Pillars map: Spawn above left/right floating platforms
            (Vec3::new(-650.0, 150.0, 10.0), Vec3::new(650.0, 150.0, 10.0))
        }
        crate::maps::ActiveMap::StadiumMap => {
            // Stadium map: Spawn on left/right sides of the massive central floor
            (Vec3::new(-350.0, -100.0, 10.0), Vec3::new(350.0, -100.0, 10.0))
        }
        crate::maps::ActiveMap::ChasmBridge => {
            (Vec3::new(-1300.0, 300.0, 10.0), Vec3::new(1300.0, 300.0, 10.0))
        }
        crate::maps::ActiveMap::Gridlock => {
            (Vec3::new(-800.0, 200.0, 10.0), Vec3::new(800.0, 200.0, 10.0))
        }
        crate::maps::ActiveMap::Hourglass => {
            (Vec3::new(-1100.0, -100.0, 10.0), Vec3::new(1100.0, -100.0, 10.0))
        }
        crate::maps::ActiveMap::IceTemple => {
            (Vec3::new(-1000.0, 250.0, 10.0), Vec3::new(1000.0, 250.0, 10.0))
        }
        crate::maps::ActiveMap::IndustrialFoundry => {
            (Vec3::new(-1200.0, 200.0, 10.0), Vec3::new(1200.0, 200.0, 10.0))
        }
        crate::maps::ActiveMap::VerticalHelix => {
            (Vec3::new(-700.0, -300.0, 10.0), Vec3::new(700.0, -300.0, 10.0))
        }
        crate::maps::ActiveMap::TectonicFissure => {
            (Vec3::new(-950.0, -350.0, 10.0), Vec3::new(950.0, -350.0, 10.0))
        }
        crate::maps::ActiveMap::ZenGarden => {
            (Vec3::new(-1100.0, -100.0, 10.0), Vec3::new(1100.0, -100.0, 10.0))
        }
        crate::maps::ActiveMap::SpaceStation => {
            (Vec3::new(-1250.0, 100.0, 10.0), Vec3::new(1250.0, 100.0, 10.0))
        }
        crate::maps::ActiveMap::AncientColiseum => {
            (Vec3::new(-800.0, 350.0, 10.0), Vec3::new(800.0, 350.0, 10.0))
        }
    };

    // Player 1 (Blue) - Base mass = 1.0 (Normal)
    commands.spawn((
        Player::P1,
        Collider::Circle { radius: 40.0 * p1_scale },
        Transform::from_translation(p1_spawn),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        Velocity(Vec2::ZERO),
        Acceleration(Vec2::ZERO),
        Grounded(true),
        WallContact::default(),
        JumpAllowance { value: 1 },
    )).insert((
        ControllerInput::default(),
        Mass(1.0),
        Health { current: p1_stats.health_max, max: p1_stats.health_max },
        PlayerAim::default(),
        ProceduralLimbs::default(),
        PlayerStatsComponent {
            movement_speed: p1_stats.movement_speed,
            jump_force: p1_stats.jump_force,
            player_scale: p1_stats.player_scale,
            health_max: p1_stats.health_max,
            block_duration: p1_stats.block_duration,
            block_cooldown: p1_stats.block_cooldown,
            block_border_boost: p1_stats.block_border_boost,
        },
        BlockComponent {
            active_timer: 0.0,
            cooldown_timer: 0.0,
            block_duration: p1_stats.block_duration,
            block_cooldown: p1_stats.block_cooldown,
            control_lockout_timer: 0.0,
        },
        Weapon {
            max_ammo: p1_stats.max_ammo,
            current_ammo: p1_stats.max_ammo,
            fire_cooldown: 0.0,
            fire_rate: p1_stats.fire_rate,
            reload_timer: 0.0,
            reload_time: p1_stats.reload_time,
            time_since_last_shot: 0.0,
        },
    )).with_children(|parent| {
        // Spawn visual body offset upwards by 15px to float perfectly on legs!
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(40.0 * p1_scale))),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.5, 1.0))),
            Transform::from_xyz(0.0, 15.0 * p1_scale, 0.0),
        ));
    });

    // Player 2 (Orange) - Base mass = 1.0 (Identical start-of-game balance)
    commands.spawn((
        Player::P2,
        Collider::Circle { radius: 40.0 * p2_scale },
        Transform::from_translation(p2_spawn),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        Velocity(Vec2::ZERO),
        Acceleration(Vec2::ZERO),
        Grounded(true),
        WallContact::default(),
        JumpAllowance { value: 1 },
    )).insert((
        ControllerInput::default(),
        Mass(1.0),
        Health { current: p2_stats.health_max, max: p2_stats.health_max },
        PlayerAim::default(),
        ProceduralLimbs::default(),
        PlayerStatsComponent {
            movement_speed: p2_stats.movement_speed,
            jump_force: p2_stats.jump_force,
            player_scale: p2_stats.player_scale,
            health_max: p2_stats.health_max,
            block_duration: p2_stats.block_duration,
            block_cooldown: p2_stats.block_cooldown,
            block_border_boost: p2_stats.block_border_boost,
        },
        BlockComponent {
            active_timer: 0.0,
            cooldown_timer: 0.0,
            block_duration: p2_stats.block_duration,
            block_cooldown: p2_stats.block_cooldown,
            control_lockout_timer: 0.0,
        },
        Weapon {
            max_ammo: p2_stats.max_ammo,
            current_ammo: p2_stats.max_ammo,
            fire_cooldown: 0.0,
            fire_rate: p2_stats.fire_rate,
            reload_timer: 0.0,
            reload_time: p2_stats.reload_time,
            time_since_last_shot: 0.0,
        },
    )).with_children(|parent| {
        // Spawn visual body offset upwards by 15px to float perfectly on legs!
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(40.0 * p2_scale))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.2))),
            Transform::from_xyz(0.0, 15.0 * p2_scale, 0.0),
        ));
    });
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&Player, &mut ControllerInput)>,
    gamepads: Query<&Gamepad>,
    lobby_slots: Res<LobbySlots>,
    kb_controls: Res<KeyboardControls>,
    ctrl_controls: Res<ControllerControls>,
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
                        if stick.y < -0.5 { fast_fall = true; }
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
                        if stick.y < -0.5 { fast_fall = true; }
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

fn draw_filled_rect(
    gizmos: &mut Gizmos,
    center: Vec2,
    size: Vec2,
    color: Color,
) {
    let half_width = size.x / 2.0;
    let half_height = size.y / 2.0;
    let steps = (size.y.round() as i32).max(1);
    
    for i in 0..steps {
        let t = if steps > 1 {
            (i as f32) / ((steps - 1) as f32)
        } else {
            0.5
        };
        let y = center.y - half_height + t * size.y;
        
        gizmos.line_2d(
            Vec2::new(center.x - half_width, y),
            Vec2::new(center.x + half_width, y),
            color,
        );
    }
}

fn draw_health_bars(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Health, &Player, &PlayerStatsComponent)>,
) {
    for (transform, health, player, stats) in query.iter() {
        let scale = stats.player_scale;
        let player_pos = transform.translation.xy();
        // Position the health bar about 90 pixels above the parent physical center (65px above visual floating body)
        let bar_center = player_pos + Vec2::new(0.0, 90.0 * scale);
        let bar_width = 64.0 * scale;
        let bar_height = 6.0 * scale;

        // 1. Draw solid outer boundary / background bar (dark solid)
        draw_filled_rect(
            &mut gizmos,
            bar_center,
            Vec2::new(bar_width, bar_height),
            Color::srgb(0.15, 0.15, 0.15),
        );

        // 2. Draw actual remaining health (green solid bar)
        let health_pct = (health.current / health.max).clamp(0.0, 1.0);
        if health_pct > 0.0 {
            let fg_width = bar_width * health_pct;
            // Shift the center as the health bar shrinks from left-to-right
            let fg_center = bar_center - Vec2::new((bar_width - fg_width) / 2.0, 0.0);
            
            let color = match player {
                Player::P1 => Color::srgb(0.2, 0.8, 0.2), // Vibrant Green
                Player::P2 => Color::srgb(0.2, 0.8, 0.2),
            };

            // Draw solid foreground inside with 2.0 pixels padding
            draw_filled_rect(
                &mut gizmos,
                fg_center,
                Vec2::new(fg_width, bar_height - 2.0),
                color,
            );
        }
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

fn despawn_gameplay_entities(
    mut commands: Commands,
    players_q: Query<Entity, With<Player>>,
    projectiles_q: Query<Entity, With<crate::physics::weapon::Projectile>>,
    particles_q: Query<Entity, With<crate::physics::particles::Particle>>,
) {
    for entity in players_q.iter() {
        commands.entity(entity).despawn();
    }
    for entity in projectiles_q.iter() {
        commands.entity(entity).despawn();
    }
    for entity in particles_q.iter() {
        commands.entity(entity).despawn();
    }
}

