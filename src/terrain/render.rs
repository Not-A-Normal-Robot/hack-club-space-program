use crate::{
    components::{camera::SimCameraZoom, frames::RootSpacePosition},
    terrain::{TerrainGen, TerrainPoint},
};
use bevy::{mesh::Indices, prelude::*};
use core::{f64::consts::TAU, num::NonZeroU8};

// Math based off a sketch:
// https://www.desmos.com/calculator/sgyaomwmk6

/// The amount of vertices to use for the extremely-zoomed-out mesh.
///
/// Is at most [`LOD_VERTS`].
pub const MIN_LOD_VERTS: u32 = 8;

/// How many vertices for each LoD level.
///
/// Is a multiple of [`LOD_DIVISIONS`] as well as [`MIN_LOD_VERTS`].
pub const LOD_VERTS: u32 = 8;

/// How much smaller the next LoD level is compared to the previous one.
/// (Level 0 = full revolution)
pub const LOD_DIVISIONS: u32 = 8;

/// The length that a finer division covers in terms of the
/// coarser division's verts.
pub const LOD_VERTS_PER_DIVISION: u32 = LOD_VERTS / LOD_DIVISIONS;

const _LOD_ASSERTIONS: () = {
    assert!(MIN_LOD_VERTS <= LOD_VERTS);
    assert!(LOD_VERTS.is_multiple_of(LOD_DIVISIONS));
    assert!(LOD_VERTS.is_multiple_of(MIN_LOD_VERTS));
};

/// Vertex and index buffers for terrain.
///
/// Note: This assumes the [`PrimitiveTopology`][bevy::mesh::PrimitiveTopology]
/// is [`TriangleList`][bevy::mesh::PrimitiveTopology::TriangleList]
pub struct Buffers {
    pub vertices: Vec<Vec3>,
    pub indices: Indices,
}

impl Buffers {
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Indices::U16(vec![]),
        }
    }
}

impl TerrainGen {
    /// Gets the LoD vector array at a certain LoD level.
    pub fn gen_lod(&self, lod_level: u8, focus: f64) -> [TerrainPoint; LOD_VERTS as usize] {
        // From Desmos graph:
        // point((tau / verts) (i ⋅ iter_scale + start))

        let start = NonZeroU8::new(lod_level)
            .map(|l| lod_level_start(l, focus))
            .unwrap_or_default();
        let iter_scale = (LOD_DIVISIONS as f64).powi(-(lod_level as i32));

        core::array::from_fn(|i| {
            self.get_terrain_vector(
                const { TAU / LOD_VERTS as f64 } * (i as f64 * iter_scale + start),
            )
        })
    }
}

/// Gets the starting theta for a given LoD level and focus theta.
///
/// This is in revolutions. To get the theta in radians, multiply this by tau.
pub fn lod_level_start(lod_level: NonZeroU8, focus: f64) -> f64 {
    // From Desmos graph:
    // start = divisions^(1 - level) ⋅
    //  round( (verts / 2pi) ⋅ divisions^(level - 1) ⋅ focus - verts/(2 ⋅ divisions))
    //
    // → coeff = divisions^(1 - level)
    // frac = (verts / 2pi) ⋅ divisions^(level - 1) ⋅ focus
    // frac_offset = -verts / (2 ⋅ divisions)
    // start = coeff round(frac + frac_offset)

    const FRAC_OFFSET: f64 = LOD_VERTS as f64 / (-2.0 * LOD_DIVISIONS as f64);

    let coeff = (LOD_DIVISIONS as f64).powi(1 - lod_level.get() as i32);
    let frac = const { LOD_VERTS as f64 / TAU }
        * LOD_DIVISIONS.pow(lod_level.get().wrapping_sub(1) as u32) as f64
        * focus;

    coeff * (frac + FRAC_OFFSET).round()
}

