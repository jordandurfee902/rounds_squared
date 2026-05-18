use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::MessageWriter;
use bevy::prelude::MessageReader;
use bevy::input::keyboard::{KeyboardInput, Key};
use crate::settings::{PersistentPlayerStats, GameState, PhysicsSettings, ScoreTracker, AppSettings, KeyboardControls, ControllerControls, P1WeaponSettings, P2WeaponSettings};
use crate::player::PlayerStatsComponent;
use crate::player::{Player, Health};
use crate::physics::weapon::Weapon;

// --- Resources ---

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Paused(pub bool);

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ActiveMenu {
    pub is_settings_open: bool,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct GameplayInputDelay(pub f32);

#[derive(Resource, Default, Debug, Clone)]
pub struct ActiveSettingInput {
    pub focused_setting: Option<SettingType>,
    pub current_text: String,
}

// --- Components ---

#[derive(Component)]
pub struct MainMenuContainer;

#[derive(Component)]
pub struct PauseMenuContainer;

#[derive(Component)]
pub struct SettingsMenuContainer;

#[derive(Component)]
pub struct SettingsInnerList {
    pub scroll_offset: f32,
}

#[derive(Component)]
pub enum MenuButton {
    StartGame,
    FindMatch,
    Continue,
    OpenSettings,
    CloseSettings,
    BackToMainMenu,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SettingType {
    // Physics
    Gravity,
    PlayerAccel,
    PlayerJumpForce,
    BoundaryRestitution,
    PlayerRestitution,
    AirFriction,
    GroundFriction,
    MovementStopFriction,
    WallSlideSpeed,
    WallJumpPushForce,
    FastFallAcceleration,
    AirAccel,

    // P1 Character
    P1Health,
    P1Speed,
    P1Size,
    P1Damage,
    P1BulletRange,
    P1BulletSpeed,
    P1BulletGravity,
    P1BulletSizeMult,
    P1BulletGrowth,
    P1MaxAmmo,
    P1ReloadTime,
    P1FireRate,
    P1Bounces,
    P1BounceSpeedMultiplier,
    P1BlockDuration,
    P1BlockCooldown,
    P1BlockBorderBoost,

    // P2 Character
    P2Health,
    P2Speed,
    P2Size,
    P2Damage,
    P2BulletRange,
    P2BulletSpeed,
    P2BulletGravity,
    P2BulletSizeMult,
    P2BulletGrowth,
    P2MaxAmmo,
    P2ReloadTime,
    P2FireRate,
    P2Bounces,
    P2BounceSpeedMultiplier,
    P2BlockDuration,
    P2BlockCooldown,
    P2BlockBorderBoost,

    // Keyboard Controls
    KbMoveLeft,
    KbMoveRight,
    KbJump,
    KbFastFall,
    KbBlock,
    KbShoot,
    KbReload,

    // Controller Controls
    CtrlJump,
    CtrlBlock,
    CtrlShoot,
    CtrlReload,
}

#[derive(Component)]
pub struct SettingInputBox(pub SettingType);

#[derive(Component)]
pub struct SettingValueText(pub SettingType);

// --- Plugin Implementation ---

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

// --- Systems ---

/// Keyboard input checks for escape during Gameplay
fn pause_input_system(
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
fn pause_menu_state_watcher(
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
        spawn_settings_menu(&mut commands, false); // in-pause settings
    } else if (!should_show_settings || *state.get() != GameState::Gameplay) && has_settings {
        for entity in existing_settings.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// --- Menu Spawners ---

fn setup_main_menu(
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
        spawn_settings_menu(&mut commands, true);
    }
}

fn cleanup_main_menu(
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

fn spawn_pause_menu(commands: &mut Commands) {
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

fn spawn_settings_menu(commands: &mut Commands, is_main_menu: bool) {
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
        BackgroundColor(if is_main_menu { Color::srgba(0.01, 0.01, 0.01, 0.98) } else { Color::srgba(0.02, 0.02, 0.02, 0.85) }),
        GlobalZIndex(110),
        SettingsMenuContainer,
    )).with_children(|parent| {
        // Settings Window Box
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(15.0),
                padding: UiRect::all(Val::Px(24.0)),
                width: Val::Px(640.0),
                height: Val::Px(660.0),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(1.0, 0.55, 0.04)),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.98)),
        )).with_children(|box_parent| {
            // Header Row
            box_parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                ..default()
            }).with_children(|hdr| {
                hdr.spawn((
                    Text::new("LIVE SETTINGS CONFIGURATION"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                hdr.spawn((
                    Text::new("[Scroll Wheel or Arrow Keys to Scroll]"),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });

            // Scroll Viewport
            box_parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Px(450.0),
                overflow: Overflow::clip(),
                ..default()
            }).with_children(|scroll_viewport| {
                // Scroll inner list
                scroll_viewport.spawn((
                    Node {
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        width: Val::Percent(100.0),
                        top: Val::Px(0.0),
                        ..default()
                    },
                    SettingsInnerList { scroll_offset: 0.0 },
                )).with_children(|list| {
                    // Global Physics
                    spawn_section_header(list, "GLOBAL PHYSICS");
                    spawn_setting_row(list, "Gravity Scale", SettingType::Gravity);
                    spawn_setting_row(list, "Player Acceleration", SettingType::PlayerAccel);
                    spawn_setting_row(list, "Player Jump Force", SettingType::PlayerJumpForce);
                    spawn_setting_row(list, "Boundary Restitution", SettingType::BoundaryRestitution);
                    spawn_setting_row(list, "Player Restitution", SettingType::PlayerRestitution);
                    spawn_setting_row(list, "Air Friction", SettingType::AirFriction);
                    spawn_setting_row(list, "Ground Friction", SettingType::GroundFriction);
                    spawn_setting_row(list, "Movement Stop Friction", SettingType::MovementStopFriction);
                    spawn_setting_row(list, "Wall Slide Speed", SettingType::WallSlideSpeed);
                    spawn_setting_row(list, "Wall Jump Push Force", SettingType::WallJumpPushForce);
                    spawn_setting_row(list, "Fast Fall Acceleration", SettingType::FastFallAcceleration);
                    spawn_setting_row(list, "Air Acceleration", SettingType::AirAccel);

                    // Player 1 Stats
                    spawn_section_header(list, "PLAYER 1 CHARACTER STATISTICS");
                    spawn_setting_row(list, "Max Health", SettingType::P1Health);
                    spawn_setting_row(list, "Movement Speed", SettingType::P1Speed);
                    spawn_setting_row(list, "Player Scale Multiplier", SettingType::P1Size);
                    spawn_setting_row(list, "Bullet Damage", SettingType::P1Damage);
                    spawn_setting_row(list, "Bullet Range (seconds)", SettingType::P1BulletRange);
                    spawn_setting_row(list, "Bullet Velocity", SettingType::P1BulletSpeed);
                    spawn_setting_row(list, "Bullet Gravity Scale", SettingType::P1BulletGravity);
                    spawn_setting_row(list, "Bullet Size Multiplier", SettingType::P1BulletSizeMult);
                    spawn_setting_row(list, "Bullet Damage Growth (%)", SettingType::P1BulletGrowth);
                    spawn_setting_row(list, "Max Ammo Capacity", SettingType::P1MaxAmmo);
                    spawn_setting_row(list, "Weapon Reload Time", SettingType::P1ReloadTime);
                    spawn_setting_row(list, "Rate of Fire Delay", SettingType::P1FireRate);
                    spawn_setting_row(list, "Max Bullet Bounces", SettingType::P1Bounces);
                    spawn_setting_row(list, "Bounce Speed Multiplier", SettingType::P1BounceSpeedMultiplier);
                    spawn_setting_row(list, "Shield Block Duration", SettingType::P1BlockDuration);
                    spawn_setting_row(list, "Shield Block Cooldown", SettingType::P1BlockCooldown);
                    spawn_setting_row(list, "Border Block Force", SettingType::P1BlockBorderBoost);

                    // Player 2 Stats
                    spawn_section_header(list, "PLAYER 2 CHARACTER STATISTICS");
                    spawn_setting_row(list, "Max Health", SettingType::P2Health);
                    spawn_setting_row(list, "Movement Speed", SettingType::P2Speed);
                    spawn_setting_row(list, "Player Scale Multiplier", SettingType::P2Size);
                    spawn_setting_row(list, "Bullet Damage", SettingType::P2Damage);
                    spawn_setting_row(list, "Bullet Range (seconds)", SettingType::P2BulletRange);
                    spawn_setting_row(list, "Bullet Velocity", SettingType::P2BulletSpeed);
                    spawn_setting_row(list, "Bullet Gravity Scale", SettingType::P2BulletGravity);
                    spawn_setting_row(list, "Bullet Size Multiplier", SettingType::P2BulletSizeMult);
                    spawn_setting_row(list, "Bullet Damage Growth (%)", SettingType::P2BulletGrowth);
                    spawn_setting_row(list, "Max Ammo Capacity", SettingType::P2MaxAmmo);
                    spawn_setting_row(list, "Weapon Reload Time", SettingType::P2ReloadTime);
                    spawn_setting_row(list, "Rate of Fire Delay", SettingType::P2FireRate);
                    spawn_setting_row(list, "Max Bullet Bounces", SettingType::P2Bounces);
                    spawn_setting_row(list, "Bounce Speed Multiplier", SettingType::P2BounceSpeedMultiplier);
                    spawn_setting_row(list, "Shield Block Duration", SettingType::P2BlockDuration);
                    spawn_setting_row(list, "Shield Block Cooldown", SettingType::P2BlockCooldown);
                    spawn_setting_row(list, "Border Block Force", SettingType::P2BlockBorderBoost);

                    // Keyboard Controls
                    spawn_section_header(list, "KEYBOARD CONTROLS");
                    spawn_setting_row(list, "Move Left Key", SettingType::KbMoveLeft);
                    spawn_setting_row(list, "Move Right Key", SettingType::KbMoveRight);
                    spawn_setting_row(list, "Jump Key", SettingType::KbJump);
                    spawn_setting_row(list, "Fast Fall / Crouch Key", SettingType::KbFastFall);
                    spawn_setting_row(list, "Shield Block Action", SettingType::KbBlock);
                    spawn_setting_row(list, "Shoot Action", SettingType::KbShoot);
                    spawn_setting_row(list, "Manual Reload Key", SettingType::KbReload);

                    // Controller Controls
                    spawn_section_header(list, "CONTROLLER CONTROLS");
                    spawn_setting_row(list, "Jump Button", SettingType::CtrlJump);
                    spawn_setting_row(list, "Shield Block Button", SettingType::CtrlBlock);
                    spawn_setting_row(list, "Shoot Button", SettingType::CtrlShoot);
                    spawn_setting_row(list, "Manual Reload Button", SettingType::CtrlReload);
                });
            });

            // Back button
            spawn_button(box_parent, "SAVE & BACK", MenuButton::CloseSettings, Color::srgb(1.0, 0.55, 0.04));
        });
    });
}

