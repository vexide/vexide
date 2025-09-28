//! Angular Position Type
//!
//! Used by devices such as [`Motor`], [`RotationSensor`], and [`AdiEncoder`]
//! that are able to measure their own rotation.
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

/// Stores an angular position/rotation.
///
/// This type stores a unit-agnostic angular position (a signed displacement
/// from some rotation representing `Position::ZERO`).
///
/// This type is used by devices such as [`Motor`], [`RotationSensor`], and
/// [`AdiEncoder`] that measure their own rotation.
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
