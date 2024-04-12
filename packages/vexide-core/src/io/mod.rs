//! Serial input and output functionality.
//!
//! This module aims to provide a very similar API to the Rust standard library's `std::io` module.

mod stdio;

pub use no_std_io::io::*;
pub(crate) use stdio::STDIO_CHANNEL;
pub use stdio::{dbg, print, println, stdin, stdout, Stdin, StdinLock, Stdout, StdoutLock};
