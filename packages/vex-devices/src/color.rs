//! Generic RGB8 color type and conversion trait.
//! The [`Rgb`] and [`IntoRgb`] types are used in multiple places in the library to represent colors.

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