// --- UI Helper Functions ---

fn spawn_button(
    builder: &mut ChildSpawnerCommands,
    label: &str,
    button_action: MenuButton,
    theme_color: Color,
) {
    builder.spawn((
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.5)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            margin: UiRect { top: Val::Px(5.0), ..default() },
            ..default()
        },
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
        BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85)),
        button_action,
    )).with_children(|btn_parent| {
        btn_parent.spawn((
            Text::new(label),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(theme_color),
        ));
    });
}

fn spawn_section_header(builder: &mut ChildSpawnerCommands, title: &str) {
    builder.spawn((
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::vertical(Val::Px(6.0)),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            border: UiRect { bottom: Val::Px(1.0), ..default() },
            margin: UiRect { top: Val::Px(12.0), bottom: Val::Px(4.0), ..default() },
            ..default()
        },
        BorderColor::all(Color::srgb(0.25, 0.25, 0.25)),
    )).with_children(|header| {
        header.spawn((
            Text::new(title),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 0.83, 1.0)), // Muted Cyan
        ));
    });
}

fn spawn_setting_row(
    builder: &mut ChildSpawnerCommands,
    label: &str,
    setting_type: SettingType,
) {
    builder.spawn(Node {
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        width: Val::Percent(100.0),
        height: Val::Px(32.0),
        padding: UiRect::horizontal(Val::Px(10.0)),
        ..default()
    }).with_children(|row| {
        // Setting Label
        row.spawn((
            Text::new(label),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.75, 0.75)),
        ));

        // Input button
        row.spawn((
            Button,
            Node {
                width: Val::Px(160.0),
                height: Val::Px(26.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
            BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85)),
            SettingInputBox(setting_type),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("---"),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                SettingValueText(setting_type),
            ));
        });
    });
}

