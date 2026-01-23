use core::f64::consts::PI;

use bevy::math::DVec2;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use hack_club_space_program::{
    builders::{celestial::CelestialBodyBuilder, vessel::VesselBuilder},
    components::{
        celestial::Heightmap,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::{CelestialParent, RailMode, SurfaceAttachment},
    },
    consts::GRAVITATIONAL_CONSTANT,
    resources::ActiveVessel,
};
use keplerian_sim::{CompactOrbit2D, StateVectors2D};

mod common;

#[test]
fn test_writing_to_orbit_rails() {
    let mut app = common::setup(true);

    let body_mass = 10.0f32;
    let body_mu = body_mass as f64 * GRAVITATIONAL_CONSTANT;

    let body = app
        .world_mut()
        .spawn(
            CelestialBodyBuilder {
                mass: AdditionalMassProperties::Mass(body_mass),
                radius: 10.0,
                heightmap: Heightmap(Box::from([])),
                angle: 0.0,
            }
            .build(),
        )
        .id();

    let vessel_pos = RootSpacePosition(DVec2::new(0.0, 12.0));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(1.0, 0.0));

    let vessel = app
        .world_mut()
        .spawn(
            VesselBuilder {
                angle: 0.0,
                angvel: 0.0,
                collider: Collider::ball(1.0),
                linvel: vessel_vel,
                mass: AdditionalMassProperties::Mass(1.0),
                parent: CelestialParent { entity: body },
                rail_mode: RailMode::None,
                position: vessel_pos,
            }
            .build_rigid(),
        )
        .id();

    app.insert_resource(ActiveVessel {
        entity: vessel,
        prev_tick_parent: body,
        prev_tick_position: vessel_pos,
        prev_tick_velocity: vessel_vel,
    });

    app.update();

    let vessel_entity = app.world().get_entity(vessel).expect("vessel should exist");
    let pos = vessel_entity
        .get::<RootSpacePosition>()
        .expect("vessel should have pos");
    let vel = vessel_entity
        .get::<RootSpaceLinearVelocity>()
        .expect("vessel should have vel");
    let rail = vessel_entity
        .get::<RailMode>()
        .expect("vessel should have rail");

    let orbit = rail.as_orbit().expect("rail mode should be orbit");
    let expected_orbit = StateVectors2D {
        position: pos.0,
        velocity: vel.0,
    }
    .to_compact_orbit(
        body_mu,
        app.world()
            .get_resource::<Time<Fixed>>()
            .expect("time should exist")
            .elapsed_secs_f64(),
    );

    assert_eq!(CompactOrbit2D::from(orbit), expected_orbit);
}

#[test]
fn test_writing_to_surface_rails() {
    let mut app = common::setup(true);

    let body_mass = 10.0f32;
    let body_mu = body_mass as f64 * GRAVITATIONAL_CONSTANT;

    let body = app
        .world_mut()
        .spawn(
            CelestialBodyBuilder {
                mass: AdditionalMassProperties::Mass(body_mass),
                radius: 10.0,
                heightmap: Heightmap(Box::from([])),
                angle: 0.0,
            }
            .build(),
        )
        .id();

    let vessel_pos = RootSpacePosition(DVec2::new(0.0, 11.0));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(0.0, 0.0));

    let vessel = app
        .world_mut()
        .spawn(
            VesselBuilder {
                angle: 0.0,
                angvel: 0.0,
                collider: Collider::ball(1.0),
                linvel: vessel_vel,
                mass: AdditionalMassProperties::Mass(1.0),
                parent: CelestialParent { entity: body },
                position: vessel_pos,
                rail_mode: RailMode::None,
            }
            .build_rigid(),
        )
        .id();

    app.insert_resource(ActiveVessel {
        entity: vessel,
        prev_tick_parent: body,
        prev_tick_position: vessel_pos,
        prev_tick_velocity: vessel_vel,
    });

    app.update();

    let rail = app
        .world()
        .get::<RailMode>(vessel)
        .expect("vessel should have rail mode");

    let expected_att = SurfaceAttachment {
        angle: PI / 2.0,
        radius: vessel_pos.y,
    };

    assert_eq!(
        rail.as_attachment(),
        Some(expected_att),
        "rail didn't match expected value"
    );
}
