use bevy::prelude::*;
use bevy::picking::Pickable;
use crate::player::Player;
use crate::settings::{PersistentPlayerStats, GameState};
use crate::physics::card_selection::CARDS;

#[derive(Component)]
pub struct CardListUiContainer;

#[derive(Component)]
#[allow(dead_code)]
pub struct CardWidget {
    pub player: Player,
    pub card_index: usize,
}

#[derive(Component)]
pub struct CardPopupNode;

pub struct CardListUiPlugin;

impl Plugin for CardListUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Gameplay), spawn_card_list_ui);
        app.add_systems(Update, card_widget_hover_system.run_if(in_state(GameState::Gameplay)));
    }
}

pub fn spawn_card_list_ui(
    mut commands: Commands,
    persistent_stats: Res<PersistentPlayerStats>,
    existing_container: Query<Entity, With<CardListUiContainer>>,
) {
    // 1. Despawn existing UI if any to rebuild cleanly (in Bevy 0.18, despawn() recursively cleans up all children automatically)
    for entity in existing_container.iter() {
        commands.entity(entity).despawn();
    }

    let has_p1 = !persistent_stats.p1.cards.is_empty();
    let has_p2 = !persistent_stats.p2.cards.is_empty();

    // If neither player has any cards yet, don't draw anything (no placeholders)
    if !has_p1 && !has_p2 {
        return;
    }

    // Colors matching player visual themes
    let p1_color = Color::srgb(0.0, 0.83, 1.0); // Neon P1 Cyan/Blue
    let p2_color = Color::srgb(1.0, 0.55, 0.04); // Neon P2 Orange
    let bg_widget = Color::srgba(0.02, 0.02, 0.02, 0.85);

    // Spawn the root absolute top-right container
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(25.0),
            right: Val::Px(25.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        CardListUiContainer,
    )).with_children(|parent| {
        // Player 1 Row
        if has_p1 {
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            }).with_children(|row_parent| {
                row_parent.spawn((
                    Text::new("P1:"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(p1_color),
                    Pickable::IGNORE,
                ));

                for &card_idx in &persistent_stats.p1.cards {
                    spawn_card_widget(row_parent, Player::P1, card_idx, p1_color, bg_widget);
                }
            });
        }

        // Player 2 Row
        if has_p2 {
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            }).with_children(|row_parent| {
                row_parent.spawn((
                    Text::new("P2:"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(p2_color),
                    Pickable::IGNORE,
                ));

                for &card_idx in &persistent_stats.p2.cards {
                    spawn_card_widget(row_parent, Player::P2, card_idx, p2_color, bg_widget);
                }
            });
        }
    });
}

fn spawn_card_widget(
    builder: &mut ChildSpawnerCommands,
    player: Player,
    card_index: usize,
    player_color: Color,
    bg_color: Color,
) {
    if card_index >= CARDS.len() {
        return;
    }
    let card_def = &CARDS[card_index];
    let name_chars: String = card_def.name.chars().take(2).collect();

    builder.spawn((
        Node {
            width: Val::Px(35.0),
            height: Val::Px(35.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BorderColor::all(player_color),
        BackgroundColor(bg_color),
        Interaction::default(),
        CardWidget { player, card_index },
    )).with_children(|widget_parent| {
        // Label inside the widget
        widget_parent.spawn((
            Text::new(name_chars),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        ));

        // Hidden card details popup (GlobalZIndex(10) ensures it overlaps other rows beautifully)
        widget_parent.spawn((
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(42.0),
                right: Val::Px(0.0),
                width: Val::Px(250.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(player_color),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.95)),
            GlobalZIndex(10),
            CardPopupNode,
            Pickable::IGNORE,
        )).with_children(|popup_parent| {
            // Popup Card Title (Bold, styled matching selection cards)
            popup_parent.spawn((
                Text::new(card_def.name),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect {
                        bottom: Val::Px(6.0),
                        ..default()
                    },
                    ..default()
                },
                Pickable::IGNORE,
            ));

            // Popup Card Description
            popup_parent.spawn((
                Text::new(card_def.desc.replace('\n', " ")),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect {
                        bottom: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                },
                Pickable::IGNORE,
            ));

            // Popup Card Stats Lines
            for stat_line in card_def.stat_lines {
                let stat_color = if stat_line.starts_with('+') || stat_line.contains("Adds") {
                    Color::srgb(0.3, 0.9, 0.3) // Green for buffs
                } else {
                    Color::srgb(0.9, 0.3, 0.3) // Red for debuffs
                };

                popup_parent.spawn((
                    Text::new(*stat_line),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(stat_color),
                    Node {
                        margin: UiRect {
                            bottom: Val::Px(2.0),
                            ..default()
                        },
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
        });
    });
}

pub fn card_widget_hover_system(
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<CardWidget>)>,
    mut popup_query: Query<&mut Node, With<CardPopupNode>>,
) {
    for (interaction, children) in interaction_query.iter_mut() {
        for child in children.iter() {
            if let Ok(mut node) = popup_query.get_mut(child) {
                match *interaction {
                    Interaction::Hovered => {
                        node.display = Display::Flex;
                    }
                    _ => {
                        node.display = Display::None;
                    }
                }
            }
        }
    }
}
