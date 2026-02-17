use crate::terrain::{
    TerrainGen, TerrainPoint,
    render::{
        Buffers, LOD_DIVISIONS, LOD_VERTS, LOD_VERTS_PER_DIVISION, MIN_LOD_VERTS, lod_level_index,
        lod_level_start,
    },
};
use bevy::{math::DVec2, mesh::Indices, prelude::*};
use core::{num::NonZeroU8, ops::Deref};

/// The previous focus angle.
#[derive(Clone, Copy, Component)]
pub struct PrevFocus(pub f64);

/// A list of LoD offsets.
#[derive(Clone, Component, Debug, PartialEq)]
pub struct LodVectors(
    /// Invariant: this vector must always have a length of at least 1
    Vec<[TerrainPoint; LOD_VERTS as usize]>,
);

impl LodVectors {
    /// Generate a lowest-quality LoD vector list.
    pub fn new(terrain_gen: &TerrainGen) -> Self {
        Self(vec![terrain_gen.gen_lod(0, 0.0)])
    }

    /// Generate a fully-realized LoD vector list.
    pub fn new_full(terrain_gen: &TerrainGen, ending_level: u8, focus: f64) -> Self {
        let mut this = Self::new(terrain_gen);
        if let Some(ending_level) = NonZeroU8::new(ending_level) {
            this.update_lods(terrain_gen, ending_level, f64::NAN, focus);
        }
        this
    }

