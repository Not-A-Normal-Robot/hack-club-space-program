use crate::components::{
    camera::{SimCameraOffset, SimCameraZoom},
    celestial::{CelestialBody, Terrain},
    frames::RootSpacePosition,
};
use bevy::{ecs::query::QueryData, math::DVec2, mesh::Indices, prelude::*};
use core::{f64::consts::TAU, num::NonZeroU8};
use fastnoise_lite::{FastNoiseLite, FractalType};

// Math based off a sketch:
// https://www.desmos.com/calculator/sgyaomwmk6

/// The amount of vertices to use for the extremely-zoomed-out mesh.
///
/// Is at most [`LOD_VERTS`].
pub const MIN_LOD_VERTS: u32 = 8;

/// How many vertices for each LoD level.
///
/// Is a multiple of [`LOD_DIVISIONS`] as well as [`MIN_LOD_VERTS`].
pub const LOD_VERTS: u32 = 128;

/// How much smaller the next LoD level is compared to the previous one.
/// (Level 0 = full revolution)
pub const LOD_DIVISIONS: u32 = 8;

/// The length that a finer division covers in terms of the
/// coarser division's verts.
pub const LOD_VERTS_PER_DIVISION: u32 = LOD_VERTS / LOD_DIVISIONS;

const LOD_ASSERTIONS: () = {
    assert!(MIN_LOD_VERTS <= LOD_VERTS);
    assert!(LOD_VERTS.is_multiple_of(LOD_DIVISIONS));
    assert!(LOD_VERTS.is_multiple_of(MIN_LOD_VERTS));
};

/// A vector relative to this object's center.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RelativeVector(DVec2);

/// Vertex and index buffers for terrain.
///
/// Note: This assumes the [`PrimitiveTopology`][bevy::mesh::PrimitiveTopology]
/// is [`TriangleList`][bevy::mesh::PrimitiveTopology::TriangleList]
struct Buffers {
    pub vertices: Box<[RelativeVector]>,
    pub indices: Indices,
}

impl Buffers {
    fn empty() -> Self {
        Self {
            vertices: Box::from([]),
            indices: Indices::U16(vec![]),
        }
    }
}

/// Do a fast, contiguous partial wrapping copy from a source array.
///
/// Amount must be less than M.
fn partial_wrapping_copy<T: Clone, const M: usize>(
    src: &[T; M],
    dest: &mut Vec<T>,
    start: usize,
    amount: usize,
) {
    debug_assert!(amount < M);

    if start + amount >= src.len() {
        dest.extend_from_slice(&src[start..src.len()]);
        dest.extend_from_slice(&src[0..start + amount - src.len()]);
    } else {
        dest.extend_from_slice(&src[start..start + amount]);
    }
}

