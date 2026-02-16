use crate::{
    components::{
        camera::{SimCamera, SimCameraOffset, SimCameraZoom},
        celestial::{CelestialBody, Terrain},
        frames::RootSpacePosition,
        terrain::{LodVectors, PrevFocus},
    },
    terrain::{
        TerrainGen,
        render::{get_focus, get_lod_level_cap},
    },
};
use bevy::{ecs::query::QueryData, prelude::*};
use core::{
    num::NonZeroU8,
    ops::{Deref, DerefMut},
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

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CelestialEntity {
    entity: Entity,
    terrain: &'static Terrain,
    body: &'static CelestialBody,
    pos: &'static RootSpacePosition,
    mesh: &'static Mesh2d,
    lod_vectors: Option<&'static mut LodVectors>,
    prev_focus: Option<&'static mut PrevFocus>,
}

#[derive(Clone, Copy)]
struct GlobalData {
    zoom: SimCameraZoom,
    cam_pos: RootSpacePosition,
}

enum CowMut<'a, T> {
    Borrowed(Mut<'a, T>),
    Owned(T),
}

impl<'a, T> Deref for CowMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(r) => r,
            Self::Owned(o) => o,
        }
    }
}

impl<'a, T> DerefMut for CowMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(r) => r,
            Self::Owned(o) => o,
        }
    }
}

fn update_mesh(
    celestial: CelestialEntityItem,
    global: GlobalData,
    meshes: &mut ResMut<Assets<Mesh>>,
    commands: &mut Commands,
) {
    // TODO: Consider celestial rotation
    let new_focus = get_focus(*celestial.pos, 0.0, global.cam_pos);
    let prev_focus = match celestial.prev_focus {
        Some(mut f) => {
            let old = *f;
            f.0 = new_focus;
            old.0
        }
        None => {
            commands
                .entity(celestial.entity)
                .insert(PrevFocus(new_focus));
            f64::NAN
        }
    };

    let terrain_gen = TerrainGen::new(*celestial.terrain);

    let distance_sq = global.cam_pos.0.distance_squared(celestial.pos.0);

    let ending_level =
        get_lod_level_cap(celestial.body.base_radius as f64, global.zoom, distance_sq)
            .map(|cap| celestial.terrain.subdivs.min(cap));

    let mut lod_vectors = match celestial.lod_vectors {
        Some(v) => CowMut::Borrowed(v),
        // None => CowMut::Owned(LodVectors::new_full(celestial.terrain, ending_level, focus))
        None => CowMut::Owned(match ending_level {
            Some(level) => LodVectors::new_full(&terrain_gen, level, new_focus),
            None => LodVectors::new(&terrain_gen),
        }),
    };

    if let Some(ending_level) = ending_level.and_then(NonZeroU8::new) {
        lod_vectors.update_lods(&terrain_gen, ending_level, prev_focus, new_focus);
    }

    todo!("Generate mesh");
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
