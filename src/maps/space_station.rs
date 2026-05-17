use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_space_station(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.85, 0.88, 0.9); // Polar Silver White
    let step_color = Color::srgb(0.65, 0.68, 0.72); // Heavy Steel Gray
    let pillar_color = Color::srgb(0.3, 0.2, 0.7); // Orbital Plasma Purple

    // 1. Central engine reactor core (Huge central block)
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 160.0), Vec3::new(0.0, -250.0, 5.0), pillar_color);

    // 2. High suspended orbital observation bridge
    spawn_platform(commands, meshes, materials, Vec2::new(750.0, 35.0), Vec3::new(0.0, 280.0, 5.0), platform_color);

    // 3. Side docking stations (Spacious spawn platforms)
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 45.0), Vec3::new(-1250.0, 0.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 45.0), Vec3::new(1250.0, 0.0, 5.0), platform_color);

    // 4. Staggered vertical solar panels on sides
    spawn_platform(commands, meshes, materials, Vec2::new(40.0, 350.0), Vec3::new(-850.0, 150.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(40.0, 350.0), Vec3::new(850.0, 150.0, 5.0), step_color);
}
