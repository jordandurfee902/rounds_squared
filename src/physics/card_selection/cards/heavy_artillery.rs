use crate::settings::PlayerStats;
use super::Card;

pub struct HeavyArtillery;

impl Card for HeavyArtillery {
    fn name(&self) -> &'static str {
        "Heavy Artillery"
    }

    fn desc(&self) -> &'static str {
        "Slow, massive,\nhigh-gravity warheads."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "+80% Bullet Damage",
            "+30% Bullet Size Mult",
            "+200 Downward Gravity",
            "+1.0s Reload Time",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.bullet_damage *= 1.80;
        stats.bullet_size_mult *= 1.30;
        stats.bullet_gravity -= 200.0;
        stats.reload_time += 1.0;
    }
}
