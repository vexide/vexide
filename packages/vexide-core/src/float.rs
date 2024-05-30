//! Floating Point Numbers
//!
//! This module provides implementations of math functions of floating point
//! primitive types (`f32`, `f64`).

use core::ffi::{c_int, c_double, c_float};

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
    /// This function currently corresponds to the `fdim` from libc on Unix and
    /// Windows. Note that this might change in the future.
    #[deprecated(
        since = "0.1.0",
        note = "you probably meant `(self - other).abs()`: \
                this operation is `(self - other).max(0.0)` \
                except that `abs_sub` also propagates NaNs (also \
                known as `fdim` in C). If you truly need the positive \
                difference, consider using that expression or the C function \
                `fdim`, depending on how you wish to handle NaN (please consider \
                filing an issue describing your use-case too)."
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

/// Errno
///
/// This is required, since the version of libm we used (an optimized version) from
/// the arm-none-eabi-gcc toolchain requires this symbol to be present here. In the
/// future, we may be able to get our own version compiled with -fno-math-errno.
#[allow(non_upper_case_globals)]
static mut errno: c_int = 0;

/// Returns the a pointer to errno in memory
///
/// See above for why this has to exist.
#[no_mangle]
pub unsafe extern "C" fn __errno() -> *mut c_int {
    unsafe {
        core::ptr::addr_of_mut!(errno)
    }
}

extern "C" {
    //
    // f32 bindings
    //
    fn floorf(arg: c_float) -> c_float;
    fn ceilf(arg: c_float) -> c_float;
    fn roundf(arg: c_float) -> c_float;
    fn rintf(arg: c_float) -> c_float;
    fn truncf(arg: c_float) -> c_float;
    fn fabsf(arg: c_float) -> c_float;
    fn copysignf(x: c_float, y: c_float) -> c_float;
    fn fmaf(x: c_float, y: c_float, z: c_float) -> c_float;
    fn powf(base: c_float, exponent: c_float) -> c_float;
    fn sqrtf(arg: c_float) -> c_float;
    fn expf(arg: c_float) -> c_float;
    fn exp2f(n: c_float) -> c_float;
    fn logf(arg: c_float) -> c_float;
    fn log2f(arg: c_float) -> c_float;
    fn log10f(arg: c_float) -> c_float;
    fn fdimf(x: c_float, y: c_float) -> c_float;
    fn cbrtf(arg: c_float) -> c_float;
    fn hypotf(x: c_float, y: c_float) -> c_float;
    fn sinf(arg: c_float) -> c_float;
    fn cosf(arg: c_float) -> c_float;
    fn tanf(arg: c_float) -> c_float;
    fn asinf(arg: c_float) -> c_float;
    fn acosf(arg: c_float) -> c_float;
    fn atanf(arg: c_float) -> c_float;
    fn atan2f(y: c_float, x: c_float) -> c_float;
    fn expm1f(arg: c_float) -> c_float;
    fn log1pf(arg: c_float) -> c_float;
    fn sinhf(arg: c_float) -> c_float;
    fn coshf(arg: c_float) -> c_float;
    fn tanhf(arg: c_float) -> c_float;
    fn asinhf(arg: c_float) -> c_float;
    fn acoshf(arg: c_float) -> c_float;
    fn atanhf(arg: c_float) -> c_float;

    //
    // f64 bindings
    //
    fn floor(arg: c_double) -> c_double;
    fn ceil(arg: c_double) -> c_double;
    fn round(arg: c_double) -> c_double;
    fn rint(arg: c_double) -> c_double;
    fn trunc(arg: c_double) -> c_double;
    fn fabs(arg: c_double) -> c_double;
    fn copysign(x: c_double, y: c_double) -> c_double;
    fn fma(x: c_double, y: c_double, z: c_double) -> c_double;
    fn pow(base: c_double, exponent: c_double) -> c_double;
    fn sqrt(arg: c_double) -> c_double;
    fn exp(arg: c_double) -> c_double;
    fn exp2(n: c_double) -> c_double;
    fn log(arg: c_double) -> c_double;
    fn log2(arg: c_double) -> c_double;
    fn log10(arg: c_double) -> c_double;
    fn fdim(x: c_double, y: c_double) -> c_double;
    fn cbrt(arg: c_double) -> c_double;
    fn hypot(x: c_double, y: c_double) -> c_double;
    fn sin(arg: c_double) -> c_double;
    fn cos(arg: c_double) -> c_double;
    fn tan(arg: c_double) -> c_double;
    fn asin(arg: c_double) -> c_double;
    fn acos(arg: c_double) -> c_double;
    fn atan(arg: c_double) -> c_double;
    fn atan2(y: c_double, x: c_double) -> c_double;
    fn expm1(arg: c_double) -> c_double;
    fn log1p(arg: c_double) -> c_double;
    fn sinh(arg: c_double) -> c_double;
    fn cosh(arg: c_double) -> c_double;
    fn tanh(arg: c_double) -> c_double;
    fn asinh(arg: c_double) -> c_double;
    fn acosh(arg: c_double) -> c_double;
    fn atanh(arg: c_double) -> c_double;
}

