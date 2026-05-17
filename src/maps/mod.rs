pub mod default_map;
pub mod pillars_map;
pub mod stadium_map;
pub mod chasm_bridge;
pub mod gridlock;
pub mod hourglass;
pub mod ice_temple;
pub mod industrial_foundry;
pub mod vertical_helix;
pub mod tectonic_fissure;
pub mod zen_garden;
pub mod space_station;
pub mod ancient_coliseum;

use std::time::{SystemTime, UNIX_EPOCH};
use bevy::prelude::Resource;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveMap {
    #[default]
    DefaultMap,
    PillarsMap,
    StadiumMap,
    ChasmBridge,
    Gridlock,
    Hourglass,
    IceTemple,
    IndustrialFoundry,
    VerticalHelix,
    TectonicFissure,
    ZenGarden,
    SpaceStation,
    AncientColiseum,
}

impl ActiveMap {
    /// Selects a pseudo-random map based on the microsecond time delta since UNIX epoch
    pub fn select_random() -> Self {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let micros = duration.as_micros();
        let selected = match micros % 13 {
            0 => ActiveMap::DefaultMap,
            1 => ActiveMap::PillarsMap,
            2 => ActiveMap::StadiumMap,
            3 => ActiveMap::ChasmBridge,
            4 => ActiveMap::Gridlock,
            5 => ActiveMap::Hourglass,
            6 => ActiveMap::IceTemple,
            7 => ActiveMap::IndustrialFoundry,
            8 => ActiveMap::VerticalHelix,
            9 => ActiveMap::TectonicFissure,
            10 => ActiveMap::ZenGarden,
            11 => ActiveMap::SpaceStation,
            _ => ActiveMap::AncientColiseum,
        };
        println!("MAP SELECTION: Selected map {:?}", selected);
        selected
    }
}
