use bevy::prelude::*;
use crate::physics::*;
use crate::physics::weapon::Weapon;
use crate::settings::{PersistentPlayerStats, GameState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Gameplay), spawn_players.after(crate::map::spawn_platforms))
           .add_systems(Update, (
               player_input.before(crate::physics::forces::apply_gravity_and_movement),
               player_block_system,
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

#[derive(Component, Debug, Clone)]
pub struct PlayerStatsComponent {
    pub movement_speed: f32,
    pub jump_force: f32,
    pub player_scale: f32,
    pub health_max: f32,
    pub block_duration: f32,
    pub block_cooldown: f32,
    pub block_border_boost: f32,
}

#[derive(Component, Debug, Clone)]
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
        // Spawn visual body offset upwards by 25px to float perfectly on legs!
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(40.0 * p1_scale))),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.5, 1.0))),
            Transform::from_xyz(0.0, 25.0 * p1_scale, 0.0),
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
        // Spawn visual body offset upwards by 25px to float perfectly on legs!
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(40.0 * p2_scale))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.2))),
            Transform::from_xyz(0.0, 25.0 * p2_scale, 0.0),
        ));
    });
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut ControllerInput)>,
    gamepads: Query<&Gamepad>,
) {
    let gamepad = gamepads.iter().next();

    for (player, mut input) in query.iter_mut() {
        let mut move_dir = 0.0;
        let mut jump = false;
        let mut fast_fall = false;

        match player {
            Player::P1 => {
                if keys.pressed(KeyCode::KeyA) { move_dir -= 1.0; }
                if keys.pressed(KeyCode::KeyD) { move_dir += 1.0; }
                if keys.just_pressed(KeyCode::KeyW) { jump = true; }
                if keys.pressed(KeyCode::KeyS) { fast_fall = true; }
            }
            Player::P2 => {
                if let Some(gp) = gamepad {
                    let stick = gp.left_stick();
                    move_dir = stick.x;
                    if gp.just_pressed(GamepadButton::South) {
                        jump = true;
                    }
                    if stick.y < -0.5 {
                        fast_fall = true;
                    }
                } else {
                    if keys.pressed(KeyCode::ArrowLeft) { move_dir -= 1.0; }
                    if keys.pressed(KeyCode::ArrowRight) { move_dir += 1.0; }
                    if keys.just_pressed(KeyCode::ArrowUp) { jump = true; }
                    if keys.pressed(KeyCode::ArrowDown) { fast_fall = true; }
                }
            }
        }

        input.move_dir = move_dir;
        input.jump = jump;
        input.fast_fall = fast_fall;
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
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    mut query: Query<(&Player, &mut BlockComponent)>,
) {
    let dt = time.delta_secs().min(0.05);
    let gamepad = gamepads.iter().next();

    for (player, mut block) in query.iter_mut() {
        if block.active_timer > 0.0 {
            block.active_timer = (block.active_timer - dt).max(0.0);
        }
        if block.cooldown_timer > 0.0 {
            block.cooldown_timer = (block.cooldown_timer - dt).max(0.0);
        }
        if block.control_lockout_timer > 0.0 {
            block.control_lockout_timer = (block.control_lockout_timer - dt).max(0.0);
        }

        let block_pressed = match player {
            Player::P1 => mouse.just_pressed(MouseButton::Right),
            Player::P2 => {
                if let Some(gp) = gamepad {
                    gp.just_pressed(GamepadButton::LeftTrigger)
                } else {
                    keys.just_pressed(KeyCode::KeyU)
                }
            }
        };

        if block_pressed && block.cooldown_timer <= 0.0 {
            block.active_timer = block.block_duration;
            block.cooldown_timer = block.block_cooldown;
        }
    }
}

