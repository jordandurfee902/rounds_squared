use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_gridlock(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.85, 0.2, 0.6); // Neon Magenta
    let step_color = Color::srgb(0.7, 0.15, 0.5); // Hot Pink
    let pillar_color = Color::srgb(0.35, 0.1, 0.5); // Deep Cyber Violet

    // 1. Central grid layers (Low platform & High platform)
    spawn_platform(commands, meshes, materials, Vec2::new(600.0, 40.0), Vec3::new(0.0, -300.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(350.0, 40.0), Vec3::new(0.0, 200.0, 5.0), platform_color);

    // 2. High flanking neon step blocks
    let cube_size = Vec2::new(120.0, 120.0);
    spawn_platform(commands, meshes, materials, cube_size, Vec3::new(-800.0, 50.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, cube_size, Vec3::new(800.0, 50.0, 5.0), pillar_color);

    // 3. Staggered vaporwave platforms for parkour climbs
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(-450.0, -50.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(450.0, -50.0, 5.0), step_color);

    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(-450.0, 450.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(450.0, 450.0, 5.0), step_color);

    // 4. Outer boundary vertical bumper walls
    spawn_platform(commands, meshes, materials, Vec2::new(50.0, 500.0), Vec3::new(-1250.0, 100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(50.0, 500.0), Vec3::new(1250.0, 100.0, 5.0), pillar_color);
}
