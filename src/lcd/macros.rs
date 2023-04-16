use core::{fmt, ffi::c_char};
use crate::bindings;

pub fn _print(args: fmt::Arguments, line: u16) {
    unsafe { bindings::lcd_print(line as i16, args.as_str().unwrap().as_ptr() as *const c_char); }
}

#[macro_export]
macro_rules! print {
    ($line:literal, $($arg:tt)*) => {
        $crate::lcd::macros::_print(core::format_args!($($arg)*), $line);
    };
}

#[macro_export]
macro_rules! println {
    ($line:literal) => {
        $crate::print!($line, "\n");
    };
    ($line:literal, $($arg:tt)*) => {
        $crate::print!($line, "{}\n", core::format_args!($($arg)*));
    };
}