use bevy::prelude::*;
use crate::player::{Player, Health};
use crate::physics::weapon::Projectile;
use crate::settings::{GameState, PersistentPlayerStats};
use super::defs::*;

pub fn check_player_death(
    mut commands: Commands,
    players_query: Query<(Entity, &Player, &Health)>,
    proj_query: Query<Entity, With<Projectile>>,
    particle_query: Query<Entity, With<crate::physics::particles::Particle>>,
    physics_query: Query<Entity, Or<(With<crate::physics::Platform>, With<crate::physics::components::MovingPlatform>, With<crate::physics::components::PhysicsObject>, With<crate::physics::components::RopeSwing>)>>,
    mut score: ResMut<crate::settings::ScoreTracker>,
    mut active_map: ResMut<crate::maps::ActiveMap>,
    mut next_state: ResMut<NextState<GameState>>,
    mut rollback_rng: ResMut<crate::net::RollbackRng>,
) {
    let total_players = players_query.iter().count();
    let alive_players: Vec<Player> = players_query.iter()
        .filter(|(_, _, health)| health.current > 0.0)
        .map(|(_, &player, _)| player)
        .collect();

    let round_over = if total_players >= 2 {
        alive_players.len() <= 1
    } else {
        alive_players.is_empty()
    };

    if round_over {
        let winner = if alive_players.len() == 1 { Some(alive_players[0]) } else { None };
        if let Some(w) = winner {
            score.wins[w.index()] += 1;
        }

        // Deterministically select a new map using the synchronized RollbackRng seed!
        let rand_val = (rollback_rng.next_f32() * 13.0) as u32;
        let selected_map = match rand_val {
            0 => crate::maps::ActiveMap::DefaultMap,
            1 => crate::maps::ActiveMap::PillarsMap,
            2 => crate::maps::ActiveMap::StadiumMap,
            3 => crate::maps::ActiveMap::ChasmBridge,
            4 => crate::maps::ActiveMap::Gridlock,
            5 => crate::maps::ActiveMap::Hourglass,
            6 => crate::maps::ActiveMap::IceTemple,
            7 => crate::maps::ActiveMap::IndustrialFoundry,
            8 => crate::maps::ActiveMap::VerticalHelix,
            9 => crate::maps::ActiveMap::TectonicFissure,
            10 => crate::maps::ActiveMap::ZenGarden,
            11 => crate::maps::ActiveMap::SpaceStation,
            _ => crate::maps::ActiveMap::AncientColiseum,
        };
        *active_map = selected_map;

        // Despawn both players from map
        for (entity, _, _) in players_query.iter() {
            commands.entity(entity).despawn();
        }

        // Clean up active bullets
        for proj_entity in proj_query.iter() {
            commands.entity(proj_entity).despawn();
        }

        // Sweep and despawn every single particle effect!
        for particle_entity in particle_query.iter() {
            commands.entity(particle_entity).despawn();
        }

        // Clean up all physics objects, platforms, moving platforms, and swing ropes
        for entity in physics_query.iter() {
            commands.entity(entity).despawn();
        }

        let all_players: Vec<Player> = players_query.iter().map(|(_, &p, _)| p).collect();
        let defeated: Vec<Player> = all_players.iter()
            .filter(|&&p| Some(p) != winner)
            .cloned()
            .collect();

        if defeated.is_empty() {
            next_state.set(GameState::Gameplay);
        } else {
            // Draw 5 unique random card indices
            let mut drawn = [0; 5];
            let mut available: Vec<usize> = (0..super::cards::TOTAL_CARDS_COUNT).collect();
            for i in 0..5 {
                if available.is_empty() {
                    break;
                }
                let idx = (rollback_rng.next_f32() * available.len() as f32) as usize;
                drawn[i] = available.remove(idx);
            }

            let mut queue = defeated.clone();
            let first_defeated = queue.remove(0);

            commands.insert_resource(CardSelectionState {
                selected_idx: 2, // highlight middle card by default
                selecting_player: first_defeated,
                drawn_cards: drawn,
                queue,
            });

            // Open the Card Selection UI
            next_state.set(GameState::CardSelection);
        }
    }
}