fn save_and_sync_settings(
    physics_settings: &PhysicsSettings,
    p1_base: &P1WeaponSettings,
    p2_base: &P2WeaponSettings,
    kb_controls: &KeyboardControls,
    ctrl_controls: &ControllerControls,
    persistent_stats: &mut PersistentPlayerStats,
    players_q: &mut Query<(&Player, &mut Health, &mut PlayerStatsComponent, &mut Weapon)>,
) {
    let updated_settings = AppSettings {
        physics: physics_settings.clone(),
        p1_character: p1_base.0.clone(),
        p2_character: p2_base.0.clone(),
        keyboard_controls: kb_controls.clone(),
        controller_controls: ctrl_controls.clone(),
    };
    crate::settings::save_settings(&updated_settings);

    // Sync base fields to active round stats (preserving cards/special effects)
    persistent_stats.p1.movement_speed = p1_base.0.speed;
    persistent_stats.p1.health_max = p1_base.0.health;
    persistent_stats.p1.player_scale = p1_base.0.size;
    persistent_stats.p1.bullet_range = p1_base.0.bullet_range;
    persistent_stats.p1.bullet_speed = p1_base.0.bullet_speed;
    persistent_stats.p1.bullet_gravity = p1_base.0.bullet_gravity;
    persistent_stats.p1.bullet_damage = p1_base.0.damage;
    persistent_stats.p1.bullet_size_mult = p1_base.0.bullet_size_mult;
    persistent_stats.p1.bullet_growth = p1_base.0.bullet_growth;
    persistent_stats.p1.max_ammo = p1_base.0.max_ammo;
    persistent_stats.p1.reload_time = p1_base.0.reload_time;
    persistent_stats.p1.fire_rate = p1_base.0.fire_rate;
    persistent_stats.p1.bounces = p1_base.0.bounces;
    persistent_stats.p1.bounce_speed_multiplier = p1_base.0.bounce_speed_multiplier;
    persistent_stats.p1.block_duration = p1_base.0.block_duration;
    persistent_stats.p1.block_cooldown = p1_base.0.block_cooldown;
    persistent_stats.p1.block_border_boost = p1_base.0.block_border_boost;

    persistent_stats.p2.movement_speed = p2_base.0.speed;
    persistent_stats.p2.health_max = p2_base.0.health;
    persistent_stats.p2.player_scale = p2_base.0.size;
    persistent_stats.p2.bullet_range = p2_base.0.bullet_range;
    persistent_stats.p2.bullet_speed = p2_base.0.bullet_speed;
    persistent_stats.p2.bullet_gravity = p2_base.0.bullet_gravity;
    persistent_stats.p2.bullet_damage = p2_base.0.damage;
    persistent_stats.p2.bullet_size_mult = p2_base.0.bullet_size_mult;
    persistent_stats.p2.bullet_growth = p2_base.0.bullet_growth;
    persistent_stats.p2.max_ammo = p2_base.0.max_ammo;
    persistent_stats.p2.reload_time = p2_base.0.reload_time;
    persistent_stats.p2.fire_rate = p2_base.0.fire_rate;
    persistent_stats.p2.bounces = p2_base.0.bounces;
    persistent_stats.p2.bounce_speed_multiplier = p2_base.0.bounce_speed_multiplier;
    persistent_stats.p2.block_duration = p2_base.0.block_duration;
    persistent_stats.p2.block_cooldown = p2_base.0.block_cooldown;
    persistent_stats.p2.block_border_boost = p2_base.0.block_border_boost;

    // Dynamic entity-component synchronization
    for (player, mut health, mut stats, mut weapon) in players_q.iter_mut() {
        let p_stats = match player {
            Player::P1 => &persistent_stats.p1,
            Player::P2 => &persistent_stats.p2,
        };

        stats.movement_speed = p_stats.movement_speed;
        stats.jump_force = physics_settings.player_jump_force;
        stats.player_scale = p_stats.player_scale;
        stats.health_max = p_stats.health_max;
        stats.block_duration = p_stats.block_duration;
        stats.block_cooldown = p_stats.block_cooldown;
        stats.block_border_boost = p_stats.block_border_boost;

        if health.max != p_stats.health_max {
            health.max = p_stats.health_max;
            health.current = health.current.min(health.max);
        }

        weapon.max_ammo = p_stats.max_ammo;
        weapon.fire_rate = p_stats.fire_rate;
        weapon.reload_time = p_stats.reload_time;
        weapon.current_ammo = weapon.current_ammo.min(p_stats.max_ammo);
    }
}

