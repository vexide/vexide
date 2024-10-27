//! Hardware abstractions and functionality for peripherals on the V5 Brain.
//!
//! # Overview
//!
//! This crate provides APIs for interfacing with VEX hardware.
//!
//! The V5 Brain features 21 RJ9 serial ports (known as "Smart Ports") for communicating
//! with newer V5 devices, as well as six three-wire ports with analog-to-digital conversion
//! capability for compatibility with legacy Cortex devices. The Brain also has a screen,
//! battery, and usually a controller for reading user input.
//!
//! # Features
//!
//! - [`peripherals`]: Singleton-style peripheral access.
//! - [`smart`]: Smart Ports and devices.
//! - [`adi`]: Three-wire ports and devices.
//! - [`battery`]: Battery API
//! - [`display`]: Brain Display API
//! - [`controller`]: Controller API

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
