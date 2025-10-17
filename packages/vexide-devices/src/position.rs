//! Angular Position Type
//!
//! Used by devices such as [`Motor`], [`RotationSensor`], and [`AdiEncoder`]
//! to store measurements of their rotation as an angle.
//!
//! [`AdiEncoder`]: crate::adi::encoder::AdiEncoder
//! [`RotationSensor`]: crate::smart::rotation::RotationSensor
//! [`Motor`]: crate::smart::motor::Motor

use core::{
    f64::{
        self,
        consts::{PI, TAU},
    },
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// Stores an angular position/rotation.that are able
///
/// This type stores a unit-agnostic angular position (a signed displacement
/// from some rotation representing `Position::ZERO`).
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
/// `Position::from_degrees(0) != Position::from_degrees(360)`, for instance.
///
/// # Precision
///
/// This type internally stores angles as *radians* inside of an `f64`. This may
/// be subject to change in the future.
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Position(f64);

impl Position {
    /// Position representing zero rotation.
    pub const ZERO: Self = Self(0.0);

    // MARK: Creation

    /// Creates a position from a custom tick reading with a given ticks-per-revolution value.
    ///
    /// Essentially scales this value to the internal 36000 ticks per revolution.
    #[inline]
    #[must_use]
    pub const fn from_ticks(ticks: f64, ticks_per_revolution: u32) -> Self {
        Self::from_revolutions(ticks / ticks_per_revolution as f64)
    }

    /// Creates a position from a specified number of degrees.
    #[inline]
    #[must_use]
    pub const fn from_degrees(degrees: f64) -> Self {
        Self(degrees.to_radians())
    }

    /// Creates a position from a specified number of radians.
    #[inline]
    #[must_use]
    pub const fn from_radians(radians: f64) -> Self {
        Self(radians)
    }

    /// Creates a position from a specified number of gradians.
    #[must_use]
    pub const fn from_gradians(gradians: f64) -> Self {
        Self(gradians * (PI / 200.0))
    }

    /// Creates a position from a specified number of revolutions (full turns).
    #[inline]
    #[must_use]
    pub const fn from_revolutions(turns: f64) -> Self {
        Self(turns * TAU)
    }

    // MARK: Conversion

    /// Returns this position's value scaled to a raw tick value with the provided TPR.
    #[inline]
    #[must_use]
    pub const fn as_ticks(&self, ticks_per_revolution: u32) -> f64 {
        self.0 / TAU * (ticks_per_revolution as f64)
    }

    /// Returns the number of degrees rotated in this position.
    #[inline]
    #[must_use]
    pub const fn as_degrees(&self) -> f64 {
        self.0.to_degrees()
    }

    /// Returns the number of radians rotated in this position.
    #[inline]
    #[must_use]
    pub const fn as_radians(&self) -> f64 {
        self.0
    }

    /// Returns the number of gradians rotated in this position.
    #[inline]
    #[must_use]
    pub const fn as_gradians(&self) -> f64 {
        self.0 * (200.0 / PI)
    }

    /// Returns the number of revolutions (full turns) rotated in this position.
    #[inline]
    #[must_use]
    pub fn as_revolutions(&self) -> f64 {
        self.0 / TAU
    }
}

// MARK: Operators

impl Add<Position> for Position {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Position> for Position {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<f64> for Position {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<f64> for Position {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl AddAssign<Position> for Position {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign<Position> for Position {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl MulAssign<f64> for Position {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
    }
}

impl DivAssign<f64> for Position {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.0 /= rhs;
    }
}

impl Neg for Position {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

#[cfg(test)]
mod test {
    use core::f64::consts::FRAC_PI_2;

    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn zero_is_actually_zero() {
        assert_eq!(Position::ZERO.as_radians(), 0.0);
        assert_eq!(Position::ZERO.as_degrees(), 0.0);
        assert_eq!(Position::ZERO.as_revolutions(), 0.0);
        assert_eq!(Position::ZERO + Position::ZERO, Position::ZERO);
    }

    #[test]
    fn from_units() {
        // degrees
        let pos = Position::from_degrees(180.0);
        assert!(approx_eq(pos.as_radians(), PI));
        assert!(approx_eq(pos.as_degrees(), 180.0));

        // radians
        let pos = Position::from_radians(PI);
        assert!(approx_eq(pos.as_degrees(), 180.0));

        // gradians
        let pos = Position::from_gradians(200.0);
        assert!(approx_eq(pos.as_radians(), PI));

        // revolutions
        let pos = Position::from_revolutions(0.5);
        assert!(approx_eq(pos.as_radians(), PI));
        assert!(approx_eq(pos.as_degrees(), 180.0));

        // ticks
        let pos = Position::from_ticks(18000.0, 36000);
        assert!(approx_eq(pos.as_revolutions(), 0.5));
        assert!(approx_eq(pos.as_degrees(), 180.0));
    }

    #[test]
    fn as_ticks() {
        let pos = Position::from_revolutions(1.0);
        assert!(approx_eq(pos.as_ticks(36000), 36000.0));
        assert!(approx_eq(pos.as_ticks(72000), 72000.0));
    }

    #[test]
    fn as_units() {
        let pos = Position::from_degrees(90.0);
        assert!(approx_eq(pos.as_revolutions(), 0.25));
        assert!(approx_eq(pos.as_gradians(), 100.0));
        assert!(approx_eq(pos.as_degrees(), 90.0));
        assert!(approx_eq(pos.as_radians(), FRAC_PI_2));
        assert!(approx_eq(pos.as_ticks(360), 90.0));
    }

    #[test]
    fn add_subtract() {
        let a = Position::from_degrees(90.0);
        let b = Position::from_degrees(45.0);
        let sum = a + b;
        let diff = a - b;
        assert!(approx_eq(sum.as_degrees(), 135.0));
        assert!(approx_eq(diff.as_degrees(), 45.0));

        let mut p = Position::from_degrees(60.0);
        p += Position::from_degrees(30.0);
        assert!(approx_eq(p.as_degrees(), 90.0));
        p -= Position::from_degrees(45.0);
        assert!(approx_eq(p.as_degrees(), 45.0));
    }

    #[test]
    fn multiply_div_scalar() {
        let p = Position::from_degrees(90.0);
        let doubled = p * 2.0;
        let halved = p / 2.0;
        assert!(approx_eq(doubled.as_degrees(), 180.0));
        assert!(approx_eq(halved.as_degrees(), 45.0));

        let mut p = Position::from_degrees(30.0);
        p *= 3.0;
        assert!(approx_eq(p.as_degrees(), 90.0));
        p /= 3.0;
        assert!(approx_eq(p.as_degrees(), 30.0));
    }

    #[test]
    fn negate() {
        let p = Position::from_degrees(90.0);
        let neg = -p;
        assert!(approx_eq(neg.as_degrees(), -90.0));
    }

    #[test]
    fn non_modular() {
        let a = Position::from_degrees(0.0);
        let b = Position::from_degrees(360.0);
        assert_ne!(a, b);
        assert!(approx_eq(b.as_revolutions(), 1.0));
    }
}
