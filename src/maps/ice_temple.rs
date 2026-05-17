use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_ice_temple(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.45, 0.75, 0.85); // Frost Blue
    let step_color = Color::srgb(0.35, 0.65, 0.85); // Ice Sheet
    let pillar_color = Color::srgb(0.25, 0.5, 0.75); // Deep Crystal Indigo

    // 1. Central frozen cathedral spire extending from bottom to sky
    spawn_platform(commands, meshes, materials, Vec2::new(80.0, 750.0), Vec3::new(0.0, -100.0, 5.0), pillar_color);

    // 2. Twin soaring ice shelves
    spawn_platform(commands, meshes, materials, Vec2::new(450.0, 40.0), Vec3::new(-1000.0, 100.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(450.0, 40.0), Vec3::new(1000.0, 100.0, 5.0), platform_color);

    // 3. Diagonal step panels for wall jumping off the central spire
    let frozen_panel = Vec2::new(150.0, 30.0);
    spawn_platform(commands, meshes, materials, frozen_panel, Vec3::new(-350.0, -250.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, frozen_panel, Vec3::new(350.0, -250.0, 5.0), step_color);

    spawn_platform(commands, meshes, materials, frozen_panel, Vec3::new(-350.0, 200.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, frozen_panel, Vec3::new(350.0, 200.0, 5.0), step_color);

    // 4. Low ice floor fragments
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 35.0), Vec3::new(-700.0, -400.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 35.0), Vec3::new(700.0, -400.0, 5.0), platform_color);

    // 5. Very high sky shelves
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 30.0), Vec3::new(-600.0, 550.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 30.0), Vec3::new(600.0, 550.0, 5.0), step_color);
}
