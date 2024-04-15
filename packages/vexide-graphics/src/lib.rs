//! vexide-graphics is a collection of V5 Brain display driver implementations for embedded rendering libraries.
//! This crate is part of the [`vexide`](https://crates.io/crates/vexide) ecosystem.
//!
//! # Features
//! - `embedded-graphics`: Enables support for the [`embedded-graphics`](https://crates.io/crates/embedded-graphics) crate.
//! - `slint`: Enables support for the Slint graphics library.
//!
//! # Usage
//!
//! ### Slint
//! To use Slint, call [`initialize_slint_platform`](slint::initialize_slint_platform) and then continue using Slint as normal.
//! If you get errors when building your slint files, try enabling the `EmbedResourcesKind` slint compile option.
//!
//! ### Embedded-graphics
//! To use embedded-graphics, create a new [`BrainDisplay`](embedded_graphics::BrainDisplay) using its [`new`](embedded_graphics::BrainDisplay::new) function
//! and start using it as a draw target.


#![no_std]
#![cfg_attr(feature = "embedded-graphics", feature(never_type))]
#[cfg(feature = "embedded-graphics")]
pub mod embedded_graphics;
#[cfg(feature = "slint")]
pub mod slint;
