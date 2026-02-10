use core::num::NonZeroU8;

use crate::components::{
    camera::{SimCameraOffset, SimCameraZoom},
    celestial::{CelestialBody, Terrain},
    frames::RootSpacePosition,
};
use bevy::{ecs::query::QueryData, math::DVec2, prelude::*};
use fastnoise_lite::{FastNoiseLite, FractalType};

pub const LOD_VERTS: usize = 128;
pub const LOD_DIVISIONS: usize = 8;

/// A vector relative to this object's center.
#[derive(Clone, Copy, Debug)]
struct RelativeVector(DVec2);

/// A list of LoD offsets.
#[derive(Clone, Component, Debug)]
struct LodVectors(Vec<[RelativeVector; LOD_VERTS]>);

/// The previous target angle.
#[derive(Clone, Copy, Component)]
struct PrevTarget(f64);

/// Returns the LoD level that should be updated.
///
/// For example, large target changes may require an LoD update for
/// levels 1 and above, therefore this function would return
/// `Some(NonZeroU8(1))`.
///
/// If the required LoD update level is higher than the amount of
/// subdivs, then nothing shall need to be updated.\
/// For example, if this function returns `Some(NonZeroU8(8))` even
/// though the subdiv amount is 4, nothing needs to be updated.
///
/// If this function returns `None`, nothing needs to be updated.
fn lod_regen_level(_prev_target: f64, _cur_target: f64, _lod_level: f64) -> Option<NonZeroU8> {
    todo!();
}

fn create_noisegen(terrain: Terrain) -> FastNoiseLite {
    let mut noisegen = FastNoiseLite::with_seed(terrain.seed);
    noisegen.fractal_type = FractalType::FBm;
    noisegen.octaves = terrain.octaves;
    noisegen.frequency = terrain.frequency;
    noisegen.gain = terrain.gain;
    noisegen.lacunarity = terrain.lacunarity;
    noisegen
}

fn get_terrain_height(offset: f64, multi: f64, noisegen: &FastNoiseLite, theta: f64) -> f64 {
    let (x, y) = theta.sin_cos();

    let noise = noisegen.get_noise_2d(x, y) as f64;

    noise.mul_add(multi, offset)
}

#[derive(QueryData)]
#[query_data(mutable)]
struct EntityComponents {
    terrain: &'static Terrain,
    body: &'static CelestialBody,
    pos: &'static RootSpacePosition,
    offsets: Option<&'static mut LodVectors>,
    prev_target: Option<&'static mut PrevTarget>,
}

#[derive(Clone, Copy)]
struct GlobalData {
    zoom: SimCameraZoom,
    offset: SimCameraOffset,
}

#[cfg(test)]
mod tests {
    use core::f64::consts::TAU;

    use crate::{
        components::celestial::Terrain,
        systems::terrain::{create_noisegen, get_terrain_height},
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
        let noisegen = create_noisegen(TEST_TERRAIN);

        let first: Box<[_]> = (0..1024)
            .map(|i| {
                get_terrain_height(
                    TEST_TERRAIN.offset,
                    TEST_TERRAIN.multiplier,
                    &noisegen,
                    i as f64 * 1.0 / 1024.0,
                )
            })
            .collect();

        let noisegen = create_noisegen(TEST_TERRAIN);

        let second: Box<_> = (0..1024)
            .map(|i| {
                get_terrain_height(
                    TEST_TERRAIN.offset,
                    TEST_TERRAIN.multiplier,
                    &noisegen,
                    i as f64 * 1.0 / 1024.0,
                )
            })
            .collect();

        assert_eq!(first, second);
    }

    #[test]
    fn circular() {
        let noisegen = create_noisegen(TEST_TERRAIN);
        let zero = get_terrain_height(TEST_TERRAIN.offset, TEST_TERRAIN.multiplier, &noisegen, 0.0);
        let tau = get_terrain_height(TEST_TERRAIN.offset, TEST_TERRAIN.multiplier, &noisegen, TAU);

        assert_eq!(zero, tau);
    }
}
