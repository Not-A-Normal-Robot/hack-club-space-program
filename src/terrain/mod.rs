use crate::components::main_game::{camera::SimCameraZoom, celestial::Terrain};
use bevy::{math::DVec2, prelude::*};
use fastnoise_lite::{FastNoiseLite, FractalType};

pub(crate) mod collider;
pub(crate) mod gfx;

/// A vector relative to the celestial body's center,
/// representing a point in the terrain/body boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TerrainPoint(pub(crate) DVec2);

impl TerrainPoint {
    /// Shifts this vector, then downcast it to 32-bit collider-ready vectors.
    ///
    /// For the shift, use a method similar to obtaining a `RigidSpacePosition`.
    #[must_use]
    pub(crate) fn phys_downcast(self, shift: DVec2) -> Vec2 {
        (self.0 + shift).as_vec2()
    }

    /// Transforms this vector, then downcast it to 32-bit graphics-ready vectors.
    #[must_use]
    pub(crate) fn gfx_tf_downcast(self, shift: DVec2, zoom: SimCameraZoom) -> Vec3 {
        (zoom.0 * (self.0 + shift)).as_vec2().extend(0.0)
    }
}

/// A terrain generator wrapper around Terrain and `FastNoiseLite`.
pub(crate) struct TerrainGen {
    multiplier: f64,
    offset: f64,
    noisegen: FastNoiseLite,
}

impl TerrainGen {
    #[must_use]
    pub(crate) fn new(terrain: Terrain) -> Self {
        let mut noisegen = FastNoiseLite::with_seed(terrain.seed);
        noisegen.fractal_type = FractalType::FBm;
        noisegen.octaves = terrain.octaves;
        noisegen.frequency = terrain.frequency;
        noisegen.gain = terrain.gain;
        noisegen.lacunarity = terrain.lacunarity;

        Self {
            multiplier: terrain.multiplier,
            offset: terrain.offset,
            noisegen,
        }
    }

    /// Gets the vector pointing to the surface at the
    /// given theta.
    #[must_use]
    pub(crate) fn get_terrain_vector(&self, theta: f64) -> TerrainPoint {
        let (sin, cos) = theta.sin_cos();

        let noise = self.get_terrain_altitude_unchecked(sin, cos);

        TerrainPoint(DVec2::new(noise * cos, noise * sin))
    }

    /// Gets the altitude of the terrain at the given theta.
    ///
    /// This altitude is relative to the centre of the planet.
    #[must_use]
    pub(crate) fn get_terrain_altitude(&self, theta: f64) -> f64 {
        let (sin, cos) = theta.sin_cos();

        self.get_terrain_altitude_unchecked(sin, cos)
    }

    /// Gets the altitude of the terrain at the given theta.
    ///
    /// This altitude is relative to the centre of the planet.
    ///
    /// # Unchecked Operation
    /// This operation does not check for the validity of the `sin_theta` and
    /// `cos_theta` parameters. Make sure you derive it from `theta.sin_cos()`
    /// or a similar method.
    #[must_use]
    pub(crate) fn get_terrain_altitude_unchecked(&self, sin_theta: f64, cos_theta: f64) -> f64 {
        let noise = f64::from(self.noisegen.get_noise_2d(sin_theta, cos_theta));
        noise.mul_add(self.multiplier, self.offset)
    }
}
