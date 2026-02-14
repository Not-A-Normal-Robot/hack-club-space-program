use crate::components::{
    camera::{SimCamera, SimCameraOffset, SimCameraZoom},
    celestial::{CelestialBody, Terrain},
    frames::RootSpacePosition,
    terrain::{LodVectors, PrevFocus},
};
use bevy::{
    ecs::{query::QueryData, system::SystemParam},
    prelude::*,
};

// TODO: Systems for terrain

pub type Queries<'w, 's> = (
    Query<
        'w,
        's,
        (
            &'static SimCameraZoom,
            &'static mut SimCameraOffset,
            &'static Camera,
        ),
        With<SimCamera>,
    >,
    Query<'w, 's, &'static RootSpacePosition>,
    Query<'w, 's, CelestialEntity>,
);

// #[derive(SystemParam)]
// pub struct Queries<'w, 's> {
//     camera: Query<'w, 's, (&'static SimCameraZoom, &'static SimCameraOffset)>,
//     camera_root_query: Query<'w, 's, &'static RootSpacePosition>,
//     entities: Query<'w, 's, EntityComponents>,
// }

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CelestialEntity {
    terrain: &'static Terrain,
    body: &'static CelestialBody,
    pos: &'static RootSpacePosition,
    mesh: &'static Mesh2d,
    offsets: Option<&'static mut LodVectors>,
    prev_focus: Option<&'static mut PrevFocus>,
}

#[derive(Clone, Copy)]
struct GlobalData {
    zoom: SimCameraZoom,
    cam_pos: RootSpacePosition,
}

fn update_mesh(
    _celestial: CelestialEntityItem,
    _global: GlobalData,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _commands: &mut Commands,
) {
    todo!();
}

pub fn update_meshes(
    mut queries: ParamSet<Queries>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let Some((&zoom, &offset, _)) = queries.p0().iter().find(|(_, _, camera)| camera.is_active)
    else {
        return;
    };

    let cam_pos = offset.immutably().get_root_position(queries.p1());

    let global = GlobalData { zoom, cam_pos };

    for celestial in queries.p2() {
        update_mesh(celestial, global, &mut meshes, &mut commands);
    }
}
