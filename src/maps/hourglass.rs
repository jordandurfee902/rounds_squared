use bevy::prelude::*;
use crate::map::spawn_platform;

pub fn spawn_hourglass(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let platform_color = Color::srgb(0.82, 0.68, 0.45); // Desert Sand Beige
    let step_color = Color::srgb(0.72, 0.58, 0.35); // Sunbaked Stone
    let pillar_color = Color::srgb(0.68, 0.35, 0.2); // Terracotta Clay

    // 1. Double hourglass massive columns in the middle (Funnel corridor)
    spawn_platform(commands, meshes, materials, Vec2::new(100.0, 450.0), Vec3::new(-350.0, -100.0, 5.0), pillar_color);
    spawn_platform(commands, meshes, materials, Vec2::new(100.0, 450.0), Vec3::new(350.0, -100.0, 5.0), pillar_color);

    // 2. High central keystone bridge in the middle choke
    spawn_platform(commands, meshes, materials, Vec2::new(250.0, 35.0), Vec3::new(0.0, 180.0, 5.0), platform_color);

    // 3. Side chambers (Safe outer spawn zones)
    spawn_platform(commands, meshes, materials, Vec2::new(450.0, 40.0), Vec3::new(-1100.0, -250.0, 5.0), platform_color);
    spawn_platform(commands, meshes, materials, Vec2::new(450.0, 40.0), Vec3::new(1100.0, -250.0, 5.0), platform_color);

    // 4. Staggered sandstone ascending steps
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(-850.0, 50.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(200.0, 30.0), Vec3::new(850.0, 50.0, 5.0), step_color);

    // 5. Sky shelter ledges
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 30.0), Vec3::new(-1100.0, 400.0, 5.0), step_color);
    spawn_platform(commands, meshes, materials, Vec2::new(300.0, 30.0), Vec3::new(1100.0, 400.0, 5.0), step_color);
}
