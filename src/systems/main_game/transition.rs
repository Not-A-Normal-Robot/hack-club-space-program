use core::f64::consts::PI;

use crate::{
    builders::{camera::SimCameraBuilder, celestial::CelestialBodyBuilder, vessel::VesselBuilder},
    components::main_game::{
        camera::{SimCamera, SimCameraOffset, SimCameraZoom},
        celestial::Terrain,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::{CelestialParent, RailMode},
    },
    consts::GRAVITATIONAL_CONSTANT,
    resources::simulation::ActiveVessel,
};
use bevy::{asset::RenderAssetUsages, math::DVec2, mesh::PrimitiveTopology, prelude::*};
use bevy_rapier2d::prelude::*;
use keplerian_sim::{Orbit2D, OrbitTrait2D};

const CELESTIAL_RADIUS: f64 = 6_371_137.0;
const CELESTIAL_MASS: f64 = 5.972_168e24;
const ALTITUDE: f64 = CELESTIAL_RADIUS + 100e3;

pub(crate) fn init_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // TODO: Load from save
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let mesh = meshes.add(mesh);

    let material = ColorMaterial::from_color(Color::srgba(1.0, 1.0, 1.0, 0.2));
    let material = materials.add(material);

    let body = CelestialBodyBuilder {
        name: Name::new("Body"),
        #[expect(clippy::cast_possible_truncation)]
        radius: CELESTIAL_RADIUS as f32,
        mass: CELESTIAL_MASS,
        angle: 0.0,
        mesh: Mesh2d(mesh),
        material: MeshMaterial2d(material.clone()),
    }
    .build_with_terrain(Terrain {
        seed: 2401,
        octaves: 6,
        frequency: 400.0,
        gain: 0.4,
        lacunarity: 0.6,
        offset: CELESTIAL_RADIUS,
        multiplier: CELESTIAL_RADIUS * 0.001,
        subdivs: 6,
    });
    let body = commands.spawn(body).id();

    let orbit = Orbit2D::new_circular(ALTITUDE, 0.0, CELESTIAL_MASS * GRAVITATIONAL_CONSTANT);
    let vessel_init_sv = orbit.get_state_vectors_at_true_anomaly(PI / 2.0);
    let vessel_pos = RootSpacePosition(vessel_init_sv.position);
    let vessel_vel = RootSpaceLinearVelocity(vessel_init_sv.velocity);
    let vessel_half_x = 10.0;
    let vessel_half_y = 20.0;

    let mesh = Mesh2d(meshes.add(Rectangle::new(vessel_half_x * 2.0, vessel_half_y * 2.0)));

    dbg!(vessel_init_sv);
    let vessel = VesselBuilder {
        name: Name::new("Vessel"),
        collider: Collider::cuboid(vessel_half_x, vessel_half_y),
        mass: AdditionalMassProperties::Mass(1000.0),
        parent: CelestialParent { entity: body },
        rail_mode: RailMode::None,
        position: vessel_pos,
        linvel: vessel_vel,
        angvel: 0.5,
        angle: 0.0,
        mesh,
        material: MeshMaterial2d(material),
    }
    .build_rigid();
    let vessel = commands.spawn(vessel);
    let vessel_entity = vessel.id();

    commands.spawn(
        SimCameraBuilder {
            offset: SimCameraOffset::Attached {
                entity: vessel_entity,
                last_known_pos: vessel_pos,
                offset: DVec2::ZERO,
            },
            zoom: SimCameraZoom(1.0),
            transform: Transform::IDENTITY,
        }
        .build(true),
    );

    commands.insert_resource(ActiveVessel {
        entity: vessel_entity,
        prev_tick_parent: body,
        prev_tick_position: vessel_pos,
        prev_tick_velocity: vessel_vel,
    });
}

type FilterInGameObjects = Or<(With<RigidBody>, With<SimCamera>)>;

pub(crate) fn exit_game(mut commands: Commands, sim_objects: Query<Entity, FilterInGameObjects>) {
    // TODO: Save

    for obj in sim_objects {
        commands
            .entity(obj)
            .queue_silenced(|e: EntityWorldMut<'_>| e.despawn());
    }

    commands.remove_resource::<ActiveVessel>();
    commands.remove_resource::<ClearColor>();
}
