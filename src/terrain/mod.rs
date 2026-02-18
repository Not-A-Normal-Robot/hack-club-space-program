use crate::components::{camera::SimCameraZoom, celestial::Terrain};
use bevy::math::{DVec2, Vec3};
use fastnoise_lite::{FastNoiseLite, FractalType};

pub mod render;

/// A vector relative to the celestial body's center,
/// representing a point in the terrain/body boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainPoint(pub DVec2);

impl TerrainPoint {
    /// Transforms this vector, then downcast it to 32-bit mesh-ready vectors.
    #[must_use]
    pub fn transform_downcast(self, shift: DVec2, zoom: SimCameraZoom) -> Vec3 {
        (zoom.0 * (self.0 + shift)).as_vec2().extend(0.0)
    }
}

/// A terrain generator wrapper around Terrain and FastNoiseLite.
pub struct TerrainGen {
    multiplier: f64,
    offset: f64,
    noisegen: FastNoiseLite,
}

impl TerrainGen {
    pub fn new(terrain: Terrain) -> Self {
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
    fn get_terrain_vector(&self, theta: f64) -> TerrainPoint {
        let (sin, cos) = theta.sin_cos();

        let noise = self.noisegen.get_noise_2d(sin, cos) as f64;
        let noise = noise.mul_add(self.multiplier, self.offset);

        TerrainPoint(DVec2::new(noise * cos, noise * sin))
    }
}
