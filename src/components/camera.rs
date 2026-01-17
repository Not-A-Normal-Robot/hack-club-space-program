use bevy::{math::DVec2, prelude::*};

use crate::components::frames::RootSpacePosition;

#[derive(Clone, Copy, Component)]
#[require(SimCameraZoom)]
pub enum SimCameraOffset {
    Attached {
        entity: Entity,
        last_known_pos: RootSpacePosition,
        offset: DVec2,
    },
    Detached(RootSpacePosition),
}

impl SimCameraOffset {
    pub fn get_root_position(&mut self, query: Query<&RootSpacePosition>) -> RootSpacePosition {
        let (entity, last_known_pos) = match self {
            Self::Attached {
                entity,
                last_known_pos,
                ..
            } => (*entity, *last_known_pos),
            Self::Detached(pos) => return *pos,
        };

        let attached_obj_pos = match query.get(entity) {
            Ok(p) => *p,
            Err(_) => last_known_pos,
        };

        self.get_root_position_with_attached_pos(attached_obj_pos)
    }

    /// Get the root position of the SimCamera, given the position of the attached object.
    ///
    /// # Unchecked Operation
    /// This function does no checks to whether or not the position of the
    /// object is equal to the thing it's actually attached to.
    pub fn get_root_position_with_attached_pos(
        &mut self,
        attached_obj_pos: RootSpacePosition,
    ) -> RootSpacePosition {
        let (last_known_pos, offset) = match self {
            Self::Attached {
                entity: _,
                last_known_pos,
                offset,
            } => (last_known_pos, *offset),
            Self::Detached(pos) => return *pos,
        };

        *last_known_pos = attached_obj_pos;

        RootSpacePosition(attached_obj_pos.0 + offset)
    }
}

#[derive(Clone, Copy, Component)]
pub struct SimCameraZoom(pub f64);

impl Default for SimCameraZoom {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Copy, Component)]
pub struct SimCamera;
