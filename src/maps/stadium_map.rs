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
}
