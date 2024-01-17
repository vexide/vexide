//! ADI (TriPort) devices on the Vex V5.
//!
//! Most ADI devices can be created with a `new` function that generally takes a port number.
//! Devi
//!

use snafu::Snafu;

use crate::error::{map_errno, PortError};

pub mod analog;
pub mod digital;
pub mod port;

pub mod encoder;
pub mod gyro;
pub mod motor;
pub mod potentiometer;
pub mod ultrasonic;

#[derive(Debug, Snafu)]
pub enum AdiError {
    #[snafu(display("Another resource is currently trying to access the ADI."))]
    AlreadyInUse,

    #[snafu(display(
        "The port specified has been reconfigured or is not configured for digital input."
    ))]
    DigitalInputNotConfigured,

    #[snafu(display(
        "The port type specified is invalid, and cannot be used to configure a port."
    ))]
    InvalidConfigType,

    #[snafu(display("The port has already been configured."))]
    AlreadyConfigured,

    #[snafu(display("The port specified is invalid."))]
    InvalidPort,

    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    AdiError {
        EACCES => Self::AlreadyInUse,
        EADDRINUSE => Self::DigitalInputNotConfigured,
    }
    inherit PortError;
}

#[derive(Debug, Copy, Clone)]
pub enum AdiSlot {
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,
    H = 8,
}
