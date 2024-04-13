#![no_std]
#![cfg_attr(feature = "embedded-graphics", feature(never_type))]
#[cfg(feature = "embedded-graphics")]
pub mod embedded_graphics;
#[cfg(feature = "slint")]
pub mod slint;

extern crate alloc;
