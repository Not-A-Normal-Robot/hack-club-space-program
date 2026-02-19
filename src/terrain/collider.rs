use bevy_rapier2d::rapier::prelude::Aabb;

use crate::{
    components::celestial::Terrain,
    consts::terrain::{LOD_DIVISIONS, LOD_VERTS},
};
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

/// `vessel_distance` is distance between vessel and celestial
/// body center
#[must_use]
pub fn is_vessel_within_terrain_altitude(
    aabb: Aabb,
    vessel_distance: f64,
    terrain: &Terrain,
) -> bool {
    let (terrain_min, terrain_max) = (
        terrain.offset - terrain.multiplier,
        terrain.offset + terrain.multiplier,
    );

    let vessel_length = aabb.maxs.x - aabb.mins.x;
    let vessel_height = aabb.maxs.y - aabb.mins.y;

    let vessel_size = f64::from(vessel_length.max(vessel_height));

    let (vessel_min, vessel_max) = (vessel_distance - vessel_size, vessel_distance + vessel_size);

    vessel_min < terrain_max && vessel_max > terrain_min
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

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

    fn create_terrain(height: f64) -> Terrain {
        Terrain {
            multiplier: height * 0.1,
            offset: height,
            ..Default::default()
        }
    }

    #[test]
    fn test_terrain_range_check() {
        for i in 50..100 {
            let height = f64::from(i) * 10000.0;
            let terrain = create_terrain(height);

            let min_terrain = terrain.offset - terrain.multiplier;
            let max_terrain = terrain.offset + terrain.multiplier;

            eprintln!("### {i} — {min_terrain}..{max_terrain}");

            for size in 1..10 {
                #[allow(clippy::cast_precision_loss)]
                let half_size = size as f32 * 10.0;
                let aabb = Aabb::new(
                    Vec2::splat(-half_size).into(),
                    Vec2::splat(half_size).into(),
                );
                let size = f64::from(half_size) * 2.0;

                eprintln!("## {size}");

                // Test way too close
                assert!(!is_vessel_within_terrain_altitude(
                    aabb,
                    min_terrain * 0.5,
                    &terrain
                ));

                // Test barely too close
                assert!(!is_vessel_within_terrain_altitude(
                    aabb,
                    min_terrain - size * 1.1,
                    &terrain
                ));

                // Test clipping the minimum
                assert!(is_vessel_within_terrain_altitude(
                    aabb,
                    min_terrain - size * 0.9,
                    &terrain
                ));

                // Test in the middle
                assert!(is_vessel_within_terrain_altitude(aabb, height, &terrain));

                // Test clipping the maximum
                assert!(is_vessel_within_terrain_altitude(
                    aabb,
                    max_terrain + size * 0.9,
                    &terrain
                ));

                // Test barely too far
                assert!(!is_vessel_within_terrain_altitude(
                    aabb,
                    max_terrain + size * 1.1,
                    &terrain
                ));

                // Test way too far
                assert!(!is_vessel_within_terrain_altitude(
                    aabb,
                    max_terrain * 1.5,
                    &terrain
                ));
            }
        }
    }
}
