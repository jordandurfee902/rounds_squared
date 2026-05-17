use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_pillars_map(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.2, 0.45, 0.45);
    let step_color = Color::srgb(0.25, 0.5, 0.5);
    let pillar_color = Color::srgb(0.15, 0.35, 0.35);

    // 1. Two main floor foundations separated by a central gap/hazard drop!
    spawn_platform(commands, meshes, materials, Vec2::new(550.0, 50.0), Vec3::new(-600.0, -450.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(550.0, 50.0), Vec3::new(600.0, -450.0, 5.0), platform_color);

    // 2. Central giant vertical pillars for wall-jumping out of the chasm
    spawn_platform(commands, meshes, materials, Vec2::new(60.0, 600.0), Vec3::new(-160.0, -180.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(60.0, 600.0), Vec3::new(160.0, -180.0, 5.0), pillar_color);

    // 3. Floating platforms above the floors
    spawn_platform(commands, meshes, materials, Vec2::new(350.0, 30.0), Vec3::new(-650.0, -100.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(350.0, 30.0), Vec3::new(650.0, -100.0, 5.0), step_color);

    // 4. Central high bridges connecting the pillars to the wings
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 40.0), Vec3::new(-450.0, 200.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 40.0), Vec3::new(450.0, 200.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(0.0, 400.0, 5.0), step_color);

    // 5. Far left and far right outer vertical leap walls
    spawn_platform(commands, meshes, materials, Vec2::new(40.0, 400.0), Vec3::new(-1200.0, 100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(40.0, 400.0), Vec3::new(1200.0, 100.0, 5.0), pillar_color);

    // 6. Sky islands
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 35.0), Vec3::new(-700.0, 600.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 35.0), Vec3::new(700.0, 600.0, 5.0), platform_color);
}
