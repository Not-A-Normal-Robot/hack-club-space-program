//! # Reference Frames
//!
//! Root Space converts into Parent Space
//! Parent Space converts into Rigid Space position (with its own rotation)
//! Parent Space position + Rigid Space rotation = Camera Space transform

use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::components::SimCameraZoom;

/// Coordinates relative to root body.
///
/// Used for orbital physics and as source of truth.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct RootSpacePosition(pub DVec2);

impl RootSpacePosition {
    pub fn to_rigid_space_position(
        self,
        active_vessel_pos: RootSpacePosition,
    ) -> RigidSpacePosition {
        let position = self.0 - active_vessel_pos.0;
        RigidSpacePosition(Vec2::new(position.x as f32, position.y as f32))
    }

    pub fn to_camera_space_transform(
        self,
        rotation: Quat,
        camera_offset: RootSpacePosition,
        camera_zoom: SimCameraZoom,
    ) -> CameraSpaceTransform {
        let offset = (self.0 - camera_offset.0) * camera_zoom.0;

        CameraSpaceTransform(Transform {
            rotation,
            translation: Vec3::new(offset.x as f32, offset.y as f32, 0.0),
            scale: Vec3::splat(camera_zoom.0 as f32),
        })
    }
}

/// Coordinates relative to root body.
///
/// Used for orbital physics and as source of truth.
#[derive(Clone, Copy, Component, Debug, Default, PartialEq)]
pub struct RootSpaceLinearVelocity(pub DVec2);

impl RootSpaceLinearVelocity {
    pub fn to_rigid_space_velocity(
        self,
        active_vessel_vel: RootSpaceLinearVelocity,
        angvel: f32,
    ) -> RigidSpaceVelocity {
        let linvel = self.0 - active_vessel_vel.0;
        RigidSpaceVelocity {
            linvel: Vec2::new(linvel.x as f32, linvel.y as f32),
            angvel,
        }
    }
}

/// Coordinates relative to active vessel.
///
/// Single precision, and unscaled. Used to be transformed to RigidSpaceTransform.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct RigidSpacePosition(pub Vec2);

impl RigidSpacePosition {
    pub const ZERO: Self = Self(Vec2::ZERO);

    pub fn to_root_space_position(
        &self,
        active_vessel_pos: RootSpacePosition,
    ) -> RootSpacePosition {
        let rigid = DVec2::new(self.0.x as f64, self.0.y as f64);

        RootSpacePosition(active_vessel_pos.0 + rigid)
    }

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
/// Single precision, and unscaled. Used for bevy_rapier2d.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct RigidSpaceTransform(pub Transform);

impl RigidSpaceTransform {
    pub fn position(&self) -> RigidSpacePosition {
        RigidSpacePosition(self.0.translation.truncate())
    }
}

/// Coordinates relative to active vessel.
///
/// Single precision, and unscaled. Used for bevy_rapier2d.
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
        let linvel = DVec2::new(self.linvel.x as f64, self.linvel.y as f64);
        RootSpaceLinearVelocity(active_vessel_vel.0 + linvel)
    }
}

/// Coordinates relative to camera.
///
/// Single precision, and scaled to camera zoom amount.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct CameraSpaceTransform(pub Transform);

#[cfg(test)]
mod tests {
    use bevy::math::{DVec2, Vec2};

    use crate::components::frames::{
        RigidSpaceVelocityImpl as _, RootSpaceLinearVelocity, RootSpacePosition,
    };

    #[test]
    fn root_rigid_conversion() {
        const REFERENCE_POS: RootSpacePosition = RootSpacePosition(DVec2::new(5.0, 9.0));
        const PARENTSPACE_POS: RootSpacePosition = RootSpacePosition(DVec2::new(-4.0, -3.0));

        let rigid = PARENTSPACE_POS.to_rigid_space_position(REFERENCE_POS);

        assert_eq!(rigid.0, Vec2::new(-9.0, -12.0));
        assert_eq!(rigid.to_root_space_position(REFERENCE_POS), PARENTSPACE_POS);

        const REFERENCE_VEL: RootSpaceLinearVelocity =
            RootSpaceLinearVelocity(DVec2::new(5.0, 9.0));
        const PARENTSPACE_VEL: RootSpaceLinearVelocity =
            RootSpaceLinearVelocity(DVec2::new(-4.0, -3.0));

        let rigid = PARENTSPACE_VEL.to_rigid_space_velocity(REFERENCE_VEL, 0.0);

        assert_eq!(rigid.linvel, Vec2::new(-9.0, -12.0));
        assert_eq!(
            rigid.to_root_space_linear_velocity(REFERENCE_VEL),
            PARENTSPACE_VEL
        );
    }
}
