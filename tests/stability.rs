use bevy::prelude::*;
use bevy_rapier2d::prelude::{AdditionalMassProperties, Collider};
use hack_club_space_program::{
    builders::{celestial::CelestialBodyBuilder, vessel::VesselBuilder},
    components::main_game::{
        celestial::Terrain,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::{CelestialParent, RailMode},
    },
    consts::{GRAVITATIONAL_CONSTANT, GRAVITY_MIN_RADIUS},
    resources::simulation::ActiveVessel,
};
use keplerian_sim::{Orbit2D, OrbitTrait2D};

mod common;

const BODY_RADIUS: f64 = 6_371_137.0;
const BODY_MASS: f64 = 5.972_168e24;
const BODY_MU: f64 = BODY_MASS * GRAVITATIONAL_CONSTANT;

const ORBIT_RADIUS: f64 = BODY_RADIUS + 100e3;

const TICKS: usize = 10000;
const TOLERANCE: f64 = 1e-11;

struct OrbitSim {
    position: RootSpacePosition,
    velocity: RootSpaceLinearVelocity,
}

impl OrbitSim {
    fn step(&mut self, delta_secs: f64) {
        // Velocity Verlet
        // p(t + Δt) = p(t) + v(t) * Δt + 0.5a(t) * Δt^2;
        // v(t + Δt) = v(t) + 0.5 * (a(t) + a(t + Δt)) * Δt;
        let r_sq = self.position.length_squared().max(GRAVITY_MIN_RADIUS);
        let accel = -BODY_MU * self.position.0 / (r_sq.sqrt() * r_sq);
        self.position.0 += self.velocity.0 * delta_secs + 0.5 * accel * delta_secs.powi(2);
        let new_r_sq = self.position.length_squared().max(GRAVITY_MIN_RADIUS);
        let new_accel = -BODY_MU * self.position.0 / (new_r_sq.sqrt() * new_r_sq);
        self.velocity.0 += 0.5 * (accel + new_accel) * delta_secs;
    }
}

#[test]
fn test_orbit_stability() {
    let mut app = common::setup_default();

    let (mesh, material) = common::empty_mesh_material(&mut app);

    let body = app
        .world_mut()
        .spawn(
            #[expect(clippy::cast_possible_truncation)]
            CelestialBodyBuilder {
                name: Name::new("Earth"),
                mesh: mesh.clone(),
                material: material.clone(),
                angle: 0.0,
                mass: BODY_MASS,
                radius: BODY_RADIUS as f32,
            }
            .build_with_terrain(Terrain {
                frequency: 2.0,
                gain: 0.5,
                seed: 17,
                lacunarity: 1.0,
                octaves: 3,
                offset: 1.0,
                multiplier: 1.0,
                subdivs: 4,
            }),
        )
        .id();

    let vessel_orbit = Orbit2D::new_circular(ORBIT_RADIUS, 0.0, BODY_MU);
    let vessel_init_sv = vessel_orbit.get_state_vectors_at_eccentric_anomaly(0.0);

    dbg!(vessel_init_sv);

    let vessel = app
        .world_mut()
        .spawn(
            VesselBuilder {
                name: Name::new("Satellite"),
                collider: Collider::ball(0.0),
                mass: AdditionalMassProperties::Mass(1.0),
                parent: CelestialParent { entity: body },
                rail_mode: RailMode::None,
                position: RootSpacePosition(vessel_init_sv.position),
                linvel: RootSpaceLinearVelocity(vessel_init_sv.velocity),
                mesh,
                material,
                angvel: 0.0,
                angle: 0.0,
            }
            .build_rigid(),
        )
        .id();

    app.insert_resource(ActiveVessel {
        entity: vessel,
        prev_tick_position: RootSpacePosition(vessel_init_sv.position),
        prev_tick_velocity: RootSpaceLinearVelocity(vessel_init_sv.velocity),
        prev_tick_parent: body,
    });

    for i in 0..TICKS {
        app.update();
        eprintln!("Tick {i}");

        let vessel_pos = app
            .world()
            .get::<RootSpacePosition>(vessel)
            .expect("Vessel should have root position");
        let vessel_vel = app
            .world()
            .get::<RootSpaceLinearVelocity>(vessel)
            .expect("Vessel should have RSLV");

        dbg!(vessel_pos, vessel_vel);

        let vessel_altitude = vessel_pos.0.length();
        let vessel_speed = vessel_vel.0.length();

        let altitude_frac = vessel_altitude / ORBIT_RADIUS;
        let speed_frac = vessel_speed / vessel_init_sv.velocity.length();

        assert!(
            (altitude_frac - 1.0).abs() < TOLERANCE,
            "Altitude {vessel_altitude} is too far away from initial altitude {ORBIT_RADIUS}"
        );
        assert!(
            (speed_frac - 1.0).abs() < TOLERANCE,
            "Speed {vessel_speed} is too far away from initial speed of {}",
            vessel_init_sv.velocity.length()
        );
    }
}

