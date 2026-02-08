use bevy::prelude::*;

use crate::components::camera::{SimCamera, SimCameraOffset, SimCameraZoom};

pub struct SimCameraBuilder {
    offset: SimCameraOffset,
    zoom: SimCameraZoom,
    /// The transform of this camera.
    ///
    /// This should only be for rotation, as
    /// translation and scaling is already
    /// handled using double-precision using
    /// SimCameraOffset and SimCameraZoom.
    transform: Transform,
}

impl SimCameraBuilder {
    pub const fn base_bundle() -> impl Bundle {
        (Camera2d, SimCamera)
    }
    pub fn build(self, active: bool) -> impl Bundle {
        (
            Self::base_bundle(),
            Camera {
                is_active: active,
                ..Default::default()
            },
            self.offset,
            self.zoom,
            self.transform,
        )
    }
    pub fn with_camera(self, camera: Camera) -> impl Bundle {
        (
            Self::base_bundle(),
            camera,
            self.offset,
            self.zoom,
            self.transform,
        )
    }
}
