use bevy::prelude::*;
use crate::settings::PhysicsSettings;
use super::components::*;

pub fn apply_gravity_and_movement(
    time: Res<Time>,
    settings: Res<PhysicsSettings>,
    mut query: Query<(&mut Velocity, &Mass, &Grounded, &WallContact, &ControllerInput)>,
) {
    let dt = time.delta_secs();
    for (mut velocity, mass, grounded, wall, input) in query.iter_mut() {
        if grounded.0 {
            continue;
        }

        // Weight/Mass dictates how quickly they fall.
        // We scale the downward gravity acceleration by the player's Mass.
        let mut gravity_accel = settings.gravity * mass.0;

        // Fast Falling: pressing down (fast_fall) increases downward velocity significantly
        if input.fast_fall && velocity.0.y < 50.0 {
            gravity_accel += -settings.fast_fall_acceleration;
        }

        velocity.0.y += gravity_accel * dt;

        // Wall Cling: pushing into a vertical surface slows descent
        // Only clings when moving downwards (falling)
        if velocity.0.y < 0.0 {
            if (wall.left && input.move_dir < -0.1) || (wall.right && input.move_dir > 0.1) {
                if velocity.0.y < -settings.wall_slide_speed {
                    velocity.0.y = -settings.wall_slide_speed;
                }
            }
        }
    }
}

pub fn player_movement(
    time: Res<Time>,
    settings: Res<PhysicsSettings>,
    mut query: Query<(&mut Velocity, &Grounded, &WallContact, &ControllerInput)>,
) {
    let dt = time.delta_secs();
    for (mut velocity, grounded, wall, input) in query.iter_mut() {
        let target_speed = input.move_dir * settings.player_speed;

        // Ground Movement vs Air Strafing
        if grounded.0 {
            if input.move_dir == 0.0 {
                // Ground sliding braking friction
                let damping = (1.0 - settings.movement_stop_friction * dt).max(0.0);
                velocity.0.x *= damping;
            } else {
                // Ground acceleration phase
                let diff = target_speed - velocity.0.x;
                velocity.0.x += diff * settings.player_accel * dt;
            }
        } else {
            // Air strafe (maintain high degree of directional control in mid-air)
            if input.move_dir != 0.0 {
                let diff = target_speed - velocity.0.x;
                velocity.0.x += diff * settings.air_accel * dt;
            }
        }

        // Jumping and Wall Leaping (Anchor Mechanic)
        if input.jump {
            if grounded.0 {
                // Normal Jump: momentum carries over from ground movement
                velocity.0 = calculate_jump(velocity.0, settings.player_jump_force);
            } else if wall.left {
                // Wall Leap off left wall: push up and outward to the right
                velocity.0.y = settings.player_jump_force;
                velocity.0.x = settings.wall_jump_push_force;
            } else if wall.right {
                // Wall Leap off right wall: push up and outward to the left
                velocity.0.y = settings.player_jump_force;
                velocity.0.x = -settings.wall_jump_push_force;
            }
        }
    }
}

pub fn apply_friction(
    time: Res<Time>,
    settings: Res<PhysicsSettings>,
    mut query: Query<(&mut Velocity, &Grounded)>,
) {
    let dt = time.delta_secs();
    for (mut velocity, grounded) in query.iter_mut() {
        // Friction creates a short slide when stopping and bounds velocities in air
        let friction = if grounded.0 {
            settings.ground_friction
        } else {
            settings.air_friction
        };
        let damping = (1.0 - friction * dt).max(0.0);
        velocity.0.x *= damping;
    }
}

// Pure movement calculation helper functions
#[allow(dead_code)]
pub fn calculate_horizontal_movement(
    current_vel: Vec2,
    direction: f32,
    speed: f32,
    acceleration: f32,
    stop_friction: f32,
    dt: f32,
) -> Vec2 {
    let mut new_vel = current_vel;
    
    if direction == 0.0 {
        let damping = (1.0 - stop_friction * dt).max(0.0);
        new_vel.x *= damping;
    } else {
        let target_vel = direction * speed;
        let diff = target_vel - current_vel.x;
        new_vel.x += diff * acceleration * dt;
    }
    
    new_vel
}

#[allow(dead_code)]
pub fn calculate_jump(current_vel: Vec2, jump_force: f32) -> Vec2 {
    let mut new_vel = current_vel;
    new_vel.y = jump_force;
    new_vel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump_overrides_vertical_velocity() {
        let vel = Vec2::new(0.0, -500.0);
        let new_vel = calculate_jump(vel, 800.0);
        assert_eq!(new_vel.y, 800.0);
    }

    #[test]
    fn test_movement_reaches_target_speed() {
        let mut vel = Vec2::ZERO;
        let speed = 500.0;
        let acceleration = 10.0;
        let dt = 0.1;
        
        // Simulate a few frames of movement (passing stop_friction as 0.0 for tests)
        for _ in 0..100 {
            vel = calculate_horizontal_movement(vel, 1.0, speed, acceleration, 0.0, dt);
        }
        
        assert!((vel.x - speed).abs() < 1.0);
    }

    #[test]
    fn test_movement_does_not_exceed_speed() {
        let mut vel = Vec2::new(600.0, 0.0);
        let speed = 500.0;
        let acceleration = 10.0;
        let dt = 0.1;
        
        vel = calculate_horizontal_movement(vel, 1.0, speed, acceleration, 0.0, dt);
        assert!(vel.x < 600.0);
        assert!(vel.x >= 500.0);
    }
}
