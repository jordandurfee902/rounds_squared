use bevy::prelude::*;
use crate::settings::{GameState, InputDevice, LobbySlots};
use crate::physics::menu_ui::types::{ActiveMenu, SettingsMenuContainer};

pub struct LobbyUiPlugin;

impl Plugin for LobbyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Lobby), setup_lobby_ui)
           .add_systems(OnExit(GameState::Lobby), cleanup_lobby_ui)
           .add_systems(Update, (
               lobby_join_system,
               lobby_button_system,
               lobby_render_update_system,
           ).run_if(in_state(GameState::Lobby)));
    }
}

// --- Marker Components ---

#[derive(Component)]
struct LobbyContainer;

#[derive(Component)]
struct LobbySlotCard(usize);

#[derive(Component)]
struct LobbyStatusText(usize);

#[derive(Component)]
struct LobbyPromptText;

#[derive(Component)]
enum LobbyButtonAction {
    StartGame,
    OpenSettings,
}

// --- Helper for Player Colors ---

fn player_color_from_index(idx: usize) -> Color {
    match idx {
        0 => Color::srgb(0.0, 0.83, 1.0),   // Cyan / P1
        1 => Color::srgb(1.0, 0.55, 0.04),  // Orange / P2
        2 => Color::srgb(0.2, 0.9, 0.2),    // Green / P3
        3 => Color::srgb(0.9, 0.2, 0.2),    // Red / P4
        4 => Color::srgb(0.7, 0.2, 0.9),    // Purple / P5
        5 => Color::srgb(0.9, 0.9, 0.1),    // Yellow / P6
        6 => Color::srgb(0.9, 0.3, 0.6),    // Pink / P7
        7 => Color::srgb(0.1, 0.7, 0.7),    // Teal / P8
        _ => Color::WHITE,
    }
}

// --- Setup System ---

fn setup_lobby_ui(
    mut commands: Commands,
    mut lobby_slots: ResMut<LobbySlots>,
    is_networked_opt: Option<Res<crate::net::IsNetworked>>,
    local_idx_opt: Option<Res<crate::net::LocalPlayerIndex>>,
) {
    let is_networked = is_networked_opt.map(|n| n.0).unwrap_or(false);
    let local_idx = local_idx_opt.map(|idx| idx.0).unwrap_or(0);

    // Reset all slots on enter
    for slot in lobby_slots.slots.iter_mut() {
        *slot = None;
    }

    // Default local host (player 0) is KeyboardMouse
    if !is_networked || local_idx == 0 {
        lobby_slots.slots[0] = Some(InputDevice::KeyboardMouse);
    }

    let is_host = !is_networked || local_idx == 0;

    let title_text = if is_networked {
        if is_host { "ONLINE LOBBY (HOST)" } else { "ONLINE LOBBY (CLIENT)" }
    } else {
        "LOCAL LOBBY"
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.01, 0.01, 0.01, 0.98)),
        LobbyContainer,
    )).with_children(|parent| {
        // Glowing Title
        parent.spawn((
            Text::new(title_text),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 0.83, 1.0)),
            Node {
                margin: UiRect { bottom: Val::Px(10.0), ..default() },
                ..default()
            },
        ));

        // Subtitle Instructions
        let sub_text = if is_networked {
            "PRESS [SPACE] ON KEYBOARD OR [A] ON CONTROLLER TO READY UP"
        } else {
            "PRESS [SPACE] OR CONTROLLER [A] TO ADD PLAYERS (UP TO 8)"
        };
        parent.spawn((
            Text::new(sub_text),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            Node {
                margin: UiRect { bottom: Val::Px(40.0), ..default() },
                ..default()
            },
        ));

        // 8 Slots Horizontal Row
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Percent(95.0),
            column_gap: Val::Px(15.0),
            ..default()
        }).with_children(|row| {
            for i in 0..8 {
                let p_color = player_color_from_index(i);
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(160.0),
                        height: Val::Px(180.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
                    BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85)),
                    LobbySlotCard(i),
                )).with_children(|card| {
                    card.spawn((
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            border_radius: BorderRadius::all(Val::Px(12.0)),
                            margin: UiRect { bottom: Val::Px(15.0), ..default() },
                            ..default()
                        },
                        BackgroundColor(p_color),
                    ));

                    // Slot label
                    card.spawn((
                        Text::new(format!("SLOT {}", i + 1)),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(p_color),
                        Node {
                            margin: UiRect { bottom: Val::Px(10.0), ..default() },
                            ..default()
                        },
                    ));

                    // Status details
                    card.spawn((
                        Text::new(format!("P{} INACTIVE", i + 1)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.4, 0.4)),
                        LobbyStatusText(i),
                    ));
                });
            }
        });

        // Host Button Controls / Client Waiting Prompt
        if is_host {
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect { top: Val::Px(40.0), ..default() },
                ..default()
            }).with_children(|btns| {
                spawn_lobby_button(btns, "GAME SETTINGS", LobbyButtonAction::OpenSettings, Color::srgb(1.0, 0.55, 0.04));
                spawn_lobby_button(btns, "START GAME", LobbyButtonAction::StartGame, Color::srgb(0.0, 1.0, 0.5));
            });
        } else {
            parent.spawn((
                Text::new("WAITING FOR HOST TO START GAME..."),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect { top: Val::Px(40.0), ..default() },
                    ..default()
                },
                LobbyPromptText,
            ));
        }
    });
}

