use crate::{
    components::{
        celestial::{CelestialBody, Terrain},
        frames::RootSpacePosition,
        relations::CelestialChildren,
        terrain::collider::{PrevColliderPoints, PrevIndexRanges},
        vessel::Vessel,
    },
    resources::ActiveVessel,
    terrain::collider::{gen_idx_ranges, gen_points, verts_at_lod_level},
};
use bevy::{ecs::query::QueryData, prelude::*};
use bevy_rapier2d::prelude::Collider;
use core::ops::RangeInclusive;

type CelestialQuery<'w, 's> = Query<'w, 's, CelestialComponents, With<CelestialBody>>;
type VesselQuery<'w, 's> = Query<'w, 's, VesselData, (With<Vessel>, Without<CelestialBody>)>;

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
    entity: Entity,
    position: &'static RootSpacePosition,
}

fn gen_theta_ranges(
    celestial_position: RootSpacePosition,
    children: &CelestialChildren,
    vessel_query: VesselQuery,
) -> Vec<RangeInclusive<f64>> {
    todo!();
}

fn update_collider(
    mut celestial: CelestialComponentsItem,
    vessel_query: VesselQuery,
    active_vessel: &ActiveVessel,
    commands: &mut Commands,
) {
    let rigid_pos = celestial.position.0 - active_vessel.prev_tick_position.0;

    let theta_ranges = gen_theta_ranges(*celestial.position, celestial.children, vessel_query);
    let verts = verts_at_lod_level(celestial.terrain.subdivs);
    let idx_ranges = gen_idx_ranges(&theta_ranges, verts);

    let is_range_changed = celestial.prev_ranges.is_none_or(|rs| *rs.0 == *idx_ranges);

    let mut new_terrain_pts = None;

    let collider_pts: Vec<_> = if !is_range_changed && let Some(ref points) = celestial.prev_pts {
        points
            .0
            .iter()
            .map(|point| point.phys_downcast(rigid_pos))
            .collect()
    } else {
        let terrain_pts = gen_points(&idx_ranges);
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

    *celestial.collider = Collider::compound(vec![
        (
            Vec2::ZERO,
            0.0,
            #[expect(clippy::cast_possible_truncation)]
            Collider::ball((celestial.terrain.offset - celestial.terrain.multiplier) as f32),
        ),
        (Vec2::ZERO, 0.0, Collider::polyline(collider_pts, None)),
    ]);
}

pub fn update_terrain_colliders(
    mut celestial_query: CelestialQuery,
    vessel_query: VesselQuery,
    mut commands: Commands,
    active_vessel: Res<ActiveVessel>,
) {
    for celestial in celestial_query {
        update_collider(celestial, vessel_query, &active_vessel, &mut commands);
    }
}
