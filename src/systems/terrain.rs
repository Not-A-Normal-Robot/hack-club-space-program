use crate::components::{
    camera::{SimCameraOffset, SimCameraZoom},
    celestial::{CelestialBody, Terrain},
    frames::RootSpacePosition,
};
use bevy::{ecs::query::QueryData, math::DVec2, prelude::*};
use core::{f64::consts::TAU, num::NonZeroU8};
use fastnoise_lite::{FastNoiseLite, FractalType};

// Math based off a sketch:
// https://www.desmos.com/calculator/5vxao6sxgq

/// How many vertices for each LoD level.
pub const LOD_VERTS: usize = 128;

/// How much smaller the next LoD level is compared to the previous one.
/// (Level 0 = full revolution)
pub const LOD_DIVISIONS: usize = 8;

/// A vector relative to this object's center.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RelativeVector(DVec2);

/// A list of LoD offsets.
#[derive(Clone, Component, Debug, PartialEq)]
struct LodVectors(Vec<[RelativeVector; LOD_VERTS]>);

impl LodVectors {
    /// Updates the LoD vectors starting from a given
    /// level up to (and including) a final level.
    fn update_lods(
        &mut self,
        terrain_gen: TerrainGen,
        starting_level: NonZeroU8,
        ending_level: u8,
    ) {
        let starting_level = starting_level.get();

        todo!();
    }
}

/// The previous focus angle.
#[derive(Clone, Copy, Component)]
struct PrevFocus(f64);

impl PrevFocus {
    /// Updates this struct's value.
    ///
    /// Returns the LoD level meshes that should be updated.
    ///
    /// For example, large focus changes may require an LoD update for
    /// levels 1 and above, therefore this function would return
    /// `Some(NonZeroU8(1))`.
    ///
    /// If the required LoD update level is higher than the amount of
    /// subdivs, then nothing shall need to be updated.\
    /// For example, if this function returns `Some(NonZeroU8(8))` even
    /// though the subdiv amount is 4, nothing needs to be updated.
    ///
    /// If this function returns `None`, nothing needs to be updated.
    fn update(&mut self, cur_focus: f64, _lod_level: f64) -> Option<NonZeroU8> {
        let _prev_focus = self.0;
        self.0 = cur_focus;
        todo!();
    }
}

/// A terrain generator wrapper around Terrain and FastNoiseLite.
struct TerrainGen {
    multiplier: f64,
    offset: f64,
    noisegen: FastNoiseLite,
}

impl TerrainGen {
    fn new(terrain: Terrain) -> Self {
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
    fn get_terrain_vector(&self, theta: f64) -> RelativeVector {
        let (sin, cos) = theta.sin_cos();

        let noise = self.noisegen.get_noise_2d(sin, cos) as f64;
        let noise = noise.mul_add(self.multiplier, self.offset);

        RelativeVector(DVec2::new(noise * cos, noise * sin))
    }

    /// Gets the LoD vector array at a certain LoD level.
    fn gen_lod(&self, lod_level: f64) -> [RelativeVector; LOD_VERTS] {
        todo!();
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct EntityComponents {
    terrain: &'static Terrain,
    body: &'static CelestialBody,
    pos: &'static RootSpacePosition,
    offsets: Option<&'static mut LodVectors>,
    prev_focus: Option<&'static mut PrevFocus>,
}

#[derive(Clone, Copy)]
struct GlobalData {
    zoom: SimCameraZoom,
    offset: SimCameraOffset,
}

/// Gets the starting theta for a given LoD level and focus theta.
const fn lod_level_start(lod_level: u8, focus: f64) -> f64 {
    // https://www.desmos.com/calculator/xmmdndxdwj
    // start = 2pi ⋅ divisions^(1 - level) ⋅
    //  round( (verts ⋅ divisions^(level - 1) ⋅ focus) / 2pi - verts/(2 ⋅ divisions))
    //
    // → coeff = 2pi ⋅ divisions^(1 - level)
    // frac = (verts / 2pi) ⋅ divisions^(level - 1) ⋅ focus
    // frac_offset = -verts / (2 ⋅ divisions)
    // start = coeff round(frac + frac_offset)

    const FRAC_OFFSET: f64 = LOD_VERTS as f64 / (-2.0 * LOD_DIVISIONS as f64);

    let coeff = TAU * LOD_DIVISIONS.pow(1 - lod_level as u32) as f64;
    let frac =
        const { LOD_VERTS as f64 / TAU } * LOD_DIVISIONS.pow(lod_level as u32 - 1) as f64 * focus;

    coeff * (frac + FRAC_OFFSET).round()
}

#[cfg(test)]
mod tests {
    use core::f64::consts::TAU;

    use crate::{
        components::celestial::Terrain,
        systems::terrain::TerrainGen,
        // systems::terrain::{create_terrain_gen, get_terrain_height},
    };

    const TEST_TERRAIN: Terrain = Terrain {
        seed: 0xabcba,
        octaves: 8,
        frequency: 1.0,
        gain: 0.5,
        lacunarity: 2.0,
        offset: 20000000.0,
        multiplier: 10.0,
        subdivs: 2,
    };

    #[test]
    fn determinism() {
        let terrain_gen = TerrainGen::new(TEST_TERRAIN);

        let first: Box<[_]> = (0..1024)
            .map(|i| terrain_gen.get_terrain_vector(i as f64 * 1.0 / 1024.0))
            .collect();

        let terrain_gen = TerrainGen::new(TEST_TERRAIN);

        let second: Box<_> = (0..1024)
            .map(|i| terrain_gen.get_terrain_vector(i as f64 * 1.0 / 1024.0))
            .collect();

        assert_eq!(first, second);
    }

    #[test]
    fn circular() {
        let terrain_gen = TerrainGen::new(TEST_TERRAIN);
        let zero = terrain_gen.get_terrain_vector(0.0);
        let tau = terrain_gen.get_terrain_vector(TAU);

        assert_eq!(zero, tau);
    }
}
