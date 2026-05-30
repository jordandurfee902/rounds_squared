use bevy::prelude::*;
use bevy::input::keyboard::{KeyboardInput, Key};
use crate::settings::GameState;

#[derive(Resource, Default, Debug, Clone)]
pub struct ActiveJoinCodeTyping {
    pub is_typing: bool,
    pub code: String,
}

#[derive(Component)]
pub struct OnlineMenuContainer;

#[derive(Component)]
pub struct JoinCodeDisplayText;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnlineMenuButton {
    PublicMatch,
    HostGame,
    JoinGame,
    ConnectWithCode,
    BackToMainMenu,
    BackToOnlineMenu,
}

#[cfg(target_os = "windows")]
pub fn copy_to_clipboard(text: &str) {
    use std::process::Command;
    let _ = Command::new("powershell")
        .args(["-Command", &format!("Set-Clipboard -Value '{}'", text)])
        .spawn();
}

#[cfg(not(target_os = "windows"))]
pub fn copy_to_clipboard(_text: &str) {
    // Fallback for non-windows platforms
}

pub fn generate_join_code() -> String {
    #[cfg(target_arch = "wasm32")]
    let seed = js_sys::Date::now() as u64;

    #[cfg(not(target_arch = "wasm32"))]
    let seed = {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64
    };

    let mut x = seed;
    x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let code = 100000 + (x % 900000);
    code.to_string()
}

pub fn setup_online_menu(mut commands: Commands) {
    commands.insert_resource(ActiveJoinCodeTyping {
        is_typing: false,
        code: String::new(),
    });
}

pub fn cleanup_online_menu(
    mut commands: Commands,
    container_q: Query<Entity, With<OnlineMenuContainer>>,
) {
    for entity in container_q.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<ActiveJoinCodeTyping>();
}

pub fn spawn_online_button(
    builder: &mut ChildSpawnerCommands,
    label: &str,
    button_action: OnlineMenuButton,
    theme_color: Color,
) {
    builder.spawn((
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.5)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            margin: UiRect { top: Val::Px(8.0), ..default() },
            ..default()
        },
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
        BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85)),
        button_action,
    )).with_children(|btn_parent| {
        btn_parent.spawn((
            Text::new(label),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(theme_color),
        ));
    });
}

pub fn online_menu_ui_watcher(
    mut commands: Commands,
    typing: Res<ActiveJoinCodeTyping>,
    container_q: Query<Entity, With<OnlineMenuContainer>>,
    mut last_is_typing: Local<Option<bool>>,
) {
    let container_empty = container_q.is_empty();
    if container_empty || last_is_typing.is_none() || last_is_typing.unwrap() != typing.is_typing {
        *last_is_typing = Some(typing.is_typing);
        for entity in container_q.iter() {
            commands.entity(entity).despawn();
        }
        setup_online_menu_inner(&mut commands, &typing);
    }
}

fn setup_online_menu_inner(
    commands: &mut Commands,
    typing: &ActiveJoinCodeTyping,
) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.01, 0.01, 0.01, 0.98)),
        OnlineMenuContainer,
    )).with_children(|parent| {
        if !typing.is_typing {
            parent.spawn((
                Text::new("ONLINE MULTIPLAYER"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.5)),
                Node {
                    margin: UiRect { bottom: Val::Px(10.0), ..default() },
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("SELECT LOBBY CONFIGURATION"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect { bottom: Val::Px(45.0), ..default() },
                    ..default()
                },
            ));

            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                width: Val::Px(360.0),
                ..default()
            }).with_children(|menu_parent| {
                spawn_online_button(menu_parent, "PUBLIC MATCHMAKING", OnlineMenuButton::PublicMatch, Color::srgb(0.0, 1.0, 0.5));
                spawn_online_button(menu_parent, "HOST PRIVATE GAME", OnlineMenuButton::HostGame, Color::srgb(0.0, 0.83, 1.0));
                spawn_online_button(menu_parent, "JOIN PRIVATE GAME", OnlineMenuButton::JoinGame, Color::srgb(1.0, 0.55, 0.04));
                spawn_online_button(menu_parent, "BACK TO MAIN MENU", OnlineMenuButton::BackToMainMenu, Color::srgb(0.9, 0.2, 0.2));
            });
        } else {
            parent.spawn((
                Text::new("JOIN PRIVATE GAME"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.55, 0.04)),
                Node {
                    margin: UiRect { bottom: Val::Px(10.0), ..default() },
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("ENTER THE 6-DIGIT ROOM CODE PROVIDED BY HOST"),
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

            parent.spawn((
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(20.0)),
                    margin: UiRect { bottom: Val::Px(30.0), ..default() },
                    width: Val::Px(420.0),
                    ..default()
                },
                BorderColor::all(Color::srgb(1.0, 0.55, 0.04)),
                BackgroundColor(Color::srgba(0.03, 0.03, 0.03, 0.95)),
            )).with_children(|box_parent| {
                let mut display = String::new();
                for i in 0..6 {
                    if i == 3 {
                        display.push_str(" ");
                    }
                    if i < typing.code.len() {
                        display.push(typing.code.chars().nth(i).unwrap());
                    } else {
                        display.push('_');
                    }
                    if i < 5 {
                        display.push(' ');
                    }
                }
                box_parent.spawn((
                    Text::new(display),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    JoinCodeDisplayText,
                ));
            });

            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                width: Val::Px(320.0),
                ..default()
            }).with_children(|menu_parent| {
                spawn_online_button(menu_parent, "CONNECT TO HOST", OnlineMenuButton::ConnectWithCode, Color::srgb(0.0, 1.0, 0.5));
                spawn_online_button(menu_parent, "BACK", OnlineMenuButton::BackToOnlineMenu, Color::srgb(0.9, 0.2, 0.2));
            });
        }
    });
}

