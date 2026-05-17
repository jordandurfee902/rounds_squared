use bevy::prelude::*;
use crate::settings::{GameState, InputDevice, LobbySlots};

pub struct LobbyUiPlugin;

impl Plugin for LobbyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Lobby), setup_lobby_ui)
           .add_systems(OnExit(GameState::Lobby), cleanup_lobby_ui)
           .add_systems(Update, (
               lobby_join_system,
               lobby_render_update_system,
           ).run_if(in_state(GameState::Lobby)));
    }
}

// --- Marker Components ---

#[derive(Component)]
struct LobbyContainer;

#[derive(Component)]
struct PlayerSlotCard(crate::player::Player);

#[derive(Component)]
struct PlayerSlotStatusText(crate::player::Player);

#[derive(Component)]
struct LobbyPromptText;

// --- Setup System ---

fn setup_lobby_ui(
    mut commands: Commands,
    mut lobby_slots: ResMut<LobbySlots>,
    is_networked_opt: Option<Res<crate::net::IsNetworked>>,
) {
    let is_networked = is_networked_opt.map(|n| n.0).unwrap_or(false);
    let title_text = if is_networked { "ONLINE MULTIPLAYER LOBBY" } else { "LOCAL MULTIPLAYER LOBBY" };
    let sub_text = if is_networked {
        "PRESS [SPACE] ON KEYBOARD OR [A] ON CONTROLLER TO ASSIGN YOUR LOCAL CONTROLS"
    } else {
        "PRESS [SPACE] ON KEYBOARD OR [A] ON CONTROLLER TO JOIN"
    };

    // Reset any existing assignments
    lobby_slots.p1 = None;
    lobby_slots.p2 = None;

    // Spawn Root UI Node (Fullscreen)
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
        // Glowing Lobby Title
        parent.spawn((
            Text::new(title_text),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 0.83, 1.0)), // Neon Cyan
            Node {
                margin: UiRect { bottom: Val::Px(10.0), ..default() },
                ..default()
            },
        ));

        // Subtitle instructions
        parent.spawn((
            Text::new(sub_text),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            Node {
                margin: UiRect { bottom: Val::Px(60.0), ..default() },
                ..default()
            },
        ));

        // Slots Row containing P1 and P2 panels side-by-side
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Percent(85.0),
            column_gap: Val::Px(60.0),
            ..default()
        }).with_children(|row| {
            // Player 1 Card (Blue)
            spawn_player_card(row, crate::player::Player::P1);

            // Player 2 Card (Orange)
            spawn_player_card(row, crate::player::Player::P2);
        });

        // Bottom Join Prompt / Countdown Text
        parent.spawn((
            Text::new("WAITING FOR PLAYERS TO JOIN..."),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node {
                margin: UiRect { top: Val::Px(60.0), ..default() },
                ..default()
            },
            LobbyPromptText,
        ));
    });
}

