//! ADI (TriPort) devices on the Vex V5.
//! 
//! Most ADI devices can be created with a `new` function that generally takes a port number.
//! Devi
//! 

use crate::error::{map_errno, PortError};


use snafu::Snafu;



pub mod port;
pub mod analog;
pub mod digital;

pub mod encoder;
pub mod motor;
pub mod ultrasonic;
pub mod gyro;
pub mod potentiometer;


#[derive(Debug, Snafu)]
pub enum AdiError {
    #[snafu(display("Another resource is currently trying to access the ADI."))]
    AlreadyInUse,

    #[snafu(display("The port specified has been reconfigured or is not configured for digital input."))]
    DigitalInputNotConfigured,

    #[snafu(display("The port specified cannot be configured due to an invalid configuration type."))]
    InvalidConfigType,

    #[snafu(display("The port has already been configured."))]
    AlreadyConfigured,

    #[snafu(display("The port specified is invalid."))]
    InvalidPort,

    #[snafu(display("Generic exception."))]
    Generic,
    
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    AdiError {
        EACCES => Self::AlreadyInUse,
        EADDRINUSE => Self::DigitalInputNotConfigured,
        _ => Self::Generic
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