fn spawn_lobby_button(
    builder: &mut ChildSpawnerCommands,
    label: &str,
    action: LobbyButtonAction,
    color: Color,
) {
    builder.spawn((
        Button,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            margin: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BorderColor::all(Color::srgb(0.4, 0.4, 0.4)),
        BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.9)),
        action,
    )).with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(color),
        ));
    });
}

// --- Cleanup System ---

fn cleanup_lobby_ui(
    mut commands: Commands,
    query: Query<Entity, With<LobbyContainer>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// --- Join & Action Logic System ---

fn lobby_join_system(
    mut state: ResMut<NextState<GameState>>,
    mut lobby_slots: ResMut<LobbySlots>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
    is_networked_opt: Option<Res<crate::net::IsNetworked>>,
    local_idx_opt: Option<Res<crate::net::LocalPlayerIndex>>,
) {
    let is_networked = is_networked_opt.map(|n| n.0).unwrap_or(false);

    if is_networked {
        let local_idx = local_idx_opt.map(|idx| idx.0).unwrap_or(0);

        // Keyboard Join
        if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
            if lobby_slots.slots[local_idx].is_none() {
                lobby_slots.slots[local_idx] = Some(InputDevice::KeyboardMouse);
            }
        }

        // Gamepad Join
        for (gp_entity, gp) in gamepads.iter() {
            if gp.just_pressed(GamepadButton::South) {
                if lobby_slots.slots[local_idx].is_none() {
                    lobby_slots.slots[local_idx] = Some(InputDevice::Gamepad(gp_entity));
                }
            }
        }

        // Un-join
        if keys.just_pressed(KeyCode::Backspace) {
            if lobby_slots.slots[local_idx].is_some() {
                lobby_slots.slots[local_idx] = None;
            }
        }

        for (gp_entity, gp) in gamepads.iter() {
            if gp.just_pressed(GamepadButton::East) {
                if lobby_slots.slots[local_idx] == Some(InputDevice::Gamepad(gp_entity)) {
                    lobby_slots.slots[local_idx] = None;
                }
            }
        }

        // Return to main menu if escape is pressed
        if keys.just_pressed(KeyCode::Escape) {
            state.set(GameState::MainMenu);
        }
    } else {
        // LOCAL MULTIPLAYER LOBBY JOINING
        // Keyboard Join Detection
        if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
            let already_joined = lobby_slots.slots.iter().any(|s| matches!(s, Some(InputDevice::KeyboardMouse)));
            if !already_joined {
                if let Some(slot) = lobby_slots.slots.iter_mut().find(|s| s.is_none()) {
                    *slot = Some(InputDevice::KeyboardMouse);
                }
            }
        }

        // Gamepad Join Detection
        for (gp_entity, gp) in gamepads.iter() {
            if gp.just_pressed(GamepadButton::South) {
                let already_joined = lobby_slots.slots.iter().any(|s| matches!(s, Some(InputDevice::Gamepad(e)) if *e == gp_entity));
                if !already_joined {
                    if let Some(slot) = lobby_slots.slots.iter_mut().find(|s| s.is_none()) {
                        *slot = Some(InputDevice::Gamepad(gp_entity));
                    }
                }
            }
        }

        // Un-join
        if keys.just_pressed(KeyCode::Backspace) {
            for slot in lobby_slots.slots.iter_mut() {
                if matches!(slot, Some(InputDevice::KeyboardMouse)) {
                    *slot = None;
                }
            }
        }

        for (gp_entity, gp) in gamepads.iter() {
            if gp.just_pressed(GamepadButton::East) {
                for slot in lobby_slots.slots.iter_mut() {
                    if matches!(slot, Some(InputDevice::Gamepad(e)) if *e == gp_entity) {
                        *slot = None;
                    }
                }
            }
        }

        // Return to main menu if empty
        let lobby_empty = lobby_slots.slots.iter().all(|s| s.is_none());
        if lobby_empty {
            if keys.just_pressed(KeyCode::Escape) {
                state.set(GameState::MainMenu);
            } else {
                for (_, gp) in gamepads.iter() {
                    if gp.just_pressed(GamepadButton::East) {
                        state.set(GameState::MainMenu);
                        break;
                    }
                }
            }
        }
    }
}

