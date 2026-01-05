use bevy::prelude::*;

use crate::{
    components::{
        ParentBody,
        frames::{ParentSpacePosition, RigidSpaceTransform},
    },
    resources::ActiveVessel,
};

/// Syncs between reference frames.
///
/// Updates parent-space position based on rigid-space transform (if any).
pub fn sync_rigid_to_parent(
    mut commands: Commands,
    parent_positionless: Query<(Entity, &RigidSpaceTransform), Without<ParentSpacePosition>>,
    mut with_parent_pos: Query<(&RigidSpaceTransform, &mut ParentSpacePosition)>,
    active_vessel: Option<Res<ActiveVessel>>,
) {
    let Some(active_vessel) = active_vessel else {
        return;
    };
    for (entity, transform) in &parent_positionless {
        let new_parent_position = transform
            .position()
            .to_parent_space_position(active_vessel.prev_tick_position);

        commands.entity(entity).insert(new_parent_position);
    }
    for (rigid, mut parent) in &mut with_parent_pos {
        *parent = rigid
            .position()
            .to_parent_space_position(active_vessel.prev_tick_position);
    }
}

/// Updates the "Last Tick Position" and "Last Parent Body" of the active vessel.
pub fn update_active_vessel_res(
    query: Query<(&ParentSpacePosition, &ParentBody)>,
    active_vessel: Option<ResMut<ActiveVessel>>,
) {
    let Some(mut active_vessel) = active_vessel else {
        return;
    };
    let Ok((position, parent)) = query.get(active_vessel.entity) else {
        return;
    };

    active_vessel.prev_tick_position = *position;
    active_vessel.prev_tick_parent = parent.0;
}
