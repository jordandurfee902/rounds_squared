use bevy::prelude::*;
use crate::player::{Player, Health};
use crate::physics::weapon::Projectile;
use crate::settings::{GameState, PersistentPlayerStats};

pub struct CardSelectionPlugin;

impl Plugin for CardSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_player_death.run_if(in_state(GameState::Gameplay)))
           .add_systems(OnEnter(GameState::CardSelection), setup_card_selection)
           .add_systems(OnExit(GameState::CardSelection), cleanup_card_selection)
           .add_systems(Update, (
               card_selection_input,
               draw_card_gizmos,
           ).run_if(in_state(GameState::CardSelection)));
    }
}

#[derive(Resource, Debug, Clone)]
pub struct CardSelectionState {
    pub selected_idx: usize,
    pub selecting_player: Player,
}

#[derive(Component, Debug, Clone)]
pub struct CardSelectionUiComponent {
    pub index: usize,
}

#[derive(Component, Debug, Clone)]
pub struct SelectionHeaderComponent;

#[derive(Debug, Clone)]
pub struct CardDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub stat_lines: &'static [&'static str],
}

pub const CARDS: [CardDef; 5] = [
    CardDef {
        name: "Fast & Light",
        desc: "Trade durability\nfor extreme speed!",
        stat_lines: &[
            "+30% Movement Speed",
            "-20% Max Health",
        ],
    },
    CardDef {
        name: "Tanky Giant",
        desc: "Grow massive and\nabsorb heavy hits.",
        stat_lines: &[
            "+40% Max Health",
            "+30% Player Scale",
            "-15% Jump Force",
        ],
    },
    CardDef {
        name: "Hyper-Shot",
        desc: "Fast-travel high\nvelocity bullet rounds.",
        stat_lines: &[
            "+35% Bullet Speed",
            "+20% Bullet Damage",
            "-1 Max Ammo",
        ],
    },
    CardDef {
        name: "Toxic Spray",
        desc: "Infect opponents with\nneon poison clouds.",
        stat_lines: &[
            "Adds Poison Trail effect",
            "+0.15 Bullet Growth",
            "+2 Max Ammo",
        ],
    },
    CardDef {
        name: "Heavy Artillery",
        desc: "Slow, massive,\nhigh-gravity warheads.",
        stat_lines: &[
            "+80% Bullet Damage",
            "+30% Bullet Size Mult",
            "+200 Downward Gravity",
            "+1.0s Reload Time",
        ],
    },
];

fn check_player_death(
    mut commands: Commands,
    players_query: Query<(Entity, &Player, &Health)>,
    proj_query: Query<Entity, With<Projectile>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut dead_player = None;
    for (_, player, health) in players_query.iter() {
        if health.current <= 0.0 {
            dead_player = Some(*player);
            break;
        }
    }

    if let Some(player_who_died) = dead_player {
        // Despawn both players from map
        for (entity, _, _) in players_query.iter() {
            commands.entity(entity).despawn();
        }

        // Clean up active bullets
        for proj_entity in proj_query.iter() {
            commands.entity(proj_entity).despawn();
        }

        // Dead player gets to choose a card!
        commands.insert_resource(CardSelectionState {
            selected_idx: 2, // highlight middle card by default
            selecting_player: player_who_died,
        });

        // Open the Card Selection UI
        next_state.set(GameState::CardSelection);
    }
}

