use bevy::prelude::*;

pub mod components;
pub mod spawning;
pub mod input;
pub mod ui;

pub use components::*;
pub use spawning::*;
pub use input::*;
pub use ui::*;

use crate::settings::GameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Gameplay), spawn_players.after(crate::map::spawn_platforms))
           .add_systems(OnExit(GameState::Gameplay), despawn_gameplay_entities)
           .add_systems(Update, (
               player_input.before(crate::physics::forces::apply_gravity_and_movement),
               player_block_system,
           ).chain().run_if(in_state(GameState::Gameplay).and(crate::physics::is_not_paused).and(resource_equals(crate::net::IsNetworked(false)))))
           .add_systems(Update, (
               draw_health_bars,
           ).run_if(in_state(GameState::Gameplay)));
    }
}
