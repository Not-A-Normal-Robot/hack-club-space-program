use crate::components::{
    frames::{RigidSpaceVelocity, RootSpaceLinearVelocity, RootSpacePosition},
    relations::ParentBody,
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Clone, Debug)]
pub struct VesselBuilder {
    pub collider: Collider,
    pub mass: AdditionalMassProperties,
    pub parent: ParentBody,
    pub position: RootSpacePosition,
    pub linvel: RootSpaceLinearVelocity,
    pub angvel: f32,
    pub angle: f32,
}

impl VesselBuilder {
    pub const fn base_bundle() -> impl Bundle {
        (
            RigidBody::Dynamic,
            Friction::coefficient(0.2),
            Restitution::coefficient(0.02),
            Ccd { enabled: true },
            // TODO: Disable sleeping
        )
    }
    pub fn build(self) -> impl Bundle {
        (
            self.collider,
            self.mass,
            self.parent,
            self.position,
            self.linvel,
            RigidSpaceVelocity {
                angvel: self.angvel,
                linvel: Vec2::NAN,
            },
            Transform::from_rotation(Quat::from_rotation_z(self.angle)),
            Self::base_bundle(),
        )
    }
}
