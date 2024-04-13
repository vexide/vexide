//! vexide-graphics is a collection of V5 Brain display driver implementations for embedded rendering libraries.
//! This crate is part of the [`vexide`](https://crates.io/crates/vexide) ecosystem.
//!
//! # Features
//! - `embedded-graphics`: Enables support for the [`embedded-graphics`](https://crates.io/crates/embedded-graphics) crate.
//! - `slint`: Enables support for the Slint graphics library.

#![no_std]
#![cfg_attr(feature = "embedded-graphics", feature(never_type))]
#[cfg(feature = "embedded-graphics")]
pub mod embedded_graphics;
#[cfg(feature = "slint")]
pub mod slint;

extern crate alloc;
