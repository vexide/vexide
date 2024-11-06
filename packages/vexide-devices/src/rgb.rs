//! RGB Color Type
//!
//! This module re-exports some types in the [`rgb`] crate for use
//! as a general container type for devices working with RGB colors.

pub use rgb::Rgb;

/// Conversion trait between [`Rgb<u8>`] and the raw `u32` bit representation
/// of it used in VEXos APIs.
pub(crate) trait RgbExt {
    #[allow(unused)]
    fn from_raw(raw: u32) -> Self;
    fn into_raw(self) -> u32;
}

impl RgbExt for rgb::Rgb<u8> {
    fn from_raw(raw: u32) -> Self {
        const BITMASK: u32 = 0b1111_1111;

        Self {
            r: ((raw >> 16) & BITMASK) as _,
            g: ((raw >> 8) & BITMASK) as _,
            b: (raw & BITMASK) as _,
        }
    }

    fn into_raw(self) -> u32 {
        (u32::from(self.r) << 16) + (u32::from(self.g) << 8) + u32::from(self.b)
    }
}
