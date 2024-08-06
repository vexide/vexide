//! Implementation of floating-point math using the pure rust port of MUSL's
//! libm. <https://github.com/rust-lang/libm/>
//!
//! This serves as the only floating-point implementation when compiling for
//! WASM, and can be forced to be used on ARM using the `force-rust-libm`
//! feature.
//!
//! This implementation exists for two reasons:
//! - You are dead-set on getting a pure-rust program build and don't want to
//!   be linked to any C libraries whatsoever.
//! - You are compiling for a WASM target, where an ARM-specific libm won't
//!   work.
//!
//! At the time of writing this, this rust implementation results in both
//! significantly larger binary sizes as well as several times slower math
//! operations than the newlib implementations, so this isn't recommended on
//! ARM.

use super::{powi_impl, Float};

impl Float for f32 {
    #[inline]
    fn floor(self) -> Self {
        libm::floorf(self)
    }

    #[inline]
    fn ceil(self) -> Self {
        libm::ceilf(self)
    }

    #[inline]
    fn round(self) -> Self {
        libm::roundf(self)
    }

    #[inline]
    fn round_ties_even(self) -> Self {
        libm::rintf(self)
    }

    #[inline]
    fn trunc(self) -> Self {
        libm::truncf(self)
    }

    #[inline]
    fn fract(self) -> Self {
        self - self.trunc()
    }

    #[inline]
    fn abs(self) -> Self {
        libm::fabsf(self)
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
        libm::copysignf(self, sign)
    }

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        libm::fmaf(self, a, b)
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
        libm::powf(self, n)
    }

    #[inline]
    fn sqrt(self) -> Self {
        libm::sqrtf(self)
    }

    #[inline]
    fn exp(self) -> Self {
        libm::expf(self)
    }

    #[inline]
    fn exp2(self) -> Self {
        libm::exp2f(self)
    }

    #[inline]
    fn ln(self) -> Self {
        libm::logf(self)
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        self.ln() / base.ln()
    }

    #[inline]
    fn log2(self) -> Self {
        libm::log2f(self)
    }

    #[inline]
    fn log10(self) -> Self {
        libm::log10f(self)
    }

    #[inline]
    fn abs_sub(self, other: Self) -> Self {
        libm::fdimf(self, other)
    }

    #[inline]
    fn cbrt(self) -> Self {
        libm::cbrtf(self)
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        libm::hypotf(self, other)
    }

    #[inline]
    fn sin(self) -> Self {
        libm::sinf(self)
    }

    #[inline]
    fn cos(self) -> Self {
        libm::cosf(self)
    }

    #[inline]
    fn tan(self) -> Self {
        libm::tanf(self)
    }

    #[inline]
    fn asin(self) -> Self {
        libm::asinf(self)
    }

    #[inline]
    fn acos(self) -> Self {
        libm::acosf(self)
    }

    #[inline]
    fn atan(self) -> Self {
        libm::atanf(self)
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        libm::atan2f(self, other)
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }

    #[inline]
    fn exp_m1(self) -> Self {
        libm::expm1f(self)
    }

    #[inline]
    fn ln_1p(self) -> Self {
        libm::log1pf(self)
    }

    #[inline]
    fn sinh(self) -> Self {
        libm::sinhf(self)
    }

    #[inline]
    fn cosh(self) -> Self {
        libm::coshf(self)
    }

    #[inline]
    fn tanh(self) -> Self {
        libm::tanhf(self)
    }

    #[inline]
    fn asinh(self) -> Self {
        libm::asinhf(self)
    }

    #[inline]
    fn acosh(self) -> Self {
        libm::acoshf(self)
    }

    #[inline]
    fn atanh(self) -> Self {
        libm::atanhf(self)
    }
}

impl Float for f64 {
    #[inline]
    fn floor(self) -> Self {
        libm::floor(self)
    }

    #[inline]
    fn ceil(self) -> Self {
        libm::ceil(self)
    }

    #[inline]
    fn round(self) -> Self {
        libm::round(self)
    }

    #[inline]
    fn round_ties_even(self) -> Self {
        libm::rint(self)
    }

    #[inline]
    fn trunc(self) -> Self {
        libm::trunc(self)
    }

    #[inline]
    fn fract(self) -> Self {
        self - self.trunc()
    }

    #[inline]
    fn abs(self) -> Self {
        libm::fabs(self)
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
        libm::copysign(self, sign)
    }

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        libm::fma(self, a, b)
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
        libm::pow(self, n)
    }

    #[inline]
    fn sqrt(self) -> Self {
        libm::sqrt(self)
    }

    #[inline]
    fn exp(self) -> Self {
        libm::exp(self)
    }

    #[inline]
    fn exp2(self) -> Self {
        libm::exp2(self)
    }

    #[inline]
    fn ln(self) -> Self {
        libm::log(self)
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        self.ln() / base.ln()
    }

    #[inline]
    fn log2(self) -> Self {
        libm::log2(self)
    }

    #[inline]
    fn log10(self) -> Self {
        libm::log10(self)
    }

    #[inline]
    fn abs_sub(self, other: Self) -> Self {
        libm::fdim(self, other)
    }

    #[inline]
    fn cbrt(self) -> Self {
        libm::cbrt(self)
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        libm::hypot(self, other)
    }

    #[inline]
    fn sin(self) -> Self {
        libm::sin(self)
    }

    #[inline]
    fn cos(self) -> Self {
        libm::cos(self)
    }

    #[inline]
    fn tan(self) -> Self {
        libm::tan(self)
    }

    #[inline]
    fn asin(self) -> Self {
        libm::asin(self)
    }

    #[inline]
    fn acos(self) -> Self {
        libm::acos(self)
    }

    #[inline]
    fn atan(self) -> Self {
        libm::atan(self)
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        libm::atan2(self, other)
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }

    #[inline]
    fn exp_m1(self) -> Self {
        libm::expm1(self)
    }

    #[inline]
    fn ln_1p(self) -> Self {
        libm::log1p(self)
    }

    #[inline]
    fn sinh(self) -> Self {
        libm::sinh(self)
    }

    #[inline]
    fn cosh(self) -> Self {
        libm::cosh(self)
    }

    #[inline]
    fn tanh(self) -> Self {
        libm::tanh(self)
    }

    #[inline]
    fn asinh(self) -> Self {
        libm::asinh(self)
    }

    #[inline]
    fn acosh(self) -> Self {
        libm::acosh(self)
    }

    #[inline]
    fn atanh(self) -> Self {
        libm::atanh(self)
    }
}