// --- Interaction / Live Adjust System ---

fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, &mut BackgroundColor, Option<&MenuButton>, Option<&SettingInputBox>),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<NextState<GameState>>,
    mut paused: ResMut<Paused>,
    mut menu: ResMut<ActiveMenu>,
    mut exit: MessageWriter<AppExit>,
    mut physics_settings: ResMut<PhysicsSettings>,
    mut p1_base: ResMut<P1WeaponSettings>,
    mut p2_base: ResMut<P2WeaponSettings>,
    mut persistent_stats: ResMut<PersistentPlayerStats>,
    mut kb_controls: ResMut<KeyboardControls>,
    mut ctrl_controls: ResMut<ControllerControls>,
    mut active_input: ResMut<ActiveSettingInput>,
    mut players_q: Query<(&Player, &mut Health, &mut PlayerStatsComponent, &mut Weapon)>,
) {
    for (interaction, mut border, mut bg, btn, input_box) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(1.0, 0.55, 0.04, 0.35));
                *border = BorderColor::all(Color::srgb(1.0, 0.55, 0.04));

                // Auto-commit active typable input first before switching focus or closing!
                if let Some(setting) = active_input.focused_setting {
                    let is_control = matches!(
                        setting,
                        SettingType::KbMoveLeft | SettingType::KbMoveRight | SettingType::KbJump | SettingType::KbFastFall |
                        SettingType::KbBlock | SettingType::KbShoot | SettingType::KbReload |
                        SettingType::CtrlJump | SettingType::CtrlBlock | SettingType::CtrlShoot | SettingType::CtrlReload
                    );
                    if is_control {
                        apply_control_setting(setting, &active_input.current_text, &mut kb_controls, &mut ctrl_controls);
                        save_and_sync_settings(&physics_settings, &p1_base, &p2_base, &kb_controls, &ctrl_controls, &mut persistent_stats, &mut players_q);
                    } else if let Ok(val) = active_input.current_text.parse::<f32>() {
                        apply_setting_value(setting, val, &mut physics_settings, &mut p1_base, &mut p2_base);
                        save_and_sync_settings(&physics_settings, &p1_base, &p2_base, &kb_controls, &ctrl_controls, &mut persistent_stats, &mut players_q);
                    }
                }
                
                // 1. Process standard menu buttons
                if let Some(menu_btn) = btn {
                    active_input.focused_setting = None;
                    active_input.current_text.clear();

                    match menu_btn {
                        MenuButton::StartGame => {
                            state.set(GameState::Lobby);
                        }
                        MenuButton::FindMatch => {
                            state.set(GameState::OnlineMenu);
                        }
                        MenuButton::Continue => {
                            paused.0 = false;
                        }
                        MenuButton::OpenSettings => {
                            menu.is_settings_open = true;
                        }
                        MenuButton::CloseSettings => {
                            menu.is_settings_open = false;
                        }
                        MenuButton::BackToMainMenu => {
                            paused.0 = false;
                            menu.is_settings_open = false;
                            state.set(GameState::MainMenu);
                        }
                        MenuButton::Quit => {
                            exit.write(AppExit::Success);
                        }
                    }
                }

                // 2. Process settings typable inputs
                if let Some(in_box) = input_box {
                    active_input.focused_setting = Some(in_box.0);
                    active_input.current_text = get_setting_value(in_box.0, &physics_settings, &persistent_stats, &p1_base, &p2_base, &kb_controls, &ctrl_controls);
                }
            }
            Interaction::Hovered => {
                // If it is a menu button, glow it orange. If it's an input box and NOT focused, glow it subtle orange.
                let is_focused = if let Some(in_box) = input_box {
                    active_input.focused_setting == Some(in_box.0)
                } else {
                    false
                };

                if !is_focused {
                    *border = BorderColor::all(Color::srgb(1.0, 0.55, 0.04));
                    *bg = BackgroundColor(Color::srgba(1.0, 0.55, 0.04, 0.15));
                }
            }
            Interaction::None => {
                let is_focused = if let Some(in_box) = input_box {
                    active_input.focused_setting == Some(in_box.0)
                } else {
                    false
                };

                if !is_focused {
                    *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
                    *bg = BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.85));
                }
            }
        }
    }
}

