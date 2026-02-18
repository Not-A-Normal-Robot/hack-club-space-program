use core::f64::consts::PI;
use std::sync::LazyLock;

use bevy::math::DVec2;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use hack_club_space_program::{
    builders::{celestial::CelestialBodyBuilder, vessel::VesselBuilder},
    components::{
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::{CelestialParent, RailMode, SurfaceAttachment},
    },
    consts::GRAVITATIONAL_CONSTANT,
    resources::ActiveVessel,
};
use keplerian_sim::{CompactOrbit2D, Orbit2D, OrbitTrait2D, StateVectors2D};

use crate::common::assert_sv_close;

mod common;

#[test]
fn test_writing_to_orbit_rails() {
    let mut app = common::setup_default();

    let (mesh, material) = common::empty_mesh_material(&mut app);

    let body_mass = 10.0f32;
    let body_mu = body_mass as f64 * GRAVITATIONAL_CONSTANT;

    let body = app
        .world_mut()
        .spawn(
            CelestialBodyBuilder {
                name: Name::new("Body"),
                mass: AdditionalMassProperties::Mass(body_mass),
                radius: 10.0,
                mesh,
                material,
                angle: 0.0,
            }
            .build_without_terrain(),
        )
        .id();

    let vessel_pos = RootSpacePosition(DVec2::new(0.0, 12.0));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(1.0, 0.0));

    let vessel = app
        .world_mut()
        .spawn(
            VesselBuilder {
                name: Name::new("Vessel"),
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
    let mut app = common::setup_default();

    let (mesh, material) = common::empty_mesh_material(&mut app);

    let body_mass = 10.0f32;

    let body = app
        .world_mut()
        .spawn(
            CelestialBodyBuilder {
                name: Name::new("Body"),
                mass: AdditionalMassProperties::Mass(body_mass),
                radius: 10.0,
                angle: 0.0,
                mesh,
                material,
            }
            .build_without_terrain(),
        )
        .id();

    let vessel_pos = RootSpacePosition(DVec2::new(0.0, 11.0));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(0.0, 0.0));

    let vessel = app
        .world_mut()
        .spawn(
            VesselBuilder {
                name: Name::new("Vessel"),
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

/// Environment:
/// - Alpha (1e6 radius)
///     - Alpharove (π radians, 1e6 alt) => (-1e6 0) (0 0)
///     - Alphasat (circular 2e6 peri, π mean) => (derive sv from orbit)
///     - Beta (1e5 radius) (circular 1e7 peri, 0 mean) => (derive sv from orbit)
///         - Betarove (3/2π radians, 1e5 alt) => (0 -1e5) (0 0)
///         - LOADED Betabase (beta pos + beta radius) (alpha vel + beta vel)
#[test]
fn test_rail_to_sv() {
    let mut app = common::setup_default();

    let (mesh, material) = common::empty_mesh_material(&mut app);

    const ALPHA_RADIUS: f64 = 1e6;
    const ALPHA_MASS: f64 = 1e20;

    const ALPHAROVE_ATTACHMENT: SurfaceAttachment = SurfaceAttachment {
        angle: PI,
        radius: ALPHA_RADIUS,
    };
    static ALPHASAT_ORBIT: LazyLock<Orbit2D> =
        LazyLock::new(|| Orbit2D::new_circular(2e6, PI, ALPHA_MASS * GRAVITATIONAL_CONSTANT));

    const BETA_RADIUS: f64 = 1e5;
    const BETA_MASS: f64 = 1e18;
    static BETA_ORBIT: LazyLock<Orbit2D> =
        LazyLock::new(|| Orbit2D::new_circular(2e5, 0.0, ALPHA_MASS * GRAVITATIONAL_CONSTANT));

    const BETAROVE_ATTACHMENT: SurfaceAttachment = SurfaceAttachment {
        angle: 1.5 * PI,
        radius: BETA_RADIUS,
    };

    static BETABASE_POS: LazyLock<RootSpacePosition> = LazyLock::new(|| {
        RootSpacePosition(DVec2::new(BETA_ORBIT.get_periapsis() + BETA_RADIUS, 0.0))
    });
    static BETABASE_VEL: LazyLock<RootSpaceLinearVelocity> =
        LazyLock::new(|| RootSpaceLinearVelocity(BETA_ORBIT.get_velocity_at_time(0.0)));

    let alpha = app
        .world_mut()
        .spawn(
            CelestialBodyBuilder {
                name: Name::new("Alpha"),
                radius: ALPHA_RADIUS as f32,
                mass: AdditionalMassProperties::Mass(ALPHA_MASS as f32),
                angle: 0.0,
                mesh: mesh.clone(),
                material: material.clone(),
            }
            .build_without_terrain(),
        )
        .id();

    let alpharove = app
        .world_mut()
        .spawn(
            VesselBuilder {
                name: Name::new("AlphaRove"),
                collider: Collider::ball(0.01),
                mass: AdditionalMassProperties::Mass(0.1),
                parent: CelestialParent { entity: alpha },
                rail_mode: RailMode::Surface(ALPHAROVE_ATTACHMENT),
                position: RootSpacePosition(DVec2::NAN),
                linvel: RootSpaceLinearVelocity(DVec2::NAN),
                angvel: 0.0,
                angle: 0.0,
            }
            .build_on_rails(),
        )
        .id();

    let alphasat = app
        .world_mut()
        .spawn(
            VesselBuilder {
                name: Name::new("AlphaSat"),
                collider: Collider::ball(0.01),
                mass: AdditionalMassProperties::Mass(0.1),
                parent: CelestialParent { entity: alpha },
                rail_mode: RailMode::Orbit(*ALPHASAT_ORBIT),
                position: RootSpacePosition(DVec2::NAN),
                linvel: RootSpaceLinearVelocity(DVec2::NAN),
                angvel: 0.0,
                angle: 0.0,
            }
            .build_on_rails(),
        )
        .id();

    let beta = app
        .world_mut()
        .spawn(
            CelestialBodyBuilder {
                name: Name::new("Beta"),
                radius: BETA_RADIUS as f32,
                mass: AdditionalMassProperties::Mass(BETA_MASS as f32),
                angle: 0.0,
                mesh: mesh.clone(),
                material: material.clone(),
            }
            .build_without_terrain(),
        )
        .insert((
            CelestialParent { entity: alpha },
            RailMode::Orbit(*BETA_ORBIT),
        ))
        .id();

    let betarove = app
        .world_mut()
        .spawn(
            VesselBuilder {
                name: Name::new("BetaRove"),
                collider: Collider::ball(0.01),
                mass: AdditionalMassProperties::Mass(0.1),
                parent: CelestialParent { entity: beta },
                rail_mode: RailMode::Surface(BETAROVE_ATTACHMENT),
                position: RootSpacePosition(DVec2::NAN),
                linvel: RootSpaceLinearVelocity(DVec2::NAN),
                angvel: 0.0,
                angle: 0.0,
            }
            .build_on_rails(),
        )
        .id();

    let betabase = app
        .world_mut()
        .spawn(
            VesselBuilder {
                name: Name::new("BetaBase"),
                collider: Collider::ball(0.0),
                mass: AdditionalMassProperties::Mass(0.0),
                parent: CelestialParent { entity: beta },
                rail_mode: RailMode::None,
                position: *BETABASE_POS,
                linvel: *BETABASE_VEL,
                angvel: 0.0,
                angle: 0.0,
            }
            .build_rigid(),
        )
        .id();

    dbg!(alpha, alpharove, alphasat, beta, betarove, betabase);

    app.world_mut().insert_resource(ActiveVessel {
        entity: betabase,
        prev_tick_parent: beta,
        prev_tick_position: *BETABASE_POS,
        prev_tick_velocity: *BETABASE_VEL,
    });

    app.update();
    let time = app.world().resource::<Time<Fixed>>();

    let alpharove_ref = app
        .world()
        .get_entity(alpharove)
        .expect("alpharove should exist");
    let alpharove_attachment = alpharove_ref
        .get::<RailMode>()
        .copied()
        .expect("alpharove should have rail mode")
        .as_attachment()
        .expect("alpharove rail mode should be attachment");
    assert_eq!(
        alpharove_attachment, ALPHAROVE_ATTACHMENT,
        "alpharove attachment should not change"
    );
    assert_sv_close(
        alpharove_ref,
        RootSpacePosition(DVec2::new(-ALPHA_RADIUS, 0.0)),
        RootSpaceLinearVelocity(DVec2::ZERO),
        1e-12,
    );

    let alphasat_ref = app
        .world()
        .get_entity(alphasat)
        .expect("alphasat should exist");
    let alphasat_orbit = alphasat_ref
        .get::<RailMode>()
        .copied()
        .expect("alphasat should have rail mode")
        .as_orbit()
        .expect("alphasat rail mode should be orbit");
    assert_eq!(
        alphasat_orbit, *ALPHASAT_ORBIT,
        "alphasat orbit should not change"
    );
    let alphasat_expected_sv = alphasat_orbit.get_state_vectors_at_time(time.elapsed_secs_f64());
    assert_sv_close(
        alphasat_ref,
        RootSpacePosition(alphasat_expected_sv.position),
        RootSpaceLinearVelocity(alphasat_expected_sv.velocity),
        1e-12,
    );

    let beta_ref = app.world().get_entity(beta).expect("beta should exist");
    let beta_orbit = beta_ref
        .get::<RailMode>()
        .copied()
        .expect("beta should have rail mode")
        .as_orbit()
        .expect("beta rail mode should be orbit");
    assert_eq!(beta_orbit, *BETA_ORBIT, "beta orbit should not change");
    let beta_expected_sv = beta_orbit.get_state_vectors_at_time(time.elapsed_secs_f64());
    assert_sv_close(
        beta_ref,
        RootSpacePosition(beta_expected_sv.position),
        RootSpaceLinearVelocity(beta_expected_sv.velocity),
        1e-12,
    );

    let betarove_ref = app
        .world()
        .get_entity(betarove)
        .expect("betarove should exist");
    let betarove_attachment = betarove_ref
        .get::<RailMode>()
        .copied()
        .expect("betarove should have rail mode")
        .as_attachment()
        .expect("betarove rail mode should be attachment");
    assert_eq!(
        betarove_attachment, BETAROVE_ATTACHMENT,
        "betarove attachment should not change"
    );
    let betarove_expected_pos =
        RootSpacePosition(beta_expected_sv.position + DVec2::new(0.0, -BETA_RADIUS));
    let betarove_expected_vel = RootSpaceLinearVelocity(beta_expected_sv.velocity);
    assert_sv_close(
        betarove_ref,
        betarove_expected_pos,
        betarove_expected_vel,
        1e-12,
    );

    let betabase_ref = app
        .world()
        .get_entity(betabase)
        .expect("betabase should exist");
    let betabase_expected_pos =
        RootSpacePosition(beta_expected_sv.position + DVec2::new(BETA_RADIUS, 0.0));
    let betabase_expected_vel = RootSpaceLinearVelocity(beta_expected_sv.velocity);
    assert_sv_close(
        betabase_ref,
        betabase_expected_pos,
        betabase_expected_vel,
        1e-10,
    );

    // Long term testing
    (0..1000).for_each(|_| app.update());

    let beta_ref = app.world().get_entity(beta).expect("beta should exist");
    let beta_pos = beta_ref
        .get::<RootSpacePosition>()
        .copied()
        .expect("beta should have pos");
    let beta_vel = beta_ref
        .get::<RootSpaceLinearVelocity>()
        .copied()
        .expect("beta should have vel");

    let betabase_ref = app
        .world()
        .get_entity(betabase)
        .expect("betabase should exist");
    let betabase_expected_pos = RootSpacePosition(beta_pos.0 + DVec2::new(BETA_RADIUS, 0.0));
    let betabase_expected_vel = beta_vel;
    assert_sv_close(
        betabase_ref,
        betabase_expected_pos,
        betabase_expected_vel,
        1e-7,
    );
}
