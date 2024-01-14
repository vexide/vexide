//! Print to and handle button presses on the LLEMU
//!
//! Anything this module provides is only availible when not using a custom ui made with LVGL.
//! This module is specific to the premade interface (LLEMU).

use snafu::Snafu;

use crate::{lvgl::colors::LcdColor, sync::Mutex};

#[macro_use]
pub mod macros;
pub mod buttons;

pub(crate) mod writer;

/// Sets the background color of the LCD.
pub fn set_background_color(color: LcdColor) {
    unsafe {
        pros_sys::lcd_initialize();
        pros_sys::lcd_set_background_color(*color);
    }
}

/// Sets the text color of the LCD.
pub fn set_text_color(color: LcdColor) {
    unsafe {
        pros_sys::lcd_initialize();
        pros_sys::lcd_set_background_color(*color);
    }
}

lazy_static::lazy_static! {
    pub(crate) static ref WRITER: Mutex<writer::ConsoleLcd> = {
        Mutex::new(writer::ConsoleLcd::new())
    };
}

#[derive(Debug, Snafu)]
pub enum LcdError {
    #[snafu(display("LCD not initialized"))]
    NotInitialized,
}
