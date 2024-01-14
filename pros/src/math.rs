//! Math utilities.

/// Rust currently does not support the `abs` function for `f32` and `f64` types in `no_std` mode.
/// (See [rust-lang/rust#50145](https://github.com/rust-lang/rust/issues/50145))
/// This trait is a workaround for that.
pub trait Abs {
    /// Returns the absolute value of `self`.
    fn abs(self) -> Self;
}

impl Abs for f64 {
    fn abs(self) -> Self {
        f64::from_bits(self.to_bits() & !(1 << 63))
    }
}

impl Abs for f32 {
    fn abs(self) -> Self {
        f32::from_bits(self.to_bits() & !(1 << 31))
    }
}