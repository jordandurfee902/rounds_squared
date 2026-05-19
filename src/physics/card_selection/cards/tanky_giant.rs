use crate::settings::PlayerStats;
use super::Card;

pub struct TankyGiant;

impl Card for TankyGiant {
    fn name(&self) -> &'static str {
        "Tanky Giant"
    }

    fn desc(&self) -> &'static str {
        "Grow massive and\nabsorb heavy hits."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "+40% Max Health",
            "+30% Player Scale",
            "-15% Jump Force",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.health_max *= 1.40;
        stats.player_scale *= 1.30;
        stats.jump_force *= 0.85;
    }
}
