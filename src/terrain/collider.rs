use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::rapier::prelude::Aabb;

use crate::{
    components::celestial::Terrain,
    consts::terrain::{LOD_DIVISIONS, LOD_VERTS},
};
use core::{
    f64::consts::{PI, TAU},
    ops::{Range, RangeInclusive},
};

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

/// Gets a conservative (larger-than-needed) theta range
/// enveloping the given AABB.
///
/// Returns a theta range. The start of the range
/// will always be in the range 0..=tau, and the
/// end of the range will always be in the range
/// 0..=4pi.
#[must_use]
fn get_theta_range(
    aabb: Aabb,
    vessel_rel_pos: DVec2,
    celestial_rotation: f64,
    terrain: &Terrain,
) -> RangeInclusive<f64> {
    let length = aabb.maxs.x - aabb.mins.x;
    let width = aabb.maxs.y - aabb.mins.y;

    let size = f64::from(length.hypot(width));

    let conservative_radius = terrain.offset - terrain.multiplier;

    // rads = tau * size / circumference
    // circumference = tau * radius
    // → rads = size / radius

    let range_length = (size / conservative_radius).clamp(0.0, TAU);
    let range_half_length = range_length / 2.0;
    let range_center =
        (vessel_rel_pos.to_angle() - celestial_rotation.rem_euclid(TAU)).rem_euclid(TAU);

    let range_min = range_center - range_half_length;
    let range_max = range_center + range_half_length;

    let range = if range_min.is_sign_negative() {
        (range_min + TAU)..=(range_max + TAU)
    } else {
        range_min..=range_max
    };

    if cfg!(debug_assertions) {
        assert!(range.start() < range.end());
        assert!(range.start().is_sign_positive());
        assert!(range.end().is_sign_positive());
        assert!(*range.start() <= TAU);
        assert!(*range.end() <= 4.0 * PI);
    }

    range
}

#[must_use]
fn gen_index_ranges(theta_ranges: &[RangeInclusive<f64>], lod_level: u8) -> Vec<Range<u32>> {
    let verts = verts_at_lod_level(lod_level);
    let verts_f64 = f64::from(verts);

    #[expect(clippy::cast_possible_truncation)]
    let to_index = |theta: f64| {
        let theta_revs = theta / TAU;
        let vert_number = theta_revs * verts_f64;
        vert_number as u64
    };

    let pre_modulo_ranges: Box<[_]> = theta_ranges
        .iter()
        .map(|range| {
            let start = *range.start();
            let end = *range.end();

            to_index(start)..=to_index(end)
        })
        .collect();

    let wrapped_ranges = wrap_ranges(&pre_modulo_ranges, verts);
    drop(pre_modulo_ranges);

    simplify_ranges(&wrapped_ranges)
}

/// Wrap the ranges such that things wrap around `verts`.
/// Also makes them no longer inclusive at the end.
fn wrap_ranges(ranges: &[RangeInclusive<u64>], verts: u32) -> Vec<Range<u32>> {
    let verts_u64 = verts as u64;
    let mut wrapped_ranges = Vec::with_capacity(ranges.len());

    for range in ranges {
        let start = *range.start();
        let end_exclusive = *range.end() + 1;

        let new_start = start % verts_u64;
        let diff = start - new_start;
        let new_end_excl = end_exclusive - diff;

        #[expect(clippy::cast_possible_truncation)]
        let new_start = new_start as u32;

        if new_end_excl >= verts as u64 {
            wrapped_ranges.push(new_start..verts);
            let wrapped_end = new_end_excl - verts_u64;
            #[expect(clippy::cast_possible_truncation)]
            wrapped_ranges.push(0..wrapped_end as u32);
        } else {
            #[expect(clippy::cast_possible_truncation)]
            let new_end_excl = new_end_excl as u32;

            wrapped_ranges.push(new_start..new_end_excl);
        }
    }

    wrapped_ranges
}

/// Simplifies ranges by combining them.
fn simplify_ranges(ranges: &[Range<u32>]) -> Vec<Range<u32>> {
    todo!();
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
