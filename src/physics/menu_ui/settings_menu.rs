use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::prelude::{MessageWriter, MessageReader};
use bevy::input::keyboard::{KeyboardInput, Key};
use crate::settings::{PersistentPlayerStats, GameState, PhysicsSettings, ScoreTracker, AppSettings, KeyboardControls, ControllerControls, P1WeaponSettings, P2WeaponSettings};
use crate::player::PlayerStatsComponent;
use crate::player::{Player, Health};
use crate::physics::weapon::Weapon;
use super::types::*;

pub fn spawn_settings_menu(commands: &mut Commands, is_main_menu: bool) {
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

pub fn settings_value_sync_system(
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

pub fn settings_keyboard_input_system(
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
    stats: &PersistentPlayerStats,
    _p1_base: &P1WeaponSettings,
    _p2_base: &P2WeaponSettings,
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

        SettingType::P1Health => format!("{:.0}", stats.p1.health_max),
        SettingType::P1Speed => format!("{:.0}", stats.p1.movement_speed),
        SettingType::P1Size => format!("{:.2}", stats.p1.player_scale),
        SettingType::P1Damage => format!("{:.1}", stats.p1.bullet_damage),
        SettingType::P1BulletRange => format!("{:.2}", stats.p1.bullet_range),
        SettingType::P1BulletSpeed => format!("{:.0}", stats.p1.bullet_speed),
        SettingType::P1BulletGravity => format!("{:.0}", stats.p1.bullet_gravity),
        SettingType::P1BulletSizeMult => format!("{:.2}", stats.p1.bullet_size_mult),
        SettingType::P1BulletGrowth => format!("{:.1}", stats.p1.bullet_growth),
        SettingType::P1MaxAmmo => format!("{}", stats.p1.max_ammo),
        SettingType::P1ReloadTime => format!("{:.2}", stats.p1.reload_time),
        SettingType::P1FireRate => format!("{:.3}", stats.p1.fire_rate),
        SettingType::P1Bounces => format!("{}", stats.p1.bounces),
        SettingType::P1BounceSpeedMultiplier => format!("{:.2}", stats.p1.bounce_speed_multiplier),
        SettingType::P1BlockDuration => format!("{:.2}", stats.p1.block_duration),
        SettingType::P1BlockCooldown => format!("{:.2}", stats.p1.block_cooldown),
        SettingType::P1BlockBorderBoost => format!("{:.0}", stats.p1.block_border_boost),

        SettingType::P2Health => format!("{:.0}", stats.p2.health_max),
        SettingType::P2Speed => format!("{:.0}", stats.p2.movement_speed),
        SettingType::P2Size => format!("{:.2}", stats.p2.player_scale),
        SettingType::P2Damage => format!("{:.1}", stats.p2.bullet_damage),
        SettingType::P2BulletRange => format!("{:.2}", stats.p2.bullet_range),
        SettingType::P2BulletSpeed => format!("{:.0}", stats.p2.bullet_speed),
        SettingType::P2BulletGravity => format!("{:.0}", stats.p2.bullet_gravity),
        SettingType::P2BulletSizeMult => format!("{:.2}", stats.p2.bullet_size_mult),
        SettingType::P2BulletGrowth => format!("{:.1}", stats.p2.bullet_growth),
        SettingType::P2MaxAmmo => format!("{}", stats.p2.max_ammo),
        SettingType::P2ReloadTime => format!("{:.2}", stats.p2.reload_time),
        SettingType::P2FireRate => format!("{:.3}", stats.p2.fire_rate),
        SettingType::P2Bounces => format!("{}", stats.p2.bounces),
        SettingType::P2BounceSpeedMultiplier => format!("{:.2}", stats.p2.bounce_speed_multiplier),
        SettingType::P2BlockDuration => format!("{:.2}", stats.p2.block_duration),
        SettingType::P2BlockCooldown => format!("{:.2}", stats.p2.block_cooldown),
        SettingType::P2BlockBorderBoost => format!("{:.0}", stats.p2.block_border_boost),

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

pub fn reset_and_cleanup_gameplay(
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
