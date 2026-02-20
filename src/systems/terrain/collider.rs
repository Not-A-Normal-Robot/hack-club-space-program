use bevy::{ecs::query::QueryData, prelude::*};
use bevy_rapier2d::prelude::Collider;

use crate::{
    components::{frames::RootSpacePosition, relations::CelestialChildren},
    systems::terrain::{CameraQuery, GlobalData},
};

type Queries<'w, 's> = (
    CameraQuery<'w, 's>,
    Query<'w, 's, &'static RootSpacePosition>,
    Query<'w, 's, CelestialComponents>,
);

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CelestialComponents {
    entity: Entity,
    position: &'static RootSpacePosition,
    collider: &'static mut Collider,
    children: &'static CelestialChildren,
}

#[derive(QueryData)]
pub struct VesselData {
    entity: Entity,
    position: &'static RootSpacePosition,
}

fn update_collider(
    celestial: CelestialComponentsItem,
    global: GlobalData,
    commands: &mut Commands,
) {
}

pub fn update_terrain_colliders(mut queries: ParamSet<Queries>, mut commands: Commands) {
    let Some((&zoom, &offset, _)) = queries.p0().iter().find(|(_, _, camera)| camera.is_active)
    else {
        #[cfg(feature = "trace")]
        trace!("Could not find active sim camera for terrain collider rebuilding");
        return;
    };

    let cam_pos = offset.immutably().get_root_position(queries.p1());

    let global = GlobalData { zoom, cam_pos };

    for celestial in queries.p2() {
        update_collider(celestial, global, &mut commands);
    }
}
