use crate::settings::PlayerStats;
use super::Card;

pub struct FastAndLight;

impl Card for FastAndLight {
    fn name(&self) -> &'static str {
        "Fast & Light"
    }

    fn desc(&self) -> &'static str {
        "Trade durability\nfor extreme speed!"
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "+30% Movement Speed",
            "-20% Max Health",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.movement_speed *= 1.30;
        stats.health_max *= 0.80;
    }
}
