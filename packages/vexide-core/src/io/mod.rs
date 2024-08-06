//! Serial input and output functionality.
//!
//! This module aims to provide a very similar API to the Rust standard library's `std::io` module.

mod stdio;

pub(crate) use stdio::STDIO_CHANNEL;
pub use stdio::{stdin, stdout, Stdin, StdinLock, Stdout, StdoutLock};
