use crate::{
    components::{
        celestial::{CelestialBody, Terrain},
        frames::RootSpacePosition,
        relations::CelestialChildren,
        terrain::collider::{PrevColliderPoints, PrevIndexRanges},
        vessel::Vessel,
    },
    resources::ActiveVessel,
    terrain::collider::{
        create_index_buffer, gen_idx_ranges, gen_points, get_theta_range, verts_at_lod_level,
    },
};
use bevy::{ecs::query::QueryData, prelude::*};
use bevy_rapier2d::prelude::{Collider, RigidBody, RigidBodyDisabled, TriMeshFlags};
use core::ops::RangeInclusive;

type CelestialQuery<'w, 's> = Query<'w, 's, CelestialComponents, With<CelestialBody>>;
type VesselQuery<'w, 's> = Query<
    'w,
    's,
    VesselData,
    (
        With<Vessel>,
        Without<CelestialBody>,
        With<RigidBody>,
        Without<RigidBodyDisabled>,
    ),
>;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CelestialComponents {
    entity: Entity,
    position: &'static RootSpacePosition,
    collider: &'static mut Collider,
    children: &'static CelestialChildren,
    terrain: &'static Terrain,
    prev_ranges: Option<&'static mut PrevIndexRanges>,
    prev_pts: Option<&'static mut PrevColliderPoints>,
}

#[derive(QueryData)]
pub struct VesselData {
    position: &'static RootSpacePosition,
    collider: &'static Collider,
}

fn gen_theta_ranges(
    celestial_position: RootSpacePosition,
    terrain: &Terrain,
    children: &CelestialChildren,
    vessel_query: VesselQuery,
) -> Vec<RangeInclusive<f64>> {
    let iter = children
        .iter()
        .filter_map(|entity| vessel_query.get(entity).ok());

    let size = iter.size_hint().1.unwrap_or_else(|| iter.size_hint().0);
    let mut vec = Vec::with_capacity(size);

    for vessel in iter {
        let vessel_rel_pos = vessel.position.0 - celestial_position.0;
        let aabb = vessel.collider.raw.compute_local_aabb();
        // TODO: Consider celestial rotation
        let range = get_theta_range(aabb, vessel_rel_pos, 0.0, terrain);
        vec.push(range);
    }

    vec
}

fn update_collider(
    mut celestial: CelestialComponentsItem,
    vessel_query: VesselQuery,
    active_vessel: &ActiveVessel,
    commands: &mut Commands,
) {
    let rigid_pos = celestial.position.0 - active_vessel.prev_tick_position.0;

    let theta_ranges = gen_theta_ranges(
        *celestial.position,
        celestial.terrain,
        celestial.children,
        vessel_query,
    );
    let verts = verts_at_lod_level(celestial.terrain.subdivs);
    let idx_ranges = gen_idx_ranges(&theta_ranges, verts);

    if idx_ranges.is_empty() {
        return; // No nearby vessels, just ignore
    }

    let is_range_changed = celestial.prev_ranges.is_none_or(|rs| *rs.0 == *idx_ranges);

    let mut new_terrain_pts = None;

    let collider_pts: Vec<_> = if !is_range_changed && let Some(ref points) = celestial.prev_pts {
        points
            .0
            .iter()
            .map(|point| point.phys_downcast(rigid_pos))
            .collect()
    } else {
        let terrain_pts = gen_points(*celestial.terrain, &idx_ranges);
        let collider_pts: Vec<_> = terrain_pts
            .iter()
            .map(|point| point.phys_downcast(rigid_pos))
            .collect();
        new_terrain_pts = Some(terrain_pts);
        collider_pts
    };

    if let Some(terrain_pts) = new_terrain_pts {
        if let Some(ref mut old_points) = celestial.prev_pts {
            old_points.0 = terrain_pts;
        } else {
            commands
                .entity(celestial.entity)
                .insert(PrevColliderPoints(terrain_pts));
        }
    }

    let idx_buffer = create_index_buffer(collider_pts.len() as u32);
    let trimesh = Collider::trimesh_with_flags(
        collider_pts,
        idx_buffer,
        TriMeshFlags::DELETE_DEGENERATE_TRIANGLES | TriMeshFlags::DELETE_BAD_TOPOLOGY_TRIANGLES,
    );

    let ball = Collider::ball((celestial.terrain.offset - celestial.terrain.multiplier) as f32);

    *celestial.collider = match trimesh {
        Ok(trimesh) => trimesh, // TODO: Ball with trimesh
        Err(e) => {
            error!("Error building terrain tri mesh: {e}");
            ball
        }
    };
}

pub fn update_terrain_colliders(
    celestial_query: CelestialQuery,
    vessel_query: VesselQuery,
    mut commands: Commands,
    active_vessel: Res<ActiveVessel>,
) {
    for celestial in celestial_query {
        update_collider(celestial, vessel_query, &active_vessel, &mut commands);
    }
}
