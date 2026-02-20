use bevy::{math::DVec2, prelude::*};
use bevy_rapier2d::rapier::prelude::Aabb;

use crate::{
    components::celestial::Terrain,
    consts::terrain::{LOD_DIVISIONS, LOD_VERTS},
};
use core::{
    f64::consts::TAU,
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

    if range_min.is_sign_negative() {
        (range_min + TAU)..=(range_max + TAU)
    } else {
        range_min..=range_max
    }
}

/// Converts a theta range into a index range.
#[must_use]
fn theta_to_idx_range(range: RangeInclusive<f64>, verts: u32) -> Range<u64> {
    let verts_f64 = f64::from(verts);

    #[expect(clippy::cast_possible_truncation)]
    #[expect(
        clippy::cast_sign_loss,
        reason = "get_theta_range output is always positive"
    )]
    let to_index = |theta: f64| {
        let theta_revs = theta / TAU;
        let vert_number = theta_revs * verts_f64;
        vert_number as u64
    };

    let start = *range.start();
    let end = *range.end();

    to_index(start)..to_index(end) + 1
}

/// Wrap the ranges such that things wrap around correctly based on `verts`.
#[must_use]
fn wrap_ranges(ranges: &[Range<u64>], verts: u32) -> Vec<Range<u32>> {
    let verts_u64 = u64::from(verts);
    let mut wrapped_ranges = Vec::with_capacity(ranges.len());

    for range in ranges {
        let start = range.start;
        let end_exclusive = range.end;

        let new_start = start % verts_u64;
        let diff = start - new_start;
        let new_end_excl = end_exclusive - diff;

        #[expect(clippy::cast_possible_truncation)]
        let new_start = new_start as u32;

        if new_end_excl > verts_u64 {
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

/// Merges ranges by combining them if possible.
#[must_use]
fn merge_ranges(mut ranges: Vec<Range<u32>>) -> Vec<Range<u32>> {
    ranges.sort_unstable_by_key(|range| range.start);

    let mut merged = Vec::with_capacity(ranges.len());

    let mut ranges_iter = ranges.into_iter();
    let Some(mut current_range) = ranges_iter.next() else {
        return merged;
    };

    for range in ranges_iter {
        if range.start <= current_range.end {
            current_range.end = current_range.end.max(range.end);
        } else {
            merged.push(current_range);
            current_range = range;
        }
    }

    merged.push(current_range);

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;
    use core::f64::consts::PI;

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

    #[test]
    fn test_wrap_ranges() {
        #[expect(clippy::single_range_in_vec_init)]
        let test_cases = [
            // 1. Simple case, no wrap
            (vec![0..3, 5..7], 10, vec![0..3, 5..7]),
            // 2. Single range wraps around verts
            (vec![8..12], 10, vec![8..10, 0..2]),
            // 3. Range exactly equal to verts (full wrap)
            (vec![0..10], 10, vec![0..10]),
            // 4. Multiple ranges, mixed wrap
            (vec![3..5, 9..14], 10, vec![3..5, 9..10, 0..4]),
            // 5. Range starting beyond verts
            (vec![12..15], 10, vec![2..5]),
            // 6. Zero-length ranges
            (vec![0..0, 10..10], 10, vec![0..0, 0..0]),
        ];

        for (input, verts, expected) in test_cases {
            let output = wrap_ranges(&input, verts);

            assert_eq!(output, expected);
        }
    }

    #[test]
    fn test_merge_ranges() {
        #[expect(clippy::single_range_in_vec_init)]
        let test_cases = [
            (vec![], vec![]),
            (vec![0..5], vec![0..5]),
            (vec![0..3, 5..7, 10..12], vec![0..3, 5..7, 10..12]),
            (vec![1..4, 3..6, 5..8], vec![1..8]),
            (vec![0..2, 2..4, 4..6], vec![0..6]),
            (vec![0..2, 1..3, 5..7, 6..9], vec![0..3, 5..9]),
            (vec![1..2, 2..3, 5..5, 7..8], vec![1..3, 5..5, 7..8]),
        ];

        for (input, expected) in test_cases {
            let output = merge_ranges(input);

            assert_eq!(output, expected);
        }
    }

    #[test]
    fn test_theta_range() {
        const ANGLE_ITERS: u32 = 8192;
        const SIZE_ITERS: u32 = 32;
        const CEL_SIZE_ITERS: u32 = 16;
        const CEL_ROT_ITERS: u32 = 16;

        const _TOTAL_ITERS: u32 = {
            const ITERS: u32 = ANGLE_ITERS * SIZE_ITERS * CEL_SIZE_ITERS * CEL_ROT_ITERS;
            assert!(
                ITERS < 100_000_000,
                "too many iters, you're killing your pc vro ;-;"
            );
            ITERS
        };

        for i in 1..=CEL_SIZE_ITERS {
            let cel_radius = f64::from(i) * 1000.0;
            let terrain = Terrain {
                offset: cel_radius,
                ..Default::default()
            };

            for i in 0..CEL_ROT_ITERS {
                let cel_rot = f64::from(i) * TAU / f64::from(CEL_ROT_ITERS);

                for i in 0..ANGLE_ITERS {
                    let angle = f64::from(i) * TAU / f64::from(ANGLE_ITERS);
                    let vessel_rel_pos = DVec2::from_angle(angle) * cel_radius;

                    let surf_angle = angle - cel_rot;

                    for i in 1..=SIZE_ITERS {
                        #[expect(clippy::cast_precision_loss)]
                        let size = i as f32 * 10.0;
                        let half_size = size / 2.0;
                        let aabb = Aabb::new(
                            Vec2::splat(-half_size).into(),
                            Vec2::splat(half_size).into(),
                        );

                        let angular_size = f64::from(size) / cel_radius;

                        let range = get_theta_range(aabb, vessel_rel_pos, cel_rot, &terrain);

                        assert!(range.start() < range.end());
                        assert!(range.start().is_sign_positive());
                        assert!(range.end().is_sign_positive());
                        assert!(*range.start() <= TAU);
                        assert!(*range.end() <= 4.0 * PI);

                        let res_span = range.end() - range.start();

                        assert!(res_span >= angular_size);

                        assert!(range.contains(&surf_angle) || range.contains(&(surf_angle + TAU)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_index_ranges() {
        const ANGLE_ITERS: u16 = 64;

        for level in 0..8 {
            let verts = verts_at_lod_level(level);

            for start in 0..ANGLE_ITERS {
                let start = TAU * f64::from(start) / f64::from(ANGLE_ITERS);

                for end in 0..ANGLE_ITERS {
                    let end = start + TAU * f64::from(end) / f64::from(ANGLE_ITERS);

                    let range = start..=end;
                    let range_span = end - start;

                    let res = theta_to_idx_range(range, verts);

                    assert!(res.end >= res.start);

                    let res_span = res.end - res.start;
                    #[expect(clippy::cast_precision_loss)]
                    let res_span_revs = res_span as f64 / f64::from(verts);
                    let res_span_rads = res_span_revs * TAU;
                    assert!(res_span_rads >= range_span);
                }
            }
        }
    }
}
