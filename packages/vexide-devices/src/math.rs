//! Math-related Container Types
//!
//! This module re-exports several math-related types from the [`mint`] crate
//! for usage in vexide's device APIs.

use core::{
    f64::{
        self,
        consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI, TAU},
    },
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Range, Sub, SubAssign},
};

#[cfg(not(feature = "std"))]
pub use libm::{floorf, roundf, truncf};
pub use mint::{EulerAngles, Point2, Quaternion, Vector3};

// MARK: libm stubs

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

#[cfg(feature = "std")]
#[inline]
#[allow(
    clippy::missing_const_for_fn,
    reason = "The const-ness of these functions should be consistent
    on std and no-std."
)]
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

/// An unbounded angular position.
///
/// This type stores a unit-agnostic angle (a signed displacement from some
/// rotation representing `Angle::ZERO`).
///
/// This type is used by devices such as [`Motor`], [`RotationSensor`], and
/// [`AdiEncoder`] to store measurements of their rotation as an angle.
///
/// [`RotationSensor`]: crate::smart::rotation::RotationSensor
/// [`Motor`]: crate::smart::motor::Motor
/// [`AdiEncoder`]: crate::adi::encoder::AdiEncoder
///
/// # Non-modularity
///
/// This type is unbounded and is NOT modular 2π. This means that
/// `Angle::from_degrees(0) != Angle::from_degrees(360)`, for instance.
///
/// # Precision
///
/// This type internally stores angles as *radians* inside of an `f64`.
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Angle {
    radians: f64,
}

impl Angle {
    /// Angle representing zero rotation.
    pub const ZERO: Self = Self { radians: 0.0 };

    /// Angle representing a sixteenth turn around a full circle.
    pub const SIXTEENTH_TURN: Self = Self { radians: FRAC_PI_8 };

    /// Angle representing an eighth turn around a full circle.
    pub const EIGHTH_TURN: Self = Self { radians: FRAC_PI_4 };

    /// Angle representing a quarter turn around a full circle.
    pub const QUARTER_TURN: Self = Self { radians: FRAC_PI_2 };

    /// Angle representing a half turn around a full circle.
    pub const HALF_TURN: Self = Self { radians: PI };

    /// Angle representing a full turn around a circle.
    pub const FULL_TURN: Self = Self { radians: TAU };

    // MARK: Angle Conversion

    /// Creates a position from a specified number of degrees.
    #[inline]
    #[must_use]
    pub const fn from_degrees(degrees: f64) -> Self {
        Self {
            radians: degrees.to_radians(),
        }
    }

    /// Creates a position from a specified number of radians.
    #[inline]
    #[must_use]
    pub const fn from_radians(radians: f64) -> Self {
        Self { radians }
    }

    /// Creates a position from a specified number of gradians.
    #[must_use]
    pub const fn from_gradians(gradians: f64) -> Self {
        Self {
            radians: gradians * (PI / 200.0),
        }
    }

    /// Creates a position from a specified number of revolutions (full turns).
    #[inline]
    #[must_use]
    pub const fn from_turns(turns: f64) -> Self {
        Self {
            radians: turns * TAU,
        }
    }

    /// Creates an angle from a custom tick reading with a given ticks-per-revolution value.
    #[inline]
    #[must_use]
    pub(crate) const fn from_ticks(ticks: f64, ticks_per_revolution: u32) -> Self {
        Self::from_turns(ticks / ticks_per_revolution as f64)
    }

    // MARK: Angle Conversion

    /// Returns the number of degrees rotated in this position.
    #[inline]
    #[must_use]
    pub const fn as_degrees(&self) -> f64 {
        self.radians.to_degrees()
    }

    /// Returns the number of radians rotated in this position.
    #[inline]
    #[must_use]
    pub const fn as_radians(&self) -> f64 {
        self.radians
    }

    /// Returns the number of gradians rotated in this position.
    #[inline]
    #[must_use]
    pub const fn as_gradians(&self) -> f64 {
        self.radians * (200.0 / PI)
    }

    /// Returns the number of revolutions (full turns) rotated in this position.
    #[inline]
    #[must_use]
    pub const fn as_turns(&self) -> f64 {
        self.radians / TAU
    }

    /// Returns this angle's value scaled to a raw tick value with the provided TPR.
    #[inline]
    #[must_use]
    pub(crate) const fn as_ticks(&self, ticks_per_revolution: u32) -> f64 {
        self.radians / TAU * (ticks_per_revolution as f64)
    }
}

// MARK: Angle Math
impl Angle {
    /// Normalizes the angle to the bounds [min, max).
    #[inline]
    #[must_use]
    pub fn wrapped(&self, range: Range<Angle>) -> Self {
        let start = range.start.radians;
        let end = range.end.radians;

        #[cfg(not(feature = "std"))]
        return Self {
            radians: rem_euclid(self.radians - start, end - start) + start,
        };

        #[cfg(feature = "std")]
        Self {
            radians: (self.radians - start).rem_euclid(end - start) + start,
        }
    }

