use bevy::prelude::*;
use bevy_rapier2d::prelude::RigidBody;

/// The heightmap of a celestial body.
///
/// The heightmap is used for determining the terrain
/// offsets of a celestial body.
///
/// An empty ([]) heightmap means there is no
/// additional collider on top of the regular ball
/// collider.
///
/// Entries in the array indicate an offset to the
/// height. This means that if a celestial body is
/// 6 000 units wide and you want to have a 2.0-unit
/// tall mountain somewhere, you set the heightmap
/// at that location to 2.0, not 6002.
///
/// TODO: Finer details for heightmaps determined using seeded Perlin noise
#[derive(Clone, Component, Debug, Default)]
pub struct Heightmap(pub Box<[f32]>);

impl Heightmap {
    pub fn empty() -> Self {
        Self(Box::from([]))
    }
}

#[derive(Clone, Copy, Component)]
#[require(Heightmap)]
#[require(RigidBody::KinematicVelocityBased)]
pub struct CelestialBody {
    pub radius: f32,
}
