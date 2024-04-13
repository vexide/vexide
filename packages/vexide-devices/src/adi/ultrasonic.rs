//! ADI ultrasonic sensor.

use snafu::Snafu;
use vex_sdk::vexDeviceAdiValueGet;

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// ADI Range Finders.
///
/// Requires two ports - one for pinging, and one for listening for the response.
/// This ping port ("output") must be indexed directly below the echo ("input") port.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiUltrasonic {
    port_ping: AdiPort,
    port_echo: AdiPort,
}

impl AdiUltrasonic {
    /// Create a new ultrasonic sensor from a ping and echo [`AdiPort`].
    pub fn new(ports: (AdiPort, AdiPort)) -> Result<Self, UltrasonicError> {
        let port_ping = ports.0;
        let port_echo = ports.1;

        // Port error handling - two-wire devices are a little weird with this sort of thing.
        if port_ping.internal_expander_index() != port_echo.internal_expander_index() {
            // Ping and echo must be plugged into the same ADI expander.
            return Err(UltrasonicError::ExpanderPortMismatch);
        } else if port_ping.index() % 2 == 0 {
            // Ping must be on an odd indexed port (A, C, E, G).
            return Err(UltrasonicError::BadPingPort);
        } else if port_echo.index() != (port_ping.index() - 1) {
            // Echo must be directly next to ping on the higher port index.
            return Err(UltrasonicError::BadEchoPort);
        }

        port_ping.configure(AdiDeviceType::Ultrasonic);

        Ok(Self {
            port_ping,
            port_echo,
        })
    }

    /// Get the distance reading of the ultrasonic sensor in centimeters.
    ///
    /// Round and/or fluffy objects can cause inaccurate values to be returned.
    pub fn distance(&self) -> Result<u16, UltrasonicError> {
        self.port_ping.validate_expander()?;

        match unsafe {
            vexDeviceAdiValueGet(
                self.port_ping.device_handle(),
                self.port_ping.internal_index(),
            )
        } {
            -1 => Err(UltrasonicError::NoReading),
            val => Ok(val as u16),
        }
    }
}

impl AdiDevice for AdiUltrasonic {
    type PortIndexOutput = (u8, u8);

    fn port_index(&self) -> Self::PortIndexOutput {
        (self.port_ping.index(), self.port_echo.index())
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port_ping.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Ultrasonic
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with an ultrasonic range finder.
pub enum UltrasonicError {
    /// The sensor is unable to return a valid reading.
    NoReading,

    /// The index of the ping ("output") wire must be on an odd numbered port (A, C, E, G).
    BadPingPort,

    /// The echo ("input") wire must be plugged in directly above the ping wire.
    BadEchoPort,

    /// The specified ping and echo ports belong to different ADI expanders.
    ExpanderPortMismatch,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
