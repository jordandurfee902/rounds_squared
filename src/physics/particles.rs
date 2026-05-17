use bevy::prelude::*;

// --- LIGHTWEIGHT PARTICLE TYPES ---
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleType {
    StandardSpark,
    LightBeam { direction: Vec2, length: f32, rotation_speed: f32 },
    Shockwave { min_radius: f32, max_radius: f32 },
}

// --- LIGHTWEIGHT PARTICLE COMPONENT ---
#[derive(Component, Debug, Clone)]
pub struct Particle {
    pub velocity: Vec2,
    pub color: Color,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub particle_type: ParticleType,
}

// --- COLOR TEMPERATURE SHIFTING HELPER ---
pub fn get_tiered_color(base_color: Color, damage: f32) -> Color {
    if damage < 25.0 {
        base_color
    } else if damage < 55.0 {
        // High Damage: Vibrant Neon Cyan (P1 Blue) or Molten Gold (P2 Orange)
        let is_orange = base_color.to_srgba().red > 0.4;
        if is_orange {
            Color::srgb(1.0, 0.85, 0.0) // Molten Gold
        } else {
            Color::srgb(0.0, 0.95, 1.0) // Vibrant Neon Cyan
        }
    } else if damage < 90.0 {
        // Extreme Damage: Volatile Plasma Magenta
        Color::srgb(0.95, 0.0, 0.8) // Plasma Magenta
    } else {
        // Cosmic Damage: Glowing Celestial Neon Violet
        Color::srgb(0.6, 0.0, 1.0) // Cosmic Neon Violet
    }
}

// --- OPTIMIZED GIZMO-DRIVEN RENDERER & PHYSICS SYSTEM ---
pub fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
    mut gizmos: Gizmos,
) {
    let dt = time.delta_secs().min(0.05); // cap delta to secure physics on lag spikes
    
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Apply visual air damping / drag so sparks burst out rapidly then decelerate elegantly
        particle.velocity *= 0.90;
        
        // Move particle coordinate space
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        let life_pct = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        let pos = transform.translation.xy();

        match particle.particle_type {
            ParticleType::StandardSpark => {
                let current_size = particle.size * life_pct;
                // Draw neon outer glow
                gizmos.circle_2d(pos, current_size, particle.color);
                // Draw white-hot center core for premium feel!
                if current_size > 1.2 {
                    gizmos.circle_2d(pos, current_size * 0.4, Color::WHITE);
                }
            }
            ParticleType::LightBeam { direction, length, rotation_speed } => {
                // Calculate rotated direction over time to look like twisting electric spikes!
                let rotation = rotation_speed * (particle.max_lifetime - particle.lifetime);
                let cos_r = rotation.cos();
                let sin_r = rotation.sin();
                let rotated_dir = Vec2::new(
                    direction.x * cos_r - direction.y * sin_r,
                    direction.x * sin_r + direction.y * cos_r,
                );
                let beam_length = length * life_pct;
                let end_pos = pos + rotated_dir * beam_length;
                
                // Draw primary laser line with fading alpha
                let alpha = life_pct * 0.7;
                let color_srgba = particle.color.to_srgba();
                let glow_color = Color::srgba(color_srgba.red, color_srgba.green, color_srgba.blue, alpha);
                
                gizmos.line_2d(pos, end_pos, glow_color);
                // Secondary core white laser line
                gizmos.line_2d(pos + rotated_dir * 0.5, end_pos, Color::srgba(1.0, 1.0, 1.0, alpha));
            }
            ParticleType::Shockwave { min_radius, max_radius } => {
                // Starts exactly at the edge of the bullet (min_radius) and grows to max_radius!
                let current_rad = min_radius + (max_radius - min_radius) * (1.0 - life_pct);
                let alpha = life_pct * 0.55;
                
                // Neon glow outer ring
                let color_srgba = particle.color.to_srgba();
                let glow_color = Color::srgba(color_srgba.red, color_srgba.green, color_srgba.blue, alpha);
                gizmos.circle_2d(pos, current_rad, glow_color);
                
                // Core white razor ring
                let white_alpha = life_pct * 0.8;
                let inner_white = Color::srgba(1.0, 1.0, 1.0, white_alpha);
                gizmos.circle_2d(pos, current_rad * 0.96, inner_white);
            }
        }
    }
}

// --- ZERO-DEPENDENCY LIGHTWEIGHT LCG RANDOM GENERATOR ---
pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        Self { state: seed.wrapping_add(12345) }
    }

    /// Generates a float in the range [0.0, 1.0)
    pub fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state & 0x7FFFFFFF) as f32 / 2147483648.0
    }

    /// Generates a float in the range [min, max)
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