/// Dynamic value syncer system keeping text value indicators perfectly matches active configurations
fn settings_value_sync_system(
    physics_settings: Res<PhysicsSettings>,
    persistent_stats: Res<PersistentPlayerStats>,
    p1_base: Res<P1WeaponSettings>,
    p2_base: Res<P2WeaponSettings>,
    kb_controls: Res<KeyboardControls>,
    ctrl_controls: Res<ControllerControls>,
    active_input: Res<ActiveSettingInput>,
    mut query: Query<(&mut Text, &SettingValueText, &ChildOf)>,
    mut border_q: Query<&mut BorderColor, With<SettingInputBox>>,
) {
    for (mut text, label, parent) in query.iter_mut() {
        let setting_type = label.0;

        if active_input.focused_setting == Some(setting_type) {
            text.0 = format!("{}|", active_input.current_text);
            
            // Set focused glowing border
            if let Ok(mut border) = border_q.get_mut(parent.parent()) {
                *border = BorderColor::all(Color::srgb(0.0, 0.83, 1.0)); // Glowing Cyan active typing border!
            }
        } else {
            text.0 = get_setting_value(setting_type, &physics_settings, &persistent_stats, &p1_base, &p2_base, &kb_controls, &ctrl_controls);
            
            // Standard border
            if let Ok(mut border) = border_q.get_mut(parent.parent()) {
                *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }
}

/// Listens for typed numbers, periods, backspaces, and applies them to resources and serialization files when Enter is pressed!
fn settings_keyboard_input_system(
    mut events: MessageReader<KeyboardInput>,
    mut active_input: ResMut<ActiveSettingInput>,
    mut physics_settings: ResMut<PhysicsSettings>,
    mut p1_base: ResMut<P1WeaponSettings>,
    mut p2_base: ResMut<P2WeaponSettings>,
    mut persistent_stats: ResMut<PersistentPlayerStats>,
    mut kb_controls: ResMut<KeyboardControls>,
    mut ctrl_controls: ResMut<ControllerControls>,
    mut players_q: Query<(&Player, &mut Health, &mut PlayerStatsComponent, &mut Weapon)>,
) {
    let Some(setting) = active_input.focused_setting else {
        return;
    };

    for event in events.read() {
        if event.state.is_pressed() {
            let is_control = matches!(
                setting,
                SettingType::KbMoveLeft | SettingType::KbMoveRight | SettingType::KbJump | SettingType::KbFastFall |
                SettingType::KbBlock | SettingType::KbShoot | SettingType::KbReload |
                SettingType::CtrlJump | SettingType::CtrlBlock | SettingType::CtrlShoot | SettingType::CtrlReload
            );

            match &event.logical_key {
                Key::Character(c) => {
                    // Filter: allow numeric/period/minus, or any character for control binds
                    if is_control || c.chars().all(|ch| ch.is_ascii_digit() || ch == '.' || ch == '-') {
                        active_input.current_text.push_str(&c);
                    }
                }
                Key::Backspace => {
                    active_input.current_text.pop();
                }
                Key::Enter => {
                    if is_control {
                        apply_control_setting(setting, &active_input.current_text, &mut kb_controls, &mut ctrl_controls);
                        save_and_sync_settings(&physics_settings, &p1_base, &p2_base, &kb_controls, &ctrl_controls, &mut persistent_stats, &mut players_q);
                    } else if let Ok(val) = active_input.current_text.parse::<f32>() {
                        // Apply and sync instantly
                        apply_setting_value(setting, val, &mut physics_settings, &mut p1_base, &mut p2_base);
                        save_and_sync_settings(&physics_settings, &p1_base, &p2_base, &kb_controls, &ctrl_controls, &mut persistent_stats, &mut players_q);
                    }
                    active_input.focused_setting = None;
                    active_input.current_text.clear();
                }
                Key::Escape => {
                    active_input.focused_setting = None;
                    active_input.current_text.clear();
                }
                _ => {}
            }
        }
    }
}

/// Allows scrolling with mouse wheel inside the settings viewport
fn settings_scroll_system(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut Node, &mut SettingsInnerList)>,
) {
    let mut scroll_dy = 0.0;
    for event in mouse_wheel_events.read() {
        scroll_dy += event.y * 30.0; // 30px per scroll tick
    }
    if scroll_dy != 0.0 {
        for (mut node, mut list) in query.iter_mut() {
            list.scroll_offset = (list.scroll_offset + scroll_dy).clamp(-2100.0, 0.0);
            node.top = Val::Px(list.scroll_offset);
        }
    }
}

/// Allows scrolling with Arrow keys / WASD when no text field is focused
fn settings_keyboard_scroll_system(
    keys: Res<ButtonInput<KeyCode>>,
    active_input: Res<ActiveSettingInput>,
    mut query: Query<(&mut Node, &mut SettingsInnerList)>,
) {
    if active_input.focused_setting.is_none() {
        let mut scroll_dy = 0.0;
        if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
            scroll_dy += 12.0;
        }
        if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
            scroll_dy -= 12.0;
        }
        if scroll_dy != 0.0 {
            for (mut node, mut list) in query.iter_mut() {
                list.scroll_offset = (list.scroll_offset + scroll_dy).clamp(-2100.0, 0.0);
                node.top = Val::Px(list.scroll_offset);
            }
        }
    }
}

