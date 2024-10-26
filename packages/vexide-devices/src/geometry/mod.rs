//! Container types for geometry.

use core::{
    fmt::{self, Display},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

pub use mint::{EulerAngles, Quaternion, Vector3};

#[cfg(feature = "nalgebra")]
mod nalgebra;

/// A point in 2D cartesian space.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Point2<T> {
    /// The x component of the point.
    pub x: T,

    /// The y component of the point.
    pub y: T,
}

impl<T> Point2<T> {
    /// Creates a new point.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Sets the point's x component.
    pub fn set_x(&mut self, x: T) {
        self.x = x;
    }

    /// Sets the point's y component.
    pub fn set_y(&mut self, y: T) {
        self.y = y;
    }
}

impl<T: Copy> Point2<T> {
    /// Returns the point's x component.
    pub const fn x(&self) -> T {
        self.x
    }

    /// Returns the point's y component.
    pub const fn y(&self) -> T {
        self.y
    }
}

// Display format (x, y)

impl<T: Display> Display for Point2<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// Operator overloads

impl<T: Add<Output = T>> Add for Point2<T> {
    type Output = Self;

    /// Vector addition
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl<T: Add<Output = T> + Copy> Add<T> for Point2<T> {
    type Output = Self;

    /// Scalar addition
    fn add(self, rhs: T) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl<T: Sub<Output = T>> Sub for Point2<T> {
    type Output = Self;

    /// Vector subtraction
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl<T: Sub<Output = T> + Copy> Sub<T> for Point2<T> {
    type Output = Self;

    /// Scalar subtraction
    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl<T: Mul<Output = T> + Copy> Mul<T> for Point2<T> {
    type Output = Self;

    // Scalar multiplication
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Div<Output = T> + Copy> Div<T> for Point2<T> {
    type Output = Self;

    // Scalar division
    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: AddAssign> AddAssign for Point2<T> {
    /// Vector add-assignment.
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl<T: AddAssign + Copy> AddAssign<T> for Point2<T> {
    /// Scalar add-assignment.
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs;
        self.y += rhs;
    }
}

impl<T: SubAssign> SubAssign for Point2<T> {
    /// Vector sub-assignment.
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
impl<T: SubAssign + Copy> SubAssign<T> for Point2<T> {
    /// Scalar sub-assignment.
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

impl<T: MulAssign + Copy> MulAssign<T> for Point2<T> {
    /// Scalar mul-assignment.
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T: DivAssign + Copy> DivAssign<T> for Point2<T> {
    /// Scalar mul-assignment.
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl<T: Neg<Output = T>> Neg for Point2<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

// Mint conversions

impl<T> mint::IntoMint for Point2<T> {
    type MintType = mint::Point2<T>;
}

impl<T> From<mint::Point2<T>> for Point2<T> {
    fn from(mint: mint::Point2<T>) -> Self {
        Self {
            x: mint.x,
            y: mint.y,
        }
    }
}

impl<T> From<Point2<T>> for mint::Point2<T> {
    fn from(point: Point2<T>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

// Basic type conversions

impl<T> From<(T, T)> for Point2<T> {
    fn from(tuple: (T, T)) -> Self {
        Self {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl<T: Copy> From<[T; 2]> for Point2<T> {
    fn from(array: [T; 2]) -> Self {
        Self {
            x: array[0],
            y: array[1],
        }
    }
}
