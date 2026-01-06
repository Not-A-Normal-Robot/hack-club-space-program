use bevy::prelude::*;

use crate::{
    components::{
        ParentBody,
        frames::{
            ParentSpaceLinearVelocity, ParentSpacePosition, RigidSpaceTransform,
            RigidSpaceVelocity, RigidSpaceVelocityImpl,
        },
    },
    resources::ActiveVessel,
};

/// Updates parent-space position based on rigid-space transform (if any).
pub fn sync_rigid_pos_to_parent(
    mut commands: Commands,
    parent_positionless: Query<
        (Entity, &RigidSpaceTransform, &ParentBody),
        Without<ParentSpacePosition>,
    >,
    mut with_parent_pos: Query<(&RigidSpaceTransform, &mut ParentSpacePosition, &ParentBody)>,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        return;
    };
    for (entity, transform, parent) in &parent_positionless {
        if parent.0 != active_vessel.prev_tick_parent {
            continue;
        }

        let new_parent_position = transform
            .position()
            .to_parent_space_position(active_vessel.prev_tick_position);

        commands.entity(entity).insert(new_parent_position);
    }
    for (rigid, mut parent_space_pos, parent) in &mut with_parent_pos {
        if parent.0 != active_vessel.prev_tick_parent {
            continue;
        }

        *parent_space_pos = rigid
            .position()
            .to_parent_space_position(active_vessel.prev_tick_position);
    }
}

/// Updates parent-space position based on rigid-space transform (if any).
pub fn sync_rigid_vel_to_parent(
    mut commands: Commands,
    parent_velless: Query<
        (Entity, &RigidSpaceVelocity, &ParentBody),
        Without<ParentSpaceLinearVelocity>,
    >,
    mut with_parent_vel: Query<(
        &RigidSpaceVelocity,
        &mut ParentSpaceLinearVelocity,
        &ParentBody,
    )>,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        return;
    };
    for (entity, velocity, parent) in &parent_velless {
        if parent.0 != active_vessel.prev_tick_parent {
            continue;
        }

        let new_parent_velocity =
            velocity.to_parent_space_linear_velocity(active_vessel.prev_tick_velocity);

        commands.entity(entity).insert(new_parent_velocity);
    }
    for (rigid_vel, mut parent_space_vel, parent) in &mut with_parent_vel {
        if parent.0 != active_vessel.prev_tick_parent {
            continue;
        }

        *parent_space_vel =
            rigid_vel.to_parent_space_linear_velocity(active_vessel.prev_tick_velocity);
    }
}

/// Updates the "Last Tick Position" and "Last Parent Body" of the active vessel.
pub fn update_active_vessel_res(
    query: Query<(
        &ParentSpacePosition,
        &ParentSpaceLinearVelocity,
        &ParentBody,
    )>,
    active_vessel: Option<ResMut<ActiveVessel>>,
) {
    let Some(mut active_vessel) = active_vessel else {
        return;
    };
    let Ok((position, velocity, parent)) = query.get(active_vessel.entity) else {
        return;
    };

    active_vessel.prev_tick_position = *position;
    active_vessel.prev_tick_parent = parent.0;
    active_vessel.prev_tick_velocity = *velocity;
}