// --- Value Helpers ---

fn get_setting_value(
    setting: SettingType,
    physics: &PhysicsSettings,
    _stats: &PersistentPlayerStats,
    p1_base: &P1WeaponSettings,
    p2_base: &P2WeaponSettings,
    kb: &KeyboardControls,
    ctrl: &ControllerControls,
) -> String {
    match setting {
        SettingType::Gravity => format!("{:.0}", physics.gravity),
        SettingType::PlayerAccel => format!("{:.1}", physics.player_accel),
        SettingType::PlayerJumpForce => format!("{:.0}", physics.player_jump_force),
        SettingType::BoundaryRestitution => format!("{:.2}", physics.boundary_restitution),
        SettingType::PlayerRestitution => format!("{:.2}", physics.player_restitution),
        SettingType::AirFriction => format!("{:.2}", physics.air_friction),
        SettingType::GroundFriction => format!("{:.2}", physics.ground_friction),
        SettingType::MovementStopFriction => format!("{:.1}", physics.movement_stop_friction),
        SettingType::WallSlideSpeed => format!("{:.0}", physics.wall_slide_speed),
        SettingType::WallJumpPushForce => format!("{:.0}", physics.wall_jump_push_force),
        SettingType::FastFallAcceleration => format!("{:.0}", physics.fast_fall_acceleration),
        SettingType::AirAccel => format!("{:.1}", physics.air_accel),

        SettingType::P1Health => format!("{:.0}", p1_base.0.health),
        SettingType::P1Speed => format!("{:.0}", p1_base.0.speed),
        SettingType::P1Size => format!("{:.2}", p1_base.0.size),
        SettingType::P1Damage => format!("{:.1}", p1_base.0.damage),
        SettingType::P1BulletRange => format!("{:.2}", p1_base.0.bullet_range),
        SettingType::P1BulletSpeed => format!("{:.0}", p1_base.0.bullet_speed),
        SettingType::P1BulletGravity => format!("{:.0}", p1_base.0.bullet_gravity),
        SettingType::P1BulletSizeMult => format!("{:.2}", p1_base.0.bullet_size_mult),
        SettingType::P1BulletGrowth => format!("{:.1}", p1_base.0.bullet_growth),
        SettingType::P1MaxAmmo => format!("{}", p1_base.0.max_ammo),
        SettingType::P1ReloadTime => format!("{:.2}", p1_base.0.reload_time),
        SettingType::P1FireRate => format!("{:.3}", p1_base.0.fire_rate),
        SettingType::P1Bounces => format!("{}", p1_base.0.bounces),
        SettingType::P1BounceSpeedMultiplier => format!("{:.2}", p1_base.0.bounce_speed_multiplier),
        SettingType::P1BlockDuration => format!("{:.2}", p1_base.0.block_duration),
        SettingType::P1BlockCooldown => format!("{:.2}", p1_base.0.block_cooldown),
        SettingType::P1BlockBorderBoost => format!("{:.0}", p1_base.0.block_border_boost),

        SettingType::P2Health => format!("{:.0}", p2_base.0.health),
        SettingType::P2Speed => format!("{:.0}", p2_base.0.speed),
        SettingType::P2Size => format!("{:.2}", p2_base.0.size),
        SettingType::P2Damage => format!("{:.1}", p2_base.0.damage),
        SettingType::P2BulletRange => format!("{:.2}", p2_base.0.bullet_range),
        SettingType::P2BulletSpeed => format!("{:.0}", p2_base.0.bullet_speed),
        SettingType::P2BulletGravity => format!("{:.0}", p2_base.0.bullet_gravity),
        SettingType::P2BulletSizeMult => format!("{:.2}", p2_base.0.bullet_size_mult),
        SettingType::P2BulletGrowth => format!("{:.1}", p2_base.0.bullet_growth),
        SettingType::P2MaxAmmo => format!("{}", p2_base.0.max_ammo),
        SettingType::P2ReloadTime => format!("{:.2}", p2_base.0.reload_time),
        SettingType::P2FireRate => format!("{:.3}", p2_base.0.fire_rate),
        SettingType::P2Bounces => format!("{}", p2_base.0.bounces),
        SettingType::P2BounceSpeedMultiplier => format!("{:.2}", p2_base.0.bounce_speed_multiplier),
        SettingType::P2BlockDuration => format!("{:.2}", p2_base.0.block_duration),
        SettingType::P2BlockCooldown => format!("{:.2}", p2_base.0.block_cooldown),
        SettingType::P2BlockBorderBoost => format!("{:.0}", p2_base.0.block_border_boost),

        SettingType::KbMoveLeft => kb.move_left.clone(),
        SettingType::KbMoveRight => kb.move_right.clone(),
        SettingType::KbJump => kb.jump.clone(),
        SettingType::KbFastFall => kb.fast_fall.clone(),
        SettingType::KbBlock => kb.block.clone(),
        SettingType::KbShoot => kb.shoot.clone(),
        SettingType::KbReload => kb.reload.clone(),

        SettingType::CtrlJump => ctrl.jump.clone(),
        SettingType::CtrlBlock => ctrl.block.clone(),
        SettingType::CtrlShoot => ctrl.shoot.clone(),
        SettingType::CtrlReload => ctrl.reload.clone(),
    }
}

