use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_chasm_bridge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.62, 0.45, 0.2); // Oak Brown
    let step_color = Color::srgb(0.52, 0.38, 0.15); // Weathered Wood
    let pillar_color = Color::srgb(0.2, 0.45, 0.25); // Forest Green Cliffs

    // 1. Two giant outer cliffs/towers for high-ground spawns
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 600.0), Vec3::new(-1300.0, 0.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 600.0), Vec3::new(1300.0, 0.0, 5.0), pillar_color);

    // 2. Suspended bridge planks with deliberate combat gaps in the center
    let plank_size = Vec2::new(120.0, 25.0);
    spawn_platform(commands, meshes, materials, plank_size, Vec3::new(-850.0, 100.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, plank_size, Vec3::new(-650.0, 50.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, plank_size, Vec3::new(-450.0, 0.0, 5.0), step_color);
    // Gap in middle of bridge!
    spawn_platform(commands, meshes, materials, plank_size, Vec3::new(450.0, 0.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, plank_size, Vec3::new(650.0, 50.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, plank_size, Vec3::new(850.0, 100.0, 5.0), step_color);

    // 3. Central floating island above the chasm gap
    spawn_platform(commands, meshes, materials, Vec2::new(350.0, 35.0), Vec3::new(0.0, 250.0, 5.0), platform_color);

    // 4. Lower safety platforms with a wide center pitfall
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 40.0), Vec3::new(-500.0, -350.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 40.0), Vec3::new(500.0, -350.0, 5.0), pillar_color);
}
