use crate::components::{
    celestial::{CelestialBody, Heightmap},
    frames::{RootSpaceLinearVelocity, RootSpacePosition},
};
use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;

/// Recommended additional components:
/// - [`ParentBody`][crate::components::relations::ParentBody]
#[derive(Clone, Debug)]
pub struct CelestialBodyBuilder {
    pub radius: f32,
    pub heightmap: Heightmap,
    pub mass: AdditionalMassProperties,
}

impl CelestialBodyBuilder {
    pub const fn base_bundle() -> impl Bundle {
        (
            RigidBody::KinematicVelocityBased,
            RootSpacePosition(DVec2::ZERO),
            RootSpaceLinearVelocity(DVec2::ZERO),
            Transform::IDENTITY,
            RigidBodyDisabled,
        )
    }

    pub fn build(self) -> impl Bundle {
        (
            CelestialBody {
                radius: self.radius,
            },
            Collider::ball(self.radius),
            self.heightmap,
            self.mass,
            Self::base_bundle(),
        )
    }
}
