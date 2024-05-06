//! Generic RGB8 color type and conversion trait.
//! The [`Rgb`] and [`IntoRgb`] types are used in multiple places in the library to represent colors.
//!
//! [`IntoRgb`] is a trait that allows for easy conversion between [`Rgb`] and the VEXos 0rgb format.

/// A trait for types that can be converted into an RGB8 color.
pub trait IntoRgb {
    /// Consume the value and convert it into an RGB8 color.
    fn into_rgb(self) -> Rgb;
}

impl<T: Into<u32>> IntoRgb for T {
    fn into_rgb(self: T) -> Rgb {
        Rgb::from_raw(self.into())
    }
}

/// An RGB8 color.
/// The color space will almost always be assumed as sRGB in this library.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    /// Red value of the color.
    pub r: u8,
    /// Green value of the color.
    pub g: u8,
    /// Blue value of the color.
    pub b: u8,
}

impl Rgb {
    /// "White" color as defined in the HTML 4.01 specification.
    pub const WHITE: Rgb = Rgb::from_raw(0xFFFFFF);

    /// "Silver" color as defined in the HTML 4.01 specification.
    pub const SILVER: Rgb = Rgb::from_raw(0xC0C0C0);

    /// "Gray" color as defined in the HTML 4.01 specification.
    pub const GRAY: Rgb = Rgb::from_raw(0x808080);

    /// "Black" color as defined in the HTML 4.01 specification.
    pub const BLACK: Rgb = Rgb::from_raw(0x000000);

    /// "Red" color as defined in the HTML 4.01 specification.
    pub const RED: Rgb = Rgb::from_raw(0xFF0000);

    /// "Maroon" color as defined in the HTML 4.01 specification.
    pub const MAROON: Rgb = Rgb::from_raw(0x800000);

    /// "Yellow" color as defined in the HTML 4.01 specification.
    pub const YELLOW: Rgb = Rgb::from_raw(0xFFFF00);

    /// "Olive" color as defined in the HTML 4.01 specification.
    pub const OLIVE: Rgb = Rgb::from_raw(0x808000);

    /// "Lime" color as defined in the HTML 4.01 specification.
    pub const LIME: Rgb = Rgb::from_raw(0x00FF00);

    /// "Green" color as defined in the HTML 4.01 specification.
    pub const GREEN: Rgb = Rgb::from_raw(0x008000);

    /// "Aqua" color as defined in the HTML 4.01 specification.
    pub const AQUA: Rgb = Rgb::from_raw(0x00FFFF);

    /// "Teal" color as defined in the HTML 4.01 specification.
    pub const TEAL: Rgb = Rgb::from_raw(0x008080);

    /// "Blue" color as defined in the HTML 4.01 specification.
    pub const BLUE: Rgb = Rgb::from_raw(0x0000FF);

    /// "Navy" color as defined in the HTML 4.01 specification.
    pub const NAVY: Rgb = Rgb::from_raw(0x000080);

    /// "Fuchsia" color as defined in the HTML 4.01 specification.
    pub const FUCHSIA: Rgb = Rgb::from_raw(0xFF00FF);

    /// "Purple" color as defined in the HTML 4.01 specification.
    pub const PURPLE: Rgb = Rgb::from_raw(0x800080);

    const BITMASK: u32 = 0b11111111;

    /// Create a new RGB8 color.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red,
            g: green,
            b: blue,
        }
    }

    /// Create a new RGB8 color from a raw u32 value.
    pub const fn from_raw(raw: u32) -> Self {
        Self {
            r: ((raw >> 16) & Self::BITMASK) as _,
            g: ((raw >> 8) & Self::BITMASK) as _,
            b: (raw & Self::BITMASK) as _,
        }
    }

    /// Get the red value of the color.
    pub const fn red(&self) -> u8 {
        self.r
    }

    /// Get the green value of the color.
    pub const fn green(&self) -> u8 {
        self.g
    }

    /// Get the blue value of the color.
    pub const fn blue(&self) -> u8 {
        self.b
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from(tuple: (u8, u8, u8)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }
}

impl From<Rgb> for (u8, u8, u8) {
    fn from(value: Rgb) -> (u8, u8, u8) {
        (value.r, value.g, value.b)
    }
}

impl From<Rgb> for u32 {
    fn from(value: Rgb) -> u32 {
        ((value.r as u32) << 16) + ((value.g as u32) << 8) + value.b as u32
    }
}

impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        Self::from_raw(value)
    }
}
