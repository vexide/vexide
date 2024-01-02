//! Generic position type for motors and sensors.
//!
//! Positions have many conversion functions as well as common operator implementations for ease of use.

use core::{cmp::Ordering, ops::*};

//TODO: Add more unit types to this.
/// Represents a position a motor can travel to.
/// Positions are relative to the last position the motor was zeroed to.
#[derive(Clone, Copy, Debug)]
pub enum Position {
    Degrees(f64),
    Rotations(f64),
    /// Raw encoder ticks.
    Counts(i64),
}

impl Position {
    /// Creates a position from a specified number of degrees.
    pub fn from_degrees(position: f64) -> Self {
        Self::Degrees(position)
    }

    /// Creates a position from a specified number of rotations.
    pub fn from_rotations(position: f64) -> Self {
        Self::Rotations(position)
    }

    /// Creates a position from a specified number of counts (raw encoder tics).
    pub fn from_counts(position: i64) -> Self {
        Self::Counts(position)
    }

    /// Converts a position into degrees.
    pub fn into_degrees(self) -> f64 {
        match self {
            Self::Degrees(num) => num,
            Self::Rotations(num) => num * 360.0,
            Self::Counts(num) => num as f64 * (360.0 / 4096.0),
        }
    }

    /// Converts a position into rotations.
    pub fn into_rotations(self) -> f64 {
        match self {
            Self::Degrees(num) => num / 360.0,
            Self::Rotations(num) => num,
            Self::Counts(num) => num as f64 * 4096.0,
        }
    }

    /// Converts a position into counts (raw encoder ticks).
    pub fn into_counts(self) -> i64 {
        match self {
            Self::Degrees(num) => (num * 4096.0 / 360.0) as i64,
            Self::Rotations(num) => (num * 4096.0) as i64,
            Self::Counts(num) => num,
        }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_degrees(self.into_degrees() + rhs.into_degrees())
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_degrees(self.into_degrees() - rhs.into_degrees())
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<Self> for Position {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_degrees(self.into_degrees() * rhs.into_degrees())
    }
}

impl MulAssign<Self> for Position {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div<Self> for Position {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::from_degrees(self.into_degrees() / rhs.into_degrees())
    }
}

impl DivAssign<Self> for Position {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Rem<Self> for Position {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::from_degrees(self.into_degrees() % rhs.into_degrees())
    }
}

impl RemAssign<Self> for Position {
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl Neg for Position {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_degrees(-self.into_degrees())
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.into_degrees() == other.into_degrees()
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.into_degrees().partial_cmp(&other.into_degrees())
    }
}
