//! Newtonian gravity application for loaded vessels

use bevy::{ecs::query::QueryData, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    components::{
        celestial::CelestialBody, frames::RootSpacePosition, relations::CelestialParent,
        vessel::Vessel,
    },
    consts::{FilterLoadedVessels, FilterUnloadedVessels, GRAVITATIONAL_CONSTANT},
};

#[derive(QueryData)]
#[query_data(mutable)]
pub struct VesselData {
    name: NameOrEntity,
    pos: &'static RootSpacePosition,
    parent: &'static CelestialParent,
    force: &'static mut ExternalForce,
    mass: &'static AdditionalMassProperties,
}

#[derive(QueryData)]
pub struct ParentData {
    pos: &'static RootSpacePosition,
    mass: &'static AdditionalMassProperties,
}

fn apply_gravity_inner(
    mut vessel: VesselDataItem,
    celestials: Query<ParentData, (With<CelestialBody>, Without<Vessel>)>,
) {
    let Ok(parent) = celestials.get(vessel.parent.entity) else {
        error!("Vessel {} is missing a parent!", vessel.name);
        return;
    };

    let rel_pos = vessel.pos.0 - parent.pos.0;

    // Gravity: a_g = GM / r^2
    let r_sq = rel_pos.length_squared().max(1e-9);
    let grav_direction = -rel_pos.normalize_or_zero();

    let m1 = match vessel.mass {
        AdditionalMassProperties::Mass(m) => *m,
        AdditionalMassProperties::MassProperties(prop) => prop.mass,
    } as f64;
    let m2 = match parent.mass {
        AdditionalMassProperties::Mass(m) => *m,
        AdditionalMassProperties::MassProperties(prop) => prop.mass,
    } as f64;

    let force = GRAVITATIONAL_CONSTANT * m1 * m2 / r_sq;
    let force = force * grav_direction;

    vessel.force.force = Vec2::new(force.x as f32, force.y as f32);
}

pub fn apply_gravity(
    mut vessels: Query<VesselData, FilterLoadedVessels>,
    celestials: Query<ParentData, (With<CelestialBody>, Without<Vessel>)>,
) {
    vessels.iter_mut().for_each(|vessel| {
        apply_gravity_inner(vessel, celestials);
    });
}

pub fn unapply_gravity_to_unloaded(mut vessels: Query<&mut ExternalForce, FilterUnloadedVessels>) {
    vessels
        .iter_mut()
        .for_each(|mut force| *force = ExternalForce::default())
}
