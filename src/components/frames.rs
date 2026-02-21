//! # Reference Frames
//!
//! Root Space converts into Rigid Space position (with its own rotation)
//! Root Space position + Rigid Space rotation + Camera offset = Camera Space transform

use crate::components::camera::SimCameraZoom;
use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;
use std::fmt::Display;

macro_rules! wrapper {
    ($( $outer:ty : $inner:ty ),* $(,)?) => {
        $(
            impl ::core::ops::Deref for $outer {
                type Target = $inner;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl From<$outer> for $inner {
                fn from(value: $outer) -> Self {
                    value.0
                }
            }
        )*
    };
}

/// Coordinates relative to root body.
///
/// Used for orbital physics and as source of truth.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct RootSpacePosition(pub DVec2);

impl RootSpacePosition {
    #[must_use]
    pub fn to_rigid_space_position(
        self,
        active_vessel_pos: RootSpacePosition,
    ) -> RigidSpacePosition {
        let position = self.0 - active_vessel_pos.0;
        RigidSpacePosition(position.as_vec2())
    }

    #[must_use]
    #[expect(clippy::cast_possible_truncation)]
    pub fn to_camera_space_transform(
        self,
        rotation: Quat,
        camera_offset: RootSpacePosition,
        camera_zoom: SimCameraZoom,
    ) -> CameraSpaceTransform {
        let offset = (self.0 - camera_offset.0) * camera_zoom.0;

        CameraSpaceTransform(Transform {
            rotation,
            translation: offset.as_vec2().extend(0.0),
            scale: Vec3::splat(camera_zoom.0 as f32),
        })
    }
}

impl Display for RootSpacePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.7e}m, {:.7e}m]@root", self.x, self.y)
    }
}

/// Coordinates relative to root body.
///
/// Used for orbital physics and as source of truth.
#[derive(Clone, Copy, Component, Debug, Default, PartialEq)]
pub struct RootSpaceLinearVelocity(pub DVec2);

impl RootSpaceLinearVelocity {
    #[must_use]
    pub fn to_rigid_space_linear_velocity(
        self,
        active_vessel_vel: RootSpaceLinearVelocity,
    ) -> RigidSpaceLinearVelocity {
        let vel = self.0 - active_vessel_vel.0;
        RigidSpaceLinearVelocity(vel.as_vec2())
    }
}

impl Display for RootSpaceLinearVelocity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.7e}m/s, {:.7e}m/s]@root", self.x, self.y)
    }
}

/// Coordinates relative to active vessel.
///
/// Single precision, and unscaled. Used to be transformed to [`RigidSpaceTransform`].
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct RigidSpacePosition(pub Vec2);

impl RigidSpacePosition {
    pub const ZERO: Self = Self(Vec2::ZERO);

    #[must_use]
    pub fn to_root_space_position(
        &self,
        active_vessel_pos: RootSpacePosition,
    ) -> RootSpacePosition {
        let rigid = DVec2::new(f64::from(self.0.x), f64::from(self.0.y));

        RootSpacePosition(active_vessel_pos.0 + rigid)
    }

    #[must_use]
    pub fn to_rigid_space_transform(&self, rotation: Quat, scale: Vec3) -> RigidSpaceTransform {
        RigidSpaceTransform(Transform {
            translation: self.0.extend(0.0),
            rotation,
            scale,
        })
    }
}

/// Coordinates relative to active vessel.
///
/// Single precision, and unscaled. Used for [`bevy_rapier2d`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidSpaceTransform(pub Transform);

impl RigidSpaceTransform {
    #[must_use]
    pub fn position(&self) -> RigidSpacePosition {
        RigidSpacePosition(self.0.translation.truncate())
    }
}

/// Coordinates relative to active vessel.
///
/// Single precision, and unscaled. Used as intermediary for [`bevy_rapier2d`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidSpaceLinearVelocity(pub Vec2);

