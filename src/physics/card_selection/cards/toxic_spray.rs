use bevy::prelude::*;
use crate::settings::PlayerStats;
use super::Card;

pub struct ToxicSpray;

impl Card for ToxicSpray {
    fn name(&self) -> &'static str {
        "Toxic Spray"
    }

    fn desc(&self) -> &'static str {
        "Infect opponents with\nneon poison clouds."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "Adds Poison Trail effect",
            "+0.15 Bullet Growth",
            "+2 Max Ammo",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        if !stats.special_effects.contains(&"PoisonCloud".to_string()) {
            stats.special_effects.push("PoisonCloud".to_string());
        }
        stats.bullet_growth += 0.15;
        stats.max_ammo += 2;
    }
}