// Spawns a beautiful, rounded glassmorphic slot card
fn spawn_player_card(
    builder: &mut ChildSpawnerCommands,
    player: crate::player::Player,
) {
    let (label, text_color) = match player {
        crate::player::Player::P1 => ("PLAYER 1", Color::srgb(0.0, 0.83, 1.0)), // Neon Blue
        crate::player::Player::P2 => ("PLAYER 2", Color::srgb(1.0, 0.55, 0.04)), // Neon Orange
    };

    builder.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(360.0),
            height: Val::Px(240.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(12.0)),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
        BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85)),
        PlayerSlotCard(player),
    )).with_children(|card| {
        // Player Label (Title)
        card.spawn((
            Text::new(label),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(text_color),
            Node {
                margin: UiRect { bottom: Val::Px(30.0), ..default() },
                ..default()
            },
        ));

        // Waiting / Joined Device Name
        card.spawn((
            Text::new("WAITING FOR BATTLE..."),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(0.4, 0.4, 0.4)),
            PlayerSlotStatusText(player),
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

        // --- ONLINE MULTIPLAYER LOBBY JOINING ---
        // 1. Keyboard Join
        if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
            if local_idx == 0 {
                if lobby_slots.p1.is_none() {
                    lobby_slots.p1 = Some(InputDevice::KeyboardMouse);
                }
            } else {
                if lobby_slots.p2.is_none() {
                    lobby_slots.p2 = Some(InputDevice::KeyboardMouse);
                }
            }
        }

        // 2. Gamepad Join
        for (gp_entity, gp) in gamepads.iter() {
            if gp.just_pressed(GamepadButton::South) {
                if local_idx == 0 {
                    if lobby_slots.p1.is_none() {
                        lobby_slots.p1 = Some(InputDevice::Gamepad(gp_entity));
                    }
                } else {
                    if lobby_slots.p2.is_none() {
                        lobby_slots.p2 = Some(InputDevice::Gamepad(gp_entity));
                    }
                }
            }
        }

        // 3. Un-join
        if keys.just_pressed(KeyCode::Backspace) {
            if local_idx == 0 && lobby_slots.p1.is_some() {
                lobby_slots.p1 = None;
            } else if local_idx == 1 && lobby_slots.p2.is_some() {
                lobby_slots.p2 = None;
            }
        }

        for (gp_entity, gp) in gamepads.iter() {
            if gp.just_pressed(GamepadButton::East) {
                if local_idx == 0 && lobby_slots.p1 == Some(InputDevice::Gamepad(gp_entity)) {
                    lobby_slots.p1 = None;
                } else if local_idx == 1 && lobby_slots.p2 == Some(InputDevice::Gamepad(gp_entity)) {
                    lobby_slots.p2 = None;
                }
            }
        }

        // 4. Return to main menu if escape is pressed
        if keys.just_pressed(KeyCode::Escape) {
            state.set(GameState::MainMenu);
        }
    } else {
        // --- LOCAL MULTIPLAYER LOBBY JOINING ---
        // 1. Keyboard Join Detection
        if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
            // Only join if keyboard is not already assigned
            let already_joined = lobby_slots.p1 == Some(InputDevice::KeyboardMouse) 
                || lobby_slots.p2 == Some(InputDevice::KeyboardMouse);

            if !already_joined {
                if lobby_slots.p1.is_none() {
                    lobby_slots.p1 = Some(InputDevice::KeyboardMouse);
                } else if lobby_slots.p2.is_none() {
                    lobby_slots.p2 = Some(InputDevice::KeyboardMouse);
                }
            } else {
                // If already joined and both are ready, keyboard space can start the game!
                if lobby_slots.p1.is_some() && lobby_slots.p2.is_some() {
                    state.set(GameState::Gameplay);
                }
            }
        }

        // 2. Gamepad Join Detection
        for (gp_entity, gp) in gamepads.iter() {
            if gp.just_pressed(GamepadButton::South) {
                let already_joined = lobby_slots.p1 == Some(InputDevice::Gamepad(gp_entity))
                    || lobby_slots.p2 == Some(InputDevice::Gamepad(gp_entity));

                if !already_joined {
                    if lobby_slots.p1.is_none() {
                        lobby_slots.p1 = Some(InputDevice::Gamepad(gp_entity));
                    } else if lobby_slots.p2.is_none() {
                        lobby_slots.p2 = Some(InputDevice::Gamepad(gp_entity));
                    }
                } else {
                    // If already joined and both are ready, controller A can start the game!
                    if lobby_slots.p1.is_some() && lobby_slots.p2.is_some() {
                        state.set(GameState::Gameplay);
                    }
                }
            }
        }

        // 3. Un-join / Go Back Cancel Logic
        // Keyboard backspace un-joins keyboard player
        if keys.just_pressed(KeyCode::Backspace) {
            if lobby_slots.p1 == Some(InputDevice::KeyboardMouse) {
                lobby_slots.p1 = None;
            } else if lobby_slots.p2 == Some(InputDevice::KeyboardMouse) {
                lobby_slots.p2 = None;
            }
        }

        // Gamepad B / East un-joins controller
        for (gp_entity, gp) in gamepads.iter() {
            if gp.just_pressed(GamepadButton::East) {
                if lobby_slots.p1 == Some(InputDevice::Gamepad(gp_entity)) {
                    lobby_slots.p1 = None;
                } else if lobby_slots.p2 == Some(InputDevice::Gamepad(gp_entity)) {
                    lobby_slots.p2 = None;
                }
            }
        }

        // Return to main menu if no one is joined and Escape or Gamepad East is pressed
        let lobby_empty = lobby_slots.p1.is_none() && lobby_slots.p2.is_none();
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

// --- Live Render System ---

fn lobby_render_update_system(
    lobby_slots: Res<LobbySlots>,
    mut card_query: Query<(&mut BorderColor, &mut BackgroundColor, &PlayerSlotCard)>,
    mut text_query: Query<(&mut Text, &mut TextColor, &PlayerSlotStatusText)>,
    mut prompt_query: Query<(&mut Text, &mut TextColor, &LobbyPromptText), Without<PlayerSlotStatusText>>,
    is_networked_opt: Option<Res<crate::net::IsNetworked>>,
    local_idx_opt: Option<Res<crate::net::LocalPlayerIndex>>,
) {
    // 1. Update Card panels
    for (mut border, mut bg, card) in card_query.iter_mut() {
        let slot = match card.0 {
            crate::player::Player::P1 => &lobby_slots.p1,
            crate::player::Player::P2 => &lobby_slots.p2,
        };

        if slot.is_some() {
            // Neon cyan border for P1, neon orange border for P2
            match card.0 {
                crate::player::Player::P1 => {
                    *border = BorderColor::all(Color::srgb(0.0, 0.83, 1.0));
                    *bg = BackgroundColor(Color::srgba(0.0, 0.83, 1.0, 0.08));
                }
                crate::player::Player::P2 => {
                    *border = BorderColor::all(Color::srgb(1.0, 0.55, 0.04));
                    *bg = BackgroundColor(Color::srgba(1.0, 0.55, 0.04, 0.08));
                }
            }
        } else {
            *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
            *bg = BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85));
        }
    }

    // 2. Update status texts
    for (mut text, mut text_color, status) in text_query.iter_mut() {
        let slot = match status.0 {
            crate::player::Player::P1 => &lobby_slots.p1,
            crate::player::Player::P2 => &lobby_slots.p2,
        };

        if let Some(device) = slot {
            match device {
                InputDevice::KeyboardMouse => {
                    text.0 = "KEYBOARD & MOUSE".to_string();
                    *text_color = TextColor(Color::WHITE);
                }
                InputDevice::Gamepad(gp_entity) => {
                    text.0 = format!("CONTROLLER ({:?})", gp_entity.index());
                    *text_color = TextColor(Color::WHITE);
                }
            }
        } else {
            text.0 = "WAITING FOR BATTLE...".to_string();
            *text_color = TextColor(Color::srgb(0.4, 0.4, 0.4));
        }
    }

    // 3. Update bottom prompt
    if let Some((mut prompt, mut prompt_color, _)) = prompt_query.iter_mut().next() {
        let is_networked = is_networked_opt.map(|n| n.0).unwrap_or(false);
        if lobby_slots.p1.is_some() && lobby_slots.p2.is_some() {
            if is_networked {
                prompt.0 = "BOTH PLAYERS READY! LAUNCHING MATCH...".to_string();
            } else {
                prompt.0 = "BOTH PLAYERS READY! PRESS [SPACE] OR [A] TO BATTLE!".to_string();
            }
            *prompt_color = TextColor(Color::srgb(0.0, 1.0, 0.5)); // Glow Green
        } else {
            if is_networked {
                let local_idx = local_idx_opt.map(|idx| idx.0).unwrap_or(0);
                let local_joined = if local_idx == 0 { lobby_slots.p1.is_some() } else { lobby_slots.p2.is_some() };
                if local_joined {
                    prompt.0 = "WAITING FOR REMOTE PLAYER TO READY UP...".to_string();
                } else {
                    prompt.0 = "CHOOSE YOUR INPUT DEVICE TO READY UP!".to_string();
                }
            } else {
                prompt.0 = "WAITING FOR BOTH PLAYERS TO JOIN...".to_string();
            }
            *prompt_color = TextColor(Color::srgb(0.5, 0.5, 0.5));
        }
    }
}