// --- MODULAR CONTAINERIZED BURST SPAWNER ---
pub fn spawn_spark_burst(
    commands: &mut Commands,
    pos: Vec2,
    color: Color,
    count: usize,
    seed_offset: u32,
) {
    // Seed using player positions and offset variables
    let mut rng = SimpleRng::new(
        (pos.x.abs() as u32)
            .wrapping_add((pos.y.abs() as u32).wrapping_mul(1000))
            .wrapping_add(seed_offset),
    );

    for _ in 0..count {
        let angle = rng.range(0.0, std::f32::consts::TAU);
        let speed = rng.range(120.0, 360.0);
        let size = rng.range(1.5, 3.5);
        let lifetime = rng.range(0.2, 0.4);

        commands.spawn((
            Particle {
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                color,
                size,
                lifetime,
                max_lifetime: lifetime,
                particle_type: ParticleType::StandardSpark,
            },
            Transform::from_xyz(pos.x, pos.y, 8.0),
        ));
    }
}

// --- DYNAMIC TRAILS SPAWNER (SCALES BY TIRED DAMAGE COLOR, TEARDROP FOR METEOR) ---
pub fn spawn_trail_particle(
    commands: &mut Commands,
    pos: Vec2,
    color: Color,
    damage: f32,
    bullet_velocity: Vec2,
    seed_offset: u32,
) {
    let mut rng = SimpleRng::new(
        (pos.x.abs() as u32)
            .wrapping_add((pos.y.abs() as u32).wrapping_mul(1000))
            .wrapping_add(seed_offset),
    );

    // Apply tiered energy colors based on damage!
    let final_color = get_tiered_color(color, damage);

    if damage >= 90.0 {
        // --- HIGH-DENSITY TEARDROP METEOR TRAIL ---
        // Spawn 3 particles per frame distributed wide and drifting inwards to create a perfect teardrop comets tail!
        let forward = bullet_velocity.normalize_or_zero();
        let right = Vec2::new(-forward.y, forward.x);
        // Estimate bullet radius for spread
        let radius = damage.sqrt() * 1.5; 

        for i in 0..3 {
            let mut particle_rng = SimpleRng::new(
                (pos.x.abs() as u32)
                    .wrapping_add((pos.y.abs() as u32).wrapping_mul(1000))
                    .wrapping_add(seed_offset)
                    .wrapping_add(i as u32 * 77),
            );
            
            let spread_pct = particle_rng.range(-1.0, 1.0);
            // Spawn spread wide perpendicular to movement
            let spawn_offset = right * (spread_pct * radius);

            // Velocity: drift backward AND pull inwards towards the center tail line!
            let drift = -bullet_velocity * particle_rng.range(0.08, 0.18);
            let inward_dir = -right * (spread_pct * particle_rng.range(80.0, 140.0));
            let velocity = drift + inward_dir;

            // Size: central particles are thick, outer particles are tapered and thin
            let size = particle_rng.range(0.8, 1.8) * (1.0 - spread_pct.abs() * 0.4);
            // Lifetime: central particles trail longer, outer particles burn out quickly
            let lifetime = particle_rng.range(0.20, 0.40) * (1.0 - spread_pct.abs() * 0.5);

            commands.spawn((
                Particle {
                    velocity,
                    color: final_color,
                    size,
                    lifetime,
                    max_lifetime: lifetime,
                    particle_type: ParticleType::StandardSpark,
                },
                Transform::from_xyz(pos.x + spawn_offset.x, pos.y + spawn_offset.y, 8.0),
            ));
        }
    } else {
        // --- STANDARD BULLET TRAIL ---
        // Trail velocity opposes bullet velocity with slight scatter
        let angle = rng.range(0.0, std::f32::consts::TAU);
        let scatter_speed = rng.range(15.0, 60.0);
        let drift = -bullet_velocity * 0.12;
        let velocity = drift + Vec2::new(angle.cos() * scatter_speed, angle.sin() * scatter_speed);

        // Size does NOT inflate: cap individual sizes at standard small trail values
        let size = rng.range(0.5, 1.2);
        let lifetime = rng.range(0.15, 0.35);

        commands.spawn((
            Particle {
                velocity,
                color: final_color,
                size,
                lifetime,
                max_lifetime: lifetime,
                particle_type: ParticleType::StandardSpark,
            },
            Transform::from_xyz(pos.x, pos.y, 8.0),
        ));
    }
}