    /// Updates the LoD vectors.
    pub fn update_lods(
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

    /// The index buffer for minimal quality rendering (far away)
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
    fn create_min_buffer(&self, shift: DVec2) -> Buffers {
        // SAFETY: LoD 0 is always loaded, never mutated, and always created when
        // using the constructors.
        let vecs = unsafe { self.0.first().unwrap_unchecked() };

        let vertices = (0..MIN_LOD_VERTS)
            .map(|i| vecs[(i * (LOD_VERTS / MIN_LOD_VERTS)) as usize].shift_downcast(shift))
            .collect();

        Buffers {
            vertices,
            indices: Indices::U16(Vec::from(const { Self::create_min_index_buffer() })),
        }
    }

    /// Creates a vertex and index buffer from the vectors for just the zeroth LoD.
    ///
    /// This doesn't need updating the LoD vectors as the zeroth LoD never changes.
    fn create_zeroth_buffer(&self, shift: DVec2) -> Buffers {
        let Some(vecs) = self.0.first() else {
            return Buffers::empty();
        };

        Buffers {
            vertices: vecs.iter().map(|v| v.shift_downcast(shift)).collect(),
            indices: Indices::U16(Vec::from(const { Self::create_zeroth_index_buffer() })),
        }
    }

    /// Creates a vertex buffer from the vectors.
    ///
    /// # Unchecked Operation
    /// This function assumes you have updated the LoD vectors.
    #[must_use]
    fn create_unshifted_vertex_buffer(
        &self,
        focus: f64,
        max_level: NonZeroU8,
    ) -> Box<[TerrainPoint]> {
        let max_level = max_level.get().min((self.0.len() - 1) as u8);

        // +1 vert in the center of the body
        let vertex_count = LOD_VERTS * max_level as u32 + 1;
        let mut vertices: Vec<TerrainPoint> = Vec::with_capacity(vertex_count as usize);

        vertices.push(TerrainPoint(DVec2::ZERO));

        // Zeroth LoD cutoffs need special care since it may pass through index 0
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
            (lod_1_start_idx + LOD_VERTS_PER_DIVISION as usize + 1) % LOD_VERTS as usize,
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

    /// `len` is the length of the vertex buffer and must be >= 3
    fn create_index_buffer_inner_16(len: u16) -> Indices {
        debug_assert!(len >= 3);

        let mut buf = Box::new_uninit_slice((len as usize - 1) * 3);

        for i in 1..len - 1 {
            let offset = 3 * (i - 1) as usize;

            unsafe {
                buf.get_unchecked_mut(offset).write(0);
                buf.get_unchecked_mut(offset + 1).write(i);
                buf.get_unchecked_mut(offset + 2).write(i + 1);
            }
        }

        let last = 3 * (len - 2) as usize;

        unsafe {
            buf.get_unchecked_mut(last).write(0);
            buf.get_unchecked_mut(last + 1).write(len - 1);
            buf.get_unchecked_mut(last + 2).write(1);
        }

        let buf = unsafe { buf.assume_init() };

        let len = buf.len();
        let ptr = Box::into_raw(buf) as *mut u16;

        let vec = unsafe { Vec::from_raw_parts(ptr, len, len) };

        Indices::U16(vec)
    }

    /// `len` is the length of the vertex buffer and must be >= 3
    #[cold]
    fn create_index_buffer_inner_32(len: u32) -> Indices {
        let mut buf = Box::new_uninit_slice((len as usize - 1) * 3);

        for i in 1..len - 1 {
            let offset = 3 * (i - 1) as usize;

            unsafe {
                buf.get_unchecked_mut(offset).write(0);
                buf.get_unchecked_mut(offset + 1).write(i);
                buf.get_unchecked_mut(offset + 2).write(i + 1);
            }
        }

        let last = 3 * (len - 2) as usize;

        unsafe {
            buf.get_unchecked_mut(last).write(0);
            buf.get_unchecked_mut(last + 1).write(len - 1);
            buf.get_unchecked_mut(last + 2).write(1);
        }

        let buf = unsafe { buf.assume_init() };

        let len = buf.len();
        let ptr = Box::into_raw(buf) as *mut u32;

        let vec = unsafe { Vec::from_raw_parts(ptr, len, len) };

        Indices::U32(vec)
    }

    /// Create an index buffer from the given vertex buffer.
    ///
    /// # Unchecked Operation
    /// This function assumes you got the `vertex_buffer` argument from
    /// [`Self::create_vertex_buffer`]. If you get it from somewhere else,
    /// you may get an invalid index buffer.
    fn create_index_buffer(vertices: usize) -> Indices {
        if let Ok(len) = vertices.try_into() {
            Self::create_index_buffer_inner_16(len)
        } else {
            Self::create_index_buffer_inner_32(vertices as u32)
        }
    }

    /// Create a vertex and index buffer from the vectors.
    ///
    /// # Unchecked Operation
    /// This function assumes you have updated the LoD vectors.
    fn create_buffers_inner(&self, focus: f64, max_level: NonZeroU8, shift: DVec2) -> Buffers {
        let vertices: Vec<Vec3> = self
            .create_unshifted_vertex_buffer(focus, max_level)
            .into_iter()
            .map(|point| point.shift_downcast(shift))
            .collect();
        let indices = Self::create_index_buffer(vertices.len());

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
    pub fn create_buffers(&self, focus: f64, max_level: Option<u8>, shift: DVec2) -> Buffers {
        match max_level {
            None => self.create_min_buffer(shift),
            Some(0) => self.create_zeroth_buffer(shift),
            Some(max_level) => {
                self.create_buffers_inner(focus, NonZeroU8::new(max_level).unwrap(), shift)
            }
        }
    }
}

impl Deref for LodVectors {
    type Target = Vec<[TerrainPoint; LOD_VERTS as usize]>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::Indices;

    #[test]
    #[ignore = "takes a few dozen secs"]
    fn test_index_buffer() {
        let buf = LodVectors::create_index_buffer_inner_16(7);
        assert_eq!(
            buf,
            Indices::U16(vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1])
        );

        for i in 3..u16::MAX {
            let buf = LodVectors::create_index_buffer_inner_16(i);
            let Indices::U16(buf) = buf else {
                panic!("buf isn't u16")
            };

            for (tri, slice) in buf.chunks(3).enumerate() {
                let tri = tri as u16;
                let [zero, cur, next] = [slice[0], slice[1], slice[2]];
                assert_eq!(zero, 0);
                assert_eq!(cur, tri + 1);

                if tri + 2 == i {
                    assert_eq!(next, 1);
                } else {
                    assert_eq!(next, tri + 2);
                }
            }
        }

        for i in (u16::MAX as u32)..70000 {
            let buf = LodVectors::create_index_buffer_inner_32(i);
            let Indices::U32(buf) = buf else {
                panic!("buf isn't u32")
            };

            for (tri, slice) in buf.chunks(3).enumerate() {
                let tri = tri as u32;
                let [zero, cur, next] = [slice[0], slice[1], slice[2]];
                assert_eq!(zero, 0);
                assert_eq!(cur, tri + 1);

                if tri + 2 == i {
                    assert_eq!(next, 1);
                } else {
                    assert_eq!(next, tri + 2);
                }
            }
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