#[test]
fn test_simplified_model() {
    let vessel_orbit = Orbit2D::new_circular(ORBIT_RADIUS, 0.0, BODY_MU);
    let vessel_init_sv = vessel_orbit.get_state_vectors_at_eccentric_anomaly(0.0);

    dbg!(vessel_init_sv);

    let mut sim_sv = OrbitSim {
        position: RootSpacePosition(vessel_init_sv.position),
        velocity: RootSpaceLinearVelocity(vessel_init_sv.velocity),
    };

    for i in 0..TICKS {
        sim_sv.step(1.0 / 64.0);

        let vessel_altitude = sim_sv.position.length();
        let vessel_speed = sim_sv.velocity.length();

        let altitude_frac = vessel_altitude / ORBIT_RADIUS;
        let speed_frac = vessel_speed / vessel_init_sv.velocity.length();

        assert!(
            (altitude_frac - 1.0).abs() < TOLERANCE,
            "Tick {i}: Altitude {vessel_altitude} is too far away from initial altitude {ORBIT_RADIUS}"
        );
        assert!(
            (speed_frac - 1.0).abs() < TOLERANCE,
            "Tick {i}: Speed {vessel_speed} is too far away from initial speed of {}",
            vessel_init_sv.velocity.length()
        );
    }
}

#[test]
fn test_equal_to_simplified() {
    let mut app = common::setup_default();

    let (mesh, material) = common::empty_mesh_material(&mut app);

    let body = app
        .world_mut()
        .spawn(
            #[expect(clippy::cast_possible_truncation)]
            CelestialBodyBuilder {
                name: Name::new("Earth"),
                mesh: mesh.clone(),
                material: material.clone(),
                angle: 0.0,
                mass: BODY_MASS,
                radius: BODY_RADIUS as f32,
            }
            .build_with_terrain(Terrain {
                frequency: 2.0,
                gain: 0.5,
                seed: 17,
                lacunarity: 1.0,
                octaves: 3,
                offset: 1.0,
                multiplier: 1.0,
                subdivs: 4,
            }),
        )
        .id();

    let vessel_orbit = Orbit2D::new_circular(ORBIT_RADIUS, 0.0, BODY_MU);
    let vessel_init_sv = vessel_orbit.get_state_vectors_at_eccentric_anomaly(0.0);

    dbg!(vessel_init_sv);

    let mut sim_sv = OrbitSim {
        position: RootSpacePosition(vessel_init_sv.position),
        velocity: RootSpaceLinearVelocity(vessel_init_sv.velocity),
    };

    let vessel = app
        .world_mut()
        .spawn(
            VesselBuilder {
                name: Name::new("Satellite"),
                collider: Collider::ball(0.0),
                mass: AdditionalMassProperties::Mass(1.0),
                parent: CelestialParent { entity: body },
                rail_mode: RailMode::None,
                position: RootSpacePosition(vessel_init_sv.position),
                linvel: RootSpaceLinearVelocity(vessel_init_sv.velocity),
                mesh,
                material,
                angvel: 0.0,
                angle: 0.0,
            }
            .build_rigid(),
        )
        .id();

    app.insert_resource(ActiveVessel {
        entity: vessel,
        prev_tick_position: RootSpacePosition(vessel_init_sv.position),
        prev_tick_velocity: RootSpaceLinearVelocity(vessel_init_sv.velocity),
        prev_tick_parent: body,
    });

    for i in 0..TICKS {
        app.update();
        sim_sv.step(app.world().resource::<Time<Fixed>>().delta_secs_f64());

        let app_pos = app
            .world()
            .get::<RootSpacePosition>(vessel)
            .copied()
            .expect("vessel should have root position");
        let app_vel = app
            .world()
            .get::<RootSpaceLinearVelocity>(vessel)
            .copied()
            .expect("vessel should have RSLV");

        assert_eq!(app_pos, sim_sv.position, "position mismatch at tick {i}");
        assert_eq!(app_vel, sim_sv.velocity, "velocity mismatch at tick {i}");
    }
}
