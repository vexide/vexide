//! Floating Point Numbers
//!
//! This module provides implementations of math functions of floating point
//! primitive types (`f32`, `f64`).

//! Provides implementations for the `critical_section` crate on the V5 brain and in WASM environments.

#[cfg(all(target_arch = "arm", target_os = "none", not(feature = "force_rust_libm")))]
mod newlib;

#[cfg(any(target_arch = "wasm32", feature = "force_rust_libm"))]
mod rust;

/// Used to make [`powi_impl`] generic across f32 and f64.
pub(crate) trait One {
    const ONE: Self;
}

impl One for f64 {
    const ONE: Self = 1.0;
}

impl One for f32 {
    const ONE: Self = 1.0;
}

/// Implementation of an integer power function using exponentiation by squaring.
///
/// Adapted from <https://github.com/rust-num/num-traits/blob/7ec3d41d39b28190ec1d42db38021107b3951f3a/src/pow.rs#L23>
#[inline]
pub(crate) fn powi_impl<T: One + Copy + core::ops::Mul<T, Output = T>>(
    mut base: T,
    mut exp: usize,
) -> T {
    if exp == 0 {
        return T::ONE;
    }

    while exp & 1 == 0 {
        base = base * base;
        exp >>= 1;
    }
    if exp == 1 {
        return base;
    }

    let mut acc = base;
    while exp > 1 {
        exp >>= 1;
        base = base * base;
        if exp & 1 == 1 {
            acc = acc * base;
        }
    }
    acc
}

/// Floating-point math functions
///
/// This extension trait defines the missing implementations of floating point
/// math in `core` present in rust's `std` crate.
pub trait Float: Sized {
    /// Returns the largest integer less than or equal to `self`.
    ///
    /// This function always returns the precise result.
    fn floor(self) -> Self;

    /// Returns the smallest integer greater than or equal to `self`.
    ///
    /// This function always returns the precise result.
    fn ceil(self) -> Self;

    /// Returns the nearest integer to `self`. If a value is half-way between two
    /// integers, round away from `0.0`.
    ///
    /// This function always returns the precise result.
    fn round(self) -> Self;

    /// Returns the nearest integer to a number. Rounds half-way cases to the number
    /// with an even least significant digit.
    ///
    /// This function always returns the precise result.
    fn round_ties_even(self) -> Self;

    /// Returns the integer part of `self`.
    /// This means that non-integer numbers are always truncated towards zero.
    ///
    /// This function always returns the precise result.
    fn trunc(self) -> Self;

    /// Returns the fractional part of `self`.
    ///
    /// This function always returns the precise result.
    fn fract(self) -> Self;

    /// Computes the absolute value of `self`.
    ///
    /// This function always returns the precise result.
    fn abs(self) -> Self;

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `INFINITY`
    /// - `-1.0` if the number is negative, `-0.0` or `NEG_INFINITY`
    /// - NaN if the number is NaN
    fn signum(self) -> Self;

    /// Returns a number composed of the magnitude of `self` and the sign of
    /// `sign`.
    ///
    /// Equal to `self` if the sign of `self` and `sign` are the same, otherwise
    /// equal to `-self`. If `self` is a NaN, then a NaN with the sign bit of
    /// `sign` is returned. Note, however, that conserving the sign bit on NaN
    /// across arithmetical operations is not generally guaranteed.
    /// See [explanation of NaN as a special value](primitive@f32) for more info.
    fn copysign(self, sign: Self) -> Self;

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error, yielding a more accurate result than an unfused multiply-add.
    ///
    /// Using `mul_add` *may* be more performant than an unfused multiply-add if
    /// the target architecture has a dedicated `fma` CPU instruction. However,
    /// this is not always true, and will be heavily dependant on designing
    /// algorithms with specific target hardware in mind.
    fn mul_add(self, a: Self, b: Self) -> Self;

    /// Calculates Euclidean division, the matching method for `rem_euclid`.
    ///
    /// This computes the integer `n` such that
    /// `self = n * rhs + self.rem_euclid(rhs)`.
    /// In other words, the result is `self / rhs` rounded to the integer `n`
    /// such that `self >= n * rhs`.
    fn div_euclid(self, rhs: Self) -> Self;