    /// Computes the arcsine of a number. Return value is in the range
    /// [-pi/2, pi/2] or NaN if the angle is outside the range [-1, 1].
    #[inline]
    #[must_use]
    pub fn asin(y: f64) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            radians: libm::asin(y),
        };

        #[cfg(feature = "std")]
        Self { radians: y.asin() }
    }

    /// Computes the arccosine of a number. Return value is in the range
    /// [0, pi] or NaN if the angle is outside the range [-1, 1].
    #[inline]
    #[must_use]
    pub fn acos(x: f64) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            radians: libm::acos(x),
        };

        #[cfg(feature = "std")]
        Self { radians: x.acos() }
    }

    /// Computes the arctangent of an angle. Return value is in radians in the
    /// range [-pi/2, pi/2];
    #[inline]
    #[must_use]
    pub fn atan(tan: f64) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            radians: libm::atan(tan),
        };

        #[cfg(feature = "std")]
        Self {
            radians: tan.atan(),
        }
    }

    /// Computes the four quadrant arctangent angle of `y` and `x`.
    #[inline]
    #[must_use]
    pub fn atan2(y: f64, x: f64) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            radians: libm::atan2(y, x),
        };

        #[cfg(feature = "std")]
        Self {
            radians: y.atan2(x),
        }
    }

    /// Computes the absolute value of `self`.
    #[allow(clippy::missing_const_for_fn)]
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn abs(self) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            radians: libm::fabs(self.radians),
        };

        #[cfg(feature = "std")]
        Self {
            radians: self.radians.abs(),
        }
    }

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// - NaN if the number is NaN
    #[allow(clippy::missing_const_for_fn)]
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn signum(self) -> f64 {
        #[cfg(not(feature = "std"))]
        return if self.radians.is_nan() {
            f64::NAN
        } else {
            libm::copysign(1.0, self.radians)
        };

        #[cfg(feature = "std")]
        self.radians.signum()
    }

    /// Returns an angle composed of the magnitude of `self` and the sign of
    /// `sign`.
    ///
    /// Equal to `self` if the sign of `self` and `sign` are the same, otherwise equal to `-self`.
    /// If `self` is a NaN, then a NaN with the same payload as `self` and the sign bit of `sign` is
    /// returned.
    ///
    /// If `sign` is a NaN, then this operation will still carry over its sign into the result. Note
    /// that IEEE 754 doesn't assign any meaning to the sign bit in case of a NaN, and as Rust
    /// doesn't guarantee that the bit pattern of NaNs are conserved over arithmetic operations, the
    /// result of `copysign` with `sign` being a NaN might produce an unexpected or non-portable
    /// result. See the [specification of NaN bit patterns](primitive@f32#nan-bit-patterns) for more
    /// info.
    #[allow(clippy::missing_const_for_fn)]
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn copysign(self, sign: Self) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            radians: libm::copysign(self.radians, sign.radians),
        };

        #[cfg(feature = "std")]
        Self {
            radians: self.radians.copysign(sign.radians),
        }
    }

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error, yielding a more accurate result than an unfused multiply-add.
    ///
    /// Using `mul_add` *may* be more performant than an unfused multiply-add if
    /// the target architecture has a dedicated `fma` CPU instruction. However,
    /// this is not always true, and will be heavily dependant on designing
    /// algorithms with specific target hardware in mind.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn mul_add(self, a: f64, b: Self) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            radians: libm::fma(self.radians, a, b.radians),
        };

        #[cfg(feature = "std")]
        Self {
            radians: self.radians.mul_add(a, b.radians),
        }
    }

    /// The positive difference of two numbers.
    ///
    /// * If `self <= other`: `0.0`
    /// * Else: `self - other`
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn abs_sub(self, other: Self) -> Self {
        #[cfg(not(feature = "std"))]
        return Self {
            radians: libm::fdim(self.radians, other.radians),
        };

        #[cfg(feature = "std")]
        #[allow(deprecated)]
        Self {
            radians: self.radians.abs_sub(other.radians),
        }
    }

    /// Computes the sine of an angle.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn sin(self) -> f64 {
        #[cfg(not(feature = "std"))]
        return libm::sin(self.radians);

        #[cfg(feature = "std")]
        self.radians.sin()
    }

    /// Computes the cosine of an angle.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn cos(self) -> f64 {
        #[cfg(not(feature = "std"))]
        return libm::cos(self.radians);

        #[cfg(feature = "std")]
        self.radians.cos()
    }

    /// Computes the tangent of an angle.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn tan(self) -> f64 {
        #[cfg(not(feature = "std"))]
        return libm::tan(self.radians);

        #[cfg(feature = "std")]
        self.radians.tan()
    }

    /// Simultaneously computes the sine and cosine of the number, `x`. Returns
    /// `(sin(x), cos(x))`.
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn sin_cos(self) -> (f64, f64) {
        #[cfg(not(feature = "std"))]
        return (self.sin(), self.cos());

        #[cfg(feature = "std")]
        self.radians.sin_cos()
    }
}

// MARK: Angle Operators

impl Add<Angle> for Angle {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            radians: self.radians + rhs.radians,
        }
    }
}

