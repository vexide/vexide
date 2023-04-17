use core::fmt::{self, Write};

use super::WRITER;

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
    ($line:literal) => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", core::format_args!($($arg)*));
    };
}