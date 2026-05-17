use bevy::prelude::*;
use crate::physics::{Collider, Platform};

use crate::settings::GameState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Gameplay), spawn_platforms)
           .add_systems(OnExit(GameState::Gameplay), despawn_platforms);
    }
}

fn spawn_platform(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    size: Vec2,
    pos: Vec3,
    color: Color,
) {
    commands.spawn((
        Platform,
        Collider::Rect { size },
        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(pos),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    ));
}

fn spawn_platforms(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.3, 0.3, 0.3);
    let step_color = Color::srgb(0.35, 0.35, 0.35);
    let pillar_color = Color::srgb(0.25, 0.25, 0.25);

    // 1. Center Foundation Platform
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(800.0, 50.0),
        Vec3::new(0.0, -400.0, 5.0),
        platform_color,
    );

    // 2. Left Staircase ascending to Left Wing
    let step_size = Vec2::new(180.0, 30.0);
    spawn_platform(&mut commands, &mut meshes, &mut materials, step_size, Vec3::new(-500.0, -550.0, 5.0), step_color);
    spawn_platform(&mut commands, &mut meshes, &mut materials, step_size, Vec3::new(-700.0, -450.0, 5.0), step_color);
    spawn_platform(&mut commands, &mut meshes, &mut materials, step_size, Vec3::new(-900.0, -350.0, 5.0), step_color);
    spawn_platform(&mut commands, &mut meshes, &mut materials, step_size, Vec3::new(-1100.0, -250.0, 5.0), step_color);

    // Left Ledge Wing
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(600.0, 40.0),
        Vec3::new(-1450.0, -150.0, 5.0),
        platform_color,
    );

    // 3. Right Staircase ascending to Right Wing
    spawn_platform(&mut commands, &mut meshes, &mut materials, step_size, Vec3::new(500.0, -550.0, 5.0), step_color);
    spawn_platform(&mut commands, &mut meshes, &mut materials, step_size, Vec3::new(700.0, -450.0, 5.0), step_color);
    spawn_platform(&mut commands, &mut meshes, &mut materials, step_size, Vec3::new(900.0, -350.0, 5.0), step_color);
    spawn_platform(&mut commands, &mut meshes, &mut materials, step_size, Vec3::new(1100.0, -250.0, 5.0), step_color);

    // Right Ledge Wing
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(600.0, 40.0),
        Vec3::new(1450.0, -150.0, 5.0),
        platform_color,
    );

    // 4. Middle Floating Islands (High ground tier)
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(500.0, 40.0),
        Vec3::new(0.0, 200.0, 5.0),
        platform_color,
    );

    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(400.0, 40.0),
        Vec3::new(-600.0, 450.0, 5.0),
        platform_color,
    );

    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(400.0, 40.0),
        Vec3::new(600.0, 450.0, 5.0),
        platform_color,
    );

    // 5. Vertical Wall-Cling & Leap Pillars (Dynamic platform traversal helpers)
    let pillar_size = Vec2::new(50.0, 450.0);
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        pillar_size,
        Vec3::new(-300.0, -50.0, 5.0),
        pillar_color,
    );

    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        pillar_size,
        Vec3::new(300.0, -50.0, 5.0),
        pillar_color,
    );

    // 6. Very High Sky Ledges
    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(500.0, 40.0),
        Vec3::new(-1100.0, 700.0, 5.0),
        platform_color,
    );

    spawn_platform(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(500.0, 40.0),
        Vec3::new(1100.0, 700.0, 5.0),
        platform_color,
    );
}

fn despawn_platforms(
    mut commands: Commands,
    query: Query<Entity, With<Platform>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
