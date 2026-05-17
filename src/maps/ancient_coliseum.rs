use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_ancient_coliseum(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.78, 0.75, 0.65); // Antique Marble
    let step_color = Color::srgb(0.68, 0.65, 0.55); // Aged Stone
    let pillar_color = Color::srgb(0.6, 0.48, 0.25); // Royal Gold Bronze

    // 1. Classical gladiator ring central pillars (Four supportive column arrays)
    spawn_platform(commands, meshes, materials, Vec2::new(60.0, 550.0), Vec3::new(-450.0, -100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(60.0, 550.0), Vec3::new(-150.0, -100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(60.0, 550.0), Vec3::new(150.0, -100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(60.0, 550.0), Vec3::new(450.0, -100.0, 5.0), pillar_color);

    // 2. High supportive Coliseum archways (Walkable platforms spanning the pillars)
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 35.0), Vec3::new(-300.0, 200.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 35.0), Vec3::new(300.0, 200.0, 5.0), platform_color);

    // 3. Central sky altar stone
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 40.0), Vec3::new(0.0, 450.0, 5.0), pillar_color);

    // 4. Staggered outer stadium wings for spacious spawning
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 45.0), Vec3::new(-950.0, 0.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(400.0, 45.0), Vec3::new(950.0, 0.0, 5.0), platform_color);

    // 5. Low ground stone slabs
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 40.0), Vec3::new(-900.0, -350.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 40.0), Vec3::new(900.0, -350.0, 5.0), step_color);
}
