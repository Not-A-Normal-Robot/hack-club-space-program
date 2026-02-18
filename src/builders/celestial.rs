use crate::components::{
    celestial::{CelestialBody, Terrain},
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
    pub mass: f32,
    pub angle: f32,
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<M>,
}

impl<M: Material2d> CelestialBodyBuilder<M> {
    #[must_use] 
    pub const fn base_bundle() -> impl Bundle {
        (
            RigidBody::KinematicVelocityBased,
            RootSpacePosition(DVec2::ZERO),
            RootSpaceLinearVelocity(DVec2::ZERO),
            Friction::new(0.7),
        )
    }

    #[must_use] 
    pub fn build_without_terrain(self) -> impl Bundle {
        (
            self.name,
            CelestialBody {
                base_radius: self.radius,
            },
            Collider::ball(self.radius),
            AdditionalMassProperties::MassProperties(MassProperties {
                local_center_of_mass: Vec2::ZERO,
                mass: self.mass,
                ..Default::default()
            }),
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

    #[must_use] 
    pub fn build_with_terrain(self, terrain: Terrain) -> impl Bundle {
        (self.build_without_terrain(), terrain)
    }
}