/// Implementation of an integer power function using exponentiation by squaring.
///
/// Adapted from <https://github.com/rust-num/num-traits/blob/7ec3d41d39b28190ec1d42db38021107b3951f3a/src/pow.rs#L23>
fn powif32_impl(mut base: f32, mut exp: usize) -> f32 {
    if exp == 0 {
        return 1.0;
    }

    while exp & 1 == 0 {
        base = base.clone() * base;
        exp >>= 1;
    }
    if exp == 1 {
        return base;
    }

    let mut acc = base.clone();
    while exp > 1 {
        exp >>= 1;
        base = base.clone() * base;
        if exp & 1 == 1 {
            acc = acc * base.clone();
        }
    }
    acc
}

/// Implementation of an integer power function using exponentiation by squaring.
///
/// Adapted from <https://github.com/rust-num/num-traits/blob/7ec3d41d39b28190ec1d42db38021107b3951f3a/src/pow.rs#L23>
fn powif64_impl(mut base: f64, mut exp: usize) -> f64 {
    if exp == 0 {
        return 1.0;
    }

    while exp & 1 == 0 {
        base = base.clone() * base;
        exp >>= 1;
    }
    if exp == 1 {
        return base;
    }

    let mut acc = base.clone();
    while exp > 1 {
        exp >>= 1;
        base = base.clone() * base;
        if exp & 1 == 1 {
            acc = acc * base.clone();
        }
    }
    acc
}

impl Float for f32 {
    #[inline]
    fn floor(self) -> Self {
        unsafe { floorf(self) }
    }

    #[inline]
    fn ceil(self) -> Self {
        unsafe { ceilf(self) }
    }

    #[inline]
    fn round(self) -> Self {
        unsafe { roundf(self) }
    }

    #[inline]
    fn round_ties_even(self) -> Self {
        unsafe { rintf(self) }
    }

    #[inline]
    fn trunc(self) -> Self {
        unsafe { truncf(self) }
    }

    #[inline]
    fn fract(self) -> Self {
        self - self.trunc()
    }

    #[inline]
    fn abs(self) -> Self {
        unsafe { fabsf(self) }
    }

    #[inline]
    fn signum(self) -> Self {
        if self.is_nan() {
            Self::NAN
        } else {
            1.0_f32.copysign(self)
        }
    }

    #[inline]
    fn copysign(self, sign: Self) -> Self {
        unsafe { copysignf(self, sign) }
    }

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        unsafe { fmaf(self, a, b) }
    }

    #[inline]
    fn div_euclid(self, rhs: Self) -> Self {
        let q = (self / rhs).trunc();
        if self % rhs < 0.0 {
            return if rhs > 0.0 { q - 1.0 } else { q + 1.0 };
        }
        q
    }

    #[inline]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < 0.0 {
            r + rhs.abs()
        } else {
            r
        }
    }

    #[inline]
    fn powi(mut self, mut exp: i32) -> Self {
        if exp < 0 {
            exp = exp.wrapping_neg();
            self = self.recip();
        }
        // It should always be possible to convert a positive `i32` to a `usize`.
        // Note, `i32::MIN` will wrap and still be negative, so we need to convert
        // to `u32` without sign-extension before growing to `usize`.
        powif32_impl(self, exp as usize)
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        unsafe { powf(self, n) }
    }

    #[inline]
    fn sqrt(self) -> Self {
        unsafe { sqrtf(self) }
    }

    #[inline]
    fn exp(self) -> Self {
        unsafe { expf(self) }
    }

    #[inline]
    fn exp2(self) -> Self {
        unsafe { exp2f(self) }
    }

    #[inline]
    fn ln(self) -> Self {
        unsafe { logf(self) }
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        self.ln() / base.ln()
    }

    #[inline]
    fn log2(self) -> Self {
        unsafe { log2f(self) }
    }

    #[inline]
    fn log10(self) -> Self {
        unsafe { log10f(self) }
    }

    #[inline]
    fn abs_sub(self, other: Self) -> Self {
        unsafe { fdimf(self, other) }
    }

    #[inline]
    fn cbrt(self) -> Self {
        unsafe { cbrtf(self) }
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        unsafe { hypotf(self, other) }
    }

    #[inline]
    fn sin(self) -> Self {
        unsafe { sinf(self) }
    }

    #[inline]
    fn cos(self) -> Self {
        unsafe { cosf(self) }
    }

    #[inline]
    fn tan(self) -> Self {
        unsafe { tanf(self) }
    }

    #[inline]
    fn asin(self) -> Self {
        unsafe { asinf(self) }
    }

    #[inline]
    fn acos(self) -> Self {
        unsafe { acosf(self) }
    }

    #[inline]
    fn atan(self) -> Self {
        unsafe { atanf(self) }
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        unsafe { atan2f(self, other) }
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos()) // TODO: Benchmark this against sincosf in libm
    }

    #[inline]
    fn exp_m1(self) -> Self {
        unsafe { expm1f(self) }
    }

    #[inline]
    fn ln_1p(self) -> Self {
        unsafe { log1pf(self) }
    }

    #[inline]
    fn sinh(self) -> Self {
        unsafe { sinhf(self) }
    }

    #[inline]
    fn cosh(self) -> Self {
        unsafe { coshf(self) }
    }

    #[inline]
    fn tanh(self) -> Self {
        unsafe { tanhf(self) }
    }

    #[inline]
    fn asinh(self) -> Self {
        unsafe { asinhf(self) }
    }

    #[inline]
    fn acosh(self) -> Self {
        unsafe { acoshf(self) }
    }

    #[inline]
    fn atanh(self) -> Self {
        unsafe { atanhf(self) }
    }
}

