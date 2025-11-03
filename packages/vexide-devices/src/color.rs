//! Color types.
//!
//! This module re-exports some types in the [`rgb`] crate for use as a general container type for
//! devices working with RGB colors.

pub use rgb::Rgb;

/// Conversion trait between [`Rgb<u8>`] and the raw `u32` bit representation of it used in VEXos
/// APIs.
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

/// A color stored in ARGB format.
#[repr(C, align(4))]
#[derive(Clone, Copy, Default, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Argb {
    /// Alpha channel
    pub a: u8,
    /// Red channel
    pub r: u8,
    /// Green channel
    pub g: u8,
    /// Blue channel
    pub b: u8,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn convert_from_raw() {
        assert_eq!(Rgb::from_raw(0), Rgb::new(0, 0, 0));
        assert_eq!(Rgb::from_raw(0xFFF_FFF), Rgb::new(255, 255, 255));
        assert_eq!(Rgb::from_raw(0x00A_CE6), Rgb::new(0, 172, 230));
    }

    #[test]
    fn convert_to_raw() {
        assert_eq!(Rgb::new(0, 0, 0).into_raw(), 0);
        assert_eq!(Rgb::new(255, 255, 255).into_raw(), 0xFFF_FFF);
        assert_eq!(Rgb::new(0, 172, 230).into_raw(), 0x00A_CE6);
    }
}
