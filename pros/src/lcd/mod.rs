use snafu::Snafu;

use crate::multitasking::mutex::Mutex;

#[macro_use]
pub mod macros;
pub mod buttons;

pub(crate) mod writer;

lazy_static::lazy_static! {
    pub(crate) static ref WRITER: Mutex<writer::ConsoleLcd> = {
        unsafe { pros_sys::lcd_initialize() };
        Mutex::new(writer::ConsoleLcd::new())
    };
}

#[derive(Debug, Snafu)]
pub enum LcdError {
    #[snafu(display("LCD not initialized"))]
    NotInitialized,
}
impl core::error::Error for LcdError {}
