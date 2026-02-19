use crate::consts::terrain::{LOD_DIVISIONS, LOD_VERTS};
use core::f64::consts::TAU;

/// An angle restricted to be in the range 0..tau.
///
/// The checks only happen in debug mode and are removed in release mode.
struct Angle(f64);

impl Angle {
    /// # Panics
    /// Panics in debug mode if the angle is
    /// outside of the range [-pi, +pi].
    #[must_use]
    fn new(angle: f64) -> Self {
        debug_assert!((0.0..TAU).contains(&angle));
        Self(angle)
    }
}

#[must_use]
pub const fn verts_at_lod_level(level: u8) -> u32 {
    // compiler explorer asm output showed that the compiler
    // wasn't able to optimize power-of-two powering as best it can
    // therefore this logic is for extra optimization
    if const { LOD_DIVISIONS.is_power_of_two() } {
        const ILOG2: u32 = LOD_DIVISIONS.ilog2();

        if const { ILOG2.is_power_of_two() } {
            const LOGLOG: u32 = ILOG2.ilog2();

            LOD_VERTS << (level << LOGLOG)
        } else {
            LOD_VERTS << (level as u32 * ILOG2)
        }
    } else {
        LOD_VERTS * LOD_DIVISIONS.pow(level as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verts_at_lod_level() {
        let expected = [
            LOD_VERTS,
            LOD_VERTS * LOD_DIVISIONS,
            LOD_VERTS * LOD_DIVISIONS.pow(2),
            LOD_VERTS * LOD_DIVISIONS.pow(3),
            LOD_VERTS * LOD_DIVISIONS.pow(4),
            LOD_VERTS * LOD_DIVISIONS.pow(5),
            LOD_VERTS * LOD_DIVISIONS.pow(6),
            LOD_VERTS * LOD_DIVISIONS.pow(7),
            LOD_VERTS * LOD_DIVISIONS.pow(8),
            LOD_VERTS * LOD_DIVISIONS.pow(9),
        ];

        for (level, expected) in expected.into_iter().enumerate() {
            assert_eq!(verts_at_lod_level(level.try_into().unwrap()), expected);
        }
    }
}
