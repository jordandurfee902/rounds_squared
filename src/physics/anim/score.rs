use bevy::prelude::*;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};

/// Renders a UI score overlay in the top left corner with rows of blue and orange circles.
pub fn draw_score_overlay(
    mut gizmos: Gizmos,
    score: Res<crate::settings::ScoreTracker>,
    state: Res<State<crate::settings::GameState>>,
) {
    if *state.get() == crate::settings::GameState::MainMenu || *state.get() == crate::settings::GameState::Lobby {
        return;
    }
    let half_width = TARGET_WIDTH / 2.0;
    let half_height = TARGET_HEIGHT / 2.0;
    let radius = 18.0; // Larger size to look very premium
    let spacing = 48.0; // Larger spacing to comfortably fit the larger size
    
    // Player 1 (Blue) Row
    let p1_color = Color::srgb(0.2, 0.5, 1.0);
    let p1_y = half_height - 50.0;
    for i in 0..score.p1_wins {
        let pos = Vec2::new(-half_width + 45.0 + i as f32 * spacing, p1_y);
        // Draw beautiful solid filled blue circle
        let max_r = radius as i32;
        for r in 0..=max_r {
            gizmos.circle_2d(pos, r as f32, p1_color);
        }
    }
    
    // Player 2 (Orange) Row
    let p2_color = Color::srgb(1.0, 0.6, 0.2);
    let p2_y = half_height - 100.0;
    for i in 0..score.p2_wins {
        let pos = Vec2::new(-half_width + 45.0 + i as f32 * spacing, p2_y);
        // Draw beautiful solid filled orange circle
        let max_r = radius as i32;
        for r in 0..=max_r {
            gizmos.circle_2d(pos, r as f32, p2_color);
        }
    }
}
