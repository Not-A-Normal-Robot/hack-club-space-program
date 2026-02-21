/// The amount of vertices to use for the extremely-zoomed-out mesh.
///
/// Is at most [`LOD_VERTS`].
pub const MIN_LOD_VERTS: u16 = 8;

/// How many vertices for each LoD level.
///
/// Is a multiple of [`LOD_DIVISIONS`] as well as [`MIN_LOD_VERTS`].
pub const LOD_VERTS: u32 = 512;

/// How much smaller the next LoD level is compared to the previous one.
/// (Level 0 = full revolution)
pub const LOD_DIVISIONS: u32 = 4;

/// The length that a finer division covers in terms of the
/// coarser division's verts.
pub const LOD_VERTS_PER_DIVISION: u32 = LOD_VERTS / LOD_DIVISIONS;

const _LOD_ASSERTIONS: () = {
    assert!(MIN_LOD_VERTS as u32 <= LOD_VERTS);
    assert!(LOD_VERTS.is_multiple_of(LOD_DIVISIONS));
    assert!(LOD_VERTS.is_multiple_of(MIN_LOD_VERTS as u32));
    assert!((LOD_VERTS / MIN_LOD_VERTS as u32) < u16::MAX as u32);
    assert!((LOD_VERTS as u128) < isize::MAX as u128);
    assert!(LOD_VERTS < i32::MAX as u32);
    assert!(LOD_VERTS < u16::MAX as u32);
};
