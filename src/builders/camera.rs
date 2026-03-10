use bevy::prelude::*;

use crate::components::main_game::camera::{SimCamera, SimCameraOffset, SimCameraZoom};

pub struct SimCameraBuilder {
    pub offset: SimCameraOffset,
    pub zoom: SimCameraZoom,
    /// The transform of this camera.
    ///
    /// This should only be for rotation, as
    /// translation and scaling is already
    /// handled using double-precision using
    /// `SimCameraOffset` and `SimCameraZoom`.
    pub transform: Transform,
}

impl SimCameraBuilder {
    #[must_use]
    pub(crate) const fn base_bundle() -> impl Bundle {
        (Camera2d, SimCamera)
    }
    #[must_use]
    pub fn build(self, active: bool) -> impl Bundle {
        (
            Self::base_bundle(),
            Camera {
                is_active: active,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..Default::default()
            },
            self.offset,
            self.zoom,
            self.transform,
        )
    }
    #[must_use]
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

pub(crate) struct UiCameraBuilder;

#[cfg_attr(not(feature = "not-headless"), expect(dead_code))]
impl UiCameraBuilder {
    #[must_use]
    pub(crate) const fn build() -> impl Bundle {
        (Camera2d, IsDefaultUiCamera)
    }

    #[must_use]
    pub(crate) const fn with_extra(extra: impl Bundle) -> impl Bundle {
        (Self::build(), extra)
    }
}
