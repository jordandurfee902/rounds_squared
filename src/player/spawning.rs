use bevy::prelude::*;
use crate::physics::{Collider, Velocity, Acceleration, Grounded, WallContact, JumpAllowance, Mass, StandingOn};
use crate::physics::weapon::Weapon;
use crate::physics::anim::ProceduralLimbs;
use crate::settings::{PersistentPlayerStats, PhysicsSettings, LobbySlots};
use super::components::*;

pub fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    persistent_stats: Res<PersistentPlayerStats>,
    active_map: Res<crate::maps::ActiveMap>,
    physics_settings: Res<PhysicsSettings>,
    lobby_slots: Res<LobbySlots>,
) {
    let base_x = match *active_map {
        crate::maps::ActiveMap::DefaultMap => 1350.0,
        crate::maps::ActiveMap::PillarsMap => 650.0,
        crate::maps::ActiveMap::StadiumMap => 350.0,
        crate::maps::ActiveMap::ChasmBridge => 1300.0,
        crate::maps::ActiveMap::Gridlock => 800.0,
        crate::maps::ActiveMap::Hourglass => 1100.0,
        crate::maps::ActiveMap::IceTemple => 1000.0,
        crate::maps::ActiveMap::IndustrialFoundry => 1200.0,
        crate::maps::ActiveMap::VerticalHelix => 700.0,
        crate::maps::ActiveMap::TectonicFissure => 950.0,
        crate::maps::ActiveMap::ZenGarden => 1100.0,
        crate::maps::ActiveMap::SpaceStation => 1250.0,
        crate::maps::ActiveMap::AncientColiseum => 800.0,
    };
    let base_y = match *active_map {
        crate::maps::ActiveMap::DefaultMap => 100.0,
        crate::maps::ActiveMap::PillarsMap => 150.0,
        crate::maps::ActiveMap::StadiumMap => -100.0,
        crate::maps::ActiveMap::ChasmBridge => 300.0,
        crate::maps::ActiveMap::Gridlock => 200.0,
        crate::maps::ActiveMap::Hourglass => -100.0,
        crate::maps::ActiveMap::IceTemple => 250.0,
        crate::maps::ActiveMap::IndustrialFoundry => 200.0,
        crate::maps::ActiveMap::VerticalHelix => -300.0,
        crate::maps::ActiveMap::TectonicFissure => -350.0,
        crate::maps::ActiveMap::ZenGarden => -100.0,
        crate::maps::ActiveMap::SpaceStation => 100.0,
        crate::maps::ActiveMap::AncientColiseum => 350.0,
    };

    let mut active_indices = Vec::new();
    for i in 0..8 {
        if lobby_slots.slots[i].is_some() {
            active_indices.push(i);
        }
    }
    if active_indices.is_empty() {
        active_indices.push(0);
        active_indices.push(1);
    }

    for &i in active_indices.iter() {
        let player = Player::from_index(i);
        let stats = &persistent_stats.players[i];
        let scale = stats.player_scale;

        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let step = (i / 2) as f32;
        let spawn_x = side * (base_x * 2.0 - step * 150.0 * 2.0);
        let spawn_y = base_y * 2.0 + step * 50.0 * 2.0;
        let spawn_pos = Vec3::new(spawn_x, spawn_y, 10.0);

        let p_color = player.color();

        commands.spawn((
            player,
            Collider::Circle { radius: physics_settings.player_base_radius * scale },
            Transform::from_translation(spawn_pos),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            Velocity(Vec2::ZERO),
            Acceleration(Vec2::ZERO),
            Grounded(true),
            WallContact::default(),
            JumpAllowance { value: physics_settings.max_jump_allowance },
            StandingOn(None),
        )).insert((
            crate::physics::components::ControllerInput::default(),
            Mass(physics_settings.player_base_mass),
            Health { current: stats.health_max, max: stats.health_max },
            crate::physics::anim::PlayerAim::default(),
            ProceduralLimbs::default(),
            PlayerStatsComponent {
                movement_speed: stats.movement_speed,
                jump_force: stats.jump_force,
                player_scale: stats.player_scale,
                health_max: stats.health_max,
                block_duration: stats.block_duration,
                block_cooldown: stats.block_cooldown,
                block_border_boost: stats.block_border_boost,
            },
            BlockComponent {
                active_timer: 0.0,
                cooldown_timer: 0.0,
                block_duration: stats.block_duration,
                block_cooldown: stats.block_cooldown,
                control_lockout_timer: 0.0,
            },
            Weapon {
                max_ammo: stats.max_ammo,
                current_ammo: stats.max_ammo,
                fire_cooldown: 0.0,
                fire_rate: stats.fire_rate,
                reload_timer: 0.0,
                reload_time: stats.reload_time,
                time_since_last_shot: 0.0,
            },
        )).with_children(|parent| {
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(physics_settings.player_base_radius * scale))),
                MeshMaterial2d(materials.add(p_color)),
                Transform::from_xyz(0.0, physics_settings.player_visual_offset * scale, 0.0),
            ));
        });
    }
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
