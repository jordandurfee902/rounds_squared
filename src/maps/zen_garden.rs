use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_zen_garden(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.85, 0.55, 0.65); // Sakura Blossom Pink
    let step_color = Color::srgb(0.75, 0.45, 0.55); // Pale Blossom
    let pillar_color = Color::srgb(0.42, 0.15, 0.15); // Pagoda Mahogany Wood

    // 1. Central Pagoda Structure (Wall columns and multi-tiered horizontal roofs)
    spawn_platform(commands, meshes, materials, Vec2::new(50.0, 450.0), Vec3::new(-200.0, -100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(50.0, 450.0), Vec3::new(200.0, -100.0, 5.0), pillar_color);

    // Pagoda Roof Tier 1 (Low interior roof ceiling)
    spawn_platform(commands, meshes, materials, Vec2::new(550.0, 40.0), Vec3::new(0.0, 150.0, 5.0), platform_color);
    // Pagoda Roof Tier 2 (High crown pagoda arch)
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 35.0), Vec3::new(0.0, 400.0, 5.0), platform_color);

    // 2. Symmetric outer zen pond stepping stones
    spawn_platform(commands, meshes, materials, Vec2::new(350.0, 40.0), Vec3::new(-1100.0, -200.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(350.0, 40.0), Vec3::new(1100.0, -200.0, 5.0), step_color);

    // 3. Staggered garden terraces (Mid sky shelves)
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(-700.0, 100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(700.0, 100.0, 5.0), pillar_color);
}
