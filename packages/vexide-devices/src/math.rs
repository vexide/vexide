//! Math-related Container Types
//!
//! This module re-exports several math-related types from the [`mint`] crate
//! for usage in vexide's device APIs.

pub use mint::{EulerAngles, Point2, Quaternion, Vector3};

/// Internal stub for f64::rem_euclid used by IMU and GPS.
///
/// TODO: Remove once core_float_math is stablized.
#[inline]
pub(crate) fn rem_euclid(x: f64, rhs: f64) -> f64 {
    let r = x % rhs;
    if r < 0.0 {
        r + libm::fabs(rhs)
    } else {
        r
    }
}