impl Float for f64 {
    #[inline]
    fn floor(self) -> Self {
        unsafe { floor(self) }
    }

    #[inline]
    fn ceil(self) -> Self {
        unsafe { ceil(self) }
    }

    #[inline]
    fn round(self) -> Self {
        unsafe { round(self) }
    }

    #[inline]
    fn round_ties_even(self) -> Self {
        unsafe { rint(self) }
    }

    #[inline]
    fn trunc(self) -> Self {
        unsafe { trunc(self) }
    }

    #[inline]
    fn fract(self) -> Self {
        self - self.trunc()
    }

    #[inline]
    fn abs(self) -> Self {
        unsafe { fabs(self) }
    }

    #[inline]
    fn signum(self) -> Self {
        if self.is_nan() {
            Self::NAN
        } else {
            1.0_f64.copysign(self)
        }
    }

    #[inline]
    fn copysign(self, sign: Self) -> Self {
        unsafe { copysign(self, sign) }
    }

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        unsafe { fma(self, a, b) }
    }

    #[inline]
    fn div_euclid(self, rhs: Self) -> Self {
        let q = (self / rhs).trunc();
        if self % rhs < 0.0 {
            return if rhs > 0.0 { q - 1.0 } else { q + 1.0 };
        }
        q
    }

    #[inline]
    fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < 0.0 {
            r + rhs.abs()
        } else {
            r
        }
    }

    #[inline]
    fn powi(mut self, mut exp: i32) -> Self {
        if exp < 0 {
            exp = exp.wrapping_neg();
            self = self.recip();
        }
        // It should always be possible to convert a positive `i32` to a `usize`.
        // Note, `i32::MIN` will wrap and still be negative, so we need to convert
        // to `u32` without sign-extension before growing to `usize`.
        powif64_impl(self, exp as usize)
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        unsafe { pow(self, n) }
    }

    #[inline]
    fn sqrt(self) -> Self {
        unsafe { sqrt(self) }
    }

    #[inline]
    fn exp(self) -> Self {
        unsafe { exp(self) }
    }

    #[inline]
    fn exp2(self) -> Self {
        unsafe { exp2(self) }
    }

    #[inline]
    fn ln(self) -> Self {
        unsafe { log(self) }
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        self.ln() / base.ln()
    }

    #[inline]
    fn log2(self) -> Self {
        unsafe { log2(self) }
    }

    #[inline]
    fn log10(self) -> Self {
        unsafe { log10(self) }
    }

    #[inline]
    fn abs_sub(self, other: Self) -> Self {
        unsafe { fdim(self, other) }
    }

    #[inline]
    fn cbrt(self) -> Self {
        unsafe { cbrt(self) }
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        unsafe { hypot(self, other) }
    }

    #[inline]
    fn sin(self) -> Self {
        unsafe { sin(self) }
    }

    #[inline]
    fn cos(self) -> Self {
        unsafe { cos(self) }
    }

    #[inline]
    fn tan(self) -> Self {
        unsafe { tan(self) }
    }

    #[inline]
    fn asin(self) -> Self {
        unsafe { asin(self) }
    }

    #[inline]
    fn acos(self) -> Self {
        unsafe { acos(self) }
    }

    #[inline]
    fn atan(self) -> Self {
        unsafe { atan(self) }
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        unsafe { atan2(self, other) }
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos()) // TODO: Benchmark this against sincos in libm
    }

    #[inline]
    fn exp_m1(self) -> Self {
        unsafe { expm1(self) }
    }

    #[inline]
    fn ln_1p(self) -> Self {
        unsafe { log1p(self) }
    }

    #[inline]
    fn sinh(self) -> Self {
        unsafe { sinh(self) }
    }

    #[inline]
    fn cosh(self) -> Self {
        unsafe { cosh(self) }
    }

    #[inline]
    fn tanh(self) -> Self {
        unsafe { tanh(self) }
    }

    #[inline]
    fn asinh(self) -> Self {
        unsafe { asinh(self) }
    }

    #[inline]
    fn acosh(self) -> Self {
        unsafe {acosh(self)}
    }

    #[inline]
    fn atanh(self) -> Self {
        unsafe { atanh(self) }
    }
}
