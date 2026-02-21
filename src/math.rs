use bevy::math::Quat;

/// Gets the rotation of the quaternion, assuming the
/// quaternion stays in the 2D XY plane.
#[must_use]
pub fn quat_to_rot(quat: Quat) -> f64 {
    2.0 * f64::from(quat.z).atan2(f64::from(quat.w))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{Quat, Vec3};
    use core::f64::consts::TAU;

    #[test]
    #[expect(clippy::cast_precision_loss)]
    #[expect(clippy::cast_possible_truncation)]
    fn test_quat_to_rot() {
        const ITERS: usize = 1024;

        for i in 0..ITERS {
            let angle = 2.0 * TAU * i as f64 / ITERS as f64;

            let quat = Quat::from_axis_angle(Vec3::Z, angle as f32);

            let (mut z_axis, mut real_angle) = quat.to_axis_angle();

            if z_axis.dot(Vec3::Z) < 0.0 {
                (real_angle, z_axis) = (-real_angle, -z_axis);
            }

            let (real_angle, z_axis) = (real_angle, z_axis);

            let calculated_angle = quat_to_rot(quat);

            if real_angle > 0.01 {
                assert!(
                    (z_axis - Vec3::Z).length() < 1e-5,
                    "{z_axis} {real_angle} isn't close to Z axis {angle}"
                );
            }
            assert!(
                (f64::from(real_angle).rem_euclid(TAU) - angle.rem_euclid(TAU)).abs() < 1e-5,
                "{real_angle} isn't near {angle}"
            );
            assert!(
                ((calculated_angle).rem_euclid(TAU) - angle.rem_euclid(TAU)).abs() < 1e-5,
                "{calculated_angle} isn't near {angle}"
            );
        }
    }
}