// --- Button Interaction Handling ---

fn lobby_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, &mut BackgroundColor, &LobbyButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut menu: ResMut<ActiveMenu>,
    lobby_slots: Res<LobbySlots>,
) {
    let active_count = lobby_slots.slots.iter().filter(|s| s.is_some()).count();
    let can_start = active_count >= 2;

    for (interaction, mut border, mut bg, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                match action {
                    LobbyButtonAction::StartGame => {
                        if can_start {
                            next_state.set(GameState::Gameplay);
                        }
                    }
                    LobbyButtonAction::OpenSettings => {
                        menu.is_settings_open = true;
                    }
                }
            }
            Interaction::Hovered => {
                if matches!(action, LobbyButtonAction::StartGame) && !can_start {
                    *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
                    *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.5));
                } else {
                    *border = BorderColor::all(Color::srgb(1.0, 1.0, 1.0));
                    *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
                }
            }
            Interaction::None => {
                if matches!(action, LobbyButtonAction::StartGame) && !can_start {
                    *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
                    *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.5));
                } else {
                    *border = BorderColor::all(Color::srgb(0.4, 0.4, 0.4));
                    *bg = BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.9));
                }
            }
        }
    }
}

// --- Live Render System ---

fn lobby_render_update_system(
    mut commands: Commands,
    lobby_slots: Res<LobbySlots>,
    mut card_query: Query<(&mut BorderColor, &mut BackgroundColor, &LobbySlotCard)>,
    mut text_query: Query<(&mut Text, &mut TextColor, &LobbyStatusText)>,
    mut start_btn_query: Query<(&mut BackgroundColor, &mut BorderColor, &Children), (With<LobbyButtonAction>, Without<LobbySlotCard>)>,
    mut btn_text_query: Query<&mut TextColor, (Without<LobbyStatusText>, Without<LobbySlotCard>)>,
    menu: Res<ActiveMenu>,
    existing_settings: Query<Entity, With<SettingsMenuContainer>>,
) {
    // 1. Manage Settings Menu Overlay
    let has_settings = !existing_settings.is_empty();
    if menu.is_settings_open && !has_settings {
        crate::physics::menu_ui::spawn_settings_menu(&mut commands, false);
    } else if !menu.is_settings_open && has_settings {
        for ent in existing_settings.iter() {
            commands.entity(ent).despawn();
        }
    }

    // 2. Update Slot Cards styling
    for (mut border, mut bg, card) in card_query.iter_mut() {
        let idx = card.0;
        let slot = &lobby_slots.slots[idx];
        let p_color = player_color_from_index(idx);

        if slot.is_some() {
            *border = BorderColor::all(p_color);
            *bg = BackgroundColor(p_color.with_alpha(0.08));
        } else {
            *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
            *bg = BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85));
        }
    }

    // 3. Update Status Text inside Slot Cards
    for (mut text, mut text_color, status) in text_query.iter_mut() {
        let idx = status.0;
        let slot = &lobby_slots.slots[idx];

        if let Some(device) = slot {
            match device {
                InputDevice::KeyboardMouse => {
                    text.0 = format!("P{} ACTIVE (KB/M)", idx + 1);
                    *text_color = TextColor(Color::WHITE);
                }
                InputDevice::Gamepad(gp_entity) => {
                    text.0 = format!("P{} ACTIVE (GP{})", idx + 1, gp_entity.index());
                    *text_color = TextColor(Color::WHITE);
                }
            }
        } else {
            text.0 = format!("P{} INACTIVE", idx + 1);
            *text_color = TextColor(Color::srgb(0.4, 0.4, 0.4));
        }
    }

    // 4. Update Start Button (disabled styling if active players < 2)
    let active_count = lobby_slots.slots.iter().filter(|s| s.is_some()).count();
    let can_start = active_count >= 2;

    for (mut bg, mut border, children) in start_btn_query.iter_mut() {
        if !can_start {
            *bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.5));
            *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
            for child in children.iter() {
                if let Ok(mut text_col) = btn_text_query.get_mut(child) {
                    *text_col = TextColor(Color::srgb(0.3, 0.3, 0.3));
                }
            }
        }
    }
}
