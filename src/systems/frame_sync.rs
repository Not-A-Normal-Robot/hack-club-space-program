use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, RigidBody};

use crate::{
    components::{
        ParentBody, SimCamera, SimCameraOffset, SimCameraZoom,
        frames::{
            RigidSpaceTransform, RigidSpaceVelocity, RigidSpaceVelocityImpl,
            RootSpaceLinearVelocity, RootSpacePosition,
        },
    },
    resources::ActiveVessel,
};

/// Updates rigid-space transform based on root-space position (if any),
/// and updates rigid-space velocity based on root-space velocity (if any).
#[expect(clippy::type_complexity)]
pub fn sync_root_to_rigid(
    mut commands: Commands,
    tf_query: Query<(Entity, Option<&mut RigidSpaceTransform>, &RootSpacePosition)>,
    vel_query: Query<
        (
            Entity,
            Option<&mut RigidSpaceVelocity>,
            Option<&RootSpaceLinearVelocity>,
        ),
        With<Collider>,
    >,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        return;
    };

    for (entity, rigid_tf, root_pos) in tf_query {
        if let Some(mut rigid_tf) = rigid_tf {
            let rotation = rigid_tf.0.rotation;
            *rigid_tf = root_pos
                .to_rigid_space_position(active_vessel.prev_tick_position)
                .to_rigid_space_transform(rotation, Vec3::ONE);
        } else {
            commands.entity(entity).insert(
                root_pos
                    .to_rigid_space_position(active_vessel.prev_tick_position)
                    .to_rigid_space_transform(Quat::IDENTITY, Vec3::ONE),
            );
        }
    }

    for (entity, rigid_vel, root_vel) in vel_query {
        let root_vel = root_vel.copied().unwrap_or_default();
        if let Some(mut rigid_vel) = rigid_vel {
            *rigid_vel = root_vel
                .to_rigid_space_velocity(active_vessel.prev_tick_velocity, rigid_vel.angvel);
        } else {
            commands
                .entity(entity)
                .insert(root_vel.to_rigid_space_velocity(active_vessel.prev_tick_velocity, 0.0));
        }
    }
}

/// Updates root-space position based on rigid-space transform (if any).
pub fn sync_rigid_pos_to_root(
    mut commands: Commands,
    root_positionless: Query<
        (Entity, &RigidSpaceTransform, &ParentBody),
        Without<RootSpacePosition>,
    >,
    mut with_root_pos: Query<(&RigidSpaceTransform, &mut RootSpacePosition, &ParentBody)>,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        return;
    };
    for (entity, transform, parent) in &root_positionless {
        if parent.0 != active_vessel.prev_tick_parent {
            continue;
        }

        let new_root_position = transform
            .position()
            .to_root_space_position(active_vessel.prev_tick_position);

        commands.entity(entity).insert(new_root_position);
    }
    for (rigid, mut root_space_pos, parent) in &mut with_root_pos {
        if parent.0 != active_vessel.prev_tick_parent {
            continue;
        }

        let new_pos = rigid
            .position()
            .to_root_space_position(active_vessel.prev_tick_position);

        *root_space_pos = new_pos;
    }
}

/// Updates root-space position based on rigid-space transform (if any).
pub fn sync_rigid_vel_to_root(
    mut commands: Commands,
    root_velless: Query<
        (Entity, &RigidSpaceVelocity, &ParentBody),
        Without<RootSpaceLinearVelocity>,
    >,
    mut with_root_vel: Query<(
        &RigidSpaceVelocity,
        &mut RootSpaceLinearVelocity,
        &ParentBody,
    )>,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        return;
    };
    for (entity, velocity, parent) in &root_velless {
        if parent.0 != active_vessel.prev_tick_parent {
            continue;
        }

        let new_root_velocity =
            dbg!(velocity.to_root_space_linear_velocity(active_vessel.prev_tick_velocity));

        commands.entity(entity).insert(new_root_velocity);
    }
    for (rigid_vel, mut root_space_vel, parent) in &mut with_root_vel {
        if parent.0 != active_vessel.prev_tick_parent {
            continue;
        }
        dbg!(rigid_vel);

        *root_space_vel =
            dbg!(rigid_vel.to_root_space_linear_velocity(active_vessel.prev_tick_velocity));
    }
}

pub fn apply_root_velocity(
    vels: Query<(&RootSpaceLinearVelocity, &mut RootSpacePosition, &RigidBody)>,
    time: Res<Time>,
) {
    vels.into_iter()
        .for_each(|(root_vel, mut root_pos, _)| root_pos.0 += root_vel.0 * time.delta_secs_f64());
}

/// Updates the last tick position and last parent body of the active vessel.
#[expect(clippy::type_complexity)]
pub fn update_active_vessel_resource(
    query: Query<(
        &RootSpacePosition,
        &RootSpaceLinearVelocity,
        &ParentBody,
        Option<&mut RigidSpaceTransform>,
        Option<&mut RigidSpaceVelocity>,
    )>,
    active_vessel: Option<ResMut<ActiveVessel>>,
) {
    let Some(mut active_vessel) = active_vessel else {
        return;
    };
    let Ok((position, velocity, parent, active_transform, active_velocity)) =
        query.get(active_vessel.entity)
    else {
        return;
    };

    active_vessel.prev_tick_parent = parent.0;
    active_vessel.prev_tick_position = *position;
    active_vessel.prev_tick_velocity = *velocity;

    let Some(&active_transform) = active_transform else {
        return;
    };

    let Some(&active_velocity) = active_velocity else {
        return;
    };

    for (_, _, _, transform, velocity) in query {
        if let Some(mut transform) = transform {
            transform.0.translation -= active_transform.0.translation;
        }
        if let Some(mut velocity) = velocity {
            velocity.linvel -= active_velocity.linvel;
        }
    }
}

/// Sets transform into the rigid transform so that Rapier can process it
pub fn pre_rapier_frame_switch(query: Query<(&RigidSpaceTransform, &mut Transform)>) {
    query.into_iter().for_each(|(rigid, mut tf)| *tf = rigid.0);
}

/// Sets transform into the camera transform so Bevy can render it
pub fn post_rapier_frame_switch(
    query: Query<(&mut RigidSpaceTransform, &mut Transform, &RootSpacePosition)>,
    sim_camera: Query<(&mut SimCameraOffset, &SimCameraZoom, &Camera), With<SimCamera>>,
    camera_offset_query: Query<&RootSpacePosition>,
) {
    let Some((mut cam_offset, &zoom, _)) = sim_camera.into_iter().find(|&(.., c)| c.is_active)
    else {
        return;
    };

    let cam_offset = cam_offset.get_root_position(camera_offset_query);

    query.into_iter().for_each(|(mut rigid, mut tf, root_pos)| {
        rigid.0 = *tf;
        *tf = root_pos
            .to_camera_space_transform(tf.rotation, cam_offset, zoom)
            .0;
    });
}