fn setup_card_selection(
    mut commands: Commands,
    state: Res<CardSelectionState>,
) {
    let x_offsets = [-1140.0, -570.0, 0.0, 570.0, 1140.0];

    // Spawn Title Bounding Prompt
    let title_color = match state.selecting_player {
        Player::P1 => Color::srgb(0.3, 0.8, 1.0), // Blue
        Player::P2 => Color::srgb(1.0, 0.6, 0.2), // Orange
    };
    let title_text = match state.selecting_player {
        Player::P1 => "P1 DEFEATED - CHOOSE A MODIFIER CARD",
        Player::P2 => "P2 DEFEATED - CHOOSE A MODIFIER CARD",
    };

    commands.spawn((
        Text2d::new(title_text),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(title_color),
        Transform::from_xyz(0.0, 390.0, 20.0),
        SelectionHeaderComponent,
    ));

    // Spawn Sub-prompt
    commands.spawn((
        Text2d::new("Use A/D or Arrow Keys to navigate | Space or Enter to select"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        Transform::from_xyz(0.0, 330.0, 20.0),
        SelectionHeaderComponent,
    ));

    // Spawn 5 Card Nodes
    for i in 0..5 {
        let card_def = &CARDS[i];
        let x_pos = x_offsets[i];

        // Apply a highly responsive arched hand layout, rotating OUTWARDS like a fan
        let angle = -(i as f32 - 2.0) * 0.05;
        let y_pos = -20.0 * (i as f32 - 2.0).abs().powi(2) - 100.0;

        commands.spawn((
            CardSelectionUiComponent { index: i },
            Transform::from_xyz(x_pos, y_pos, 15.0).with_rotation(Quat::from_rotation_z(angle)),
            Visibility::default(),
            InheritedVisibility::default(),
        )).with_children(|parent| {
            // Title Text
            parent.spawn((
                Text2d::new(card_def.name),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Transform::from_xyz(0.0, 290.0, 1.0),
            ));

            // Description Text
            parent.spawn((
                Text2d::new(card_def.desc),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Transform::from_xyz(0.0, 150.0, 1.0),
            ));

            // Stat Modifications lines
            for (stat_idx, stat_line) in card_def.stat_lines.iter().enumerate() {
                let y_pos = -30.0 - (stat_idx as f32 * 55.0);
                let text_color = if stat_line.starts_with('+') || stat_line.contains("Adds") {
                    Color::srgb(0.3, 0.9, 0.3) // Glowing neon green for buffs!
                } else {
                    Color::srgb(0.9, 0.3, 0.3) // Neon red for debuffs!
                };

                parent.spawn((
                    Text2d::new(*stat_line),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(text_color),
                    Transform::from_xyz(0.0, y_pos, 1.0),
                ));
            }
        });
    }
}

fn cleanup_card_selection(
    mut commands: Commands,
    ui_nodes: Query<Entity, Or<(With<CardSelectionUiComponent>, With<SelectionHeaderComponent>)>>,
) {
    for entity in ui_nodes.iter() {
        commands.entity(entity).despawn();
    }
}

fn draw_card_gizmos(
    mut gizmos: Gizmos,
    state: Res<CardSelectionState>,
    query: Query<(&Transform, &CardSelectionUiComponent)>,
) {
    let card_size = Vec2::new(465.0, 750.0);

    for (transform, comp) in query.iter() {
        let center = transform.translation.xy();
        let is_hovered = state.selected_idx == comp.index;
        
        // Tilt OUTWARDS to match the fanned cards layout (milder 0.05 angle prevents overlaps)
        let angle = -(comp.index as f32 - 2.0) * 0.05;

        if is_hovered {
            let hover_color = match state.selecting_player {
                Player::P1 => Color::srgb(0.3, 0.8, 1.0), // Blue neon glow for P1 choice
                Player::P2 => Color::srgb(1.0, 0.6, 0.2), // Orange neon glow for P2 choice
            };
            // Double outline swept rotated frames
            draw_rotated_rect(&mut gizmos, center, card_size, angle, hover_color);
            draw_rotated_rect(&mut gizmos, center, card_size + Vec2::new(8.0, 8.0), angle, hover_color);
        } else {
            draw_rotated_rect(&mut gizmos, center, card_size, angle, Color::srgb(0.2, 0.2, 0.2));
        }
    }
}

/// Sweeps 4 rotated line segments manually to achieve gorgeous rotatable card borders.
fn draw_rotated_rect(
    gizmos: &mut Gizmos,
    center: Vec2,
    size: Vec2,
    angle: f32,
    color: Color,
) {
    let hw = size.x / 2.0;
    let hh = size.y / 2.0;

    let c1 = Vec2::new(-hw, -hh);
    let c2 = Vec2::new(hw, -hh);
    let c3 = Vec2::new(hw, hh);
    let c4 = Vec2::new(-hw, hh);

    let rotate = |v: Vec2| {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec2::new(
            v.x * cos_a - v.y * sin_a + center.x,
            v.x * sin_a + v.y * cos_a + center.y,
        )
    };

    let p1 = rotate(c1);
    let p2 = rotate(c2);
    let p3 = rotate(c3);
    let p4 = rotate(c4);

    gizmos.line_2d(p1, p2, color);
    gizmos.line_2d(p2, p3, color);
    gizmos.line_2d(p3, p4, color);
    gizmos.line_2d(p4, p1, color);
}

fn card_selection_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CardSelectionState>,
    mut persistent_stats: ResMut<PersistentPlayerStats>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut step = 0i32;

    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        step = -1;
    }
    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        step = 1;
    }

    if step != 0 {
        if step == -1 {
            state.selected_idx = if state.selected_idx == 0 { 4 } else { state.selected_idx - 1 };
        } else {
            state.selected_idx = (state.selected_idx + 1) % 5;
        }
    }

    // Confirm selection
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyQ) {
        // Apply selected stats modifiers to the dead player
        let p_stats = match state.selecting_player {
            Player::P1 => &mut persistent_stats.p1,
            Player::P2 => &mut persistent_stats.p2,
        };

        match state.selected_idx {
            0 => { // Fast & Light
                p_stats.movement_speed *= 1.30;
                p_stats.health_max *= 0.80;
            }
            1 => { // Tanky Giant
                p_stats.health_max *= 1.40;
                p_stats.player_scale *= 1.30;
                p_stats.jump_force *= 0.85;
            }
            2 => { // Hyper-Shot
                p_stats.bullet_speed *= 1.35;
                p_stats.bullet_damage *= 1.20;
                p_stats.max_ammo = p_stats.max_ammo.saturating_sub(1).max(1);
            }
            3 => { // Toxic Spray
                if !p_stats.special_effects.contains(&"PoisonCloud".to_string()) {
                    p_stats.special_effects.push("PoisonCloud".to_string());
                }
                p_stats.bullet_growth += 0.15;
                p_stats.max_ammo += 2;
            }
            4 => { // Heavy Artillery
                p_stats.bullet_damage *= 1.80;
                p_stats.bullet_size_mult *= 1.30;
                p_stats.bullet_gravity += 200.0;
                p_stats.reload_time += 1.0;
            }
            _ => {}
        }

        // Return to round gameplay
        next_state.set(GameState::Gameplay);
    }
}
