//! Hardware abstractions and functionality for peripherals on the V5 Brain.
//!
//! # Overview
//!
//! This crate provides APIs for interfacing with VEX hardware and peripherals.
//!
//! The V5 Brain features 21 RJ9 serial ports (known as "Smart Ports") for communicating
//! with newer V5 devices, as well as six three-wire ports with analog-to-digital conversion
//! capability for compatibility with legacy Cortex devices. The Brain also has a screen,
//! battery, and usually a controller for reading user input.
//!
//! Hardware access begins at the [`Peripherals`](crate::peripherals::Peripherals) API, where
//! singleton access to the brain's I/O and peripherals can be obtained:
//!
//! ```
//! let peripherals = Peripherals::take().unwrap();
//!
//! // Pull out port 1 of peripherals. This is a `SmartPort` and can be used to construct any
//! // device on port 1 of the Brain that we want to control.
//! let port_1 = peripherals.port_1;
//! ```
//!
//! If you are on vexide, [`Peripherals`] is already given to you through your `main` function:
//!
//! ```
//! #![no_std]
//! #![no_main]
//!
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) {
//!     println!("o.o what's this? {:?}", peripherals);
//! }
//! ```
//!
//! For more information on peripheral access, see the [`peripherals`] module.

#![no_std]

extern crate alloc;

pub mod adi;
pub mod battery;
pub mod controller;
pub mod display;
pub mod geometry;
pub mod peripherals;
pub mod position;
pub mod rgb;
pub mod smart;

use snafu::Snafu;

#[derive(Debug, Snafu)]
/// Generic errors that can take place when using ports on the V5 Brain.
pub enum PortError {
    /// No device is plugged into the port.
    Disconnected,

    /// The incorrect device type is plugged into the port.
    IncorrectDevice,
}
