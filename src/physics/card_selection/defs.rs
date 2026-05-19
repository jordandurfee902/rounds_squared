use bevy::prelude::*;
use crate::player::Player;

#[derive(Resource, Debug, Clone)]
pub struct CardSelectionState {
    pub selected_idx: usize,
    pub selecting_player: Player,
    pub drawn_cards: [usize; 5],
    pub queue: Vec<Player>,
}

#[derive(Component, Debug, Clone)]
pub struct CardSelectionUiComponent {
    pub index: usize,
}

#[derive(Component, Debug, Clone)]
pub struct SelectionHeaderComponent;
