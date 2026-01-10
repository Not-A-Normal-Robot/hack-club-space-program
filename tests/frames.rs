//! Integration test for reference frames

use crate::common::{Assertions, AssertionsCollection};
use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;
use hack_club_space_program::{
    components::{
        CelestialBody, Heightmap, SimCamera, SimCameraTransform, Vessel,
        frames::{
            CameraSpaceTransform, RigidSpaceTransform, RigidSpaceVelocity, RootSpaceLinearVelocity,
            RootSpacePosition,
        },
    },
    resources::ActiveVessel,
};
use std::sync::LazyLock;

mod common;

#[derive(Clone, Copy)]
struct PostTickAssertions {
    pub vessel: TransformAssertions,
    pub body: TransformAssertions,
    pub extra_assertions: Option<fn(&App, TestEntities)>,
}

#[derive(Clone, Copy)]
struct TransformAssertions {
    pub root_pos: Option<RootSpacePosition>,
    pub root_vel: Option<RootSpaceLinearVelocity>,
    pub rig_tf: Option<RigidSpaceTransform>,
    pub rig_vel: Option<RigidSpaceVelocity>,
    pub cam_tf: Option<CameraSpaceTransform>,
}

impl TransformAssertions {
    fn check_assertions(&self, entity: EntityRef<'_>, camera_tf: SimCameraTransform) {
        let root_pos = entity.get::<RootSpacePosition>().cloned();
        assert_eq!(root_pos, self.root_pos);
        let root_vel = entity.get::<RootSpaceLinearVelocity>().cloned();
        assert_eq!(root_vel, self.root_vel);
        let rig_tf = entity.get::<RigidSpaceTransform>().cloned();
        assert_eq!(rig_tf, self.rig_tf);
        let rig_vel = entity.get::<RigidSpaceVelocity>().cloned();
        assert_eq!(rig_vel, self.rig_vel);
        if let Some(asserted_cam_tf) = self.cam_tf {
            let rig_tf = rig_tf
                .expect("rigid-space transform should exist for camera-space transform assertion");
            let cam_tf = root_pos
                .expect("root-space position should exist for camera-space transform assertion")
                .to_camera_space_transform(rig_tf.0.rotation, camera_tf);
            assert_eq!(cam_tf, asserted_cam_tf);
        }
    }
}

#[derive(Clone, Copy)]
struct TestEntitiesRequest {
    vessel: Entity,
    body: Entity,
    camera: Entity,
}

#[derive(Clone, Copy)]
struct TestEntities<'a> {
    vessel: EntityRef<'a>,
    body: EntityRef<'a>,
    camera: EntityRef<'a>,
}

impl<'a> TestEntities<'a> {
    fn get_entities(app: &'a App, vessel: Entity, body: Entity, camera: Entity) -> Self {
        let [vessel, body, camera] =
            [vessel, body, camera].map(|e| app.world().get_entity(e).unwrap());

        Self {
            vessel,
            body,
            camera,
        }
    }
}

impl Assertions for PostTickAssertions {
    type ExtraData = TestEntitiesRequest;
    fn check_assertions(&self, app: &App, extra: Self::ExtraData) {
        let entities = TestEntities::get_entities(app, extra.vessel, extra.body, extra.camera);
        let camera_tf = entities
            .camera
            .get::<SimCameraTransform>()
            .copied()
            .expect("could not find SimCameraTransform");

        self.body.check_assertions(entities.body, camera_tf);
        self.vessel.check_assertions(entities.vessel, camera_tf);
        if let Some(extra_assertions) = self.extra_assertions {
            extra_assertions(app, entities);
        }
    }
}

static ASSERTION_COLLECTION: LazyLock<Box<[PostTickAssertions]>> = LazyLock::new(|| {
    Box::new([PostTickAssertions {
        body: TransformAssertions {
            root_pos: Some(RootSpacePosition(DVec2::ZERO)),
            root_vel: None,
            rig_tf: Some(RigidSpaceTransform(Transform {
                translation: Vec3::new(0.5 + 1.0 / 32.0, 1.5, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            })),
            rig_vel: Some(RigidSpaceVelocity {
                angvel: 0.0,
                linvel: Vec2::new(-1.0, 0.0),
            }),
            cam_tf: Some(CameraSpaceTransform(Transform {
                translation: Vec3::new(0.5, 1.5, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            })),
        },
        vessel: TransformAssertions {
            root_pos: Some(RootSpacePosition(DVec2::new(0.5 + 1.0 / 32.0, 1.5))),
            root_vel: Some(RootSpaceLinearVelocity(DVec2::new(1.0, 0.0))),
            rig_tf: Some(RigidSpaceTransform(Transform {
                translation: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            })),
            rig_vel: Some(RigidSpaceVelocity::zero()),
            cam_tf: Some(CameraSpaceTransform(Transform {
                translation: Vec3::new(0.5 + 1.0 / 32.0, 1.5, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            })),
        },
        extra_assertions: None,
    }])
});

#[test]
fn reference_frames() {
    let mut app = common::setup(true);

    let camera = app
        .world_mut()
        .spawn((
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
        ))
        .id();

    let body = app
        .world_mut()
        .spawn((
            CelestialBody { radius: 1.0 },
            RigidBody::Fixed,
            Collider::ball(1.0),
            Heightmap(Box::from([])),
            RootSpacePosition(DVec2::ZERO),
        ))
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

    ASSERTION_COLLECTION.run_assertions_collection(
        &mut app,
        TestEntitiesRequest {
            body,
            camera,
            vessel,
        },
    );
}
