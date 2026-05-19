use bevy::prelude::*;
use crate::settings::PlayerStats;
use crate::physics::weapon::Projectile;
use super::Card;

pub struct GravityVortex;

#[derive(Component, Debug, Clone)]
pub struct GravityWell {
    pub strength: f32,
    pub radius: f32,
    pub lifetime: f32,
}

impl Card for GravityVortex {
    fn name(&self) -> &'static str {
        "Gravity Vortex"
    }

    fn desc(&self) -> &'static str {
        "Bullets spawn a gravity well on\nimpact that pulls players in."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "Adds Gravity Pull on Land",
            "-20% Bullet Speed",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.bullet_speed *= 0.80;
    }

    fn on_bullet_land(&self, commands: &mut Commands, _proj: &Projectile, pos: Vec2) {
        commands.spawn((
            GravityWell {
                strength: 320.0,
                radius: 180.0,
                lifetime: 2.5,
            },
            Transform::from_xyz(pos.x, pos.y, 5.0),
        ));
    }
}

pub fn gravity_well_system(
    mut commands: Commands,
    time: Res<Time>,
    mut wells: Query<(Entity, &Transform, &mut GravityWell)>,
    mut players: Query<(&Transform, &mut crate::physics::components::Velocity), With<crate::player::Health>>,
) {
    let dt = time.delta_secs();
    for (well_ent, well_trans, mut well) in wells.iter_mut() {
        well.lifetime -= dt;
        if well.lifetime <= 0.0 {
            commands.entity(well_ent).despawn();
            continue;
        }

        let well_pos = well_trans.translation.xy();
        for (player_trans, mut player_vel) in players.iter_mut() {
            let player_pos = player_trans.translation.xy();
            let to_well = well_pos - player_pos;
            let dist = to_well.length();
            if dist < well.radius && dist > 1.0 {
                let dir = to_well / dist;
                // Force increases as it gets closer
                let force_mult = (1.0 - dist / well.radius).clamp(0.0, 1.0);
                let accel = dir * (well.strength * force_mult);
                player_vel.0 += accel * dt;
            }
        }
    }
}

pub fn draw_gravity_wells(
    mut gizmos: Gizmos,
    time: Res<Time>,
    wells: Query<(&Transform, &GravityWell)>,
) {
    let t = time.elapsed_secs();
    for (trans, well) in wells.iter() {
        let center = trans.translation.xy();
        // Swirling vortex: draw concentric rotating circles
        for r_idx in 1..=4 {
            let radius = well.radius * (r_idx as f32 / 4.0);
            let color = Color::hsla(280.0 + (r_idx as f32 * 10.0), 0.9, 0.6, 0.4);
            gizmos.circle_2d(center, radius, color);
            
            // Add some swirling particles or dots along the circle
            let num_dots = 6;
            for d in 0..num_dots {
                let angle = (t * (5.0 - r_idx as f32) + d as f32 * (std::f32::consts::TAU / num_dots as f32)) * if r_idx % 2 == 0 { 1.0 } else { -1.0 };
                let dot_pos = center + Vec2::new(angle.cos(), angle.sin()) * radius;
                gizmos.circle_2d(dot_pos, 4.0, Color::srgb(0.9, 0.5, 1.0));
            }
        }
    }
}
