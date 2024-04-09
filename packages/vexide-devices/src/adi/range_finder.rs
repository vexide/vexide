//! ADI Ultrasonic Range Finder.

use snafu::Snafu;
use vex_sdk::vexDeviceAdiValueGet;

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// ADI Range Finders.
///
/// Requires two ports - one for pinging, and one for listening for the response.
/// This output port ("output") must be indexed directly below the input ("input") port.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiRangeFinder {
    output_port: AdiPort,
    input_port: AdiPort,
}

impl AdiRangeFinder {
    /// Create a new rangefinder sensor from a output and input [`AdiPort`].
    pub fn new(ports: (AdiPort, AdiPort)) -> Result<Self, RangeFinderError> {
        let mut output_port = ports.0;
        let input_port = ports.1;

        // Port error handling - two-wire devices are a little weird with this sort of thing.
        if output_port.internal_expander_index() != input_port.internal_expander_index() {
            // Output and input must be plugged into the same ADI expander.
            return Err(RangeFinderError::ExpanderPortMismatch);
        } else if output_port.index() % 2 == 0 {
            // Output must be on an odd indexed port (A, C, E, G).
            return Err(RangeFinderError::BadOutputPort);
        } else if input_port.index() != (output_port.index() - 1) {
            // Input must be directly next to output on the higher port index.
            return Err(RangeFinderError::BadInputPort);
        }

        output_port.configure(AdiDeviceType::RangeFinder)?;

        Ok(Self {
            output_port,
            input_port,
        })
    }

    /// Get the distance reading of the rangefinder sensor in centimeters.
    ///
    /// Round and/or fluffy objects can cause inaccurate values to be returned.
    pub fn distance(&self) -> Result<u16, RangeFinderError> {
        self.output_port.validate_expander()?;

        match unsafe {
            vexDeviceAdiValueGet(
                self.output_port.device_handle(),
                self.output_port.internal_index(),
            )
        } {
            -1 => Err(RangeFinderError::NoReading),
            val => Ok(val as u16),
        }
    }
}

impl AdiDevice for AdiRangeFinder {
    type PortIndexOutput = (u8, u8);

    fn port_index(&self) -> Self::PortIndexOutput {
        (self.output_port.index(), self.input_port.index())
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.output_port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::RangeFinder
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with an rangefinder range finder.
pub enum RangeFinderError {
    /// The sensor is unable to return a valid reading.
    NoReading,

    /// The index of the output wire must be on an odd numbered port (A, C, E, G).
    BadOutputPort,

    /// The input  wire must be plugged in directly above the output wire.
    BadInputPort,

    /// The specified output and input ports belong to different ADI expanders.
    ExpanderPortMismatch,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