pub fn spawn_card_selection_ui(
    commands: &mut Commands,
    state: &CardSelectionState,
) {
    let x_offsets = [-2280.0, -1140.0, 0.0, 1140.0, 2280.0];

    let title_color = state.selecting_player.color();
    let title_text = format!("{:?} WAS DEFEATED - SELECT A CARD", state.selecting_player);

    commands.spawn((
        Text2d::new(title_text),
        TextFont {
            font_size: 180.0,
            ..default()
        },
        TextColor(title_color),
        Transform::from_xyz(0.0, 780.0, 20.0),
        SelectionHeaderComponent,
    ));

    // Spawn 5 Card Nodes
    for i in 0..5 {
        let card_idx = state.drawn_cards[i];
        let card_def = super::cards::get_card(card_idx).unwrap();
        let x_pos = x_offsets[i];

        // Apply a highly responsive arched hand layout, rotating OUTWARDS like a fan
        let angle = -(i as f32 - 2.0) * 0.05;
        let y_pos = -40.0 * (i as f32 - 2.0).abs().powi(2) - 200.0;

        commands.spawn((
            CardSelectionUiComponent { index: i },
            Transform::from_xyz(x_pos, y_pos, 15.0).with_rotation(Quat::from_rotation_z(angle)),
            Visibility::default(),
            InheritedVisibility::default(),
        )).with_children(|parent| {
            // Title Text
            parent.spawn((
                Text2d::new(card_def.name()),
                TextFont {
                    font_size: 84.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Transform::from_xyz(0.0, 580.0, 1.0),
            ));

            // Description Text
            parent.spawn((
                Text2d::new(card_def.desc()),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Transform::from_xyz(0.0, 300.0, 1.0),
            ));

            // Stat Modifications lines
            for (stat_idx, stat_line) in card_def.stat_lines().iter().enumerate() {
                let y_pos = -60.0 - (stat_idx as f32 * 110.0);
                let text_color = if stat_line.starts_with('+') || stat_line.contains("Adds") {
                    Color::srgb(0.3, 0.9, 0.3) // Glowing neon green for buffs!
                } else {
                    Color::srgb(0.9, 0.3, 0.3) // Neon red for debuffs!
                };

                parent.spawn((
                    Text2d::new(*stat_line),
                    TextFont {
                        font_size: 52.0,
                        ..default()
                    },
                    TextColor(text_color),
                    Transform::from_xyz(0.0, y_pos, 1.0),
                ));
            }
        });
    }
}

pub fn setup_card_selection(
    mut commands: Commands,
    state: Res<CardSelectionState>,
) {
    spawn_card_selection_ui(&mut commands, &state);
}

pub fn cleanup_card_selection(
    mut commands: Commands,
    ui_nodes: Query<Entity, Or<(With<CardSelectionUiComponent>, With<SelectionHeaderComponent>)>>,
) {
    for entity in ui_nodes.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn draw_card_gizmos(
    mut gizmos: Gizmos,
    state: Res<CardSelectionState>,
    query: Query<(&Transform, &CardSelectionUiComponent)>,
) {
    let card_size = Vec2::new(930.0, 1500.0);

    for (transform, comp) in query.iter() {
        let center = transform.translation.xy();
        let is_hovered = state.selected_idx == comp.index;
        
        // Tilt OUTWARDS to match the fanned cards layout (milder 0.05 angle prevents overlaps)
        let angle = -(comp.index as f32 - 2.0) * 0.05;

        if is_hovered {
            let hover_color = state.selecting_player.color();
            let scaled_size = card_size * transform.scale.xy();
            // Double outline swept rotated frames
            draw_rotated_rect(&mut gizmos, center, scaled_size, angle, hover_color);
            draw_rotated_rect(&mut gizmos, center, scaled_size + Vec2::new(16.0, 16.0) * transform.scale.xy(), angle, hover_color);
        } else {
            let scaled_size = card_size * transform.scale.xy();
            draw_rotated_rect(&mut gizmos, center, scaled_size, angle, Color::srgb(0.2, 0.2, 0.2));
        }
    }
}

/// Sweeps 4 rotated line segments manually to achieve gorgeous rotatable card borders.
pub fn draw_rotated_rect(
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

pub fn card_selection_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
    lobby_slots: Res<crate::settings::LobbySlots>,
    time: Res<Time>,
    mut gamepad_cooldown: Local<f32>,
    mut state: ResMut<CardSelectionState>,
    mut persistent_stats: ResMut<PersistentPlayerStats>,
    mut next_state: ResMut<NextState<GameState>>,
    ui_nodes: Query<Entity, Or<(With<CardSelectionUiComponent>, With<SelectionHeaderComponent>)>>,
    mut rollback_rng: ResMut<crate::net::RollbackRng>,
) {
    if *gamepad_cooldown > 0.0 {
        *gamepad_cooldown = (*gamepad_cooldown - time.delta_secs()).max(0.0);
    }

    let mut step = 0i32;
    let mut confirm = false;

    let selecting_device = if state.selecting_player.index() < lobby_slots.slots.len() {
        &lobby_slots.slots[state.selecting_player.index()]
    } else {
        &None
    };

    match selecting_device {
        Some(crate::settings::InputDevice::KeyboardMouse) => {
            if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
                step = -1;
            }
            if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
                step = 1;
            }
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyQ) {
                confirm = true;
            }
        }
        Some(crate::settings::InputDevice::Gamepad(gp_entity)) => {
            if let Ok((_, gp)) = gamepads.get(*gp_entity) {
                let left_stick = gp.left_stick();
                if *gamepad_cooldown <= 0.0 {
                    if left_stick.x < -0.5 {
                        step = -1;
                        *gamepad_cooldown = 0.25;
                    } else if left_stick.x > 0.5 {
                        step = 1;
                        *gamepad_cooldown = 0.25;
                    }
                }
                if gp.just_pressed(GamepadButton::DPadLeft) {
                    step = -1;
                }
                if gp.just_pressed(GamepadButton::DPadRight) {
                    step = 1;
                }
                if gp.just_pressed(GamepadButton::South) {
                    confirm = true;
                }
            }
        }
        None => {
            // Fallback (Keyboard/Mouse)
            if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
                step = -1;
            }
            if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
                step = 1;
            }
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyQ) {
                confirm = true;
            }
        }
    }

    if step != 0 {
        if step == -1 {
            state.selected_idx = if state.selected_idx == 0 { 4 } else { state.selected_idx - 1 };
        } else {
            state.selected_idx = (state.selected_idx + 1) % 5;
        }
    }

    if confirm {
        // Apply selected stats modifiers to the selecting player
        let p_idx = state.selecting_player.index();
        let p_stats = &mut persistent_stats.players[p_idx];
        let card_idx = state.drawn_cards[state.selected_idx];
        p_stats.cards.push(card_idx);

        if let Some(card) = super::cards::get_card(card_idx) {
            card.apply(p_stats);
        }

        // Check if there are more players in the queue
        if !state.queue.is_empty() {
            // Despawn old card selection UI
            for entity in ui_nodes.iter() {
                commands.entity(entity).despawn();
            }

            // Pop next player from queue
            let next_player = state.queue.remove(0);
            state.selecting_player = next_player;
            state.selected_idx = 2;

            // Draw 5 new unique random card indices
            let mut drawn = [0; 5];
            let mut available: Vec<usize> = (0..super::cards::TOTAL_CARDS_COUNT).collect();
            for i in 0..5 {
                if available.is_empty() {
                    break;
                }
                let idx = (rollback_rng.next_f32() * available.len() as f32) as usize;
                drawn[i] = available.remove(idx);
            }
            state.drawn_cards = drawn;

            // Spawn new card selection UI in-place
            spawn_card_selection_ui(&mut commands, &state);
        } else {
            // No more players, return to gameplay
            next_state.set(GameState::Gameplay);
        }
    }
}

pub fn animate_card_selection(
    time: Res<Time>,
    state: Option<Res<CardSelectionState>>,
    mut query: Query<(&mut Transform, &CardSelectionUiComponent)>,
) {
    let Some(state) = state else { return; };
    let x_offsets = [-2280.0, -1140.0, 0.0, 1140.0, 2280.0];

    for (mut transform, comp) in query.iter_mut() {
        let i = comp.index;
        let x_pos = x_offsets[i];

        // Base fanned layout angle and Y position
        let angle = -(i as f32 - 2.0) * 0.05;
        let y_pos = -40.0 * (i as f32 - 2.0).abs().powi(2) - 200.0;

        let is_hovered = state.selected_idx == i;

        // Hover scale lerp
        let target_scale = if is_hovered { 1.08 } else { 1.0 };
        transform.scale = transform.scale.lerp(Vec3::splat(target_scale), time.delta_secs() * 15.0);

        transform.translation.x = x_pos;
        transform.translation.y = y_pos;
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
