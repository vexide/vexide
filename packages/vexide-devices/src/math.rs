//! Math-related Container Types
//!
//! This module re-exports several math-related types from the [`mint`] crate
//! for usage in vexide's device APIs.

pub use mint::{EulerAngles, Point2, Quaternion, Vector3};

/// Internal stub for f64::rem_euclid used by IMU and GPS.
///
/// TODO: Remove once core_float_math is stablized.
#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn rem_euclid(x: f64, rhs: f64) -> f64 {
    let r = x % rhs;
    if r < 0.0 {
        r + libm::fabs(rhs)
    } else {
        r
    }
}

#[cfg(not(feature = "std"))]
pub use libm::{floorf, roundf, truncf};

#[cfg(feature = "std")]
#[inline]
pub(crate) fn rem_euclid(x: f64, rhs: f64) -> f64 {
    x.rem_euclid(rhs)
}

#[cfg(feature = "std")]
#[inline]
#[allow(clippy::missing_const_for_fn)]
pub(crate) fn roundf(x: f32) -> f32 {
    x.round()
}

#[cfg(feature = "std")]
#[inline]
#[allow(clippy::missing_const_for_fn)]
pub(crate) fn floorf(x: f32) -> f32 {
    x.floor()
}

#[cfg(feature = "std")]
#[inline]
#[allow(clippy::missing_const_for_fn)]
pub(crate) fn truncf(x: f32) -> f32 {
    x.trunc()
}
