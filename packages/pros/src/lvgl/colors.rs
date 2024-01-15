//! LVGL colors and presets.

use core::ops::{Deref, DerefMut};

use pros_sys::lv_color_t;

/// A color that can be used on the LCD.
/// The color space is dependent on the LCD driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LcdColor(pub lv_color_t);

impl LcdColor {
    /// Create an RGB color without any transparency.
    pub const fn new_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(lv_color_t {
            red,
            green,
            blue,
            alpha: 0xFF,
        })
    }
    /// Create an RGBA color with a certain opacity.
    pub const fn new_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self(lv_color_t {
            red,
            green,
            blue,
            alpha,
        })
    }
}

impl From<lv_color_t> for LcdColor {
    fn from(other: lv_color_t) -> Self {
        Self(other)
    }
}

impl Deref for LcdColor {
    type Target = lv_color_t;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LcdColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl LcdColor {
    pub const WHITE: Self = Self::new_rgb(0xFF, 0xFF, 0xFF);
    pub const SILVER: Self = Self::new_rgb(0xC0, 0xC0, 0xC0);
    pub const GRAY: Self = Self::new_rgb(0x80, 0x80, 0x80);
    pub const BLACK: Self = Self::new_rgb(0x00, 0x00, 0x00);
    pub const RED: Self = Self::new_rgb(0xFF, 0x00, 0x00);
    pub const MAROON: Self = Self::new_rgb(0x80, 0x00, 0x00);
    pub const YELLOW: Self = Self::new_rgb(0xFF, 0xFF, 0x00);
    pub const OLIVE: Self = Self::new_rgb(0x80, 0x80, 0x00);
    pub const LIME: Self = Self::new_rgb(0x00, 0xFF, 0x00);
    pub const GREEN: Self = Self::new_rgb(0x00, 0x80, 0x00);
    pub const CYAN: Self = Self::new_rgb(0x00, 0xFF, 0xFF);
    pub const AQUA: Self = Self::CYAN;
    pub const TEAL: Self = Self::new_rgb(0x00, 0x80, 0x80);
    pub const BLUE: Self = Self::new_rgb(0x00, 0x00, 0xFF);
    pub const NAVY: Self = Self::new_rgb(0x00, 0x00, 0x80);
    pub const MAGENTA: Self = Self::new_rgb(0xFF, 0x00, 0xFF);
    pub const PURPLE: Self = Self::new_rgb(0x80, 0x00, 0x80);
    pub const ORANGE: Self = Self::new_rgb(0xFF, 0xA5, 0x00);
}
