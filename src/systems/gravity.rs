//! Newtonian gravity application for loaded vessels

use bevy::{ecs::query::QueryData, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    components::{
        celestial::CelestialBody,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::CelestialParent,
        vessel::Vessel,
    },
    consts::{FilterLoadedVessels, GRAVITATIONAL_CONSTANT},
};

#[derive(QueryData)]
#[query_data(mutable)]
pub struct VesselData {
    name: NameOrEntity,
    vel: &'static mut RootSpaceLinearVelocity,
    pos: &'static RootSpacePosition,
    parent: &'static CelestialParent,
}

#[derive(QueryData)]
pub struct ParentData {
    pos: &'static RootSpacePosition,
    mass: &'static AdditionalMassProperties,
}

fn apply_gravity_inner(
    mut vessel: VesselDataItem,
    celestials: Query<ParentData, (With<CelestialBody>, Without<Vessel>)>,
    time: Time,
) {
    let Ok(parent) = celestials.get(vessel.parent.entity) else {
        error!("Vessel {} is missing a parent!", vessel.name);
        return;
    };

    let rel_pos = vessel.pos.0 - parent.pos.0;

    // Gravity: a_g = GM / r^2
    let r_sq = rel_pos.length_squared();
    if r_sq < 0.0 {
        return;
    }

    let grav_direction = rel_pos.normalize_or_zero();

    let mass = match parent.mass {
        AdditionalMassProperties::Mass(m) => *m,
        AdditionalMassProperties::MassProperties(prop) => prop.mass,
    } as f64;

    let accel = GRAVITATIONAL_CONSTANT * mass / r_sq;
    let delta_speed = accel * time.delta_secs_f64();
    let dv = grav_direction * delta_speed;

    vessel.vel.0 += dv;
}

pub fn apply_gravity(
    mut vessels: Query<VesselData, FilterLoadedVessels>,
    celestials: Query<ParentData, (With<CelestialBody>, Without<Vessel>)>,
    time: Res<Time>,
) {
    vessels.iter_mut().for_each(|vessel| {
        apply_gravity_inner(vessel, celestials, *time);
    });
}
