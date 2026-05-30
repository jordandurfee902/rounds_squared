use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_stadium_map(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.5, 0.25, 0.25);
    let step_color = Color::srgb(0.55, 0.3, 0.3);
    let pillar_color = Color::srgb(0.4, 0.18, 0.18);

    // 1. Massive central arena platform
    spawn_platform(commands, meshes, materials, Vec2::new(950.0, 60.0), Vec3::new(0.0, -320.0, 5.0), platform_color);

    // 2. High central floating platform (The "Crown")
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 40.0), Vec3::new(0.0, 300.0, 5.0), step_color);

    // 3. Left and Right soaring traversable vertical towers
    spawn_platform(commands, meshes, materials, Vec2::new(70.0, 700.0), Vec3::new(-850.0, 0.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(70.0, 700.0), Vec3::new(850.0, 0.0, 5.0), pillar_color);

    // 4. Floating step ledges extending from the towers
    spawn_platform(commands, meshes, materials, Vec2::new(250.0, 30.0), Vec3::new(-600.0, 50.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(250.0, 30.0), Vec3::new(600.0, 50.0, 5.0), step_color);

    // 5. Very high sky wing platforms
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 35.0), Vec3::new(-1100.0, 550.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 35.0), Vec3::new(1100.0, 550.0, 5.0), platform_color);

    // --- Overhaul Map Additions: Destructible Hollow Squares ---
    let box_size = Vec2::new(50.0, 50.0) * 2.0;

    let x_offsets = [-300.0, 300.0];
    let mut id_counter = 300;
    for &x in x_offsets.iter() {
        for i in 0..2 {
            let y_pos = -580.0 + (i as f32 + 0.5) * box_size.y + 10.0;
            let box_pos = Vec2::new(x, y_pos);
            commands.spawn((
                crate::physics::components::Collider::Rect { size: box_size },
                Mesh2d(meshes.add(Rectangle::new(box_size.x, box_size.y))),
                MeshMaterial2d(materials.add(Color::srgba(1.0, 0.5, 0.2, 0.05))),
                Transform::from_translation(box_pos.extend(10.0)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                crate::physics::components::Velocity(Vec2::ZERO),
                crate::physics::components::Mass(0.4),
                crate::physics::components::PhysicsObject {
                    id: id_counter,
                    obj_type: crate::physics::components::PhysicsObjectType::HollowSquare,
                    health: 30.0,
                    max_health: 30.0,
                },
            ));
            id_counter += 1;
        }
    }
}
