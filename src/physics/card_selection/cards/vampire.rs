use crate::settings::PlayerStats;
use super::Card;

pub struct Vampire;

impl Card for Vampire {
    fn name(&self) -> &'static str {
        "Vampire"
    }

    fn desc(&self) -> &'static str {
        "Increases shield duration\nand reduces cooldown."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "+25% Block Duration",
            "-20% Block Cooldown",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.block_duration *= 1.25;
        stats.block_cooldown *= 0.80;
    }
}
