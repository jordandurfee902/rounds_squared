use bevy::prelude::*;
use crate::settings::GameState;
use super::types::*;
use super::settings_menu::spawn_button;

/// Keyboard input checks for escape during Gameplay
pub fn pause_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut paused: ResMut<Paused>,
    mut menu: ResMut<ActiveMenu>,
    mut active_input: ResMut<ActiveSettingInput>,
) {
    if keys.just_pressed(KeyCode::Escape) && *state.get() == GameState::Gameplay {
        active_input.focused_setting = None;
        active_input.current_text.clear();
        if menu.is_settings_open {
            menu.is_settings_open = false;
        } else {
            paused.0 = !paused.0;
        }
    }
}

/// Watches the `Paused` and `ActiveMenu` resources in Gameplay state and manages the overlay UI nodes
pub fn pause_menu_state_watcher(
    mut commands: Commands,
    state: Res<State<GameState>>,
    paused: Res<Paused>,
    menu: Res<ActiveMenu>,
    existing_pause: Query<Entity, With<PauseMenuContainer>>,
    existing_settings: Query<Entity, With<SettingsMenuContainer>>,
) {
    if *state.get() != GameState::Gameplay {
        // Despawn if leaving Gameplay
        for entity in existing_pause.iter() {
            commands.entity(entity).despawn();
        }
        for entity in existing_settings.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // 1. Manage standard Pause Menu
    let should_show_pause = paused.0 && !menu.is_settings_open;
    let has_pause = !existing_pause.is_empty();

    if should_show_pause && !has_pause {
        spawn_pause_menu(&mut commands);
    } else if (!should_show_pause || *state.get() != GameState::Gameplay) && has_pause {
        for entity in existing_pause.iter() {
            commands.entity(entity).despawn();
        }
    }

    // 2. Manage Settings overlay
    let should_show_settings = paused.0 && menu.is_settings_open;
    let has_settings = !existing_settings.is_empty();

    if should_show_settings && !has_settings {
        super::settings_menu::spawn_settings_menu(&mut commands, false); // in-pause settings
    } else if (!should_show_settings || *state.get() != GameState::Gameplay) && has_settings {
        for entity in existing_settings.iter() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_pause_menu(commands: &mut Commands) {
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
        BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.85)),
        GlobalZIndex(100),
        PauseMenuContainer,
    )).with_children(|parent| {
        // Pause Window Box
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(35.0)),
                width: Val::Px(420.0),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.0, 0.83, 1.0)),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.98)),
        )).with_children(|box_parent| {
            // Header
            box_parent.spawn((
                Text::new("GAME PAUSED"),
                TextFont {
                    font_size: 38.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect { bottom: Val::Px(15.0), ..default() },
                    ..default()
                },
            ));

            // Pause buttons
            spawn_button(box_parent, "CONTINUE", MenuButton::Continue, Color::srgb(0.0, 0.83, 1.0));
            spawn_button(box_parent, "SETTINGS", MenuButton::OpenSettings, Color::srgb(1.0, 0.55, 0.04));
            spawn_button(box_parent, "BACK TO MAIN MENU", MenuButton::BackToMainMenu, Color::srgb(1.0, 0.4, 0.0));
            spawn_button(box_parent, "QUIT GAME", MenuButton::Quit, Color::srgb(0.9, 0.2, 0.2));
        });
    });
}

pub fn reset_input_delay(mut delay: ResMut<GameplayInputDelay>) {
    delay.0 = 0.2; // 0.2s input delay to absorb click/keyboard releases
}

pub fn tick_input_delay(time: Res<Time>, mut delay: ResMut<GameplayInputDelay>) {
    if delay.0 > 0.0 {
        delay.0 -= time.delta_secs();
    }
}
