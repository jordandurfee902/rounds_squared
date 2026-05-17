pub mod components;
pub mod forces;
pub mod collision;
pub mod anim;

pub use components::*;
pub use forces::*;
pub use collision::*;
pub use anim::*;

use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            apply_acceleration,
            apply_gravity_and_movement,
            player_movement,
            apply_friction,
            apply_velocity,
            reset_collision_states,
            boundary_collision,
            player_collision,
            player_platform_collision,
            // Noodle animation & aim updates
            update_aim,
            update_and_draw_legs,
            draw_procedural_arms,
            draw_expressive_faces,
        ).chain());
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
