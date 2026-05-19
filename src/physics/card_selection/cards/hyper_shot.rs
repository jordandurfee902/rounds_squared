use crate::settings::PlayerStats;
use super::Card;

pub struct HyperShot;

impl Card for HyperShot {
    fn name(&self) -> &'static str {
        "Hyper-Shot"
    }

    fn desc(&self) -> &'static str {
        "Fast-travel high\nvelocity bullet rounds."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "+35% Bullet Speed",
            "+20% Bullet Damage",
            "-1 Max Ammo",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.bullet_speed *= 1.35;
        stats.bullet_damage *= 1.20;
        stats.max_ammo = stats.max_ammo.saturating_sub(1).max(1);
    }
}