fn apply_control_setting(
    setting: SettingType,
    val: &str,
    kb: &mut KeyboardControls,
    ctrl: &mut ControllerControls,
) {
    match setting {
        SettingType::KbMoveLeft => kb.move_left = val.to_string(),
        SettingType::KbMoveRight => kb.move_right = val.to_string(),
        SettingType::KbJump => kb.jump = val.to_string(),
        SettingType::KbFastFall => kb.fast_fall = val.to_string(),
        SettingType::KbBlock => kb.block = val.to_string(),
        SettingType::KbShoot => kb.shoot = val.to_string(),
        SettingType::KbReload => kb.reload = val.to_string(),

        SettingType::CtrlJump => ctrl.jump = val.to_string(),
        SettingType::CtrlBlock => ctrl.block = val.to_string(),
        SettingType::CtrlShoot => ctrl.shoot = val.to_string(),
        SettingType::CtrlReload => ctrl.reload = val.to_string(),
        _ => {}
    }
}

fn apply_setting_value(
    setting: SettingType,
    val: f32,
    physics: &mut PhysicsSettings,
    p1_base: &mut P1WeaponSettings,
    p2_base: &mut P2WeaponSettings,
) {
    match setting {
        SettingType::Gravity => physics.gravity = val,
        SettingType::PlayerAccel => physics.player_accel = val,
        SettingType::PlayerJumpForce => physics.player_jump_force = val,
        SettingType::BoundaryRestitution => physics.boundary_restitution = val,
        SettingType::PlayerRestitution => physics.player_restitution = val,
        SettingType::AirFriction => physics.air_friction = val,
        SettingType::GroundFriction => physics.ground_friction = val,
        SettingType::MovementStopFriction => physics.movement_stop_friction = val,
        SettingType::WallSlideSpeed => physics.wall_slide_speed = val,
        SettingType::WallJumpPushForce => physics.wall_jump_push_force = val,
        SettingType::FastFallAcceleration => physics.fast_fall_acceleration = val,
        SettingType::AirAccel => physics.air_accel = val,

        SettingType::P1Health => p1_base.0.health = val,
        SettingType::P1Speed => p1_base.0.speed = val,
        SettingType::P1Size => p1_base.0.size = val,
        SettingType::P1Damage => p1_base.0.damage = val,
        SettingType::P1BulletRange => p1_base.0.bullet_range = val,
        SettingType::P1BulletSpeed => p1_base.0.bullet_speed = val,
        SettingType::P1BulletGravity => p1_base.0.bullet_gravity = val,
        SettingType::P1BulletSizeMult => p1_base.0.bullet_size_mult = val,
        SettingType::P1BulletGrowth => p1_base.0.bullet_growth = val,
        SettingType::P1MaxAmmo => p1_base.0.max_ammo = val.max(1.0) as u32,
        SettingType::P1ReloadTime => p1_base.0.reload_time = val,
        SettingType::P1FireRate => p1_base.0.fire_rate = val,
        SettingType::P1Bounces => p1_base.0.bounces = val.max(0.0) as u32,
        SettingType::P1BounceSpeedMultiplier => p1_base.0.bounce_speed_multiplier = val,
        SettingType::P1BlockDuration => p1_base.0.block_duration = val,
        SettingType::P1BlockCooldown => p1_base.0.block_cooldown = val,
        SettingType::P1BlockBorderBoost => p1_base.0.block_border_boost = val,

        SettingType::P2Health => p2_base.0.health = val,
        SettingType::P2Speed => p2_base.0.speed = val,
        SettingType::P2Size => p2_base.0.size = val,
        SettingType::P2Damage => p2_base.0.damage = val,
        SettingType::P2BulletRange => p2_base.0.bullet_range = val,
        SettingType::P2BulletSpeed => p2_base.0.bullet_speed = val,
        SettingType::P2BulletGravity => p2_base.0.bullet_gravity = val,
        SettingType::P2BulletSizeMult => p2_base.0.bullet_size_mult = val,
        SettingType::P2BulletGrowth => p2_base.0.bullet_growth = val,
        SettingType::P2MaxAmmo => p2_base.0.max_ammo = val.max(1.0) as u32,
        SettingType::P2ReloadTime => p2_base.0.reload_time = val,
        SettingType::P2FireRate => p2_base.0.fire_rate = val,
        SettingType::P2Bounces => p2_base.0.bounces = val.max(0.0) as u32,
        SettingType::P2BounceSpeedMultiplier => p2_base.0.bounce_speed_multiplier = val,
        SettingType::P2BlockDuration => p2_base.0.block_duration = val,
        SettingType::P2BlockCooldown => p2_base.0.block_cooldown = val,
        SettingType::P2BlockBorderBoost => p2_base.0.block_border_boost = val,
        _ => {}
    }
}

