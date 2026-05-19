use bevy::prelude::*;
use super::components::*;

fn draw_filled_rect(
    gizmos: &mut Gizmos,
    center: Vec2,
    size: Vec2,
    color: Color,
) {
    let half_width = size.x / 2.0;
    let half_height = size.y / 2.0;
    let steps = (size.y.round() as i32).max(1);
    
    for i in 0..steps {
        let t = if steps > 1 {
            (i as f32) / ((steps - 1) as f32)
        } else {
            0.5
        };
        let y = center.y - half_height + t * size.y;
        
        gizmos.line_2d(
            Vec2::new(center.x - half_width, y),
            Vec2::new(center.x + half_width, y),
            color,
        );
    }
}

pub fn draw_health_bars(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Health, &Player, &PlayerStatsComponent)>,
) {
    for (transform, health, player, stats) in query.iter() {
        let scale = stats.player_scale;
        let player_pos = transform.translation.xy();
        let bar_center = player_pos + Vec2::new(0.0, 90.0 * scale);
        let bar_width = 64.0 * scale;
        let bar_height = 6.0 * scale;

        // 1. Draw solid outer boundary / background bar (dark solid)
        draw_filled_rect(
            &mut gizmos,
            bar_center,
            Vec2::new(bar_width, bar_height),
            Color::srgb(0.15, 0.15, 0.15),
        );

        // 2. Draw actual remaining health (green solid bar)
        let health_pct = (health.current / health.max).clamp(0.0, 1.0);
        if health_pct > 0.0 {
            let fg_width = bar_width * health_pct;
            let fg_center = bar_center - Vec2::new((bar_width - fg_width) / 2.0, 0.0);
            
            let color = player.color();

            // Draw solid foreground inside with 2.0 pixels padding
            draw_filled_rect(
                &mut gizmos,
                fg_center,
                Vec2::new(fg_width, bar_height - 2.0),
                color,
            );
        }
    }
}