impl RigidSpaceLinearVelocity {
    #[must_use]
    pub fn to_rigid_space_velocity(self, angvel: f32) -> RigidSpaceVelocity {
        RigidSpaceVelocity {
            linvel: self.0,
            angvel,
        }
    }

    #[must_use]
    pub fn to_root_space_linear_velocity(
        self,
        active_vessel_vel: RootSpaceLinearVelocity,
    ) -> RootSpaceLinearVelocity {
        let linvel = DVec2::new(f64::from(self.0.x), f64::from(self.0.y));
        RootSpaceLinearVelocity(active_vessel_vel.0 + linvel)
    }
}

/// Coordinates relative to active vessel.
///
/// Single precision, and unscaled. Used for [`bevy_rapier2d`].
pub type RigidSpaceVelocity = Velocity;

pub trait RigidSpaceVelocityImpl {
    fn to_root_space_linear_velocity(
        self,
        active_vessel_vel: RootSpaceLinearVelocity,
    ) -> RootSpaceLinearVelocity;
}

impl RigidSpaceVelocityImpl for RigidSpaceVelocity {
    fn to_root_space_linear_velocity(
        self,
        active_vessel_vel: RootSpaceLinearVelocity,
    ) -> RootSpaceLinearVelocity {
        let linvel = DVec2::new(f64::from(self.linvel.x), f64::from(self.linvel.y));
        RootSpaceLinearVelocity(active_vessel_vel.0 + linvel)
    }
}

/// Coordinates relative to camera.
///
/// Single precision, and scaled to camera zoom amount.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct CameraSpaceTransform(pub Transform);

wrapper! {
    RootSpacePosition: DVec2,
    RootSpaceLinearVelocity: DVec2,
    RigidSpacePosition: Vec2,
    RigidSpaceTransform: Transform,
    RigidSpaceLinearVelocity: Vec2,
    CameraSpaceTransform: Transform
}

#[cfg(test)]
mod tests {
    use bevy::math::{DVec2, Vec2};

    use crate::components::frames::{
        RigidSpaceVelocity, RigidSpaceVelocityImpl as _, RootSpaceLinearVelocity, RootSpacePosition,
    };

    #[test]
    fn root_rigid_conversion() {
        const REFERENCE_POS: RootSpacePosition = RootSpacePosition(DVec2::new(5.0, 9.0));
        const ROOTSPACE_POS: RootSpacePosition = RootSpacePosition(DVec2::new(-4.0, -3.0));

        const REFERENCE_VEL: RootSpaceLinearVelocity =
            RootSpaceLinearVelocity(DVec2::new(5.0, 9.0));
        const ROOTSPACE_VEL: RootSpaceLinearVelocity =
            RootSpaceLinearVelocity(DVec2::new(-4.0, -3.0));

        const ANG_VEL: f32 = 0.0;

        let rigid = ROOTSPACE_POS.to_rigid_space_position(REFERENCE_POS);

        assert_eq!(rigid.0, Vec2::new(-9.0, -12.0));
        assert_eq!(rigid.to_root_space_position(REFERENCE_POS), ROOTSPACE_POS);

        let rigid = ROOTSPACE_VEL.to_rigid_space_linear_velocity(REFERENCE_VEL);

        assert_eq!(*rigid, Vec2::new(-9.0, -12.0));
        assert_eq!(
            rigid.to_root_space_linear_velocity(REFERENCE_VEL),
            ROOTSPACE_VEL
        );

        let rigid_full = rigid.to_rigid_space_velocity(ANG_VEL);

        assert_eq!(
            rigid_full,
            RigidSpaceVelocity {
                linvel: Vec2::new(-9.0, -12.0),
                angvel: ANG_VEL
            }
        );
        assert_eq!(
            rigid_full.to_root_space_linear_velocity(REFERENCE_VEL),
            ROOTSPACE_VEL
        );
    }
}