/// Completely clears the playfield entities and resets player upgrades to loaded JSON configurations
/// Completely clears the playfield entities and resets player upgrades to loaded configurations
fn reset_and_cleanup_gameplay(
    mut commands: Commands,
    players_q: Query<Entity, With<PlayerStatsComponent>>,
    weapons_q: Query<Entity, With<crate::physics::weapon::Weapon>>,
    bullets_q: Query<Entity, With<crate::physics::weapon::Projectile>>,
    particles_q: Query<Entity, With<crate::physics::particles::Particle>>,
    mut score: ResMut<ScoreTracker>,
    p1_base: Res<P1WeaponSettings>,
    p2_base: Res<P2WeaponSettings>,
    mut persistent_stats: ResMut<PersistentPlayerStats>,
) {
    // 1. Teardown active playfield
    for entity in players_q.iter() { commands.entity(entity).despawn(); }
    for entity in weapons_q.iter() { commands.entity(entity).despawn(); }
    for entity in bullets_q.iter() { commands.entity(entity).despawn(); }
    for entity in particles_q.iter() { commands.entity(entity).despawn(); }

    // 2. Reset scores
    score.p1_wins = 0;
    score.p2_wins = 0;

    // 3. Reset persistent statistics back to starting values from baseline resources
    persistent_stats.p1.movement_speed = p1_base.0.speed;
    persistent_stats.p1.health_max = p1_base.0.health;
    persistent_stats.p1.player_scale = p1_base.0.size;
    persistent_stats.p1.bullet_range = p1_base.0.bullet_range;
    persistent_stats.p1.bullet_speed = p1_base.0.bullet_speed;
    persistent_stats.p1.bullet_gravity = p1_base.0.bullet_gravity;
    persistent_stats.p1.bullet_damage = p1_base.0.damage;
    persistent_stats.p1.bullet_size_mult = p1_base.0.bullet_size_mult;
    persistent_stats.p1.bullet_growth = p1_base.0.bullet_growth;
    persistent_stats.p1.max_ammo = p1_base.0.max_ammo;
    persistent_stats.p1.reload_time = p1_base.0.reload_time;
    persistent_stats.p1.fire_rate = p1_base.0.fire_rate;
    persistent_stats.p1.bounces = p1_base.0.bounces;
    persistent_stats.p1.bounce_speed_multiplier = p1_base.0.bounce_speed_multiplier;
    persistent_stats.p1.block_duration = p1_base.0.block_duration;
    persistent_stats.p1.block_cooldown = p1_base.0.block_cooldown;
    persistent_stats.p1.block_border_boost = p1_base.0.block_border_boost;
    persistent_stats.p1.special_effects = p1_base.0.special_effects.clone();
    persistent_stats.p1.cards.clear();

    persistent_stats.p2.movement_speed = p2_base.0.speed;
    persistent_stats.p2.health_max = p2_base.0.health;
    persistent_stats.p2.player_scale = p2_base.0.size;
    persistent_stats.p2.bullet_range = p2_base.0.bullet_range;
    persistent_stats.p2.bullet_speed = p2_base.0.bullet_speed;
    persistent_stats.p2.bullet_gravity = p2_base.0.bullet_gravity;
    persistent_stats.p2.bullet_damage = p2_base.0.damage;
    persistent_stats.p2.bullet_size_mult = p2_base.0.bullet_size_mult;
    persistent_stats.p2.bullet_growth = p2_base.0.bullet_growth;
    persistent_stats.p2.max_ammo = p2_base.0.max_ammo;
    persistent_stats.p2.reload_time = p2_base.0.reload_time;
    persistent_stats.p2.fire_rate = p2_base.0.fire_rate;
    persistent_stats.p2.bounces = p2_base.0.bounces;
    persistent_stats.p2.bounce_speed_multiplier = p2_base.0.bounce_speed_multiplier;
    persistent_stats.p2.block_duration = p2_base.0.block_duration;
    persistent_stats.p2.block_cooldown = p2_base.0.block_cooldown;
    persistent_stats.p2.block_border_boost = p2_base.0.block_border_boost;
    persistent_stats.p2.special_effects = p2_base.0.special_effects.clone();
    persistent_stats.p2.cards.clear();
}

fn reset_input_delay(mut delay: ResMut<GameplayInputDelay>) {
    delay.0 = 0.2; // 0.2s input delay to absorb click/keyboard releases
}

fn tick_input_delay(time: Res<Time>, mut delay: ResMut<GameplayInputDelay>) {
    if delay.0 > 0.0 {
        delay.0 -= time.delta_secs();
    }
}

#[derive(Component)]
pub struct MatchmakingContainer;

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

fn spawn_online_button(
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
                        
                        state.set(GameState::Matchmaking);
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
