use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_vertical_helix(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.4, 0.75, 0.2); // Lime Green
    let step_color = Color::srgb(0.3, 0.62, 0.15); // Staggered Leaf
    let pillar_color = Color::srgb(0.12, 0.45, 0.2); // Deep Emerald DNA Core

    // 1. Two giant side walls forming outer climbing chutes
    spawn_platform(commands, meshes, materials, Vec2::new(45.0, 750.0), Vec3::new(-1100.0, 100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(45.0, 750.0), Vec3::new(1100.0, 100.0, 5.0), pillar_color);

    // 2. Central DNA core block
    spawn_platform(commands, meshes, materials, Vec2::new(100.0, 300.0), Vec3::new(0.0, 100.0, 5.0), pillar_color);

    // 3. Staggered helical ladders (Step planks rotating upwards)
    let plank = Vec2::new(250.0, 30.0);
    // Level 1 steps (Low)
    spawn_platform(commands, meshes, materials, plank, Vec3::new(-700.0, -300.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, plank, Vec3::new(700.0, -300.0, 5.0), platform_color);

    // Level 2 steps (Mid-Low offset)
    spawn_platform(commands, meshes, materials, plank, Vec3::new(-450.0, -100.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, plank, Vec3::new(450.0, -100.0, 5.0), step_color);

    // Level 3 steps (Mid-High offset closer to center)
    spawn_platform(commands, meshes, materials, plank, Vec3::new(-250.0, 300.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, plank, Vec3::new(250.0, 300.0, 5.0), platform_color);

    // Level 4 steps (High)
    spawn_platform(commands, meshes, materials, plank, Vec3::new(-650.0, 500.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, plank, Vec3::new(650.0, 500.0, 5.0), step_color);
}
