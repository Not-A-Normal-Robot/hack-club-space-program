//! Integration test for reference frames

use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;
use hack_club_space_program::{
    components::{
        CelestialBody, Heightmap, SimCamera, SimCameraTransform, Vessel,
        frames::{
            RigidSpaceTransform, RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition,
        },
    },
    resources::ActiveVessel,
};

mod common;

#[test]
fn reference_frames() {
    let mut app = common::setup(true);

    app.world_mut().spawn((
        Camera {
            is_active: true,
            ..Default::default()
        },
        Camera2d,
        SimCamera,
        SimCameraTransform {
            translation: DVec2::ZERO,
            zoom: 1.0,
        },
        Transform::from_rotation(Quat::from_rotation_z(0.0)),
    ));

    let body = app
        .world_mut()
        .spawn((CelestialBody { radius: 1.0 }, Heightmap(Box::from([]))))
        .id();

    let vessel_pos = RootSpacePosition(DVec2::new(0.5, 1.5));
    let vessel_vel = RootSpaceLinearVelocity(DVec2::new(1.0, 0.0));

    let vessel = app
        .world_mut()
        .spawn((
            Vessel,
            Collider::ball(10.0),
            RigidBody::Dynamic,
            AdditionalMassProperties::Mass(1e4),
            Transform::IDENTITY,
            RigidSpaceTransform(Transform::IDENTITY),
            RigidSpaceVelocity::zero(),
            vessel_pos,
            vessel_vel,
            GravityScale(0.0),
        ))
        .id();

    app.world_mut().insert_resource(ActiveVessel {
        entity: vessel,
        prev_tick_parent: body,
        prev_tick_position: vessel_pos,
        prev_tick_velocity: vessel_vel,
    });

    app.update();

    todo!("Write assertions");
}
