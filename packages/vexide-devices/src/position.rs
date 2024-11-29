//! Angular Position Type
//!
//! Standard return type for sensors measuring rotational position

use core::{
    f64::consts::TAU,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// A opaque fixed-point raw angular position reading from a sensor.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Position(i64);

impl Position {
    /// Arbitrary number that's large enough to represent all VEX sensors without precision loss.
    ///
    /// At this time, this represents the least common multiple between the rotation sensor's TPR and
    /// an ungeared motor encoder's TPR.
    pub(crate) const INTERNAL_TPR: u32 = 4_608_000; // LCM of 36000 and 4096

    /// Creates a position from a custom tick reading with a given ticks-per-revolution value.
    ///
    /// Essentially scales this value to the internal 36000 ticks per revolution.
    #[must_use]
    pub const fn from_ticks(ticks: i64, tpr: u32) -> Self {
        Self(ticks * Self::INTERNAL_TPR as i64 / tpr as i64)
    }

    /// Creates a position from a specified number of degrees.
    #[must_use]
    pub fn from_degrees(degrees: f64) -> Self {
        Self(((degrees / 360.0) * f64::from(Self::INTERNAL_TPR)) as i64)
    }

    /// Creates a position from a specified number of radians.
    #[must_use]
    pub fn from_radians(radians: f64) -> Self {
        Self(((radians / TAU) * f64::from(Self::INTERNAL_TPR)) as i64)
    }

    /// Creates a position from a specified number of revolutions.
    #[must_use]
    pub fn from_revolutions(revolutions: f64) -> Self {
        Self((revolutions * f64::from(Self::INTERNAL_TPR)) as i64)
    }

    /// Returns the number of degrees rotated in this position.
    ///
    /// This function's conversion from an internal representation may cause a loss of precision.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_degrees(&self) -> f64 {
        (self.0 * 360) as f64 / f64::from(Self::INTERNAL_TPR)
    }

    /// Returns the number of radians rotated in this position.
    ///
    /// This function's conversion from an internal representation may cause a loss of precision.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_radians(&self) -> f64 {
        self.0 as f64 / f64::from(Self::INTERNAL_TPR) * TAU
    }

    /// Returns the number of revolutions rotated in this position.
    ///
    /// This function's conversion from an internal representation may cause a loss of precision.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_revolutions(&self) -> f64 {
        self.0 as f64 / f64::from(Self::INTERNAL_TPR)
    }

    /// Returns this position's value scaled to another tick value with a different TPR.
    ///
    /// This function's conversion from an internal representation may cause a loss of precision.
    #[must_use]
    pub const fn as_ticks(&self, tpr: u32) -> i64 {
        (self.0 * tpr as i64) / Self::INTERNAL_TPR as i64
    }
}

impl Add<Position> for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Position> for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<i64> for Position {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<i64> for Position {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl AddAssign<Position> for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign<Position> for Position {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl MulAssign<i64> for Position {
    fn mul_assign(&mut self, rhs: i64) {
        self.0 *= rhs;
    }
}

impl DivAssign<i64> for Position {
    fn div_assign(&mut self, rhs: i64) {
        self.0 /= rhs;
    }
}

impl Neg for Position {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}
