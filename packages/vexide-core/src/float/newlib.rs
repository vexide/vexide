//! Implementation of floating-point math using newlib's libm library.
//!
//! This file leverages and links the newlib's libm implementation compiled
//! for an ARMv7a target using the softfp ABI as part of the ARM's
//! arm-none-eabi-gcc toolchain.
//!
//! More information (and source code) regarding newlib can be found here:
//! <https://sourceware.org/newlib/>

use core::ffi::{c_double, c_float};

use super::{powi_impl, Float};

/// Errno
///
/// This is required, since the version of libm we used (an optimized version) from
/// the arm-none-eabi-gcc toolchain requires this symbol to be present here. In the
/// future, we may be able to get our own version compiled with -fno-math-errno.
#[cfg(not(feature = "libc"))]
#[allow(non_upper_case_globals)]
static mut errno: core::ffi::c_int = 0;

/// Returns the a pointer to errno in memory
///
/// See above for why this has to exist.
///
/// # Safety
///
/// This function returns a raw pointer to a mutable static. It is intended for
/// interoptability with libm.
#[cfg(not(feature = "libc"))]
#[unsafe(no_mangle)] // SAFETY: libm requires this symbol to exist, and this is the only place it is defined
unsafe extern "C" fn __errno() -> *mut core::ffi::c_int {
    &raw mut errno
}

#[link(name = "m")]
unsafe extern "C" {
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
        powi_impl(self, exp as usize)
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
        (self.sin(), self.cos())
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
        powi_impl(self, exp as usize)
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
        (self.sin(), self.cos())
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
        unsafe { acosh(self) }
    }

    #[inline]
    fn atanh(self) -> Self {
        unsafe { atanh(self) }
    }
}
