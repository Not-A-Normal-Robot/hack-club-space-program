use core::{ops::Sub, time::Duration};

use crate::{
    components::{
        celestial::CelestialBody,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::{CelestialChildren, CelestialParent, RailMode, SurfaceAttachment},
        vessel::Vessel,
    },
    consts::{FilterLoadedVessels, FilterUnloadedVessels, GRAVITATIONAL_CONSTANT},
};
use bevy::{ecs::query::QueryData, math::DVec2, prelude::*};
use bevy_rapier2d::{
    plugin::{RapierContext, ReadRapierContext},
    prelude::*,
};
use keplerian_sim::{OrbitTrait2D, StateVectors2D};

type FilterUnloadedVesselOrCelestialBody = Or<(FilterUnloadedVessels, With<CelestialBody>)>;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct NodeData {
    rail_mode: &'static RailMode,
    pos: &'static mut RootSpacePosition,
    vel: &'static mut RootSpaceLinearVelocity,
    children: Option<&'static CelestialChildren>,
}

#[derive(QueryData)]
pub struct RootData {
    children: &'static CelestialChildren,
}

/// State vector query data
#[derive(QueryData)]
#[query_data(mutable)]
pub struct SvData {
    pos: &'static RootSpacePosition,
    vel: &'static RootSpaceLinearVelocity,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct ChildData {
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

const ZERO_SV: (RootSpacePosition, RootSpaceLinearVelocity) = (
    RootSpacePosition(DVec2::ZERO),
    RootSpaceLinearVelocity(DVec2::ZERO),
);

fn write_sv_to_rail_inner(
    rapier_context: &RapierContext<'_>,
    mut vessel: ChildDataItem,
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
    mut vessels: Query<ChildData, FilterLoadedVessels>,
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct RelativeStateVectors {
    position: DVec2,
    velocity: DVec2,
}

impl Sub for RelativeStateVectors {
    type Output = RelativeStateVectors;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            position: self.position - rhs.position,
            velocity: self.velocity - rhs.velocity,
        }
    }
}

fn convert_rail_to_relative_sv(rail: RailMode, time: Duration) -> RelativeStateVectors {
    match rail {
        RailMode::None => unreachable!("RailMode::None should have been skipped"),
        RailMode::Orbit(o) => {
            let sv = o.get_state_vectors_at_time(time.as_secs_f64());
            RelativeStateVectors {
                position: sv.position,
                velocity: sv.velocity,
            }
        }
        RailMode::Surface(a) => {
            // TODO: Consider celestial rotation
            let position = DVec2::from_angle(a.angle) * a.radius;
            RelativeStateVectors {
                position,
                velocity: DVec2::ZERO,
            }
        }
    }
}

/// For every node's child:
/// - Try to find it using the on_rails query
///   - Calculate new SV using rail_mode and parent_sv
///   - Calculate SV difference
///   - Recurse, changing the parent_sv and accum_shift
/// - Try to find it using the off_rails query
///   - Shift SV using accum_shift
fn write_rail_to_sv_inner(
    node: Entity,
    parent_sv: (RootSpacePosition, RootSpaceLinearVelocity),
    accum_shift: (RootSpacePosition, RootSpaceLinearVelocity),
    mut on_rails_query: Query<NodeData, FilterUnloadedVesselOrCelestialBody>,
    mut off_rails_query: Query<SvData, (With<CelestialParent>, FilterLoadedVessels)>,
    time: Time,
) {
    let Ok(mut node) = on_rails_query.get_mut(node) else {
        return;
    };

    if node.rail_mode.is_none() {
        return;
    };

    let old_rel_sv = convert_rail_to_relative_sv(*node.rail_mode, time.elapsed() - time.delta());
    let new_rel_sv = convert_rail_to_relative_sv(*node.rail_mode, time.elapsed());

    let new_root_pos = RootSpacePosition(parent_sv.0.0 + new_rel_sv.position);
    let new_root_vel = RootSpaceLinearVelocity(parent_sv.1.0 + new_rel_sv.velocity);

    let diff_rel_sv = new_rel_sv - old_rel_sv;

    *node.pos = new_root_pos;
    *node.vel = new_root_vel;

    let Some(children) = node.children else {
        return;
    };

    let children = children.clone_to_box();

    children.into_iter().for_each(|child| {
        write_rail_to_sv_inner(
            child,
            (new_root_pos, new_root_vel),
            (
                RootSpacePosition(accum_shift.0.0 + diff_rel_sv.position),
                RootSpaceLinearVelocity(accum_shift.1.0 + diff_rel_sv.velocity),
            ),
            on_rails_query.reborrow(),
            off_rails_query.reborrow(),
            time,
        );
    });
}

pub fn write_rail_to_sv(
    roots: Query<RootData, Without<CelestialParent>>,
    mut on_rails_query: Query<NodeData, FilterUnloadedVesselOrCelestialBody>,
    mut off_rails_query: Query<SvData, (With<CelestialParent>, FilterLoadedVessels)>,
    time: Res<Time>,
) {
    roots.iter().for_each(|root| {
        root.children.iter().for_each(|node| {
            write_rail_to_sv_inner(
                node,
                ZERO_SV,
                ZERO_SV,
                on_rails_query.reborrow(),
                off_rails_query.reborrow(),
                *time,
            )
        })
    });
}
