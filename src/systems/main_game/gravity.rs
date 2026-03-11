//! Newtonian gravity application for loaded vessels

use bevy::{ecs::query::QueryData, prelude::*};

use crate::{
    components::main_game::{
        celestial::CelestialBody,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::CelestialParent,
        vessel::Vessel,
    },
    consts::{FilterLoadedVessels, GRAVITATIONAL_CONSTANT, GRAVITY_MIN_RADIUS},
};

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct VesselData {
    name: NameOrEntity,
    pos: &'static mut RootSpacePosition,
    vel: &'static mut RootSpaceLinearVelocity,
    parent: &'static CelestialParent,
}

#[derive(QueryData)]
pub(crate) struct ParentData {
    pos: &'static RootSpacePosition,
    vel: &'static RootSpaceLinearVelocity,
    body_data: &'static CelestialBody,
}

fn apply_gravity_inner(
    mut vessel: VesselDataItem,
    celestials: Query<ParentData, Without<Vessel>>,
    time: &Time,
) {
    let Ok(parent) = celestials.get(vessel.parent.entity) else {
        error!("Vessel {} is missing a parent!", vessel.name);
        return;
    };

    let parent_mass = parent.body_data.mass;
    let parent_mu = parent_mass * GRAVITATIONAL_CONSTANT;

    let rel_pos = vessel.pos.0 - parent.pos.0;

    let delta_secs = time.delta_secs_f64();

    // Velocity Verlet
    // p(t + Δt) = p(t) + v(t) * Δt + 0.5a(t) * Δt^2;
    // v(t + Δt) = v(t) + 0.5 * (a(t) + a(t + Δt)) * Δt;

    let r_sq = rel_pos.length_squared().max(GRAVITY_MIN_RADIUS);
    let accel = -parent_mu * rel_pos / (r_sq.sqrt() * r_sq);
    vessel.pos.0 += vessel.vel.0 * delta_secs + 0.5 * accel * delta_secs.powi(2);

    // We assume the parent's orbit, if any, has negligible local curvature
    let new_parent_pos = RootSpacePosition(parent.pos.0 + parent.vel.0 * delta_secs);
    let new_rel_pos = vessel.pos.0 - new_parent_pos.0;
    let new_r_sq = new_rel_pos.length_squared().max(GRAVITY_MIN_RADIUS);
    let new_accel = -parent_mu * new_rel_pos / (new_r_sq.sqrt() * new_r_sq);
    vessel.vel.0 += 0.5 * (accel + new_accel) * delta_secs;
}

pub(crate) fn apply_gravity_and_velocity(
    mut vessels: Query<VesselData, FilterLoadedVessels>,
    celestials: Query<ParentData, Without<Vessel>>,
    time: Res<Time>,
) {
    vessels.iter_mut().for_each(|vessel| {
        apply_gravity_inner(vessel, celestials, &time);
    });
}
