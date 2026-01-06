use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    components::{
        CelestialBody, Heightmap, ParentBody, Vessel,
        frames::{
            RigidSpaceTransform, RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition,
        },
    },
    plugins::frame_sync::FrameSyncPlugin,
    resources::ActiveVessel,
};

const DEMO_HEIGHTMAP: [f32; 10] = [10.0, 10.0, 10.0, 10.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0];
const CELESTIAL_RADIUS: f32 = 100.0;
const ALTITUDE: f32 = 1.5 * CELESTIAL_RADIUS;

fn demo_startup(mut commands: Commands) {
    commands.spawn((
        Camera::default(),
        Camera2d,
        Transform::from_rotation(Quat::from_rotation_z(0.0)),
    ));

    let body = commands
        .spawn((
            CelestialBody {
                radius: CELESTIAL_RADIUS,
            },
            Collider::ball(CELESTIAL_RADIUS),
            AdditionalMassProperties::Mass(10.0),
            Heightmap(Box::from(DEMO_HEIGHTMAP)),
            Transform::from_xyz(0.0, -ALTITUDE, 0.0),
        ))
        .id();

    let vessel_pos = RootSpacePosition(DVec2::new(0.0, 1.5 * ALTITUDE as f64));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(1.0, 0.0));

    let vessel = commands.spawn((
        Vessel,
        Collider::ball(10.0),
        RigidBody::Dynamic,
        AdditionalMassProperties::Mass(1e4),
        ParentBody(body),
        Transform::IDENTITY,
        RigidSpaceTransform(Transform::IDENTITY),
        RigidSpaceVelocity::zero(),
        vessel_pos,
        vessel_vel,
        GravityScale(0.0),
    ));
    let vessel_entity = vessel.id();

    commands.insert_resource(ActiveVessel {
        entity: vessel_entity,
        prev_tick_parent: body,
        prev_tick_position: vessel_pos,
        prev_tick_velocity: vessel_vel,
    });
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let physics = RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0).in_fixed_schedule();

        app.add_plugins(physics)
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(FrameSyncPlugin)
            .add_systems(Startup, demo_startup);
    }
}
