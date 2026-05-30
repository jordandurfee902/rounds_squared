use bevy::prelude::*;
use crate::graphics::{TARGET_WIDTH, TARGET_HEIGHT};

/// Renders a UI score overlay in the top left corner with rows of blue and orange circles.
pub fn draw_score_overlay(
    mut gizmos: Gizmos,
    score: Res<crate::settings::ScoreTracker>,
    state: Res<State<crate::settings::GameState>>,
    lobby_slots: Res<crate::settings::LobbySlots>,
) {
    if *state.get() == crate::settings::GameState::MainMenu || *state.get() == crate::settings::GameState::Lobby {
        return;
    }
    let half_width = TARGET_WIDTH / 2.0;
    let half_height = TARGET_HEIGHT / 2.0;
    let radius = 36.0; // Larger size to look very premium
    let spacing = 96.0; // Larger spacing to comfortably fit the larger size

    let mut active_indices = Vec::new();
    for i in 0..8 {
        if lobby_slots.slots[i].is_some() {
            active_indices.push(i);
        }
    }
    if active_indices.is_empty() {
        active_indices.push(0);
        active_indices.push(1);
    }

    for (row_idx, &p_idx) in active_indices.iter().enumerate() {
        let player = crate::player::Player::from_index(p_idx);
        let p_color = player.color();
        let p_y = half_height - (100.0 + row_idx as f32 * 100.0);
        let wins = score.wins[p_idx];

        for i in 0..wins {
            let pos = Vec2::new(-half_width + 90.0 + i as f32 * spacing, p_y);
            let max_r = radius as i32;
            for r in 0..=max_r {
                gizmos.circle_2d(pos, r as f32, p_color);
            }
        }
    }
}
