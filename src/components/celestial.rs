use bevy::prelude::*;
use bevy_rapier2d::prelude::RigidBody;

#[derive(Clone, Component, Debug, Default)]
pub struct Heightmap(pub Box<[f32]>);

#[derive(Clone, Copy, Component)]
#[require(Heightmap)]
#[require(RigidBody::KinematicVelocityBased)]
pub struct CelestialBody {
    pub radius: f32,
}
