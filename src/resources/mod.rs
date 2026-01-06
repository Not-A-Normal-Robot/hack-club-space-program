use bevy::prelude::*;

use crate::components::frames::{RootSpaceLinearVelocity, RootSpacePosition};

#[derive(Resource)]
pub struct ActiveVessel {
    pub entity: Entity,
    pub prev_tick_position: RootSpacePosition,
    pub prev_tick_velocity: RootSpaceLinearVelocity,
    pub prev_tick_parent: Entity,
}
