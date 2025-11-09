//! Color type for devices and graphics.
//!
//! This module contains the [`Color`] type, which provides a zero-cost representation of RGB colors
//! used in VEXos.

/// A color stored in the 32-bit BGR0 format, with the "0" byte being reserved.
#[repr(C, align(4))]
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Color {
    /// Blue channel
    pub b: u8,

    /// Green channel
    pub g: u8,

    /// Red channel
    pub r: u8,

    /// Reserved
    _reserved: u8,
}

#[allow(clippy::unreadable_literal)]
impl Color {
    /// "White" color as defined in the HTML 4.01 specification.
    pub const WHITE: Color = Color::from_raw(0xFFFFFF);

    /// "Silver" color as defined in the HTML 4.01 specification.
    pub const SILVER: Color = Color::from_raw(0xC0C0C0);

    /// "Gray" color as defined in the HTML 4.01 specification.
    pub const GRAY: Color = Color::from_raw(0x808080);

    /// "Black" color as defined in the HTML 4.01 specification.
    pub const BLACK: Color = Color::from_raw(0x000000);

    /// "Red" color as defined in the HTML 4.01 specification.
    pub const RED: Color = Color::from_raw(0xFF0000);

    /// "Maroon" color as defined in the HTML 4.01 specification.
    pub const MAROON: Color = Color::from_raw(0x800000);

    /// "Yellow" color as defined in the HTML 4.01 specification.
    pub const YELLOW: Color = Color::from_raw(0xFFFF00);

    /// "Olive" color as defined in the HTML 4.01 specification.
    pub const OLIVE: Color = Color::from_raw(0x808000);

    /// "Lime" color as defined in the HTML 4.01 specification.
    pub const LIME: Color = Color::from_raw(0x00FF00);

    /// "Green" color as defined in the HTML 4.01 specification.
    pub const GREEN: Color = Color::from_raw(0x008000);

    /// "Aqua" color as defined in the HTML 4.01 specification.
    pub const AQUA: Color = Color::from_raw(0x00FFFF);

    /// "Teal" color as defined in the HTML 4.01 specification.
    pub const TEAL: Color = Color::from_raw(0x008080);

    /// "Blue" color as defined in the HTML 4.01 specification.
    pub const BLUE: Color = Color::from_raw(0x0000FF);

    /// "Navy" color as defined in the HTML 4.01 specification.
    pub const NAVY: Color = Color::from_raw(0x000080);

    /// "Fuchsia" color as defined in the HTML 4.01 specification.
    pub const FUCHSIA: Color = Color::from_raw(0xFF00FF);

    /// "Purple" color as defined in the HTML 4.01 specification.
    pub const PURPLE: Color = Color::from_raw(0x800080);

    /// Creates a new RGB color from the provided components.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            _reserved: 0,
            r,
            g,
            b,
        }
    }

    /// Creates a new RGB color from a raw 0RGB representation.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        unsafe { std::mem::transmute(raw.to_le()) }
    }

    /// Converts this color to a raw 0RGB representation.
    #[must_use]
    pub const fn into_raw(self) -> u32 {
        unsafe { std::mem::transmute::<Self, u32>(self).to_le() }
    }
}

impl From<u32> for Color {
    fn from(raw: u32) -> Self {
        Self::from_raw(raw)
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        value.into_raw()
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self {
            _reserved: 0,
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(value: Color) -> (u8, u8, u8) {
        (value.r, value.g, value.b)
    }
}

impl From<rgb::Rgb<u8>> for Color {
    fn from(value: rgb::Rgb<u8>) -> Self {
        Self {
            _reserved: 0,
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

impl From<Color> for rgb::Rgb<u8> {
    fn from(value: Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn convert_from_raw() {
        assert_eq!(Color::from_raw(0), Color::new(0, 0, 0));
        assert_eq!(Color::from_raw(0xFFF_FFF), Color::new(255, 255, 255));
        assert_eq!(Color::from_raw(0x00A_CE6), Color::new(0, 172, 230));
    }

    #[test]
    fn convert_to_raw() {
        assert_eq!(Color::new(0, 0, 0).into_raw(), 0);
        assert_eq!(Color::new(255, 255, 255).into_raw(), 0xFFF_FFF);
        assert_eq!(Color::new(0, 172, 230).into_raw(), 0x00A_CE6);
    }
}
