//! # Reference Frames
//!
//! Root Space converts into Parent Space
//! Parent Space converts into Rigid Space position (with its own rotation)
//! Parent Space position + Rigid Space rotation = Camera Space transform

use bevy::{math::DVec3, prelude::*};
use bevy_rapier2d::prelude::*;

/// Coordinates relative to root body.
///
/// Only used for drawing external objects.
#[derive(Component)]
pub struct RootSpacePosition(pub DVec3);

/// Coordinates relative to parent body.
///
/// Double precision, and unscaled.
#[derive(Component)]
pub struct ParentSpacePosition(pub DVec3);

/// Coordinates relative to active vessel.
///
/// Single precision, and unscaled. Used for bevy_rapier2d.
#[derive(Component)]
pub struct RigidSpaceTransform(pub Transform);

/// Coordinates relative to active vessel.
///
/// Single precision, and unscaled. Used for bevy_rapier2d.
pub type RigidSpaceVelocity = Velocity;

/// Coordinates relative to camera.
///
/// Single precision, and scaled to camera zoom amount.
pub type CameraSpaceTransform = Transform;