/// Gets the starting vertex index for a given LoD level relative to the previous LoD level.
///
/// This is for the part where the LoDs get stitched into a mesh.
/// This helps figure out where the stitches should be.
pub fn lod_level_index(lod_level: NonZeroU8, focus: f64) -> usize {
    // From Desmos graph:
    // indices = ((cur_start - prev_start) / prev_iter_scale).rem_euclid(verts)
    // prev_iter_scale = divisions^(1-o)

    let prev_iter_scale = (LOD_DIVISIONS as f64).powi(1 - (lod_level.get() as i32));
    let cur_start = lod_level_start(lod_level, focus);
    let prev_start = NonZeroU8::new(lod_level.get() - 1)
        .map(|level| lod_level_start(level, focus))
        .unwrap_or_default();

    ((cur_start - prev_start) / prev_iter_scale).rem_euclid(LOD_VERTS as f64) as usize
}

/// Finds a maximum reasonable LoD level based on certain parameters.
pub fn get_lod_level_cap(_cel_radius: f64, _zoom: SimCameraZoom, _distance_sq: f64) -> Option<u8> {
    // TODO: lod level cap would be a small opt, low prio
    Some(u8::MAX)
}

/// Gets the angle of focus on the celestial body given the camera's position.
///
/// Assumes the cel_rotation is a pure rotation around the Z-axis.
pub fn get_focus(
    cel_position: RootSpacePosition,
    cel_rotation: f64,
    cam_pos: RootSpacePosition,
) -> f64 {
    let rel_pos = cam_pos.0 - cel_position.0;
    let rel_angle = rel_pos.to_angle();
    let angle = rel_angle - cel_rotation;
    angle.rem_euclid(TAU)
}

#[cfg(test)]
mod tests {
    use bevy::math::DVec2;

    use crate::components::{celestial::Terrain, terrain::LodVectors};

    use super::*;
    use core::{
        f64::consts::{PI, TAU},
        num::NonZeroU8,
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

        let diff = zero.0 - tau.0;
        eprintln!("{diff}");

        assert!(diff.length() < 5e-9);
    }

    #[test]
    fn lod_ranges() {
        fn get_range(vecs: impl IntoIterator<Item = TerrainPoint>) -> f64 {
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
            let range = get_range(terrain_gen.gen_lod(level, 0.0));
            let expected_range = FULL_RANGE * (LOD_DIVISIONS as f64).powi(-(level as i32));
            assert!(
                (range - expected_range).abs() < TOLERANCE,
                "Expected range of {expected_range}, got {range}"
            );
        }
    }

    #[test]
    fn lod_index() {
        const FOCUS: f64 = 1.0;

        let indices = (1..=TEST_TERRAIN.subdivs)
            .map(|level| {
                let level = NonZeroU8::new(level).unwrap();
                lod_level_index(level, FOCUS)
            })
            .collect::<Box<_>>();

        assert_eq!(&*indices, [12, 59, 56, 54, 59, 56, 58, 54].as_slice());
    }

    #[test]
    #[ignore = "mostly for debugging"]
    fn print_results() {
        const FOCUS: f64 = PI;

        println!("=== LoD Vertices ===");
        println!("lod,x,y");
        let terrain_gen = TerrainGen::new(TEST_TERRAIN);
        let vecs = LodVectors::new_full(&terrain_gen, TEST_TERRAIN.subdivs, FOCUS);
        vecs.iter().enumerate().for_each(|(lod_level, vecs)| {
            vecs.iter().for_each(|vec| {
                println!("{lod_level},{},{}", vec.0.x, vec.0.y);
            })
        });

        println!("=== Indices ===");
        println!("lod,num");
        for level in 1..=TEST_TERRAIN.subdivs {
            let level = NonZeroU8::new(level).unwrap();
            let index = lod_level_index(level, FOCUS);

            println!("{level},{index}");
        }

        println!("=== Vert Buffer ===");
        println!("x,y,z");
        let buffers = vecs.create_buffers(FOCUS, TEST_TERRAIN.subdivs.into(), DVec2::ZERO);

        for Vec3 { x, y, z } in buffers.vertices.iter() {
            println!("{x},{y},{z}");
        }
    }
}
