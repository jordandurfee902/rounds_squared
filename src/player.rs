use bevy::prelude::*;
use crate::physics::*;


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_players)
           .add_systems(Update, (
               player_input.before(crate::physics::forces::apply_gravity_and_movement),
               draw_health_bars,
           ));
    }
}

#[derive(Component, PartialEq)]
pub enum Player {
    P1,
    P2,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Player 1 (Blue) - Base mass = 1.0 (Normal)
    commands.spawn((
        Player::P1,
        Collider::Circle { radius: 40.0 },
        Transform::from_xyz(-1350.0, 100.0, 10.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        Velocity(Vec2::ZERO),
        Acceleration(Vec2::ZERO),
        Grounded(true),
        WallContact::default(),
    )).insert((
        ControllerInput::default(),
        Mass(1.0),
        Health { current: 100.0, max: 100.0 },
        PlayerAim::default(),
        ProceduralLimbs::default(),
    )).with_children(|parent| {
        // Spawn visual body offset upwards by 25px to float perfectly on legs!
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(40.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.5, 1.0))),
            Transform::from_xyz(0.0, 25.0, 0.0),
        ));
    });

    // Player 2 (Orange) - Base mass = 1.0 (Identical start-of-game balance)
    commands.spawn((
        Player::P2,
        Collider::Circle { radius: 40.0 },
        Transform::from_xyz(1350.0, 100.0, 10.0),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        Velocity(Vec2::ZERO),
        Acceleration(Vec2::ZERO),
        Grounded(true),
        WallContact::default(),
    )).insert((
        ControllerInput::default(),
        Mass(1.0),
        Health { current: 100.0, max: 100.0 },
        PlayerAim::default(),
        ProceduralLimbs::default(),
    )).with_children(|parent| {
        // Spawn visual body offset upwards by 25px to float perfectly on legs!
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(40.0))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.2))),
            Transform::from_xyz(0.0, 25.0, 0.0),
        ));
    });
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut ControllerInput)>,
) {
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
                if keys.pressed(KeyCode::ArrowLeft) { move_dir -= 1.0; }
                if keys.pressed(KeyCode::ArrowRight) { move_dir += 1.0; }
                if keys.just_pressed(KeyCode::ArrowUp) { jump = true; }
                if keys.pressed(KeyCode::ArrowDown) { fast_fall = true; }
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
    query: Query<(&Transform, &Health, &Player)>,
) {
    for (transform, health, player) in query.iter() {
        let player_pos = transform.translation.xy();
        // Position the health bar about 90 pixels above the parent physical center (65px above visual floating body)
        let bar_center = player_pos + Vec2::new(0.0, 90.0);
        let bar_width = 64.0;
        let bar_height = 6.0;

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

