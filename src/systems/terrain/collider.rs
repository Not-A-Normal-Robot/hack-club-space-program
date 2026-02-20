use bevy::{ecs::query::QueryData, prelude::*};

use crate::{
    components::frames::RootSpacePosition,
    systems::terrain::{CameraQuery, GlobalData},
};

type PhyQueries<'w, 's> = (
    CameraQuery<'w, 's>,
    Query<'w, 's, &'static RootSpacePosition>,
    Query<'w, 's, CelestialPhyComponents>,
);

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CelestialPhyComponents {
    entity: Entity,
    position: &'static RootSpacePosition,
}

#[derive(QueryData)]
pub struct VesselData {
    entity: Entity,
    position: &'static RootSpacePosition,
}

fn update_collider(
    celestial: CelestialPhyComponentsItem,
    global: GlobalData,
    commands: &mut Commands,
) {
    todo!();
}

pub fn update_terrain_colliders(mut queries: ParamSet<PhyQueries>, mut commands: Commands) {
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
