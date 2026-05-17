use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_tectonic_fissure(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.2, 0.2, 0.2); // Basalt Charcoal Grey
    let step_color = Color::srgb(0.3, 0.3, 0.3); // Obsidian Slabs
    let pillar_color = Color::srgb(0.85, 0.35, 0.1); // Magma Orange Spire

    // 1. Two giant outer volcanic columns (Extreme high guards)
    spawn_platform(commands, meshes, materials, Vec2::new(250.0, 700.0), Vec3::new(-1300.0, -100.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(250.0, 700.0), Vec3::new(1300.0, -100.0, 5.0), platform_color);

    // 2. Basalt floor columns separated by wide ground fissures
    let column_size = Vec2::new(250.0, 100.0);
    spawn_platform(commands, meshes, materials, column_size, Vec3::new(-950.0, -450.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, column_size, Vec3::new(-500.0, -450.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, column_size, Vec3::new(500.0, -450.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, column_size, Vec3::new(950.0, -450.0, 5.0), platform_color);

    // 3. Central Magma venting core (Tall vertical leap pillar)
    spawn_platform(commands, meshes, materials, Vec2::new(60.0, 450.0), Vec3::new(0.0, -250.0, 5.0), pillar_color);

    // 4. Staggered sky obsidian steps
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 30.0), Vec3::new(-450.0, 0.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 30.0), Vec3::new(450.0, 0.0, 5.0), step_color);

    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 35.0), Vec3::new(0.0, 250.0, 5.0), step_color);
}
