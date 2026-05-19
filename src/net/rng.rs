use bevy::prelude::*;

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackRng {
    pub seed: u32,
}

impl RollbackRng {
    pub fn new(seed: u32) -> Self {
        Self { seed: seed.wrapping_add(54321) }
    }

    /// Generates a float in the range [0.0, 1.0) deterministically
    pub fn next_f32(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed & 0x7FFFFFFF) as f32 / 2147483648.0
    }

    /// Generates a float in the range [min, max) deterministically
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}
