//! VEX hardware abstractions and peripheral access.
//!
//! # Overview
//!
//! This crate provides APIs for interfacing with hardware and peripherals sold by VEX
//! robotics.
//!
//! The VEX V5 Brain features 21 RJ9 serial ports (known as "Smart Ports") for communicating
//! with newer V5 devices, as well as six three-wire ports with analog-to-digital conversion
//! capability for compatibility with legacy Cortex devices. The Brain also has a screen,
//! battery, and usually a controller for reading user input.
//!
//! Hardware access begins at the [`Peripherals`](crate::peripherals::Peripherals) API, where
//! a singleton to the brain's available I/O and peripherals can be obtained:
//!
//! ```
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
pub mod controller;
pub mod display;
pub mod math;
pub mod peripherals;
pub mod position;
pub mod rgb;
pub mod smart;

use smart::SmartDeviceType;
use snafu::Snafu;

/// Errors that can occur when performing operations on [Smartport-connected devices](smart).
/// Most smart devices will return this type when an error occurs.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum PortError {
    /// No device was plugged into the port, when one was expected.
    #[snafu(display("Expected a device to be connected to port {port}"))]
    Disconnected {
        /// The port that was expected to have a device
        port: u8,
    },

    /// The wrong type of device is plugged into the port.
    #[snafu(display(
        "Expected a {expected:?} device on port {port}, but found a {actual:?} device"
    ))]
    IncorrectDevice {
        /// The device type that was expected
        expected: SmartDeviceType,
        /// The device type that was found
        actual: SmartDeviceType,
        /// The port that was expected to have a device
        port: u8,
    },
}

#[cfg(feature = "std")]
impl From<PortError> for std::io::Error {
    fn from(value: PortError) -> Self {
        match value {
            PortError::Disconnected { .. } => std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                "A device is not connected to the specified port.",
            ),
            PortError::IncorrectDevice { .. } => std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                "Port is in use as another device.",
            ),
        }
    }
}

#[cfg(feature = "embedded-io")]
impl embedded_io::Error for PortError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            PortError::Disconnected { .. } => embedded_io::ErrorKind::AddrNotAvailable,
            PortError::IncorrectDevice { .. } => embedded_io::ErrorKind::AddrInUse,
        }
    }
}
