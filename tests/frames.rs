//! Integration test for reference frames

use crate::common::{Assertions, AssertionsCollection};
use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;
use hack_club_space_program::{
    components::{
        CelestialBody, Heightmap, SimCamera, SimCameraOffset, SimCameraZoom, Vessel,
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
    pub extra_assertions: Option<fn(&App, TestEntityRefs)>,
}

#[derive(Clone, Copy, Default)]
struct TransformAssertions {
    pub root_pos: Option<RootSpacePosition>,
    pub root_vel: Option<RootSpaceLinearVelocity>,
    pub rig_tf: Option<RigidSpaceTransform>,
    pub rig_vel: Option<RigidSpaceVelocity>,
    pub cam_tf: Option<CameraSpaceTransform>,
}

impl TransformAssertions {
    fn check_assertions(
        &self,
        entity: EntityRef<'_>,
        camera_offset: RootSpacePosition,
        camera_zoom: SimCameraZoom,
        object: &str,
    ) {
        if let Some(expected_root_pos) = self.root_pos {
            assert_eq!(
                dbg!(entity.get::<RootSpacePosition>().copied()),
                Some(expected_root_pos),
                "root pos didn't match expected value for {object}"
            );
        };

        if let Some(expected_root_vel) = self.root_vel {
            assert_eq!(
                dbg!(entity.get::<RootSpaceLinearVelocity>().copied()),
                Some(expected_root_vel),
                "root vel didn't match expected value for {object}"
            );
        }

        // DEBUG
        if let Some(expected_rig_vel) = self.rig_vel {
            assert_eq!(
                dbg!(entity.get::<RigidSpaceVelocity>().cloned()),
                Some(expected_rig_vel),
                "rigid vel didn't match expected value for {object}"
            );
        }

        if let Some(expected_rig_tf) = self.rig_tf {
            assert_eq!(
                dbg!(entity.get::<RigidSpaceTransform>().cloned()),
                Some(expected_rig_tf),
                "rigid tf didn't match expected value for {object}"
            );
        }

        if let Some(expected_rig_vel) = self.rig_vel {
            assert_eq!(
                dbg!(entity.get::<RigidSpaceVelocity>().cloned()),
                Some(expected_rig_vel),
                "rigid vel didn't match expected value for {object}"
            );
        }

        if let Some(asserted_cam_tf) = self.cam_tf {
            let rig_tf = dbg!(entity.get::<RigidSpaceTransform>())
                .cloned()
                .expect("rigid tf should exist for camera-space transform assertion");
            let cam_tf = dbg!(entity.get::<RootSpacePosition>())
                .cloned()
                .expect("root pos should exist for camera-space transform assertion")
                .to_camera_space_transform(rig_tf.0.rotation, camera_offset, camera_zoom);
            assert_eq!(
                cam_tf, asserted_cam_tf,
                "cam tf didn't match expected value for {object}"
            );
        }
    }
}

#[derive(Clone, Copy)]
struct TestExtraData {
    entities: TestEntities,
}

#[derive(Clone, Copy)]
struct TestEntities {
    vessel: Entity,
    body: Entity,
    camera: Entity,
}

#[derive(Clone, Copy)]
struct TestEntityRefs<'a> {
    vessel: EntityRef<'a>,
    body: EntityRef<'a>,
    camera: EntityRef<'a>,
}

