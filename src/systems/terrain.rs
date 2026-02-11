use crate::components::{
    camera::{SimCameraOffset, SimCameraZoom},
    celestial::{CelestialBody, Terrain},
    frames::RootSpacePosition,
};
use bevy::{ecs::query::QueryData, math::DVec2, prelude::*};
use core::{f64::consts::TAU, num::NonZeroU8};
use fastnoise_lite::{FastNoiseLite, FractalType};

// Math based off a sketch:
// https://www.desmos.com/calculator/vgdk9qd2ux

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
    /// Generate a lowest-quality LoD vector list.
    fn new(terrain_gen: &TerrainGen) -> Self {
        Self(vec![terrain_gen.gen_lod(0, 0.0)])
    }

    /// Generate a fully-realized LoD vector list.
    fn new_full(terrain_gen: &TerrainGen, ending_level: NonZeroU8, focus: f64) -> Self {
        let mut this = Self::new(terrain_gen);
        this.update_lods(terrain_gen, ending_level, f64::NAN, focus);
        this
    }

    /// Updates the LoD vectors starting from a given
    /// level up to (and including) a final level.
    fn update_lods(
        &mut self,
        terrain_gen: &TerrainGen,
        ending_level: NonZeroU8,
        prev_focus: f64,
        new_focus: f64,
    ) {
        debug_assert!(self.0.is_empty());

        (0..=ending_level.get()).for_each(|level| {
            let level_not_loaded = self.0.len() >= level as usize;
            let lod_needs_updating =
                lod_level_start(level, prev_focus) != lod_level_start(level, new_focus);

            if level_not_loaded || lod_needs_updating {
                let vecs = terrain_gen.gen_lod(level, new_focus);

                if level_not_loaded {
                    self.0.push(vecs);
                } else {
                    *self.0.get_mut(level as usize).unwrap() = vecs;
                }
            }
        })
    }

    /// Stitches together this LoD vectors into a mesh.
    fn create_mesh(&self) -> Mesh {
        todo!()
    }
}

/// The previous focus angle.
#[derive(Clone, Copy, Component)]
struct PrevFocus(f64);

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
    fn gen_lod(&self, lod_level: u8, focus: f64) -> [RelativeVector; LOD_VERTS] {
        // https://www.desmos.com/calculator/vgdk9qd2ux
        // point((tau / verts) (i ⋅ iter_scale + start))

        let start = lod_level_start(lod_level, focus);
        let iter_scale = (LOD_DIVISIONS as f64).powi(-(lod_level as i32));

        core::array::from_fn(|i| {
            self.get_terrain_vector(
                const { TAU / LOD_VERTS as f64 } * (i as f64 * iter_scale + start),
            )
        })
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
///
/// This is in revolutions. To get the theta in radians, multiply this by tau.
fn lod_level_start(lod_level: u8, focus: f64) -> f64 {
    // https://www.desmos.com/calculator/vgdk9qd2ux
    // start = divisions^(1 - level) ⋅
    //  round( (verts / 2pi) ⋅ divisions^(level - 1) ⋅ focus - verts/(2 ⋅ divisions))
    //
    // → coeff = divisions^(1 - level)
    // frac = (verts / 2pi) ⋅ divisions^(level - 1) ⋅ focus
    // frac_offset = -verts / (2 ⋅ divisions)
    // start = coeff round(frac + frac_offset)

    const FRAC_OFFSET: f64 = LOD_VERTS as f64 / (-2.0 * LOD_DIVISIONS as f64);

    let coeff = (LOD_DIVISIONS as f64).powi(1 - lod_level as i32);
    let frac = const { LOD_VERTS as f64 / TAU }
        * (LOD_DIVISIONS as f64).powi(lod_level as i32 - 1)
        * focus;

    coeff * (frac + FRAC_OFFSET).round()
}

#[cfg(test)]
mod tests {
    use core::{f64::consts::TAU, num::NonZeroU8};

    use crate::{
        components::celestial::Terrain,
        systems::terrain::{
            LOD_DIVISIONS, LOD_VERTS, LodVectors, PrevFocus, RelativeVector, TerrainGen,
        },
    };

    const TEST_TERRAIN: Terrain = Terrain {
        seed: 0xabcba,
        octaves: 8,
        frequency: 1.0,
        gain: 0.5,
        lacunarity: 2.0,
        offset: 20000000.0,
        multiplier: 10.0,
        subdivs: 8,
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

    #[test]
    fn lod_ranges() {
        fn get_range(vecs: impl IntoIterator<Item = RelativeVector>) -> f64 {
            let (min, max) = vecs
                .into_iter()
                .map(|v| v.0.to_angle())
                .fold(None::<(f64, f64)>, |acc, x| {
                    Some(match acc {
                        None => (x, x),
                        Some((min, max)) => (min.min(x), max.max(x)),
                    })
                })
                .unwrap();
            max - min
        }

        let terrain_gen = TerrainGen::new(TEST_TERRAIN);
        const FULL_RANGE: f64 = TAU * ((LOD_VERTS as f64 - 1.0) / LOD_VERTS as f64);
        const TOLERANCE: f64 = 1e-6;

        for level in 0..=TEST_TERRAIN.subdivs {
            let range = get_range(terrain_gen.gen_lod(level as u8, 0.0));
            let expected_range = FULL_RANGE * (LOD_DIVISIONS as f64).powi(-(level as i32));
            assert!(
                (range - expected_range).abs() < TOLERANCE,
                "Expected range of {expected_range}, got {range}"
            );
        }
    }
}
