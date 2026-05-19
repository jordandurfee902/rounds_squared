pub mod components;
pub mod forces;
pub mod collision;
pub mod anim;
pub mod weapon;
pub mod particles;
pub mod card_selection;
pub mod card_list_ui;
pub mod menu_ui;
pub mod lobby_ui;

pub use components::*;
pub use forces::*;
pub use collision::*;
pub use anim::*;

use bevy::prelude::*;
use crate::settings::GameState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(card_selection::CardSelectionPlugin);
        app.add_plugins(card_list_ui::CardListUiPlugin);
        app.add_plugins(menu_ui::MenuUiPlugin);
        app.add_plugins(lobby_ui::LobbyUiPlugin);

        // Core physics simulation & firing updates (suspended when paused)
        // Runs in Local Play OR on the Host (P1)
        app.add_systems(Update, (
            apply_acceleration,
            apply_gravity_and_movement,
            player_movement,
            crate::player::player_block_system,
            apply_friction,
            apply_velocity,
            reset_collision_states,
            boundary_collision,
            player_collision,
            player_platform_collision,
            weapon::weapon_update_system,
            weapon::weapon_fire_system,
            weapon::projectile_physics_system,
            card_selection::cards::gravity_vortex::gravity_well_system,
            crate::physics::card_selection::check_player_death,
        ).chain().run_if(in_state(GameState::Gameplay).and(is_not_paused).and(run_physics_simulation)));

        // Network synchronization systems (run in all online states)
        app.add_systems(Update, (
            crate::net::host_network_system.run_if(resource_equals(crate::net::IsNetworked(true)).and(resource_equals(crate::net::LocalPlayerIndex(0)))),
            crate::net::client_network_system.run_if(resource_equals(crate::net::IsNetworked(true)).and(resource_equals(crate::net::LocalPlayerIndex(1)))),
        ));

        // 3. Purely visual particles & projectile rendering (always in Update loop in both modes)
        app.add_systems(Update, (
            particles::update_particles,
            weapon::draw_projectiles,
            card_selection::cards::gravity_vortex::draw_gravity_wells,
        ).run_if(in_state(GameState::Gameplay).and(is_not_paused)));

        // Noodle drawing and visual systems (continue running while paused to draw visual frames)
        app.add_systems(Update, (
            update_aim,
            update_and_draw_legs,
            draw_procedural_arms,
            draw_expressive_faces,
        ).chain().after(player_platform_collision).run_if(in_state(GameState::Gameplay)));

        // Draw score UI overlay during all game states (Gameplay and Card Selection)
        app.add_systems(Update, draw_score_overlay);
    }
}

fn reset_collision_states(
    mut query: Query<(&mut Grounded, &mut WallContact)>,
) {
    for (mut grounded, mut wall) in query.iter_mut() {
        grounded.0 = false;
        wall.left = false;
        wall.right = false;
    }
}

fn apply_acceleration(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Acceleration)>,
) {
    let dt = time.delta_secs();
    for (mut velocity, acceleration) in query.iter_mut() {
        velocity.0 += acceleration.0 * dt;
    }
}

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * dt;
    }
}

pub fn is_not_paused(
    paused: Option<Res<menu_ui::Paused>>,
    delay: Option<Res<menu_ui::GameplayInputDelay>>,
) -> bool {
    let p_ok = if let Some(p) = paused { !p.0 } else { true };
    let d_ok = if let Some(d) = delay { d.0 <= 0.0 } else { true };
    p_ok && d_ok
}

pub fn run_physics_simulation(
    is_net: Res<crate::net::IsNetworked>,
    local_idx: Res<crate::net::LocalPlayerIndex>,
) -> bool {
    !is_net.0 || local_idx.0 == 0
}
