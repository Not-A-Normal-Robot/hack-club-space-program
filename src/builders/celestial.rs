use crate::components::{
    celestial::{CelestialBody, Heightmap},
    frames::{RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition},
};
use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::prelude::*;

/// Recommended additional components:
/// - [`CelestialParent`][crate::components::relations::CelestialParent]
/// - [`RailMode`][crate::components::relations::RailMode]
#[derive(Clone, Debug)]
pub struct CelestialBodyBuilder {
    pub radius: f32,
    pub heightmap: Heightmap,
    pub mass: AdditionalMassProperties,
    pub angle: f32,
}

impl CelestialBodyBuilder {
    pub const fn base_bundle() -> impl Bundle {
        (
            RigidBody::KinematicVelocityBased,
            RootSpacePosition(DVec2::ZERO),
            RootSpaceLinearVelocity(DVec2::ZERO),
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
            RigidSpaceVelocity {
                // TODO: Celestial rotation
                angvel: 0.0,
                linvel: Vec2::NAN,
            },
            Transform::from_rotation(Quat::from_rotation_z(self.angle)),
        )
    }
}
