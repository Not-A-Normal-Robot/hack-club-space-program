//! # Reference Frames
//!
//! Root Space converts into Parent Space
//! Parent Space converts into Rigid Space position (with its own rotation)
//! Parent Space position + Rigid Space rotation = Camera Space transform

use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;

/// Coordinates relative to root body.
///
/// Only used for drawing external objects.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct RootSpacePosition(pub DVec2);

/// Coordinates relative to parent body.
///
/// Double precision, and unscaled.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct ParentSpacePosition(pub DVec2);

impl ParentSpacePosition {
    pub const ZERO: Self = Self(DVec2::ZERO);

    /// Convert this parent space position into a rigid space position.
    ///
    /// # Parameters
    /// * `active_vessel_parent_pos`: The active vessel's parent-space position.
    pub fn to_rigid_space_position(
        &self,
        active_vessel_parent_pos: ParentSpacePosition,
    ) -> RigidSpacePosition {
        let rigid_double = self.0 - active_vessel_parent_pos.0;

        RigidSpacePosition(Vec2::new(rigid_double.x as f32, rigid_double.y as f32))
    }

    /// Convert this parent space position into a rigid space transform.
    ///
    /// # Parameters
    /// * `active_vessel_parent_pos`: The active vessel's parent-space position.
    pub fn to_rigid_space_transform(
        &self,
        active_vessel_parent_pos: ParentSpacePosition,
        rotation: Quat,
        scale: Vec3,
    ) -> RigidSpaceTransform {
        self.to_rigid_space_position(active_vessel_parent_pos)
            .to_rigid_space_transform(rotation, scale)
    }
}

/// Coordinates relative to parent body.
///
/// Double precision, and unscaled.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct ParentSpaceLinearVelocity(pub DVec2);

impl ParentSpaceLinearVelocity {
    pub const ZERO: Self = Self(DVec2::ZERO);

    pub fn to_rigid_space_velocity(
        &self,
        active_vessel_parent_vel: ParentSpaceLinearVelocity,
        angvel: f32,
    ) -> RigidSpaceVelocity {
        let rigid_double = self.0 - active_vessel_parent_vel.0;

        RigidSpaceVelocity {
            linvel: Vec2::new(rigid_double.x as f32, rigid_double.y as f32),
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

    pub fn to_rigid_space_transform(&self, rotation: Quat, scale: Vec3) -> RigidSpaceTransform {
        RigidSpaceTransform(Transform {
            translation: self.0.extend(0.0),
            rotation,
            scale,
        })
    }

    pub fn to_parent_space_position(
        &self,
        active_vessel_parent_pos: ParentSpacePosition,
    ) -> ParentSpacePosition {
        let rigid_pos = DVec2::new(self.0.x as f64, self.0.y as f64);

        ParentSpacePosition(active_vessel_parent_pos.0 + rigid_pos)
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
    fn to_parent_space_linear_velocity(
        &self,
        active_vessel_parent_vel: ParentSpaceLinearVelocity,
    ) -> ParentSpaceLinearVelocity;
}

impl RigidSpaceVelocityImpl for RigidSpaceVelocity {
    fn to_parent_space_linear_velocity(
        &self,
        active_vessel_parent_vel: ParentSpaceLinearVelocity,
    ) -> ParentSpaceLinearVelocity {
        let rigid_vel = DVec2::new(self.linvel.x as f64, self.linvel.y as f64);

        ParentSpaceLinearVelocity(active_vessel_parent_vel.0 + rigid_vel)
    }
}

/// Coordinates relative to camera.
///
/// Single precision, and scaled to camera zoom amount.
pub type CameraSpaceTransform = Transform;

#[cfg(test)]
mod tests {
    use bevy::math::{DVec2, Vec2};

    use crate::components::frames::{
        ParentSpaceLinearVelocity, ParentSpacePosition, RigidSpaceVelocityImpl,
    };

    #[test]
    fn parent_rigid_conversion() {
        const REFERENCE_POS: ParentSpacePosition = ParentSpacePosition(DVec2::new(5.0, 9.0));
        const PARENTSPACE_POS: ParentSpacePosition = ParentSpacePosition(DVec2::new(-4.0, -3.0));

        let rigid = PARENTSPACE_POS.to_rigid_space_position(REFERENCE_POS);

        assert_eq!(rigid.0, Vec2::new(-9.0, -12.0));
        assert_eq!(
            rigid.to_parent_space_position(REFERENCE_POS),
            PARENTSPACE_POS
        );

        const REFERENCE_VEL: ParentSpaceLinearVelocity =
            ParentSpaceLinearVelocity(DVec2::new(5.0, 9.0));
        const PARENTSPACE_VEL: ParentSpaceLinearVelocity =
            ParentSpaceLinearVelocity(DVec2::new(-4.0, -3.0));

        let rigid = PARENTSPACE_VEL.to_rigid_space_velocity(REFERENCE_VEL, 0.0);

        assert_eq!(rigid.linvel, Vec2::new(-9.0, -12.0));
        assert_eq!(
            rigid.to_parent_space_linear_velocity(REFERENCE_VEL),
            PARENTSPACE_VEL
        );
    }
}
