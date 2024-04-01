//! Std-like I/O macros and types for use in pros.
//!
//! Implements `println!`, `eprintln!` and `dbg!` on top of the `pros_sys` crate without requiring
//! the use of an allocator. (Modified version of `libc_print` crate)
//!
//! Allows you to use these macros in a #!\[no_std\] context, or in a situation where the
//! traditional Rust streams might not be available (ie: at process shutdown time).
//!
//! ## Usage
//!
//! Exactly as you'd use `println!`, `eprintln!` and `dbg!`.
//!
//! ```rust
//! # use pros::io::*;
//! // Use the default ``-prefixed macros:
//! # fn test1()
//! # {
//! println!("Hello {}!", "stdout");
//! eprintln!("Hello {}!", "stderr");
//! let a = 2;
//! let b = dbg!(a * 2) + 1;
//! assert_eq!(b, 5);
//! # }
//! ```
//!
//! Or you can import aliases to `std` names:
//!
//! ```rust
//! use pros::io::{println, eprintln, dbg};
//!
//! # fn test2()
//! # {
//! println!("Hello {}!", "stdout");
//! eprintln!("Hello {}!", "stderr");
//! let a = 2;
//! let b = dbg!(a * 2) + 1;
//! assert_eq!(b, 5);
//! # }
//! ```

// libc_print is licensed under the MIT License:

// Copyright (c) 2023 Matt Mastracci and contributors

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
#[allow(unused_imports)]
use core::{convert::TryFrom, file, line, stringify};

pub use no_std_io::io::*;

pub use crate::{dbg, eprint, eprintln, print, println};

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct __SerialWriter(i32);

impl core::fmt::Write for __SerialWriter {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        __println(self.0, s)
    }
}

impl __SerialWriter {
    #[inline]
    pub const fn new(err: bool) -> __SerialWriter {
        __SerialWriter(if err { 2 } else { 1 })
    }

    #[inline]
    pub fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        core::fmt::Write::write_fmt(self, args)
    }

    #[inline]
    pub fn write_str(&mut self, s: &str) -> core::fmt::Result {
        __println(self.0, s)
    }

    #[inline]
    pub fn write_nl(&mut self) -> core::fmt::Result {
        __println(self.0, "\n")
    }
}

#[doc(hidden)]
#[inline]
pub fn __println(handle: i32, msg: &str) -> core::fmt::Result {
    let msg = msg.as_bytes();

    let mut written = 0;
    while written < msg.len() {
        match unsafe { write(handle, &msg[written..]) } {
            // Ignore errors
            None | Some(0) => break,
            Some(res) => written += res,
        }
    }

    Ok(())
}

unsafe fn write(handle: i32, bytes: &[u8]) -> Option<usize> {
    usize::try_from(unsafe {
        pros_sys::write(
            handle,
            bytes.as_ptr().cast::<core::ffi::c_void>(),
            bytes.len(),
        )
    })
    .ok()
}

/// Macro for printing to the standard output, with a newline.
///
/// Does not panic on failure to write - instead silently ignores errors.
///
/// See [`println!`](https://doc.rust-lang.org/std/macro.println.html) for
/// full documentation.
#[macro_export]
macro_rules! println {
    () => { $crate::println!("") };
    ($($arg:tt)*) => {
        {
            #[allow(unused_must_use)]
            {
                let mut stm = $crate::io::__SerialWriter::new(false);
                stm.write_fmt(format_args!($($arg)*));
                stm.write_nl();
            }
        }
    };
}

/// Macro for printing to the standard output.
///
/// Does not panic on failure to write - instead silently ignores errors.
///
/// See [`print!`](https://doc.rust-lang.org/std/macro.print.html) for
/// full documentation.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            #[allow(unused_must_use)]
            {
                let mut stm = $crate::io::__SerialWriter::new(false);
                stm.write_fmt(format_args!($($arg)*));
            }
        }
    };
}

/// Macro for printing to the standard error, with a newline.
///
/// Does not panic on failure to write - instead silently ignores errors.
///
/// See [`eprintln!`](https://doc.rust-lang.org/std/macro.eprintln.html) for
/// full documentation.
#[macro_export]
macro_rules! eprintln {
    () => { $crate::eprintln!("") };
    ($($arg:tt)*) => {
        {
            #[allow(unused_must_use)]
            {
                let mut stm = $crate::io::__SerialWriter::new(true);
                stm.write_fmt(format_args!($($arg)*));
                stm.write_nl();
            }
        }
    };
}

/// Macro for printing to the standard error.
///
/// Does not panic on failure to write - instead silently ignores errors.
///
/// See [`eprint!`](https://doc.rust-lang.org/std/macro.eprint.html) for
/// full documentation.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        {
            #[allow(unused_must_use)]
            {
                let mut stm = $crate::io::__SerialWriter::new(true);
                stm.write_fmt(format_args!($($arg)*));
            }
        }
    };
}

/// Prints and returns the value of a given expression for quick and dirty
/// debugging.
///
/// An example:
///
/// ```rust
/// let a = 2;
/// let b = dbg!(a * 2) + 1;
/// //      ^-- prints: [src/main.rs:2] a * 2 = 4
/// assert_eq!(b, 5);
/// ```
///
/// See [dbg!](https://doc.rust-lang.org/std/macro.dbg.html) for full documentation.
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}]", $file!(), $line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::eprintln!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