pub fn online_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, &mut BackgroundColor, &OnlineMenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<NextState<GameState>>,
    mut typing_res: Option<ResMut<ActiveJoinCodeTyping>>,
    mut code_res: ResMut<crate::net::OnlineCodeResource>,
    mut commands: Commands,
) {
    for (interaction, mut border, mut bg, btn) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(1.0, 0.55, 0.04, 0.35));
                *border = BorderColor::all(Color::srgb(1.0, 0.55, 0.04));

                match btn {
                    OnlineMenuButton::PublicMatch => {
                        code_res.code = String::new();
                        code_res.is_host = false;
                        state.set(GameState::Matchmaking);
                    }
                    OnlineMenuButton::HostGame => {
                        let code = generate_join_code();
                        code_res.code = code.clone();
                        code_res.is_host = true;
                        
                        // Copy formatted code to clipboard
                        let formatted_code = format!("{} {}", &code[..3], &code[3..]);
                        copy_to_clipboard(&formatted_code);
                        
                        // Connect socket immediately as host
                        info!("HOST: Starting private match socket on room code: {}", code);
                        let room_url = format!("wss://durfdog-sets.hf.space/room_{}?next=8", code);
                        let (socket, message_loop) = matchbox_socket::WebRtcSocket::builder(&room_url)
                            .add_channel(matchbox_socket::ChannelConfig::unreliable())
                            .build();
                        bevy::tasks::IoTaskPool::get().spawn(message_loop).detach();
                        commands.insert_resource(crate::net::MatchboxSocketResource(socket));
                        
                        // Set host net resources
                        commands.insert_resource(crate::net::LocalPlayerIndex(0));
                        commands.insert_resource(crate::net::IsNetworked(true));
                        commands.insert_resource(crate::net::RollbackRng::new(98765));

                        state.set(GameState::Lobby);
                    }
                    OnlineMenuButton::JoinGame => {
                        if let Some(ref mut typing) = typing_res {
                            typing.is_typing = true;
                            typing.code.clear();
                        }
                    }
                    OnlineMenuButton::ConnectWithCode => {
                        if let Some(ref mut typing) = typing_res {
                            if typing.code.len() == 6 {
                                code_res.code = typing.code.clone();
                                code_res.is_host = false;
                                state.set(GameState::Matchmaking);
                            }
                        }
                    }
                    OnlineMenuButton::BackToMainMenu => {
                        state.set(GameState::MainMenu);
                    }
                    OnlineMenuButton::BackToOnlineMenu => {
                        // Disconnect Matchmaking socket if canceling
                        commands.remove_resource::<crate::net::MatchboxSocketResource>();
                        if let Some(ref mut typing) = typing_res {
                            typing.is_typing = false;
                            typing.code.clear();
                        }
                        state.set(GameState::OnlineMenu);
                    }
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.08, 0.08, 0.08, 0.95));
                *border = BorderColor::all(Color::srgb(1.0, 0.55, 0.04));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85));
                *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }
}

pub fn online_menu_keyboard_input_system(
    mut events: MessageReader<KeyboardInput>,
    mut typing: ResMut<ActiveJoinCodeTyping>,
    mut state: ResMut<NextState<GameState>>,
    mut code_res: ResMut<crate::net::OnlineCodeResource>,
) {
    if !typing.is_typing {
        return;
    }

    for event in events.read() {
        if event.state.is_pressed() {
            match &event.logical_key {
                Key::Character(c) => {
                    // Only allow numeric digits 0-9
                    if c.chars().all(|ch| ch.is_ascii_digit()) && typing.code.len() < 6 {
                        typing.code.push_str(&c);
                    }
                }
                Key::Backspace => {
                    typing.code.pop();
                }
                Key::Enter => {
                    if typing.code.len() == 6 {
                        code_res.code = typing.code.clone();
                        code_res.is_host = false;
                        state.set(GameState::Matchmaking);
                    }
                }
                Key::Escape => {
                    typing.is_typing = false;
                    typing.code.clear();
                }
                _ => {}
            }
        }
    }
}

pub fn join_code_ui_sync_system(
    typing: Res<ActiveJoinCodeTyping>,
    mut query: Query<&mut Text, With<JoinCodeDisplayText>>,
) {
    if typing.is_changed() {
        for mut text in query.iter_mut() {
            let mut display = String::new();
            for i in 0..6 {
                if i == 3 {
                    display.push_str(" ");
                }
                if i < typing.code.len() {
                    display.push(typing.code.chars().nth(i).unwrap());
                } else {
                    display.push('_');
                }
                if i < 5 {
                    display.push(' ');
                }
            }
            text.0 = display;
        }
    }
}
