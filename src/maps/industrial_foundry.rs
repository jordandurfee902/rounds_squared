use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_industrial_foundry(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.4, 0.48, 0.52); // Steel Slate Grey
    let step_color = Color::srgb(0.32, 0.38, 0.42); // Iron Girder
    let pillar_color = Color::srgb(0.72, 0.38, 0.15); // Rusty Amber Piston

    // 1. Double outer machinery towers (Sturdy spawn towers)
    spawn_platform(commands, meshes, materials, Vec2::new(350.0, 500.0), Vec3::new(-1200.0, -100.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(350.0, 500.0), Vec3::new(1200.0, -100.0, 5.0), platform_color);

    // 2. Giant central mechanical piston (Heavy wall-jump block)
    spawn_platform(commands, meshes, materials, Vec2::new(160.0, 400.0), Vec3::new(0.0, -150.0, 5.0), pillar_color);

    // 3. Floating heavy girders above the piston
    spawn_platform(commands, meshes, materials, Vec2::new(500.0, 40.0), Vec3::new(0.0, 300.0, 5.0), step_color);

    // 4. Staggered steel steps bridging the gap from towers to middle
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(-650.0, 100.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(650.0, 100.0, 5.0), step_color);

    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(-650.0, -250.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(650.0, -250.0, 5.0), step_color);
}
