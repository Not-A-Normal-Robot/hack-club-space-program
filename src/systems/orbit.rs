use crate::{
    components::{
        celestial::CelestialBody,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::{CelestialParent, RailMode, SurfaceAttachment},
        vessel::Vessel,
    },
    consts::{FilterLoadedVessels, GRAVITATIONAL_CONSTANT},
};
use bevy::{ecs::query::QueryData, prelude::*};
use bevy_rapier2d::{
    plugin::{RapierContext, ReadRapierContext},
    prelude::*,
};
use keplerian_sim::StateVectors2D;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct VesselData {
    entity: Entity,
    parent: &'static CelestialParent,
    rail_mode: &'static mut RailMode,
    pos: &'static RootSpacePosition,
    vel: &'static RootSpaceLinearVelocity,
}

#[derive(QueryData)]
pub struct ParentData {
    entity: Entity,
    pos: &'static RootSpacePosition,
    vel: &'static RootSpaceLinearVelocity,
    mass: &'static AdditionalMassProperties,
}

fn write_sv_to_rail_inner(
    rapier_context: &RapierContext<'_>,
    mut vessel: VesselDataItem,
    parent: ParentDataItem,
    time: &Time,
) {
    let rel_pos = vessel.pos.0 - parent.pos.0;

    let touching = rapier_context
        .contact_pair(vessel.entity, parent.entity)
        .is_some_and(|c| c.has_any_active_contact());

    if touching {
        // TODO: Consider celestial rotation
        let radius = rel_pos.length();
        let angle = rel_pos.to_angle();
        let attachment = SurfaceAttachment { angle, radius };
        *vessel.rail_mode = RailMode::Surface(attachment);
        return;
    }

    let rel_vel = vessel.vel.0 - parent.vel.0;

    let parent_mass = match parent.mass {
        AdditionalMassProperties::Mass(mass) => *mass,
        AdditionalMassProperties::MassProperties(prop) => prop.mass,
    };

    let orbit = StateVectors2D {
        position: rel_pos,
        velocity: rel_vel,
    }
    .to_cached_orbit(
        GRAVITATIONAL_CONSTANT * parent_mass as f64,
        time.elapsed_secs_f64(),
    );

    *vessel.rail_mode = RailMode::Orbit(orbit);
}

pub fn write_sv_to_rail(
    rapier_context: ReadRapierContext,
    mut vessels: Query<VesselData, FilterLoadedVessels>,
    cel_query: Query<ParentData, (With<CelestialBody>, Without<Vessel>)>,
    time: Res<Time>,
) {
    let rapier_context = rapier_context
        .single()
        .expect("there should be only one rapier context");
    vessels.iter_mut().for_each(|vessel| {
        let Ok(parent) = cel_query.get(vessel.parent.entity) else {
            return;
        };
        write_sv_to_rail_inner(&rapier_context, vessel, parent, &time);
    });
}
