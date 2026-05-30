use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::{MessageWriter, MessageReader};
use bevy::input::keyboard::{KeyboardInput, Key};
use crate::settings::{PersistentPlayerStats, GameState, PhysicsSettings, ScoreTracker, AppSettings, KeyboardControls, ControllerControls, PlayerWeaponSettings};
use crate::player::PlayerStatsComponent;
use crate::player::{Player, Health};
use crate::physics::weapon::Weapon;
use super::types::*;

pub fn spawn_settings_menu(commands: &mut Commands, is_main_menu: bool, controls_only: bool) {
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
                    if !controls_only {
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
                        spawn_setting_row(list, "Player Base Radius", SettingType::PlayerBaseRadius);
                        spawn_setting_row(list, "Player Base Mass", SettingType::PlayerBaseMass);
                        spawn_setting_row(list, "Player Visual Offset", SettingType::PlayerVisualOffset);
                        spawn_setting_row(list, "Player Aim Offset Y", SettingType::PlayerAimOffsetY);
                        spawn_setting_row(list, "Boundary Knockback Speed", SettingType::BoundaryKnockbackSpeed);
                        spawn_setting_row(list, "Boundary Damage Lockout (s)", SettingType::BoundaryDamageLockout);
                        spawn_setting_row(list, "Boundary Deflect Lockout (s)", SettingType::BoundaryDeflectLockout);
                        spawn_setting_row(list, "Spawn Grace Period (s)", SettingType::SpawnInvincibilityGracePeriod);
                        spawn_setting_row(list, "Boundary Hazard Damage", SettingType::BoundaryHazardDamage);
                        spawn_setting_row(list, "Fast Fall Stick Threshold", SettingType::FastFallStickThreshold);
                        spawn_setting_row(list, "Fast Fall Velocity Limit", SettingType::FastFallVelocityLimit);
                        spawn_setting_row(list, "Wall Cling Stick Threshold", SettingType::WallClingStickThreshold);
                        spawn_setting_row(list, "Max Jump Allowance", SettingType::MaxJumpAllowance);
                        spawn_setting_row(list, "Collision Skin Buffer", SettingType::CollisionPenetrationSkinBuffer);
                        spawn_setting_row(list, "Overlapping Push Factor", SettingType::OverlappingPushFactor);
                        spawn_setting_row(list, "Grounded Slope Threshold", SettingType::GroundedSlopeThreshold);
                        spawn_setting_row(list, "Wall Contact Slope Threshold", SettingType::WallContactSlopeThreshold);
                        spawn_setting_row(list, "Bullet Knockback Constant", SettingType::BulletKnockbackConstant);

                        for i in 1..=8 {
                            spawn_section_header(list, &format!("PLAYER {} CHARACTER STATISTICS", i));
                            spawn_setting_row(list, "Max Health", SettingType::PlayerHealth(i - 1));
                            spawn_setting_row(list, "Movement Speed", SettingType::PlayerSpeed(i - 1));
                            spawn_setting_row(list, "Player Scale Multiplier", SettingType::PlayerSize(i - 1));
                            spawn_setting_row(list, "Bullet Damage", SettingType::PlayerDamage(i - 1));
                            spawn_setting_row(list, "Bullet Range (seconds)", SettingType::PlayerBulletRange(i - 1));
                            spawn_setting_row(list, "Bullet Velocity", SettingType::PlayerBulletSpeed(i - 1));
                            spawn_setting_row(list, "Bullet Gravity Scale", SettingType::PlayerBulletGravity(i - 1));
                            spawn_setting_row(list, "Bullet Size Multiplier", SettingType::PlayerBulletSizeMult(i - 1));
                            spawn_setting_row(list, "Bullet Damage Growth (%)", SettingType::PlayerBulletGrowth(i - 1));
                            spawn_setting_row(list, "Max Ammo Capacity", SettingType::PlayerMaxAmmo(i - 1));
                            spawn_setting_row(list, "Weapon Reload Time", SettingType::PlayerReloadTime(i - 1));
                            spawn_setting_row(list, "Rate of Fire Delay", SettingType::PlayerFireRate(i - 1));
                            spawn_setting_row(list, "Max Bullet Bounces", SettingType::PlayerBounces(i - 1));
                            spawn_setting_row(list, "Bounce Speed Multiplier", SettingType::PlayerBounceSpeedMultiplier(i - 1));
                            spawn_setting_row(list, "Shield Block Duration", SettingType::PlayerBlockDuration(i - 1));
                            spawn_setting_row(list, "Shield Block Cooldown", SettingType::PlayerBlockCooldown(i - 1));
                            spawn_setting_row(list, "Border Block Force", SettingType::PlayerBlockBorderBoost(i - 1));
                        }
                    }

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

pub fn spawn_button(
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

pub fn spawn_section_header(builder: &mut ChildSpawnerCommands, title: &str) {
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

pub fn spawn_setting_row(
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

pub fn save_and_sync_settings(
    physics_settings: &PhysicsSettings,
    p_weapon: &PlayerWeaponSettings,
    kb_controls: &KeyboardControls,
    ctrl_controls: &ControllerControls,
    persistent_stats: &mut PersistentPlayerStats,
    players_q: &mut Query<(&Player, &mut Health, &mut PlayerStatsComponent, &mut Weapon)>,
) {
    let updated_settings = AppSettings {
        physics: physics_settings.clone(),
        p1_character: p_weapon.0[0].clone(),
        p2_character: p_weapon.0[1].clone(),
        p3_character: p_weapon.0[2].clone(),
        p4_character: p_weapon.0[3].clone(),
        p5_character: p_weapon.0[4].clone(),
        p6_character: p_weapon.0[5].clone(),
        p7_character: p_weapon.0[6].clone(),
        p8_character: p_weapon.0[7].clone(),
        keyboard_controls: kb_controls.clone(),
        controller_controls: ctrl_controls.clone(),
    };
    crate::settings::save_settings(&updated_settings);

    // Sync base fields to active round stats (preserving cards/special effects)
    for i in 0..8 {
        let char_settings = &p_weapon.0[i];
        persistent_stats.players[i].movement_speed = char_settings.speed;
        persistent_stats.players[i].health_max = char_settings.health;
        persistent_stats.players[i].player_scale = char_settings.size;
        persistent_stats.players[i].bullet_range = char_settings.bullet_range;
        persistent_stats.players[i].bullet_speed = char_settings.bullet_speed;
        persistent_stats.players[i].bullet_gravity = char_settings.bullet_gravity;
        persistent_stats.players[i].bullet_damage = char_settings.damage;
        persistent_stats.players[i].bullet_size_mult = char_settings.bullet_size_mult;
        persistent_stats.players[i].bullet_growth = char_settings.bullet_growth;
        persistent_stats.players[i].max_ammo = char_settings.max_ammo;
        persistent_stats.players[i].reload_time = char_settings.reload_time;
        persistent_stats.players[i].fire_rate = char_settings.fire_rate;
        persistent_stats.players[i].bounces = char_settings.bounces;
        persistent_stats.players[i].bounce_speed_multiplier = char_settings.bounce_speed_multiplier;
        persistent_stats.players[i].block_duration = char_settings.block_duration;
        persistent_stats.players[i].block_cooldown = char_settings.block_cooldown;
        persistent_stats.players[i].block_border_boost = char_settings.block_border_boost;
    }

    // Dynamic entity-component synchronization
    for (player, mut health, mut stats, mut weapon) in players_q.iter_mut() {
        let p_idx = player.index();
        let p_stats = &persistent_stats.players[p_idx];

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

pub fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, &mut BackgroundColor, Option<&MenuButton>, Option<&SettingInputBox>),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<NextState<GameState>>,
    mut paused: ResMut<Paused>,
    mut menu: ResMut<ActiveMenu>,
    mut exit: MessageWriter<AppExit>,
    mut physics_settings: ResMut<PhysicsSettings>,
    mut p_weapon: ResMut<PlayerWeaponSettings>,
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
                        save_and_sync_settings(&physics_settings, &p_weapon, &kb_controls, &ctrl_controls, &mut persistent_stats, &mut players_q);
                    } else if let Ok(val) = active_input.current_text.parse::<f32>() {
                        apply_setting_value(setting, val, &mut physics_settings, &mut p_weapon);
                        save_and_sync_settings(&physics_settings, &p_weapon, &kb_controls, &ctrl_controls, &mut persistent_stats, &mut players_q);
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
                    active_input.current_text = get_setting_value(in_box.0, &physics_settings, &persistent_stats, &p_weapon, &kb_controls, &ctrl_controls);
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

pub fn settings_value_sync_system(
    physics_settings: Res<PhysicsSettings>,
    persistent_stats: Res<PersistentPlayerStats>,
    p_weapon: Res<PlayerWeaponSettings>,
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
            text.0 = get_setting_value(setting_type, &physics_settings, &persistent_stats, &p_weapon, &kb_controls, &ctrl_controls);
            
            // Standard border
            if let Ok(mut border) = border_q.get_mut(parent.parent()) {
                *border = BorderColor::all(Color::srgb(0.2, 0.2, 0.2));
            }
        }
    }
}

pub fn settings_keyboard_input_system(
    mut events: MessageReader<KeyboardInput>,
    mut active_input: ResMut<ActiveSettingInput>,
    mut physics_settings: ResMut<PhysicsSettings>,
    mut p_weapon: ResMut<PlayerWeaponSettings>,
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
                        save_and_sync_settings(&physics_settings, &p_weapon, &kb_controls, &ctrl_controls, &mut persistent_stats, &mut players_q);
                    } else if let Ok(val) = active_input.current_text.parse::<f32>() {
                        // Apply and sync instantly
                        apply_setting_value(setting, val, &mut physics_settings, &mut p_weapon);
                        save_and_sync_settings(&physics_settings, &p_weapon, &kb_controls, &ctrl_controls, &mut persistent_stats, &mut players_q);
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

pub fn settings_scroll_system(
    mut mouse_wheel_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<(&mut Node, &mut SettingsInnerList)>,
) {
    let mut scroll_dy = 0.0;
    for event in mouse_wheel_events.read() {
        scroll_dy += event.y * 30.0; // 30px per scroll tick
    }
    if scroll_dy != 0.0 {
        for (mut node, mut list) in query.iter_mut() {
            list.scroll_offset = (list.scroll_offset + scroll_dy).clamp(-3200.0, 0.0);
            node.top = Val::Px(list.scroll_offset);
        }
    }
}

pub fn settings_keyboard_scroll_system(
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
                list.scroll_offset = (list.scroll_offset + scroll_dy).clamp(-3200.0, 0.0);
                node.top = Val::Px(list.scroll_offset);
            }
        }
    }
}

pub fn get_setting_value(
    setting: SettingType,
    physics: &PhysicsSettings,
    _stats: &PersistentPlayerStats,
    p_weapon: &PlayerWeaponSettings,
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
        SettingType::PlayerBaseRadius => format!("{:.1}", physics.player_base_radius),
        SettingType::PlayerBaseMass => format!("{:.2}", physics.player_base_mass),
        SettingType::PlayerVisualOffset => format!("{:.1}", physics.player_visual_offset),
        SettingType::PlayerAimOffsetY => format!("{:.1}", physics.player_aim_offset_y),
        SettingType::BoundaryKnockbackSpeed => format!("{:.0}", physics.boundary_knockback_speed),
        SettingType::BoundaryDamageLockout => format!("{:.2}", physics.boundary_damage_lockout),
        SettingType::BoundaryDeflectLockout => format!("{:.2}", physics.boundary_deflect_lockout),
        SettingType::SpawnInvincibilityGracePeriod => format!("{:.2}", physics.spawn_invincibility_grace_period),
        SettingType::BoundaryHazardDamage => format!("{:.1}", physics.boundary_hazard_damage),
        SettingType::FastFallStickThreshold => format!("{:.2}", physics.fast_fall_stick_threshold),
        SettingType::FastFallVelocityLimit => format!("{:.1}", physics.fast_fall_velocity_limit),
        SettingType::WallClingStickThreshold => format!("{:.2}", physics.wall_cling_stick_threshold),
        SettingType::MaxJumpAllowance => format!("{}", physics.max_jump_allowance),
        SettingType::CollisionPenetrationSkinBuffer => format!("{:.2}", physics.collision_penetration_skin_buffer),
        SettingType::OverlappingPushFactor => format!("{:.2}", physics.overlapping_push_factor),
        SettingType::GroundedSlopeThreshold => format!("{:.2}", physics.grounded_slope_threshold),
        SettingType::WallContactSlopeThreshold => format!("{:.2}", physics.wall_contact_slope_threshold),
        SettingType::BulletKnockbackConstant => format!("{:.1}", physics.bullet_knockback_constant),

        SettingType::PlayerHealth(idx) => format!("{:.0}", p_weapon.0[idx].health),
        SettingType::PlayerSpeed(idx) => format!("{:.0}", p_weapon.0[idx].speed),
        SettingType::PlayerSize(idx) => format!("{:.2}", p_weapon.0[idx].size),
        SettingType::PlayerDamage(idx) => format!("{:.1}", p_weapon.0[idx].damage),
        SettingType::PlayerBulletRange(idx) => format!("{:.2}", p_weapon.0[idx].bullet_range),
        SettingType::PlayerBulletSpeed(idx) => format!("{:.0}", p_weapon.0[idx].bullet_speed),
        SettingType::PlayerBulletGravity(idx) => format!("{:.0}", p_weapon.0[idx].bullet_gravity),
        SettingType::PlayerBulletSizeMult(idx) => format!("{:.2}", p_weapon.0[idx].bullet_size_mult),
        SettingType::PlayerBulletGrowth(idx) => format!("{:.1}", p_weapon.0[idx].bullet_growth),
        SettingType::PlayerMaxAmmo(idx) => format!("{}", p_weapon.0[idx].max_ammo),
        SettingType::PlayerReloadTime(idx) => format!("{:.2}", p_weapon.0[idx].reload_time),
        SettingType::PlayerFireRate(idx) => format!("{:.3}", p_weapon.0[idx].fire_rate),
        SettingType::PlayerBounces(idx) => format!("{}", p_weapon.0[idx].bounces),
        SettingType::PlayerBounceSpeedMultiplier(idx) => format!("{:.2}", p_weapon.0[idx].bounce_speed_multiplier),
        SettingType::PlayerBlockDuration(idx) => format!("{:.2}", p_weapon.0[idx].block_duration),
        SettingType::PlayerBlockCooldown(idx) => format!("{:.2}", p_weapon.0[idx].block_cooldown),
        SettingType::PlayerBlockBorderBoost(idx) => format!("{:.0}", p_weapon.0[idx].block_border_boost),

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

pub fn apply_control_setting(
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

pub fn apply_setting_value(
    setting: SettingType,
    val: f32,
    physics: &mut PhysicsSettings,
    p_weapon: &mut PlayerWeaponSettings,
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
        SettingType::PlayerBaseRadius => physics.player_base_radius = val,
        SettingType::PlayerBaseMass => physics.player_base_mass = val,
        SettingType::PlayerVisualOffset => physics.player_visual_offset = val,
        SettingType::PlayerAimOffsetY => physics.player_aim_offset_y = val,
        SettingType::BoundaryKnockbackSpeed => physics.boundary_knockback_speed = val,
        SettingType::BoundaryDamageLockout => physics.boundary_damage_lockout = val,
        SettingType::BoundaryDeflectLockout => physics.boundary_deflect_lockout = val,
        SettingType::SpawnInvincibilityGracePeriod => physics.spawn_invincibility_grace_period = val,
        SettingType::BoundaryHazardDamage => physics.boundary_hazard_damage = val,
        SettingType::FastFallStickThreshold => physics.fast_fall_stick_threshold = val,
        SettingType::FastFallVelocityLimit => physics.fast_fall_velocity_limit = val,
        SettingType::WallClingStickThreshold => physics.wall_cling_stick_threshold = val,
        SettingType::MaxJumpAllowance => physics.max_jump_allowance = val.max(0.0) as u32,
        SettingType::CollisionPenetrationSkinBuffer => physics.collision_penetration_skin_buffer = val,
        SettingType::OverlappingPushFactor => physics.overlapping_push_factor = val,
        SettingType::GroundedSlopeThreshold => physics.grounded_slope_threshold = val,
        SettingType::WallContactSlopeThreshold => physics.wall_contact_slope_threshold = val,
        SettingType::BulletKnockbackConstant => physics.bullet_knockback_constant = val,

        SettingType::PlayerHealth(idx) => p_weapon.0[idx].health = val,
        SettingType::PlayerSpeed(idx) => p_weapon.0[idx].speed = val,
        SettingType::PlayerSize(idx) => p_weapon.0[idx].size = val,
        SettingType::PlayerDamage(idx) => p_weapon.0[idx].damage = val,
        SettingType::PlayerBulletRange(idx) => p_weapon.0[idx].bullet_range = val,
        SettingType::PlayerBulletSpeed(idx) => p_weapon.0[idx].bullet_speed = val,
        SettingType::PlayerBulletGravity(idx) => p_weapon.0[idx].bullet_gravity = val,
        SettingType::PlayerBulletSizeMult(idx) => p_weapon.0[idx].bullet_size_mult = val,
        SettingType::PlayerBulletGrowth(idx) => p_weapon.0[idx].bullet_growth = val,
        SettingType::PlayerMaxAmmo(idx) => p_weapon.0[idx].max_ammo = val.max(1.0) as u32,
        SettingType::PlayerReloadTime(idx) => p_weapon.0[idx].reload_time = val,
        SettingType::PlayerFireRate(idx) => p_weapon.0[idx].fire_rate = val,
        SettingType::PlayerBounces(idx) => p_weapon.0[idx].bounces = val.max(0.0) as u32,
        SettingType::PlayerBounceSpeedMultiplier(idx) => p_weapon.0[idx].bounce_speed_multiplier = val,
        SettingType::PlayerBlockDuration(idx) => p_weapon.0[idx].block_duration = val,
        SettingType::PlayerBlockCooldown(idx) => p_weapon.0[idx].block_cooldown = val,
        SettingType::PlayerBlockBorderBoost(idx) => p_weapon.0[idx].block_border_boost = val,
        _ => {}
    }
}

pub fn reset_and_cleanup_gameplay(
    mut commands: Commands,
    players_q: Query<Entity, With<PlayerStatsComponent>>,
    weapons_q: Query<Entity, With<crate::physics::weapon::Weapon>>,
    bullets_q: Query<Entity, With<crate::physics::weapon::Projectile>>,
    particles_q: Query<Entity, With<crate::physics::particles::Particle>>,
    mut score: ResMut<ScoreTracker>,
    p_weapon: Res<PlayerWeaponSettings>,
    mut persistent_stats: ResMut<PersistentPlayerStats>,
) {
    // 1. Teardown active playfield
    for entity in players_q.iter() { commands.entity(entity).despawn(); }
    for entity in weapons_q.iter() { commands.entity(entity).despawn(); }
    for entity in bullets_q.iter() { commands.entity(entity).despawn(); }
    for entity in particles_q.iter() { commands.entity(entity).despawn(); }

    // 2. Reset scores
    score.wins = [0; 8];

    // 3. Reset persistent statistics back to starting values from baseline resources
    for i in 0..8 {
        let char_settings = &p_weapon.0[i];
        persistent_stats.players[i].movement_speed = char_settings.speed;
        persistent_stats.players[i].health_max = char_settings.health;
        persistent_stats.players[i].player_scale = char_settings.size;
        persistent_stats.players[i].bullet_range = char_settings.bullet_range;
        persistent_stats.players[i].bullet_speed = char_settings.bullet_speed;
        persistent_stats.players[i].bullet_gravity = char_settings.bullet_gravity;
        persistent_stats.players[i].bullet_damage = char_settings.damage;
        persistent_stats.players[i].bullet_size_mult = char_settings.bullet_size_mult;
        persistent_stats.players[i].bullet_growth = char_settings.bullet_growth;
        persistent_stats.players[i].max_ammo = char_settings.max_ammo;
        persistent_stats.players[i].reload_time = char_settings.reload_time;
        persistent_stats.players[i].fire_rate = char_settings.fire_rate;
        persistent_stats.players[i].bounces = char_settings.bounces;
        persistent_stats.players[i].bounce_speed_multiplier = char_settings.bounce_speed_multiplier;
        persistent_stats.players[i].block_duration = char_settings.block_duration;
        persistent_stats.players[i].block_cooldown = char_settings.block_cooldown;
        persistent_stats.players[i].block_border_boost = char_settings.block_border_boost;
        persistent_stats.players[i].special_effects = char_settings.special_effects.clone();
        persistent_stats.players[i].cards.clear();
    }
}
