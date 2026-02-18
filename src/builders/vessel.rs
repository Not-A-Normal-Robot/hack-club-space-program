use crate::components::{
    frames::{RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition},
    relations::{CelestialParent, RailMode},
    vessel::Vessel,
};
use bevy::{prelude::*, sprite_render::Material2d};
use bevy_rapier2d::prelude::*;

#[derive(Clone, Debug)]
pub struct VesselBuilder<M: Material2d> {
    pub name: Name,
    pub collider: Collider,
    pub mass: AdditionalMassProperties,
    pub parent: CelestialParent,
    pub rail_mode: RailMode,
    pub position: RootSpacePosition,
    pub linvel: RootSpaceLinearVelocity,
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<M>,
    pub angvel: f32,
    pub angle: f32,
}

impl<M: Material2d> VesselBuilder<M> {
    pub const fn base_bundle() -> impl Bundle {
        (
            Vessel,
            RigidBody::Dynamic,
            Friction::coefficient(0.9),
            Restitution::coefficient(0.02),
            Ccd { enabled: true },
            ExternalForce {
                force: Vec2::ZERO,
                torque: 0.0,
            },
        )
    }

    /// Builds a vessel with the rigid body properties processed (i.e., not on rails).
    ///
    /// For the on-rails version, see [`build_on_rails`][Self::build_on_rails].
    pub fn build_rigid(self) -> impl Bundle {
        (
            self.name,
            self.collider,
            self.mass,
            self.parent,
            self.rail_mode,
            self.position,
            self.linvel,
            RigidSpaceVelocity {
                angvel: self.angvel,
                linvel: Vec2::NAN,
            },
            Transform::from_rotation(Quat::from_rotation_z(self.angle)),
            self.mesh,
            self.material,
            Self::base_bundle(),
        )
    }

    /// Builds a vessel with the rigid body properties skipped (i.e., on rails).
    ///
    /// For the rigid-body version, see [`build_rigid`][Self::build_rigid].
    pub fn build_on_rails(self) -> impl Bundle {
        (self.build_rigid(), RigidBodyDisabled)
    }
}
