pub mod types;
pub mod main_menu;
pub mod pause_menu;
pub mod settings_menu;
pub mod online_menu;
pub mod matchmaking_ui;

pub use types::*;
pub use main_menu::*;
pub use pause_menu::*;
pub use settings_menu::*;
pub use online_menu::*;
pub use matchmaking_ui::*;

use bevy::prelude::*;
use crate::settings::GameState;

pub struct MenuUiPlugin;

impl Plugin for MenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Paused(false))
           .insert_resource(ActiveMenu { is_settings_open: false })
           .insert_resource(GameplayInputDelay(0.0))
           .insert_resource(ActiveSettingInput::default())
           .insert_resource(crate::net::OnlineCodeResource::default())
           // Input delay on gameplay entry
           .add_systems(OnEnter(GameState::Gameplay), reset_input_delay)
           .add_systems(Update, tick_input_delay.run_if(in_state(GameState::Gameplay)))
           // Main Menu events
           .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
           .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
           // Online Matchmaking setup menu
           .add_systems(OnEnter(GameState::OnlineMenu), setup_online_menu)
           .add_systems(OnExit(GameState::OnlineMenu), cleanup_online_menu)
           // Matchmaking screen events
           .add_systems(OnEnter(GameState::Matchmaking), setup_matchmaking_ui)
           .add_systems(OnExit(GameState::Matchmaking), cleanup_matchmaking_ui)
           // Reset/Teardown when entering Main Menu or starting a new game Lobby
           .add_systems(OnEnter(GameState::MainMenu), reset_and_cleanup_gameplay)
           .add_systems(OnEnter(GameState::Lobby), reset_and_cleanup_gameplay)
           // Pause & Settings Menu events
           .add_systems(Update, (
               pause_input_system,
               pause_menu_state_watcher,
               main_menu_ui_watcher.run_if(in_state(GameState::MainMenu)),
               button_interaction_system,
               settings_value_sync_system,
               settings_keyboard_input_system,
               settings_scroll_system,
               settings_keyboard_scroll_system,
               // Online Menu systems
               online_menu_ui_watcher.run_if(in_state(GameState::OnlineMenu)),
               online_menu_keyboard_input_system.run_if(in_state(GameState::OnlineMenu)),
               online_menu_button_system.run_if(in_state(GameState::OnlineMenu).or(in_state(GameState::Matchmaking))),
               join_code_ui_sync_system.run_if(in_state(GameState::OnlineMenu)),
           ));
    }
}
