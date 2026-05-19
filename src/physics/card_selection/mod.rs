use bevy::prelude::*;
use crate::settings::GameState;

pub mod defs;
pub mod systems;
pub mod cards;

pub use defs::*;
pub use systems::*;

pub struct CardSelectionPlugin;

impl Plugin for CardSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_player_death.run_if(in_state(GameState::Gameplay).and(crate::physics::is_not_paused).and(resource_equals(crate::net::IsNetworked(false)))))
           .add_systems(OnEnter(GameState::CardSelection), setup_card_selection)
           .add_systems(OnExit(GameState::CardSelection), cleanup_card_selection)
           .add_systems(Update, (
               card_selection_input.run_if(resource_equals(crate::net::IsNetworked(false))),
               draw_card_gizmos,
           ).run_if(in_state(GameState::CardSelection)));
    }
}
