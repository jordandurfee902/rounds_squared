use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_default_map(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.3, 0.3, 0.3);
    let step_color = Color::srgb(0.35, 0.35, 0.35);
    let pillar_color = Color::srgb(0.25, 0.25, 0.25);

    // 1. Center Foundation Platform
    spawn_platform(
        commands,
        meshes,
        materials,
        Vec2::new(800.0, 50.0),
        Vec3::new(0.0, -400.0, 5.0),
        platform_color,
    );

    // 2. Left Staircase ascending to Left Wing
    let step_size = Vec2::new(180.0, 30.0);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(-500.0, -550.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(-700.0, -450.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(-900.0, -350.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(-1100.0, -250.0, 5.0), step_color);

    // Left Ledge Wing
    spawn_platform(
        commands,
        meshes,
        materials,
        Vec2::new(600.0, 40.0),
        Vec3::new(-1450.0, -150.0, 5.0),
        platform_color,
    );

    // 3. Right Staircase ascending to Right Wing
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(500.0, -550.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(700.0, -450.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(900.0, -350.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(1100.0, -250.0, 5.0), step_color);

    // Right Ledge Wing
    spawn_platform(
        commands,
        meshes,
        materials,
        Vec2::new(600.0, 40.0),
        Vec3::new(1450.0, -150.0, 5.0),
        platform_color,
    );

    // 4. Middle Floating Islands (High ground tier)
    spawn_platform(
        commands,
        meshes,
        materials,
        Vec2::new(500.0, 40.0),
        Vec3::new(0.0, 200.0, 5.0),
        platform_color,
    );

    spawn_platform(
        commands,
        meshes,
        materials,
        Vec2::new(400.0, 40.0),
        Vec3::new(-600.0, 450.0, 5.0),
        platform_color,
    );

    spawn_platform(
        commands,
        meshes,
        materials,
        Vec2::new(400.0, 40.0),
        Vec3::new(600.0, 450.0, 5.0),
        platform_color,
    );

    // 5. Vertical Wall-Cling & Leap Pillars (Dynamic platform traversal helpers)
    let pillar_size = Vec2::new(50.0, 450.0);
    spawn_platform(
        commands,
        meshes,
        materials,
        pillar_size,
        Vec3::new(-300.0, -50.0, 5.0),
        pillar_color,
    );

    spawn_platform(
        commands,
        meshes,
        materials,
        pillar_size,
        Vec3::new(300.0, -50.0, 5.0),
        pillar_color,
    );

    // 6. Very High Sky Ledges
    spawn_platform(
        commands,
        meshes,
        materials,
        Vec2::new(500.0, 40.0),
        Vec3::new(-1100.0, 700.0, 5.0),
        platform_color,
    );

    spawn_platform(
        commands,
        meshes,
        materials,
        Vec2::new(500.0, 40.0),
        Vec3::new(1100.0, 700.0, 5.0),
        platform_color,
    );

    // --- Overhaul Map Additions: Moving Platforms & Physics Objects ---

    // 7. Horizontal Moving and Spinning Platform
    let size = Vec2::new(300.0, 40.0) * 2.0;
    let initial_pos = Vec2::new(0.0, 50.0) * 2.0;
    commands.spawn((
        crate::physics::components::Platform,
        crate::physics::components::Collider::Rect { size },
        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.2, 0.7))),
        Transform::from_translation(initial_pos.extend(5.0)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        crate::physics::components::MovingPlatform {
            id: 1,
            initial_pos,
            amplitude: Vec2::new(600.0, 0.0) * 2.0,
            frequency: Vec2::new(1.0, 0.0),
            spin_speed: 1.0,
            current_rotation: 0.0,
        },
    ));

    // 8. Rope Swing Weight (Pendulum)
    let bob_radius = 40.0 * 2.0;
    let anchor = Vec2::new(0.0, 1000.0) * 2.0;
    let bob_pos = Vec2::new(200.0, 600.0) * 2.0;
    commands.spawn((
        crate::physics::components::Collider::Circle { radius: bob_radius },
        Mesh2d(meshes.add(Circle::new(bob_radius))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.3, 0.3))),
        Transform::from_translation(bob_pos.extend(10.0)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        crate::physics::components::Velocity(Vec2::ZERO),
        crate::physics::components::Mass(2.0),
        crate::physics::components::PhysicsObject {
            id: 100,
            obj_type: crate::physics::components::PhysicsObjectType::SwingWeight,
            health: 100.0,
            max_health: 100.0,
        },
        crate::physics::components::RopeSwing {
            anchor,
            length: 400.0 * 2.0,
        },
    ));

    // 9. Stackable Crates (3 boxes stacked on the Center Foundation platform)
    let box_size = Vec2::new(60.0, 60.0) * 2.0;
    for i in 0..3 {
        let y_pos = -800.0 + 50.0 + (i as f32 + 0.5) * box_size.y + 10.0;
        let box_pos = Vec2::new(0.0, y_pos);
        commands.spawn((
            crate::physics::components::Collider::Rect { size: box_size },
            Mesh2d(meshes.add(Rectangle::new(box_size.x, box_size.y))),
            MeshMaterial2d(materials.add(Color::srgb(0.7, 0.5, 0.3))),
            Transform::from_translation(box_pos.extend(10.0)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            crate::physics::components::Velocity(Vec2::ZERO),
            crate::physics::components::Mass(1.0),
            crate::physics::components::PhysicsObject {
                id: 200 + i,
                obj_type: crate::physics::components::PhysicsObjectType::StackableBox,
                health: 100.0,
                max_health: 100.0,
            },
        ));
    }
}
