//! # vexide-devices
//!
//! Functionality for accessing hardware connected to the V5 brain.
//!
//! ## Overview
//!
//! The V5 brain features 21 RJ9 4p4c connector ports (known as "Smart ports") for communicating with newer V5 peripherals, as well as six 3-wire ports with log-to-digital conversion capability for compatibility with legacy Cortex devices. This module provides access to both smart devices and ADI devices.
//!
//! ## Organization
//!
//! - [`smart`] contains abstractions and types for smart port connected devices.
//! - [`adi`] contains abstractions for three wire ADI connected devices.
//! - [`battery`] provides functions for getting information about the currently connected
//!   battery.
//! - [`controller`] provides types for interacting with the V5 controller.

#![no_std]

extern crate alloc;

pub mod adi;
pub mod smart;

pub mod battery;
pub mod color;
pub mod controller;
pub mod geometry;
pub mod peripherals;
pub mod position;
pub mod screen;
pub mod usd;

use snafu::Snafu;

#[derive(Debug, Snafu)]
/// Generic erros that can take place when using ports on the V5 Brain.
pub enum PortError {
    /// No device is plugged into the port.
    Disconnected,

    /// The incorrect device type is plugged into the port.
    IncorrectDevice,
}