// --- DYNAMIC LANDING EXPLOSION SPAWNER (SCALES BY TIERS, INJECTS BEAMS & SHOCKWAVES) ---
pub fn spawn_damage_explosion(
    commands: &mut Commands,
    pos: Vec2,
    color: Color,
    damage: f32,
    seed_offset: u32,
) {
    let mut rng = SimpleRng::new(
        (pos.x.abs() as u32)
            .wrapping_add((pos.y.abs() as u32).wrapping_mul(1000))
            .wrapping_add(seed_offset),
    );

    // Apply tiered energy colors based on damage!
    let final_color = get_tiered_color(color, damage);

    // Estimate bullet physical radius to scatter sparks across its exact body volume
    let bullet_radius = (damage.sqrt() * 1.5).max(4.0);

    // 1. Spawning Standard Sparks (Distributed uniformly inside the bullet volume)
    let size_factor = damage.sqrt();
    let count = (size_factor * 2.5 + 8.0).round() as usize; // slightly increased count for dense meteor impacts

    for i in 0..count {
        let angle = rng.range(0.0, std::f32::consts::TAU);
        let speed = rng.range(80.0, 260.0) * size_factor.clamp(1.0, 2.5);
        // Cap spark size between 0.8 and 2.5 max
        let size = rng.range(0.8, 2.0).min(2.5);
        let lifetime = rng.range(0.25, 0.50);

        // Scatter starting positions uniformly inside the bullet circle!
        let mut particle_rng = SimpleRng::new(seed_offset.wrapping_add(i as u32 * 33));
        let spread_angle = particle_rng.range(0.0, std::f32::consts::TAU);
        let spread_radius = particle_rng.range(0.0, bullet_radius);
        let spawn_pos = pos + Vec2::new(spread_angle.cos() * spread_radius, spread_angle.sin() * spread_radius);

        commands.spawn((
            Particle {
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                color: final_color,
                size,
                lifetime,
                max_lifetime: lifetime,
                particle_type: ParticleType::StandardSpark,
            },
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, 8.0),
        ));
    }

    // 2. Procedural Light Beams (Laser Spikes offset from within the bullet body)
    if damage >= 25.0 {
        let beam_count = if damage < 55.0 {
            rng.range(4.0, 6.0).round() as usize
        } else if damage < 90.0 {
            rng.range(6.0, 10.0).round() as usize
        } else {
            rng.range(10.0, 14.0).round() as usize
        };

        let base_length = if damage < 55.0 {
            rng.range(50.0, 70.0)
        } else if damage < 90.0 {
            rng.range(80.0, 110.0)
        } else {
            rng.range(120.0, 150.0)
        };

        for i in 0..beam_count {
            // Evenly distribute beam angles with slight random jitter
            let base_angle = (i as f32 / beam_count as f32) * std::f32::consts::TAU;
            let angle = base_angle + rng.range(-0.15, 0.15);
            let direction = Vec2::new(angle.cos(), angle.sin());

            // Set higher damage beams to rotate slowly
            let rotation_speed = if damage >= 55.0 {
                rng.range(-2.0, 2.0)
            } else {
                0.0
            };

            let lifetime = rng.range(0.12, 0.22);

            // Offset the beam origin from within the bullet body
            let beam_offset = direction * rng.range(0.0, bullet_radius * 0.5);
            let spawn_pos = pos + beam_offset;

            commands.spawn((
                Particle {
                    velocity: Vec2::ZERO, // Beams are fixed at impact point
                    color: final_color,
                    size: 1.0, // Fixed thickness
                    lifetime,
                    max_lifetime: lifetime,
                    particle_type: ParticleType::LightBeam {
                        direction,
                        length: base_length,
                        rotation_speed,
                    },
                },
                Transform::from_xyz(spawn_pos.x, spawn_pos.y, 8.0),
            ));
        }
    }

    // 3. Expanding Shockwave Ring Distortion (Starts exactly at the bullet edge!)
    if damage >= 40.0 {
        let min_radius = bullet_radius;
        let max_radius = bullet_radius + if damage < 90.0 {
            rng.range(70.0, 100.0)
        } else {
            rng.range(120.0, 160.0)
        };

        let lifetime = rng.range(0.14, 0.24);

        commands.spawn((
            Particle {
                velocity: Vec2::ZERO, // Shockwave is fixed at impact point
                color: final_color,
                size: 1.0,
                lifetime,
                max_lifetime: lifetime,
                particle_type: ParticleType::Shockwave { min_radius, max_radius },
            },
            Transform::from_xyz(pos.x, pos.y, 8.0),
        ));
    }
}