    /// Calculates the least nonnegative remainder of `self (mod rhs)`.
    ///
    /// In particular, the return value `r` satisfies `0.0 <= r < rhs.abs()` in
    /// most cases. However, due to a floating point round-off error it can
    /// result in `r == rhs.abs()`, violating the mathematical definition, if
    /// `self` is much smaller than `rhs.abs()` in magnitude and `self < 0.0`.
    /// This result is not an element of the function's codomain, but it is the
    /// closest floating point number in the real numbers and thus fulfills the
    /// property `self == self.div_euclid(rhs) * rhs + self.rem_euclid(rhs)`
    /// approximately.
    ///
    /// # Precision
    ///
    /// The result of this operation is guaranteed to be the rounded
    /// infinite-precision result.
    fn rem_euclid(self, rhs: Self) -> Self;

    /// Raises a number to an integer power.
    ///
    /// Using this function is generally faster than using `powf`.
    /// It might have a different sequence of rounding operations than `powf`,
    /// so the results are not guaranteed to agree.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn powi(self, n: i32) -> Self;

    /// Raises a number to a floating point power.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn powf(self, n: Self) -> Self;

    /// Returns the square root of a number.
    ///
    /// Returns NaN if `self` is a negative number other than `-0.0`.
    ///
    /// # Precision
    ///
    /// The result of this operation is guaranteed to be the rounded
    /// infinite-precision result. It is specified by IEEE 754 as `squareRoot`
    /// and guaranteed not to change.
    fn sqrt(self) -> Self;

    /// Returns `e^(self)`, (the exponential function).
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn exp(self) -> Self;

    /// Returns `2^(self)`.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn exp2(self) -> Self;

    /// Returns the natural logarithm of the number.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn ln(self) -> Self;

    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// The result might not be correctly rounded owing to implementation details;
    /// `self.log2()` can produce more accurate results for base 2, and
    /// `self.log10()` can produce more accurate results for base 10.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn log(self, base: Self) -> Self;

    /// Returns the base 2 logarithm of the number.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn log2(self) -> Self;

    /// Returns the base 10 logarithm of the number.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn log10(self) -> Self;

    /// The positive difference of two numbers.
    ///
    /// * If `self <= other`: `0.0`
    /// * Else: `self - other`
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    /// This function currently corresponds to the `fdim` function from libm.
    #[deprecated(
        since = "0.2.0",
        note = "you probably meant `(self - other).abs()`: \
                this operation is `(self - other).max(0.0)` \
                except that `abs_sub` also propagates NaNs (also \
                known as `fdim` in C). If you truly need the positive \
                difference, consider using that expression or the C function \
                `fdim`, depending on how you wish to handle NaN."
    )]
    fn abs_sub(self, other: Self) -> Self;

    /// Returns the cube root of a number.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn cbrt(self) -> Self;

    /// Compute the distance between the origin and a point (`x`, `y`) on the
    /// Euclidean plane. Equivalently, compute the length of the hypotenuse of a
    /// right-angle triangle with other sides having length `x.abs()` and
    /// `y.abs()`.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn hypot(self, other: Self) -> Self;

    /// Computes the sine of a number (in radians).
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn sin(self) -> Self;

    /// Computes the cosine of a number (in radians).
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn cos(self) -> Self;

    /// Computes the tangent of a number (in radians).
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn tan(self) -> Self;

    /// Computes the arcsine of a number. Return value is in radians in
    /// the range [-pi/2, pi/2] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn asin(self) -> Self;

    /// Computes the arccosine of a number. Return value is in radians in
    /// the range [0, pi] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn acos(self) -> Self;

    /// Computes the arctangent of a number. Return value is in radians in the
    /// range [-pi/2, pi/2];
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn atan(self) -> Self;

    /// Computes the four quadrant arctangent of `self` (`y`) and `other` (`x`) in radians.
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn atan2(self, other: Self) -> Self;

    /// Simultaneously computes the sine and cosine of the number, `x`. Returns
    /// `(sin(x), cos(x))`.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn sin_cos(self) -> (Self, Self);

    /// Returns `e^(self) - 1` in a way that is accurate even if the
    /// number is close to zero.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn exp_m1(self) -> Self;

    /// Returns `ln(1+n)` (natural logarithm) more accurately than if
    /// the operations were performed separately.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn ln_1p(self) -> Self;

    /// Hyperbolic sine function.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn sinh(self) -> Self;

    /// Hyperbolic cosine function.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn cosh(self) -> Self;

    /// Hyperbolic tangent function.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn tanh(self) -> Self;

    /// Inverse hyperbolic sine function.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn asinh(self) -> Self;

    /// Inverse hyperbolic cosine function.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn acosh(self) -> Self;

    /// Inverse hyperbolic tangent function.
    ///
    /// # Platform-specific precision
    ///
    /// The precision of this function varies by platform and Rust version.
    fn atanh(self) -> Self;
}