impl Sub<Angle> for Angle {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            radians: self.radians - rhs.radians,
        }
    }
}

impl Mul<f64> for Angle {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            radians: self.radians * rhs,
        }
    }
}

impl Div<f64> for Angle {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            radians: self.radians / rhs,
        }
    }
}

impl AddAssign<Angle> for Angle {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.radians += rhs.radians;
    }
}

impl SubAssign<Angle> for Angle {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.radians -= rhs.radians;
    }
}

impl MulAssign<f64> for Angle {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.radians *= rhs;
    }
}

impl DivAssign<f64> for Angle {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.radians /= rhs;
    }
}

impl Neg for Angle {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            radians: -self.radians,
        }
    }
}

// MARK: Direction

/// A rotational direction.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    /// Rotates in the forward direction.
    Forward,

    /// Rotates in the reverse direction.
    Reverse,
}

impl Direction {
    /// Returns `true` if the direction is [`Forward`](Direction::Forward).
    #[must_use]
    pub const fn is_forward(&self) -> bool {
        match self {
            Self::Forward => true,
            Self::Reverse => false,
        }
    }

    /// Returns `true` if the direction is [`Reverse`](Direction::Reverse).
    #[must_use]
    pub const fn is_reverse(&self) -> bool {
        match self {
            Self::Forward => false,
            Self::Reverse => true,
        }
    }
}

impl core::ops::Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

// MARK: Tests

#[cfg(test)]
mod test {
    use core::f64::consts::FRAC_PI_2;

    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn zero_is_actually_zero() {
        assert!(approx_eq(Angle::ZERO.as_radians(), 0.0));
        assert!(approx_eq(Angle::ZERO.as_degrees(), 0.0));
        assert!(approx_eq(Angle::ZERO.as_turns(), 0.0));
        assert_eq!(Angle::ZERO + Angle::ZERO, Angle::ZERO);
    }

    #[test]
    fn from_units() {
        // degrees
        let pos = Angle::from_degrees(180.0);
        assert!(approx_eq(pos.as_radians(), PI));
        assert!(approx_eq(pos.as_degrees(), 180.0));

        // radians
        let pos = Angle::from_radians(PI);
        assert!(approx_eq(pos.as_degrees(), 180.0));

        // gradians
        let pos = Angle::from_gradians(200.0);
        assert!(approx_eq(pos.as_radians(), PI));

        // turns
        let pos = Angle::from_turns(0.5);
        assert!(approx_eq(pos.as_radians(), PI));
        assert!(approx_eq(pos.as_degrees(), 180.0));

        // ticks
        let pos = Angle::from_ticks(18000.0, 36000);
        assert!(approx_eq(pos.as_turns(), 0.5));
        assert!(approx_eq(pos.as_degrees(), 180.0));
    }

    #[test]
    fn as_ticks() {
        let pos = Angle::from_turns(1.0);
        assert!(approx_eq(pos.as_ticks(36000), 36000.0));
        assert!(approx_eq(pos.as_ticks(72000), 72000.0));
    }

    #[test]
    fn as_units() {
        let pos = Angle::from_degrees(90.0);
        assert!(approx_eq(pos.as_turns(), 0.25));
        assert!(approx_eq(pos.as_gradians(), 100.0));
        assert!(approx_eq(pos.as_degrees(), 90.0));
        assert!(approx_eq(pos.as_radians(), FRAC_PI_2));
        assert!(approx_eq(pos.as_ticks(360), 90.0));
    }

    #[test]
    fn add_subtract() {
        let a = Angle::from_degrees(90.0);
        let b = Angle::from_degrees(45.0);
        let sum = a + b;
        let diff = a - b;
        assert!(approx_eq(sum.as_degrees(), 135.0));
        assert!(approx_eq(diff.as_degrees(), 45.0));

        let mut p = Angle::from_degrees(60.0);
        p += Angle::from_degrees(30.0);
        assert!(approx_eq(p.as_degrees(), 90.0));
        p -= Angle::from_degrees(45.0);
        assert!(approx_eq(p.as_degrees(), 45.0));
    }

    #[test]
    fn multiply_div_scalar() {
        let p = Angle::from_degrees(90.0);
        let doubled = p * 2.0;
        let halved = p / 2.0;
        assert!(approx_eq(doubled.as_degrees(), 180.0));
        assert!(approx_eq(halved.as_degrees(), 45.0));

        let mut p = Angle::from_degrees(30.0);
        p *= 3.0;
        assert!(approx_eq(p.as_degrees(), 90.0));
        p /= 3.0;
        assert!(approx_eq(p.as_degrees(), 30.0));
    }

    #[test]
    fn negate() {
        let p = Angle::from_degrees(90.0);
        let neg = -p;
        assert!(approx_eq(neg.as_degrees(), -90.0));
    }

    #[test]
    fn non_modular() {
        let a = Angle::from_degrees(0.0);
        let b = Angle::from_degrees(360.0);
        assert_ne!(a, b);
        assert!(approx_eq(b.as_turns(), 1.0));
    }
}
