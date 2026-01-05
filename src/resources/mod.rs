use bevy::prelude::*;

use crate::components::frames::ParentSpacePosition;

#[derive(Resource)]
pub struct ActiveVessel {
    pub entity: Entity,
    pub prev_tick_position: ParentSpacePosition,
    pub prev_tick_parent: Entity,
}
