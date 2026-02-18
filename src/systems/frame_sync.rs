use bevy::prelude::*;
use bevy_rapier2d::prelude::RigidBody;

use crate::{
    components::{
        camera::{SimCamera, SimCameraOffset, SimCameraZoom},
        celestial::CelestialBody,
        frames::{
            RigidSpaceTransform, RigidSpaceVelocity, RigidSpaceVelocityImpl,
            RootSpaceLinearVelocity, RootSpacePosition,
        },
        relations::CelestialParent,
    },
    consts::FilterLoadedVessels,
    resources::ActiveVessel,
};

/// Updates root-space position based on rigid-space transform (if any).
///  
/// Assumes the current Transform is a rigid-space transform.
pub fn write_rigid_pos_to_root(
    mut commands: Commands,
    root_positionless: Query<
        (Entity, &Transform, &CelestialParent),
        (Without<RootSpacePosition>, FilterLoadedVessels),
    >,
    mut with_root_pos: Query<
        (&Transform, &mut RootSpacePosition, &CelestialParent),
        FilterLoadedVessels,
    >,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        return;
    };
    for (entity, transform, parent) in &root_positionless {
        if parent.entity != active_vessel.prev_tick_parent {
            continue;
        }

        let transform = RigidSpaceTransform(*transform);

        let new_root_position = transform
            .position()
            .to_root_space_position(active_vessel.prev_tick_position);

        commands.entity(entity).insert(new_root_position);
    }
    for (transform, mut root_space_pos, parent) in &mut with_root_pos {
        if parent.entity != active_vessel.prev_tick_parent {
            continue;
        }

        let transform = RigidSpaceTransform(*transform);
        let new_pos = transform
            .position()
            .to_root_space_position(active_vessel.prev_tick_position);

        *root_space_pos = new_pos;
    }
}

/// Updates root-space velocity based on rigid-space velocity (if any).
pub fn write_rigid_vel_to_root(
    mut commands: Commands,
    root_velless: Query<
        (Entity, &RigidSpaceVelocity, &CelestialParent),
        (Without<RootSpaceLinearVelocity>, FilterLoadedVessels),
    >,
    mut with_root_vel: Query<
        (
            &RigidSpaceVelocity,
            &mut RootSpaceLinearVelocity,
            &CelestialParent,
        ),
        FilterLoadedVessels,
    >,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        return;
    };
    for (entity, velocity, parent) in &root_velless {
        if parent.entity != active_vessel.prev_tick_parent {
            continue;
        }

        let new_root_velocity =
            velocity.to_root_space_linear_velocity(active_vessel.prev_tick_velocity);

        commands.entity(entity).insert(new_root_velocity);
    }
    for (rigid_vel, mut root_space_vel, parent) in &mut with_root_vel {
        if parent.entity != active_vessel.prev_tick_parent {
            continue;
        }

        *root_space_vel = rigid_vel.to_root_space_linear_velocity(active_vessel.prev_tick_velocity);
    }
}

/// Shifts all entities' RootSpacePosition based on its RootSpaceLinearVelocity
/// (if any).
pub fn apply_root_velocity(
    vels: Query<
        (&RootSpaceLinearVelocity, &mut RootSpacePosition, &RigidBody),
        FilterLoadedVessels,
    >,
    time: Res<Time>,
) {
    vels.into_iter()
        .for_each(|(root_vel, mut root_pos, _)| root_pos.0 += root_vel.0 * time.delta_secs_f64());
}

/// Updates the last tick position and last parent body of the active vessel.
pub fn update_active_vessel_resource(
    query: Query<(
        &RootSpacePosition,
        &RootSpaceLinearVelocity,
        &CelestialParent,
    )>,
    active_vessel: Option<ResMut<ActiveVessel>>,
) {
    let Some(mut active_vessel) = active_vessel else {
        return;
    };
    let Ok((position, velocity, parent)) = query.get(active_vessel.entity) else {
        return;
    };

    active_vessel.prev_tick_parent = parent.entity;
    active_vessel.prev_tick_position = *position;
    active_vessel.prev_tick_velocity = *velocity;
}

fn pre_rapier_frame_switch_inner(
    root_pos: RootSpacePosition,
    root_vel: RootSpaceLinearVelocity,
    mut transform: Mut<'_, Transform>,
    mut rigid_vel: Mut<'_, RigidSpaceVelocity>,
    active_vessel: &ActiveVessel,
) {
    transform.translation = root_pos
        .to_rigid_space_position(active_vessel.prev_tick_position)
        .0
        .extend(0.0);
    transform.scale = Vec3::ONE;
    rigid_vel.linvel = *root_vel.to_rigid_space_linear_velocity(active_vessel.prev_tick_velocity);
}

/// Sets transform into the rigid transform so that Rapier can process it
pub fn pre_rapier_frame_switch(
    query: Query<(
        &RootSpacePosition,
        &RootSpaceLinearVelocity,
        &mut Transform,
        &mut RigidSpaceVelocity,
    )>,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        warn!("active vessel resource not loaded");
        return;
    };

    query
        .into_iter()
        .for_each(|(&root_pos, &root_vel, transform, rigid_vel)| {
            pre_rapier_frame_switch_inner(root_pos, root_vel, transform, rigid_vel, &active_vessel);
        });
}

/// Sets transform into the camera transform so Bevy can render it
pub fn post_rapier_frame_switch(
    query: Query<(&mut Transform, &RootSpacePosition), Without<CelestialBody>>,
    celestials: Query<&mut Transform, With<CelestialBody>>,
    sim_camera: Query<(&mut SimCameraOffset, &SimCameraZoom, &Camera), With<SimCamera>>,
    camera_offset_query: Query<&RootSpacePosition>,
) {
    let Some((mut cam_offset, &cam_zoom, _)) = sim_camera.into_iter().find(|&(.., c)| c.is_active)
    else {
        warn!("sim camera not found");
        return;
    };

    let cam_offset = cam_offset.mutably().get_root_position(camera_offset_query);

    query.into_iter().for_each(|(mut transform, &root_pos)| {
        let rotation = transform.rotation;
        *transform = root_pos
            .to_camera_space_transform(rotation, cam_offset, cam_zoom)
            .0;
    });

    celestials.into_iter().for_each(|mut transform| {
        // Offsetting is handled at the mesh layer
        transform.translation = Vec3::ZERO;
    })
}
