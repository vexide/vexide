//! Macros for printing to the LCD screen.
//!
//! These macros are designed to be exactly the same as the standard library's  equivalents.

use core::fmt::{self, Write};

use super::WRITER;

#[doc(hidden)]
pub fn _llemu_print(args: fmt::Arguments<'_>) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
/// Print to the LLEMU without a trailing newline.
/// The syntax is exactly the same as the standard library's printing macros.
macro_rules! llemu_print {
    ($($arg:tt)*) => {
        $crate::lcd::macros::_llemu_print(core::format_args!($($arg)*));
    };
}

#[macro_export]
/// Print to the LLEMU.
/// The syntax is exactly the same as the standard library's printing macros.
macro_rules! llemu_println {
    () => {
        $crate::llemu_print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::llemu_print!("{}\n", core::format_args!($($arg)*));
    };
}

pub use llemu_print;
pub use llemu_println;
