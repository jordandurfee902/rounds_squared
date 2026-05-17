use bevy::prelude::*;
use crate::physics::{Collider, Platform};
use crate::settings::GameState;
use crate::maps::{
    ActiveMap,
    default_map::spawn_default_map,
    pillars_map::spawn_pillars_map,
    stadium_map::spawn_stadium_map,
    chasm_bridge::spawn_chasm_bridge,
    gridlock::spawn_gridlock,
    hourglass::spawn_hourglass,
    ice_temple::spawn_ice_temple,
    industrial_foundry::spawn_industrial_foundry,
    vertical_helix::spawn_vertical_helix,
    tectonic_fissure::spawn_tectonic_fissure,
    zen_garden::spawn_zen_garden,
    space_station::spawn_space_station,
    ancient_coliseum::spawn_ancient_coliseum,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveMap::DefaultMap);
        app.add_systems(OnEnter(GameState::Gameplay), spawn_platforms)
           .add_systems(OnExit(GameState::Gameplay), despawn_platforms);
    }
}

pub fn spawn_platform(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    size: Vec2,
    pos: Vec3,
    color: Color,
) {
    commands.spawn((
        Platform,
        Collider::Rect { size },
        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(pos),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
    ));
}

pub fn spawn_platforms(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    active_map: Res<ActiveMap>,
) {
    match *active_map {
        ActiveMap::DefaultMap => {
            spawn_default_map(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::PillarsMap => {
            spawn_pillars_map(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::StadiumMap => {
            spawn_stadium_map(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::ChasmBridge => {
            spawn_chasm_bridge(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::Gridlock => {
            spawn_gridlock(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::Hourglass => {
            spawn_hourglass(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::IceTemple => {
            spawn_ice_temple(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::IndustrialFoundry => {
            spawn_industrial_foundry(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::VerticalHelix => {
            spawn_vertical_helix(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::TectonicFissure => {
            spawn_tectonic_fissure(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::ZenGarden => {
            spawn_zen_garden(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::SpaceStation => {
            spawn_space_station(&mut commands, &mut meshes, &mut materials);
        }
        ActiveMap::AncientColiseum => {
            spawn_ancient_coliseum(&mut commands, &mut meshes, &mut materials);
        }
    }
}

fn despawn_platforms(
    mut commands: Commands,
    query: Query<Entity, With<Platform>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
