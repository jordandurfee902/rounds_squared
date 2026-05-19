use bevy::prelude::*;
use crate::settings::PlayerStats;
use crate::physics::weapon::Projectile;

pub mod fast_and_light;
pub mod tanky_giant;
pub mod hyper_shot;
pub mod toxic_spray;
pub mod heavy_artillery;
pub mod glass_cannon;
pub mod bullet_hell;
pub mod rubber_bullets;
pub mod vampire;
pub mod gravity_vortex;

pub trait Card: Send + Sync {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn stat_lines(&self) -> &'static [&'static str];
    fn apply(&self, stats: &mut PlayerStats);
    fn on_bullet_land(&self, _commands: &mut Commands, _proj: &Projectile, _pos: Vec2) {}
}

pub const TOTAL_CARDS_COUNT: usize = 10;

pub fn get_card(idx: usize) -> Option<&'static dyn Card> {
    match idx {
        0 => Some(&fast_and_light::FastAndLight),
        1 => Some(&tanky_giant::TankyGiant),
        2 => Some(&hyper_shot::HyperShot),
        3 => Some(&toxic_spray::ToxicSpray),
        4 => Some(&heavy_artillery::HeavyArtillery),
        5 => Some(&glass_cannon::GlassCannon),
        6 => Some(&bullet_hell::BulletHell),
        7 => Some(&rubber_bullets::RubberBullets),
        8 => Some(&vampire::Vampire),
        9 => Some(&gravity_vortex::GravityVortex),
        _ => None,
    }
}
