use crate::settings::PlayerStats;
use super::Card;

pub struct BulletHell;

impl Card for BulletHell {
    fn name(&self) -> &'static str {
        "Bullet Hell"
    }

    fn desc(&self) -> &'static str {
        "Fires rapid bullets\nwith reduced damage."
    }

    fn stat_lines(&self) -> &'static [&'static str] {
        &[
            "+100% Max Ammo",
            "-50% Fire Rate Cooldown",
            "-30% Bullet Damage",
        ]
    }

    fn apply(&self, stats: &mut PlayerStats) {
        stats.max_ammo *= 2;
        stats.fire_rate *= 0.50;
        stats.bullet_damage *= 0.70;
    }
}
