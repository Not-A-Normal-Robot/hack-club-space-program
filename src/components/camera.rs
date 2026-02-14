use crate::components::frames::RootSpacePosition;
use bevy::{math::DVec2, prelude::*};
use core::ops::Deref;

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
    pub fn immutably(&self) -> SimCameraOffsetReference {
        SimCameraOffsetReference::Immutable(self)
    }

    pub fn mutably(&mut self) -> SimCameraOffsetReference {
        SimCameraOffsetReference::Mutable(self)
    }
}

pub enum SimCameraOffsetReference<'a> {
    Mutable(&'a mut SimCameraOffset),
    Immutable(&'a SimCameraOffset),
}

impl<'a> Deref for SimCameraOffsetReference<'a> {
    type Target = SimCameraOffset;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Mutable(sim_camera_offset) => sim_camera_offset,
            Self::Immutable(sim_camera_offset) => sim_camera_offset,
        }
    }
}

impl<'a> SimCameraOffsetReference<'a> {
    /// Gets the current root position of the simulation camera.
    pub fn get_root_position(self, query: Query<&RootSpacePosition>) -> RootSpacePosition {
        let (entity, last_known_pos) = match *self {
            SimCameraOffset::Attached {
                entity,
                last_known_pos,
                ..
            } => (entity, last_known_pos),
            SimCameraOffset::Detached(pos) => return pos,
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
        self,
        attached_obj_pos: RootSpacePosition,
    ) -> RootSpacePosition {
        let offset = match self {
            Self::Mutable(SimCameraOffset::Attached {
                last_known_pos,
                offset,
                ..
            }) => {
                last_known_pos.0 = attached_obj_pos.0 + *offset;
                *offset
            }
            Self::Immutable(SimCameraOffset::Attached { offset, .. }) => *offset,
            Self::Mutable(&mut SimCameraOffset::Detached(pos))
            | Self::Immutable(&SimCameraOffset::Detached(pos)) => return pos,
        };

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