impl<'a> TestEntityRefs<'a> {
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

fn get_camera_offset(app: &App, entity_refs: &TestEntityRefs) -> RootSpacePosition {
    let mut camera_offset = entity_refs
        .camera
        .get::<SimCameraOffset>()
        .copied()
        .expect("could not find SimCameraOffset");
    let attached_pos = match camera_offset {
        SimCameraOffset::Attached { entity, .. } => app
            .world()
            .get::<RootSpacePosition>(entity)
            .copied()
            .unwrap_or(RootSpacePosition(DVec2::ZERO)),
        SimCameraOffset::Detached(pos) => pos,
    };
    camera_offset.get_root_position_with_attached_pos(attached_pos)
}

impl Assertions for PostTickAssertions {
    type ExtraData = TestExtraData;
    fn check_assertions(&self, app: &App, extra: Self::ExtraData) {
        let entities = extra.entities;
        let entity_refs =
            TestEntityRefs::get_entities(app, entities.vessel, entities.body, entities.camera);
        let camera_offset = get_camera_offset(app, &entity_refs);
        let camera_zoom = entity_refs
            .camera
            .get::<SimCameraZoom>()
            .copied()
            .expect("could not find SimCameraZoom");

        if let Some(extra_assertions) = self.extra_assertions {
            eprintln!(">>> Running extra assertions");
            extra_assertions(app, entity_refs);
        }

        eprintln!(">>> Running body assertions");
        self.body
            .check_assertions(entity_refs.body, camera_offset, camera_zoom, "body");

        eprintln!(">>> Running vessel assertions");
        self.vessel
            .check_assertions(entity_refs.vessel, camera_offset, camera_zoom, "vessel");
    }
}

static ASSERTION_COLLECTION: LazyLock<Box<[PostTickAssertions]>> = LazyLock::new(|| {
    Box::new([
        PostTickAssertions {
            body: TransformAssertions {
                root_pos: Some(RootSpacePosition(DVec2::ZERO)),
                root_vel: None,
                // rigid transform is updated in FixedPreUpdate before the rigid step
                // and so we see last tick's rigid transform here
                rig_tf: Some(RigidSpaceTransform(Transform {
                    translation: Vec3::new(-0.5, -1.5, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                })),
                rig_vel: Some(RigidSpaceVelocity {
                    angvel: 0.0,
                    linvel: Vec2::new(-1.0, 0.0),
                }),
                cam_tf: Some(CameraSpaceTransform(Transform {
                    translation: Vec3::new(-0.515625, -1.5, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                })),
            },
            vessel: TransformAssertions {
                root_pos: Some(RootSpacePosition(DVec2::new(0.5 + 1.0 / 64.0, 1.5))),
                root_vel: Some(RootSpaceLinearVelocity(DVec2::new(1.0, 0.0))),
                rig_tf: Some(RigidSpaceTransform(Transform::IDENTITY)),
                rig_vel: Some(RigidSpaceVelocity::zero()),
                cam_tf: Some(CameraSpaceTransform(Transform::IDENTITY)),
            },
            extra_assertions: Some(|app, entity_refs| {
                let camera_offset = get_camera_offset(app, &entity_refs);
                assert_eq!(
                    camera_offset,
                    RootSpacePosition(DVec2::new(0.5 + 1.0 / 64.0, 1.5)),
                    "camera offset didn't match expected value"
                );

                let active_vessel = app.world().resource::<ActiveVessel>();
                assert_eq!(
                    active_vessel.entity,
                    entity_refs.vessel.entity(),
                    "active vessel entity mismatch"
                );
                assert_eq!(
                    active_vessel.prev_tick_parent,
                    entity_refs.body.entity(),
                    "active vessel parent mismatch"
                );
                assert_eq!(
                    active_vessel.prev_tick_position,
                    RootSpacePosition(DVec2::new(0.5, 1.5)),
                    "active vessel position mismatch"
                );
                assert_eq!(
                    active_vessel.prev_tick_velocity,
                    RootSpaceLinearVelocity(DVec2::new(1.0, 0.0)),
                    "active vessel velocity mismatch"
                );
            }),
        },
        PostTickAssertions {
            body: TransformAssertions {
                rig_tf: Some(RigidSpaceTransform(Transform {
                    translation: Vec3::new(-0.515625, -1.5, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                })),
                ..Default::default()
            },
            vessel: TransformAssertions {
                rig_tf: Some(RigidSpaceTransform(Transform::IDENTITY)),
                ..Default::default()
            },
            extra_assertions: None,
        },
    ])
});

#[test]
fn reference_frames() {
    let mut app = common::setup(true);

    let body = app
        .world_mut()
        .spawn((
            CelestialBody { radius: 1.0 / 32.0 },
            AdditionalMassProperties::Mass(0.0),
            RigidBody::Fixed,
            Collider::ball(1.0 / 32.0),
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
            Collider::ball(1.0 / 512.0),
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

    let camera = app
        .world_mut()
        .spawn((
            Camera {
                is_active: true,
                ..Default::default()
            },
            Camera2d,
            SimCamera,
            SimCameraOffset::Attached {
                entity: vessel,
                last_known_pos: RootSpacePosition(DVec2::ZERO),
                offset: DVec2::ZERO,
            },
            SimCameraZoom(1.0),
            Transform::from_rotation(Quat::from_rotation_z(0.0)),
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
        TestExtraData {
            entities: TestEntities {
                body,
                camera,
                vessel,
            },
        },
    );
}
