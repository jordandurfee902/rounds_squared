use bevy::prelude::*;
use crate::physics::particles::spawn_trail_particle;
use super::components::*;

pub fn draw_projectiles(
    mut commands: Commands,
    mut gizmos: Gizmos,
    projectiles: Query<(&Transform, &Projectile)>,
) {
    let mut rng_seed = 0u32;
    for (transform, proj) in projectiles.iter() {
        rng_seed += 1;
        let scale = proj.player_scale;
        let bullet_radius = proj.damage.sqrt() * proj.size_multiplier * scale;
        let curr_pos = transform.translation.xy();

        let forward = if proj.velocity.length_squared() > 1e-4 {
            proj.velocity.normalize()
        } else {
            Vec2::X
        };
        let right = Vec2::new(-forward.y, forward.x);

        // Helper to draw a clean teardrop vector outline
        let draw_teardrop_outline = |gizmos: &mut Gizmos, center: Vec2, r: f32, color: Color| {
            let tail_tip = center - forward * (r * 2.0);
            gizmos.line_2d(center + right * r, tail_tip, color);
            gizmos.line_2d(center - right * r, tail_tip, color);
            
            let segments = 16;
            for i in 0..segments {
                let a1 = -90.0 + (i as f32 / segments as f32) * 180.0;
                let a2 = -90.0 + ((i + 1) as f32 / segments as f32) * 180.0;
                let r1 = a1.to_radians();
                let r2 = a2.to_radians();
                let p1 = center + (right * r1.cos() + forward * r1.sin()) * r;
                let p2 = center + (right * r2.cos() + forward * r2.sin()) * r;
                gizmos.line_2d(p1, p2, color);
            }
        };

        // Solid bright yellow base bullet body
        let bullet_yellow = Color::srgb(1.0, 0.88, 0.0);

        // Fill shape solid by nesting concentric outlines
        let mut r = 0.5;
        while r <= bullet_radius {
            draw_teardrop_outline(&mut gizmos, curr_pos, r, bullet_yellow);
            gizmos.circle_2d(curr_pos, r, bullet_yellow);
            r += 1.0;
        }
        // Final crisp clean outer edge of the core
        draw_teardrop_outline(&mut gizmos, curr_pos, bullet_radius, bullet_yellow);

        // Layer dynamic glowing borders based on damage
        if proj.damage >= 70.0 {
            // Extreme/Cosmic stage: dual outer glow borders (white-hot inner, celestial magenta/violet outer)
            draw_teardrop_outline(&mut gizmos, curr_pos, bullet_radius + 3.0, Color::WHITE);
            draw_teardrop_outline(&mut gizmos, curr_pos, bullet_radius + 6.0, Color::srgb(0.9, 0.1, 1.0));
        } else if proj.damage >= 30.0 {
            // Medium damage stage: themed glow border for player identity
            let glow_color = proj.owner.color();
            draw_teardrop_outline(&mut gizmos, curr_pos, bullet_radius + 3.5, glow_color);
        }

        // Special Effects overlay: Poison Cloud green outline
        if proj.special_effects.contains(&"PoisonCloud".to_string()) {
            draw_teardrop_outline(&mut gizmos, curr_pos, bullet_radius + 4.5, Color::srgb(0.2, 0.9, 0.2));
        }

        // Spawn trail particles
        spawn_trail_particle(
            &mut commands,
            curr_pos,
            bullet_yellow,
            proj.damage,
            proj.velocity,
            rng_seed * 200,
        );
    }
}
