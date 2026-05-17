use bevy::prelude::*;

// --- LIGHTWEIGHT PARTICLE COMPONENT ---
#[derive(Component, Debug, Clone)]
pub struct Particle {
    pub velocity: Vec2,
    pub color: Color,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
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

        // Decent shrink over time
        let life_pct = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        let current_size = particle.size * life_pct;
        
        // Draw the circular spark using high-performance Gizmos lines (avoids entity sprite overload)
        gizmos.circle_2d(transform.translation.xy(), current_size, particle.color);
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
            },
            Transform::from_xyz(pos.x, pos.y, 8.0),
        ));
    }
}

// --- DYNAMIC TRAILS SPAWNER (SCALES BY SQRT DAMAGE) ---
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

    // Trail velocity opposes bullet velocity with slight scatter
    let angle = rng.range(0.0, std::f32::consts::TAU);
    let scatter_speed = rng.range(15.0, 60.0);
    let drift = -bullet_velocity * 0.12;
    let velocity = drift + Vec2::new(angle.cos() * scatter_speed, angle.sin() * scatter_speed);

    // Size scales with sqrt(damage)
    let size_factor = damage.sqrt();
    let size = rng.range(0.3, 0.6) * size_factor;
    let lifetime = rng.range(0.15, 0.35);

    commands.spawn((
        Particle {
            velocity,
            color,
            size,
            lifetime,
            max_lifetime: lifetime,
        },
        Transform::from_xyz(pos.x, pos.y, 8.0),
    ));
}

// --- DYNAMIC LANDING EXPLOSION SPAWNER (SCALES BY SQRT DAMAGE) ---
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

    let size_factor = damage.sqrt();
    
    // Count scales proportionally: base damage 12 gives ~13 particles, 100 damage gives ~26 particles!
    let count = (size_factor * 2.0 + 6.0).round() as usize;

    for _ in 0..count {
        let angle = rng.range(0.0, std::f32::consts::TAU);
        // Particle velocity & expansion speed scale with sqrt(damage)
        let speed = rng.range(80.0, 240.0) * size_factor;
        // Individual particle sizes scale with sqrt(damage)
        let size = rng.range(0.8, 2.0) * size_factor;
        let lifetime = rng.range(0.25, 0.50);

        commands.spawn((
            Particle {
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                color,
                size,
                lifetime,
                max_lifetime: lifetime,
            },
            Transform::from_xyz(pos.x, pos.y, 8.0),
        ));
    }
}
