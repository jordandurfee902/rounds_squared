use bevy::prelude::*;
use super::online_menu::{spawn_online_button, OnlineMenuButton};

#[derive(Component)]
pub struct MatchmakingContainer;

pub fn setup_matchmaking_ui(
    mut commands: Commands,
    code_res: Option<Res<crate::net::OnlineCodeResource>>,
) {
    let (title_text, status_text, detail_text) = if let Some(res) = code_res {
        if res.code.is_empty() {
            (
                "PUBLIC MATCHMAKING",
                "SEARCHING FOR PEER...".to_string(),
                "Connecting to the public matchmaking lobby.\nThe game will start automatically when a peer joins."
            )
        } else {
            let raw_code = res.code.replace(" ", "");
            let formatted_code = if raw_code.len() == 6 {
                format!("{} {}", &raw_code[..3], &raw_code[3..])
            } else {
                raw_code.clone()
            };

            if res.is_host {
                (
                    "HOST PRIVATE GAME",
                    format!("ROOM CODE: {}", formatted_code),
                    "Share this code with your friend!\nThe code has been copied to your clipboard automatically."
                )
            } else {
                (
                    "JOIN PRIVATE GAME",
                    format!("CONNECTING TO ROOM: {}", formatted_code),
                    "Establishing handshake with Host...\nWaiting for connection to finalize."
                )
            }
        }
    } else {
        (
            "ONLINE MULTIPLAYER",
            "CONNECTING...".to_string(),
            "Initiating online multiplayer connection..."
        )
    };

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
        MatchmakingContainer,
    )).with_children(|parent| {
        parent.spawn((
            Text::new(title_text),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.5)),
            Node {
                margin: UiRect { bottom: Val::Px(15.0), ..default() },
                ..default()
            },
        ));

        parent.spawn((
            Text::new(status_text),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect { bottom: Val::Px(20.0), ..default() },
                ..default()
            },
        ));

        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(1.5)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                width: Val::Px(550.0),
                margin: UiRect { bottom: Val::Px(20.0), ..default() },
                ..default()
            },
            BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.95)),
        )).with_children(|box_parent| {
            box_parent.spawn((
                Text::new(detail_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });

        // Cancel Button Box
        parent.spawn(Node {
            width: Val::Px(280.0),
            ..default()
        }).with_children(|cancel_parent| {
            spawn_online_button(cancel_parent, "CANCEL MATCHMAKING", OnlineMenuButton::BackToOnlineMenu, Color::srgb(0.9, 0.2, 0.2));
        });
    });
}

pub fn cleanup_matchmaking_ui(
    mut commands: Commands,
    query: Query<Entity, With<MatchmakingContainer>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
