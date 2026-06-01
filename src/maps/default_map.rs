use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_default_map(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.25, 0.28, 0.32);
    let step_color = Color::srgb(0.30, 0.45, 0.50);
    let pillar_color = Color::srgb(0.20, 0.22, 0.25);
    let accent_color = Color::srgb(0.55, 0.35, 0.15);

    // --- SPAWN PLATFORMS (players land directly on these) ---
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(120.0, 25.0), Vec3::new(-700.0, -100.0, 5.0), step_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(120.0, 25.0), Vec3::new(700.0, -100.0, 5.0), step_color,
    );

    // --- BOTTOM CENTER ---
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(220.0, 25.0), Vec3::new(0.0, -500.0, 5.0), platform_color,
    );

    // --- LEFT WING (connects spawn area to center) ---
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(120.0, 25.0), Vec3::new(-1200.0, -350.0, 5.0), platform_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(100.0, 22.0), Vec3::new(-1000.0, -250.0, 5.0), step_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(80.0, 20.0), Vec3::new(-750.0, -170.0, 5.0), step_color,
    );

    // --- RIGHT WING (connects spawn area to center, mirrored) ---
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(120.0, 25.0), Vec3::new(1200.0, -350.0, 5.0), platform_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(100.0, 22.0), Vec3::new(1000.0, -250.0, 5.0), step_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(80.0, 20.0), Vec3::new(750.0, -170.0, 5.0), step_color,
    );

    // --- INNER STAIRCASE (center approaches) ---
    let step_size = Vec2::new(70.0, 18.0);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(-450.0, -400.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(-320.0, -310.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(-190.0, -220.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(450.0, -400.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(320.0, -310.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, step_size, Vec3::new(190.0, -220.0, 5.0), step_color);

    // --- MID TIER ---
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(180.0, 22.0), Vec3::new(0.0, -80.0, 5.0), platform_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(100.0, 20.0), Vec3::new(-600.0, -40.0, 5.0), step_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(100.0, 20.0), Vec3::new(600.0, -40.0, 5.0), step_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(70.0, 18.0), Vec3::new(-300.0, 0.0, 5.0), step_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(70.0, 18.0), Vec3::new(300.0, 0.0, 5.0), step_color,
    );

    // --- UPPER TIER ---
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(120.0, 20.0), Vec3::new(0.0, 160.0, 5.0), platform_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(110.0, 20.0), Vec3::new(-600.0, 200.0, 5.0), platform_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(110.0, 20.0), Vec3::new(600.0, 200.0, 5.0), platform_color,
    );

    // --- SKY TIER ---
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(90.0, 16.0), Vec3::new(-1000.0, 380.0, 5.0), platform_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(90.0, 16.0), Vec3::new(1000.0, 380.0, 5.0), platform_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(80.0, 16.0), Vec3::new(-1300.0, 480.0, 5.0), platform_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(80.0, 16.0), Vec3::new(1300.0, 480.0, 5.0), platform_color,
    );

    // --- WALL PILLARS ---
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(22.0, 300.0), Vec3::new(-1200.0, -50.0, 5.0), pillar_color,
    );
    spawn_platform(
        commands, meshes, materials,
        Vec2::new(22.0, 300.0), Vec3::new(1200.0, -50.0, 5.0), pillar_color,
    );

    // --- MOVING PLATFORMS ---

    // Horizontal sweeper across center
    let mp1_size = Vec2::new(110.0, 20.0) * 2.0;
    let mp1_initial = Vec2::new(0.0, 30.0) * 2.0;
    commands.spawn((
        crate::physics::components::Platform,
        crate::physics::components::Collider::Rect { size: mp1_size },
        Mesh2d(meshes.add(Rectangle::new(mp1_size.x, mp1_size.y))),
        MeshMaterial2d(materials.add(accent_color)),
        Transform::from_translation(mp1_initial.extend(5.0)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        crate::physics::components::MovingPlatform {
            id: 1,
            initial_pos: mp1_initial,
            amplitude: Vec2::new(600.0, 0.0) * 2.0,
            frequency: Vec2::new(0.7, 0.0),
            spin_speed: 0.0,
            current_rotation: 0.0,
        },
    ));

    // Vertical lifter L
    let mp2_size = Vec2::new(80.0, 18.0) * 2.0;
    let mp2_initial = Vec2::new(-900.0, 60.0) * 2.0;
    commands.spawn((
        crate::physics::components::Platform,
        crate::physics::components::Collider::Rect { size: mp2_size },
        Mesh2d(meshes.add(Rectangle::new(mp2_size.x, mp2_size.y))),
        MeshMaterial2d(materials.add(accent_color)),
        Transform::from_translation(mp2_initial.extend(5.0)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        crate::physics::components::MovingPlatform {
            id: 2,
            initial_pos: mp2_initial,
            amplitude: Vec2::new(0.0, 250.0) * 2.0,
            frequency: Vec2::new(0.0, 0.5),
            spin_speed: 0.0,
            current_rotation: 0.0,
        },
    ));

    // Vertical lifter R
    let mp3_size = Vec2::new(80.0, 18.0) * 2.0;
    let mp3_initial = Vec2::new(900.0, 60.0) * 2.0;
    commands.spawn((
        crate::physics::components::Platform,
        crate::physics::components::Collider::Rect { size: mp3_size },
        Mesh2d(meshes.add(Rectangle::new(mp3_size.x, mp3_size.y))),
        MeshMaterial2d(materials.add(accent_color)),
        Transform::from_translation(mp3_initial.extend(5.0)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        crate::physics::components::MovingPlatform {
            id: 3,
            initial_pos: mp3_initial,
            amplitude: Vec2::new(0.0, 250.0) * 2.0,
            frequency: Vec2::new(0.0, 0.5),
            spin_speed: 0.0,
            current_rotation: 0.0,
        },
    ));

    // --- ROPE SWINGS ---

    // Left rope swing weight
    let bob_radius = 20.0 * 2.0;
    let left_anchor = Vec2::new(-500.0, 800.0) * 2.0;
    let left_bob = Vec2::new(-600.0, 500.0) * 2.0;
    commands.spawn((
        crate::physics::components::Collider::Circle { radius: bob_radius },
        Mesh2d(meshes.add(Circle::new(bob_radius))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.25, 0.25))),
        Transform::from_translation(left_bob.extend(10.0)),
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
            anchor: left_anchor,
            length: 260.0 * 2.0,
        },
    ));

    // Right rope swing weight
    let right_anchor = Vec2::new(500.0, 800.0) * 2.0;
    let right_bob = Vec2::new(600.0, 500.0) * 2.0;
    commands.spawn((
        crate::physics::components::Collider::Circle { radius: bob_radius },
        Mesh2d(meshes.add(Circle::new(bob_radius))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.25, 0.25))),
        Transform::from_translation(right_bob.extend(10.0)),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        crate::physics::components::Velocity(Vec2::ZERO),
        crate::physics::components::Mass(2.0),
        crate::physics::components::PhysicsObject {
            id: 101,
            obj_type: crate::physics::components::PhysicsObjectType::SwingWeight,
            health: 100.0,
            max_health: 100.0,
        },
        crate::physics::components::RopeSwing {
            anchor: right_anchor,
            length: 260.0 * 2.0,
        },
    ));

    // --- PHYSICS OBJECTS ---

    // Center crate stack (3 boxes on center floor)
    let box_size = Vec2::new(50.0, 50.0) * 2.0;
    for i in 0..3 {
        let y_pos = -1000.0 + 25.0 + (i as f32 + 0.5) * box_size.y + 10.0;
        let box_pos = Vec2::new(0.0, y_pos);
        commands.spawn((
            crate::physics::components::Collider::Rect { size: box_size },
            Mesh2d(meshes.add(Rectangle::new(box_size.x, box_size.y))),
            MeshMaterial2d(materials.add(Color::srgb(0.65, 0.45, 0.25))),
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

    // Left crate stack (2 boxes on mid L platform)
    let small_box = Vec2::new(45.0, 45.0) * 2.0;
    for i in 0..2 {
        let y_pos = -80.0 + 20.0 + (i as f32 + 0.5) * small_box.y + 8.0;
        let box_pos = Vec2::new(-600.0, y_pos);
        commands.spawn((
            crate::physics::components::Collider::Rect { size: small_box },
            Mesh2d(meshes.add(Rectangle::new(small_box.x, small_box.y))),
            MeshMaterial2d(materials.add(Color::srgb(0.6, 0.4, 0.2))),
            Transform::from_translation(box_pos.extend(10.0)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            crate::physics::components::Velocity(Vec2::ZERO),
            crate::physics::components::Mass(0.8),
            crate::physics::components::PhysicsObject {
                id: 203 + i,
                obj_type: crate::physics::components::PhysicsObjectType::StackableBox,
                health: 80.0,
                max_health: 80.0,
            },
        ));
    }

    // Right crate stack (2 boxes on mid R platform)
    for i in 0..2 {
        let y_pos = -80.0 + 20.0 + (i as f32 + 0.5) * small_box.y + 8.0;
        let box_pos = Vec2::new(600.0, y_pos);
        commands.spawn((
            crate::physics::components::Collider::Rect { size: small_box },
            Mesh2d(meshes.add(Rectangle::new(small_box.x, small_box.y))),
            MeshMaterial2d(materials.add(Color::srgb(0.6, 0.4, 0.2))),
            Transform::from_translation(box_pos.extend(10.0)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            crate::physics::components::Velocity(Vec2::ZERO),
            crate::physics::components::Mass(0.8),
            crate::physics::components::PhysicsObject {
                id: 205 + i,
                obj_type: crate::physics::components::PhysicsObjectType::StackableBox,
                health: 80.0,
                max_health: 80.0,
            },
        ));
    }

    // Hollow square destructible barriers (mid tier flanking)
    let hs_size = Vec2::new(65.0, 65.0) * 2.0;
    for (i, x) in [-150.0, 150.0].iter().enumerate() {
        commands.spawn((
            crate::physics::components::Collider::Rect { size: hs_size },
            Mesh2d(meshes.add(Rectangle::new(hs_size.x, hs_size.y))),
            MeshMaterial2d(materials.add(Color::srgba(0.8, 0.4, 0.1, 0.3))),
            Transform::from_translation(Vec3::new(x * 2.0, -80.0 * 2.0, 10.0)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            crate::physics::components::Velocity(Vec2::ZERO),
            crate::physics::components::Mass(3.0),
            crate::physics::components::PhysicsObject {
                id: 300 + i as u32,
                obj_type: crate::physics::components::PhysicsObjectType::HollowSquare,
                health: 150.0,
                max_health: 150.0,
            },
        ));
    }
}
