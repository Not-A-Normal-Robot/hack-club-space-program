use crate::components::{
    celestial::{CelestialBody, Heightmap, Terrain},
    frames::{RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition},
};
use bevy::{math::DVec2, prelude::*, sprite_render::Material2d};
use bevy_rapier2d::prelude::*;

/// Recommended additional components:
/// - [`CelestialParent`][crate::components::relations::CelestialParent]
/// - [`RailMode`][crate::components::relations::RailMode]
#[derive(Clone, Debug)]
pub struct CelestialBodyBuilder<M: Material2d> {
    pub name: Name,
    pub radius: f32,
    pub mass: AdditionalMassProperties,
    pub angle: f32,
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<M>,
    pub terrain: Terrain,
}

impl<M: Material2d> CelestialBodyBuilder<M> {
    pub const fn base_bundle() -> impl Bundle {
        (
            RigidBody::KinematicVelocityBased,
            RootSpacePosition(DVec2::ZERO),
            RootSpaceLinearVelocity(DVec2::ZERO),
            Friction::new(0.7),
        )
    }

    pub fn build(self) -> impl Bundle {
        (
            self.name,
            CelestialBody {
                base_radius: self.radius,
            },
            Collider::ball(self.radius),
            self.mass,
            self.terrain,
            self.mesh,
            self.material,
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
