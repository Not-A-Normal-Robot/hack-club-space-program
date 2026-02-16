use bevy::math::Quat;

/// Gets the rotation of the quaternion, assuming the
/// quaternion stays in the 2D XY plane.
pub fn quat_to_rot(quat: Quat) -> f64 {
    2.0 * (quat.z as f64).atan2(quat.w as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::{Quat, Vec3};
    use core::f64::consts::TAU;

    #[test]
    fn test_quat_to_rot() {
        const ITERS: usize = 1024;

        for i in 0..ITERS {
            let angle = 2.0 * TAU * i as f64 / ITERS as f64;

            let quat = Quat::from_axis_angle(Vec3::Z, angle as f32);

            let (z_axis, real_angle) = quat.to_axis_angle();

            assert!((z_axis - Vec3::Z).length() < 1e-5);
            assert!((real_angle as f64 - angle).abs() < 1e-5);
            assert!((quat_to_rot(quat) - angle).abs() < 1e-5);
        }
    }
}
