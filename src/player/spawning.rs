use bevy::prelude::*;
use crate::physics::{Collider, Velocity, Acceleration, Grounded, WallContact, JumpAllowance, Mass};
use crate::physics::weapon::Weapon;
use crate::physics::anim::ProceduralLimbs;
use crate::settings::{PersistentPlayerStats, PhysicsSettings};
use super::components::*;

pub fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    persistent_stats: Res<PersistentPlayerStats>,
    active_map: Res<crate::maps::ActiveMap>,
    physics_settings: Res<PhysicsSettings>,
) {
    let p1_stats = &persistent_stats.p1;
    let p2_stats = &persistent_stats.p2;

    let p1_scale = p1_stats.player_scale;
    let p2_scale = p2_stats.player_scale;

    let (p1_spawn, p2_spawn) = match *active_map {
        crate::maps::ActiveMap::DefaultMap => {
            (Vec3::new(-1350.0, 100.0, 10.0), Vec3::new(1350.0, 100.0, 10.0))
        }
        crate::maps::ActiveMap::PillarsMap => {
            (Vec3::new(-650.0, 150.0, 10.0), Vec3::new(650.0, 150.0, 10.0))
        }
        crate::maps::ActiveMap::StadiumMap => {
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

    // Player 1 (Blue) - Base mass from settings
    commands.spawn((
        Player::P1,
        Collider::Circle { radius: physics_settings.player_base_radius * p1_scale },
        Transform::from_translation(p1_spawn),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        Velocity(Vec2::ZERO),
        Acceleration(Vec2::ZERO),
        Grounded(true),
        WallContact::default(),
        JumpAllowance { value: physics_settings.max_jump_allowance },
    )).insert((
        crate::physics::components::ControllerInput::default(),
        Mass(physics_settings.player_base_mass),
        Health { current: p1_stats.health_max, max: p1_stats.health_max },
        crate::physics::anim::PlayerAim::default(),
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
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(physics_settings.player_base_radius * p1_scale))),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.5, 1.0))),
            Transform::from_xyz(0.0, physics_settings.player_visual_offset * p1_scale, 0.0),
        ));
    });

    // Player 2 (Orange) - Base mass from settings
    commands.spawn((
        Player::P2,
        Collider::Circle { radius: physics_settings.player_base_radius * p2_scale },
        Transform::from_translation(p2_spawn),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        Velocity(Vec2::ZERO),
        Acceleration(Vec2::ZERO),
        Grounded(true),
        WallContact::default(),
        JumpAllowance { value: physics_settings.max_jump_allowance },
    )).insert((
        crate::physics::components::ControllerInput::default(),
        Mass(physics_settings.player_base_mass),
        Health { current: p2_stats.health_max, max: p2_stats.health_max },
        crate::physics::anim::PlayerAim::default(),
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
        parent.spawn((
            Mesh2d(meshes.add(Circle::new(physics_settings.player_base_radius * p2_scale))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.2))),
            Transform::from_xyz(0.0, physics_settings.player_visual_offset * p2_scale, 0.0),
        ));
    });
}

pub fn despawn_gameplay_entities(
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
