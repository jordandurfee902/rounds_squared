use crate::settings::PlayerStats;
use super::Card;

pub struct GlassCannon;

impl Card for GlassCannon {
    fn name(&self) -> &'static str {
        "Glass Cannon"
    }

    fn desc(&self) -> &'static str {
        "Deals massive damage\nbut reduces max health."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "+100% Bullet Damage",
            "-50% Max Health",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.bullet_damage *= 2.00;
        stats.health_max *= 0.50;
    }
}
