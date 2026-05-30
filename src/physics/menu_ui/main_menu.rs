use bevy::prelude::*;
use super::types::*;
use super::settings_menu::spawn_button;

pub fn setup_main_menu(
    mut commands: Commands,
    menu: Res<ActiveMenu>,
    existing_settings: Query<Entity, With<SettingsMenuContainer>>,
) {
    // Spawn root Main Menu UI container
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
        MainMenuContainer,
    )).with_children(|parent| {
        // Glowing Title
        parent.spawn((
            Text::new("SETS"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 0.83, 1.0)), // Neon Cyan
            Node {
                margin: UiRect { bottom: Val::Px(10.0), ..default() },
                ..default()
            },
        ));

        // Sub-title
        parent.spawn((
            Text::new("PROCEDURAL 2D ARENA SHOOTER"),
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

        // Menu Box
        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            width: Val::Px(320.0),
            ..default()
        }).with_children(|menu_parent| {
            spawn_button(menu_parent, "LOCAL PLAY", MenuButton::StartGame, Color::srgb(0.0, 0.83, 1.0));
            spawn_button(menu_parent, "ONLINE MULTIPLAYER", MenuButton::FindMatch, Color::srgb(0.0, 1.0, 0.5));
            spawn_button(menu_parent, "SETTINGS", MenuButton::OpenSettings, Color::srgb(1.0, 0.55, 0.04));
            spawn_button(menu_parent, "QUIT GAME", MenuButton::Quit, Color::srgb(0.9, 0.2, 0.2));
        });
    });

    // Check if settings were opened in main menu
    if menu.is_settings_open && existing_settings.is_empty() {
        super::settings_menu::spawn_settings_menu(&mut commands, true, false);
    }
}

pub fn cleanup_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MainMenuContainer>>,
    existing_settings: Query<Entity, With<SettingsMenuContainer>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in existing_settings.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn main_menu_ui_watcher(
    mut commands: Commands,
    menu: Res<ActiveMenu>,
    existing_settings: Query<Entity, With<SettingsMenuContainer>>,
) {
    let has_settings = !existing_settings.is_empty();
    if menu.is_settings_open && !has_settings {
        super::settings_menu::spawn_settings_menu(&mut commands, true, false);
    } else if !menu.is_settings_open && has_settings {
        for ent in existing_settings.iter() {
            commands.entity(ent).despawn();
        }
    }
}
