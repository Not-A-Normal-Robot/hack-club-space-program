use bevy::ecs::query::QueryData;

use crate::components::{
    camera::{SimCameraOffset, SimCameraZoom},
    celestial::{CelestialBody, Terrain},
    frames::RootSpacePosition,
    terrain::{LodVectors, PrevFocus},
};

// TODO: Systems for terrain

#[derive(QueryData)]
#[query_data(mutable)]
struct EntityComponents {
    terrain: &'static Terrain,
    body: &'static CelestialBody,
    pos: &'static RootSpacePosition,
    offsets: Option<&'static mut LodVectors>,
    prev_focus: Option<&'static mut PrevFocus>,
}

#[derive(Clone, Copy)]
struct GlobalData {
    zoom: SimCameraZoom,
    offset: SimCameraOffset,
}
