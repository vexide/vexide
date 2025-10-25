//! VEX hardware abstractions and peripheral access.
//!
//! # Overview
//!
//! This crate provides APIs for interfacing with hardware and peripherals sold by VEX robotics.
//!
//! The VEX V5 Brain features 21 RJ9 serial ports (known as "Smart Ports") for communicating with
//! newer V5 devices, as well as six three-wire ports with analog-to-digital conversion capability
//! for compatibility with legacy Cortex devices. The Brain also has a screen, battery, and usually
//! a controller for reading user input.
//!
//! Hardware access begins at the [`Peripherals`](crate::peripherals::Peripherals) API, where a
//! singleton to the brain's available I/O and peripherals can be obtained:
//!
//! ```
//! use vexide::peripherals::Peripherals;
//!
//! let peripherals = Peripherals::take().unwrap();
//!
//! // Pull out port 1 of peripherals. This is a `SmartPort` and can be used to construct any
//! // device on port 1 of the Brain that we want to control.
//! let port_1 = peripherals.port_1;
//! ```
//!
//! If you are using vexide's `#[vexide::main]` macro, then
//! [`Peripherals`](crate::peripherals::Peripherals) is already given to you through an
//! argument to your `main` function:
//!
//! ```
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) {
//!     println!("o.o what's this? {:?}", peripherals);
//! }
//! ```
//!
//! For more information on peripheral access, see the [`peripherals`] module.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod adi;
pub mod battery;
pub mod color;
pub mod controller;
pub mod display;
pub mod math;
pub mod peripherals;
pub mod smart;
