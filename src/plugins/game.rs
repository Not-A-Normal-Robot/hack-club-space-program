use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::{prelude::*, rapier::prelude::IntegrationParameters};

use crate::{
    builders::{celestial::CelestialBodyBuilder, vessel::VesselBuilder},
    components::{
        camera::{SimCamera, SimCameraOffset, SimCameraZoom},
        celestial::Heightmap,
        frames::{RootSpaceLinearVelocity, RootSpacePosition},
        relations::{CelestialParent, RailMode},
    },
    plugins::physics::HcspPhysicsPlugin,
    resources::ActiveVessel,
};

const DEMO_HEIGHTMAP: [f32; 10] = [10.0, 10.0, 10.0, 10.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0];
const CELESTIAL_RADIUS: f32 = 100.0;
const ALTITUDE: f32 = 1.5 * CELESTIAL_RADIUS;

fn demo_startup(mut commands: Commands) {
    commands.spawn((
        Camera {
            is_active: true,
            ..Default::default()
        },
        Camera2d,
        SimCamera,
        SimCameraOffset::Detached(RootSpacePosition(DVec2::ZERO)),
        SimCameraZoom(1.0),
        Transform::from_rotation(Quat::from_rotation_z(0.0)),
    ));

    let body = CelestialBodyBuilder {
        name: Name::new("Body"),
        radius: CELESTIAL_RADIUS,
        heightmap: Heightmap(Box::from(DEMO_HEIGHTMAP)),
        mass: AdditionalMassProperties::Mass(1e30),
        angle: 0.0,
    }
    .build();
    let body = commands.spawn(body).id();

    let vessel_pos = RootSpacePosition(DVec2::new(-1.5 * ALTITUDE as f64, 0.5 * ALTITUDE as f64));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(100.0, 0.0));

    let vessel = VesselBuilder {
        name: Name::new("Vessel"),
        collider: Collider::ball(10.0),
        mass: AdditionalMassProperties::Mass(1e6),
        parent: CelestialParent { entity: body },
        rail_mode: RailMode::None,
        position: vessel_pos,
        linvel: vessel_vel,
        angvel: 0.0,
        angle: 0.0,
    }
    .build_rigid();
    let mut vessel = commands.spawn(vessel);
    vessel.insert(Sleeping::disabled());
    let vessel_entity = vessel.id();

    commands.insert_resource(ActiveVessel {
        entity: vessel_entity,
        prev_tick_parent: body,
        prev_tick_position: vessel_pos,
        prev_tick_velocity: vessel_vel,
    });
}

pub struct GameLogicPlugin;

pub const RAPIER_CONFIGURATION: RapierConfiguration = RapierConfiguration {
    gravity: Vec2::ZERO,
    physics_pipeline_active: true,
    scaled_shape_subdivision: 10,
    force_update_from_transform_changes: false,
};

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        let physics = RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0)
            .in_fixed_schedule()
            .with_custom_initialization(
                RapierContextInitialization::InitializeDefaultRapierContext {
                    integration_parameters: IntegrationParameters::default(),
                    rapier_configuration: RAPIER_CONFIGURATION,
                },
            );

        app.add_plugins((physics, HcspPhysicsPlugin));
    }
}

pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, demo_startup)
            .add_plugins(RapierDebugRenderPlugin {
                enabled: true,
                default_collider_debug: ColliderDebug::AlwaysRender,
                mode: DebugRenderMode::all(),
                style: DebugRenderStyle {
                    rigid_body_axes_length: 20.0,
                    ..Default::default()
                },
            });
    }
}
