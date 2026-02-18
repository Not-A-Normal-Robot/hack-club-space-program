
use bevy::prelude::*;
use bevy_rapier2d::prelude::RigidBody;

/// The terrain parameters of a celestial body.
#[derive(Clone, Copy, Component, Debug, Default)]
pub struct Terrain {
    /// The seed given to the noise generator.
    pub seed: i32,
    /// The amount of octaves for the noise generator.
    pub octaves: i32,
    /// The base frequency for the noise generator.
    pub frequency: f32,
    /// The gain for the noise generator.
    pub gain: f32,
    /// The lacunarity for the noise generator.
    pub lacunarity: f32,
    /// The offset to give to the noise generator output.
    pub offset: f64,
    /// The multiplier to give to the noise generator output.
    pub multiplier: f64,
    /// The amount of subdivisions for mesh generation.
    pub subdivs: u8,
}

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
    /// The "base radius" of a celestial body.
    ///
    /// This is the "sea level" in most cases.
    ///
    /// To calculate the minimum or maximum radius,
    /// use this alongside the terrain multiplier.
    pub base_radius: f32,
}
