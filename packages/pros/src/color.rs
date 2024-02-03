//! Generic RGB8 color type and conversion trait.
//! The [`Rgb`] and [`IntoRgb`] types are used in multiple places in the library to represent colors.

/// A trait for types that can be converted into an RGB8 color.
pub trait IntoRgb {
    /// Consume the value and convert it into an RGB8 color.
    fn into_rgb(self) -> Rgb;
}

impl<T: Into<u32>> IntoRgb for T {
    fn into_rgb(self: T) -> Rgb {
        self.into() // u32
            .into() // Rgb
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
    /// HTML white.
    pub const WHITE: Self = Self::new(0xFF, 0xFF, 0xFF);
    /// HTML silver.
    pub const SILVER: Self = Self::new(0xC0, 0xC0, 0xC0);
    /// HTML gray.
    pub const GRAY: Self = Self::new(0x80, 0x80, 0x80);
    /// HTML black.
    pub const BLACK: Self = Self::new(0x00, 0x00, 0x00);
    /// HTML red.
    pub const RED: Self = Self::new(0xFF, 0x00, 0x00);
    /// HTML maroon.
    pub const MAROON: Self = Self::new(0x80, 0x00, 0x00);
    /// HTML yellow.
    pub const YELLOW: Self = Self::new(0xFF, 0xFF, 0x00);
    /// HTML olive.
    pub const OLIVE: Self = Self::new(0x80, 0x80, 0x00);
    /// HTML lime.
    pub const LIME: Self = Self::new(0x00, 0xFF, 0x00);
    /// HTML green.
    pub const GREEN: Self = Self::new(0x00, 0x80, 0x00);
    /// HTML cyan.
    pub const CYAN: Self = Self::new(0x00, 0xFF, 0xFF);
    /// HTML aqua.
    pub const AQUA: Self = Self::CYAN;
    /// HTML teal.
    pub const TEAL: Self = Self::new(0x00, 0x80, 0x80);
    /// HTML blue.
    pub const BLUE: Self = Self::new(0x00, 0x00, 0xFF);
    /// HTML navy.
    pub const NAVY: Self = Self::new(0x00, 0x00, 0x80);
    /// HTML magenta.
    pub const MAGENTA: Self = Self::new(0xFF, 0x00, 0xFF);
    /// HTML purple.
    pub const PURPLE: Self = Self::new(0x80, 0x00, 0x80);
    /// HTML orange.
    pub const ORANGE: Self = Self::new(0xFF, 0xA5, 0x00);

    /// Create a new RGB8 color.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red,
            g: green,
            b: blue,
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

const BITMASK: u32 = 0b11111111;
impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        Self {
            r: ((value >> 16) & BITMASK) as _,
            g: ((value >> 8) & BITMASK) as _,
            b: (value & BITMASK) as _,
        }
    }
}
