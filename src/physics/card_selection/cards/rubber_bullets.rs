use crate::settings::PlayerStats;
use super::Card;

pub struct RubberBullets;

impl Card for RubberBullets {
    fn name(&self) -> &'static str {
        "Rubber Bullets"
    }

    fn desc(&self) -> &'static str {
        "Bullets bounce more times\nand retain speed."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "+3 Bullet Bounces",
            "+30% Bounce Speed Mult",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.bounces += 3;
        if stats.bounce_speed_multiplier <= 0.0 {
            stats.bounce_speed_multiplier = 0.8;
        } else {
            stats.bounce_speed_multiplier *= 1.30;
        }
    }
}