/// A list of LoD offsets.
#[derive(Clone, Component, Debug, PartialEq)]
struct LodVectors(
    /// Invariant: this vector must always have a length of at least 1
    Vec<[RelativeVector; LOD_VERTS as usize]>,
);

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
        debug_assert!(!self.0.is_empty());

        (1..=ending_level.get()).for_each(|level| {
            self.update_lod(
                terrain_gen,
                NonZeroU8::new(level).unwrap(),
                prev_focus,
                new_focus,
            );
        })
    }

    fn update_lod(
        &mut self,
        terrain_gen: &TerrainGen,
        level: NonZeroU8,
        prev_focus: f64,
        new_focus: f64,
    ) {
        let level = level.get();
        let level_not_loaded = self.0.len() >= level as usize;
        let lod_needs_updating = NonZeroU8::new(level)
            .map(|level| lod_level_start(level, prev_focus) != lod_level_start(level, new_focus))
            .unwrap_or(false);

        if level_not_loaded || lod_needs_updating {
            let vecs = terrain_gen.gen_lod(level, new_focus);

            if level_not_loaded {
                self.0.push(vecs);
            } else {
                *self.0.get_mut(level as usize).unwrap() = vecs;
            }
        }
    }

    /// The index buffer for
    const fn create_min_index_buffer() -> [u16; (MIN_LOD_VERTS as usize - 1) * 3] {
        let mut arr = [0u16; _];

        let mut index = 1usize;

        while index < MIN_LOD_VERTS as usize {
            arr[3 * index - 2] = index as u16;
            arr[3 * index - 1] = match index + 1 {
                val if val == MIN_LOD_VERTS as usize => 1,
                val => val as u16,
            };

            index += 1;
        }

        arr
    }

    const fn create_zeroth_index_buffer() -> [u16; (LOD_VERTS as usize - 1) * 3] {
        let mut arr = [0u16; _];

        let mut index = 1usize;

        while index < LOD_VERTS as usize {
            arr[3 * index - 2] = index as u16;
            arr[3 * index - 1] = match index + 1 {
                val if val == LOD_VERTS as usize => 1,
                val => val as u16,
            };

            index += 1;
        }

        arr
    }

    /// Creates a very minimal vertex and index buffer
    /// for extremely-zoomed-out scenarios.
    fn create_min_buffer(&self) -> Buffers {
        // SAFETY: LoD 0 is always loaded, never mutated, and always created when
        // using the constructors.
        let vecs = unsafe { self.0.first().unwrap_unchecked() };

        let verts: Box<[_]> = (0..MIN_LOD_VERTS)
            .map(|i| vecs[(i * (LOD_VERTS / MIN_LOD_VERTS)) as usize])
            .collect();

        Buffers {
            vertices: verts,
            indices: Indices::U16(Vec::from(const { Self::create_min_index_buffer() })),
        }
    }

    /// Creates a vertex and index buffer from the vectors for just the zeroth LoD.
    ///
    /// This doesn't need updating the LoD vectors as the zeroth LoD never changes.
    fn create_zeroth_buffer(&self) -> Buffers {
        let Some(vecs) = self.0.first() else {
            return Buffers::empty();
        };

        Buffers {
            vertices: Box::from(*vecs),
            indices: Indices::U16(Vec::from(const { Self::create_min_index_buffer() })),
        }
    }

    /// Creates a vertex buffer from the vectors.
    ///
    /// # Unchecked Operation
    /// This function assumes you have updated the LoD vectors.
    fn create_vertex_buffer(&self, focus: f64, max_level: NonZeroU8) -> Box<[RelativeVector]> {
        let max_level = max_level.get().min((self.0.len() - 1) as u8);

        // +1 vert in the center of the body
        let vertex_count = LOD_VERTS * max_level as u32 + 1;
        let mut vertices: Vec<RelativeVector> = Vec::with_capacity(vertex_count as usize);
        let mut indices: Vec<u32> = Vec::with_capacity(3 * (vertex_count as usize - 1));

        vertices.push(RelativeVector(DVec2::ZERO));

        // Zeroeth LoD cutoffs need special care since it may pass through index 0
        // (+x axis)
        // I figured I could just shove all the LoD_0 verts into the beginning of the buffer
        // e.g. verts: 8, subdivs: 1, divs: 4, start idx: 3
        // => L0.4 L0.5 L0.6 L0.7 L0.0 L1.*
        // => L0[(start_idx + 1) mod VERTS], repeated USED_VERTS = VERTS*(DIVS-1)/DIVS-1 times

        let lod_0_verts = unsafe { self.0.first().unwrap_unchecked() };
        const LOD_0_USED_VERTS_COUNT: u32 = LOD_VERTS * (LOD_DIVISIONS - 1) / LOD_DIVISIONS - 1;
        let lod_1_start_idx = lod_level_index(NonZeroU8::MIN, focus);

        partial_wrapping_copy(
            lod_0_verts,
            &mut vertices,
            lod_1_start_idx + 1,
            LOD_0_USED_VERTS_COUNT as usize,
        );

        for level in 1..max_level {
            // SAFETY: We already clamped the max_level at the beginning
            // of the function.
            let verts = unsafe { self.0.get_unchecked(level as usize) };

            const SKIP_VERTS_AMOUNT: usize = (LOD_VERTS / LOD_DIVISIONS + 1) as usize;

            let next_start = lod_level_index(NonZeroU8::new(level + 1).unwrap(), focus);

            vertices.extend_from_slice(&verts[0..next_start]);
            vertices.extend_from_slice(&verts[next_start + SKIP_VERTS_AMOUNT..verts.len()]);
        }

        // SAFETY: We already clamped the max_level at the beginning
        // of the function.
        vertices.extend_from_slice(unsafe { self.0.get_unchecked(max_level as usize) });

        vertices.into()
    }

    /// Create an index buffer from the given vertex buffer.
    ///
    /// # Unchecked Operation
    /// This function assumes you got the `vertex_buffer` argument from
    /// [`Self::create_vertex_buffer`]. If you get it from somewhere else,
    /// you may get an invalid index buffer.
    fn create_index_buffer(_vertex_buffer: &[RelativeVector]) -> Indices {
        todo!("Index buffer");
    }

    /// Create a vertex and index buffer from the vectors.
    ///
    /// # Unchecked Operation
    /// This function assumes you have updated the LoD vectors.
    fn create_full_buffer(&self, focus: f64, max_level: NonZeroU8) -> Buffers {
        let vertices = self.create_vertex_buffer(focus, max_level);
        let indices = Self::create_index_buffer(&vertices);

        Buffers { vertices, indices }
    }

    /// Creates a vertex and index buffer from the vectors.
    ///
    /// A `max_level` value of `None` indicates a very minimal representation,
    /// even more so than a value of `Some(0)`.\
    /// This is reserved for when the camera is zoomed very far out or is very
    /// far away.
    ///
    /// # Unchecked Operation
    /// This function assumes you have updated the LoD vectors.
    fn create_buffers(&self, focus: f64, max_level: Option<u8>) -> Buffers {
        match max_level {
            None => self.create_min_buffer(),
            Some(0) => self.create_zeroth_buffer(),
            Some(max_level) => self.create_full_buffer(focus, NonZeroU8::new(max_level).unwrap()),
        }
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
    fn gen_lod(&self, lod_level: u8, focus: f64) -> [RelativeVector; LOD_VERTS as usize] {
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
fn lod_level_start(lod_level: NonZeroU8, focus: f64) -> f64 {
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
fn lod_level_index(lod_level: NonZeroU8, focus: f64) -> usize {
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

#[cfg(test)]
mod tests {
    use core::{f64::consts::TAU, num::NonZeroU8};

    use crate::{
        components::celestial::Terrain,
        systems::terrain::{
            LOD_DIVISIONS, LOD_VERTS, LodVectors, RelativeVector, TerrainGen, lod_level_index,
            partial_wrapping_copy,
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
    #[ignore]
    fn print_results() {
        const FOCUS: f64 = 1.0;

        println!("=== Vertices ===");
        println!("lod,x,y");
        let terrain_gen = TerrainGen::new(TEST_TERRAIN);
        let vecs = LodVectors::new_full(
            &terrain_gen,
            NonZeroU8::new(TEST_TERRAIN.subdivs).unwrap(),
            FOCUS,
        );
        vecs.0.iter().enumerate().for_each(|(lod_level, vecs)| {
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
    }

    #[test]
    fn test_partial_wrapping_copy() {
        fn slow_pwc<T: Clone, const M: usize>(
            src: &[T; M],
            dest: &mut Vec<T>,
            start: usize,
            amount: usize,
        ) {
            for i in 0..amount {
                let idx = (start + i) % M;
                dest.push(src[idx].clone());
            }
        }

        const ARRAY_SIZE: usize = 512;
        let src = {
            let mut array = [0usize; ARRAY_SIZE];
            array
                .iter_mut()
                .enumerate()
                .for_each(|(idx, entry)| *entry = idx);
            array
        };

        let mut slow_buf = Vec::new();
        let mut fast_buf = Vec::new();

        for start in 0..ARRAY_SIZE {
            for amount in 0..ARRAY_SIZE {
                slow_buf.clear();
                fast_buf.clear();

                partial_wrapping_copy(&src, &mut fast_buf, start, amount);
                slow_pwc(&src, &mut slow_buf, start, amount);

                assert_eq!(
                    slow_buf, fast_buf,
                    "buffer inequality at start={start}, amount={amount}, size={ARRAY_SIZE}"
                );
            }
        }
    }
}
