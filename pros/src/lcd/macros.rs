//! Macros for printing to the LCD screen.
//!
//! These macros are designed to be exactly the same as the standard library's  equivalents.

use core::fmt::{self, Write};

use super::WRITER;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::lcd::macros::_print(core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", core::format_args!($($arg)*));
    };
}
