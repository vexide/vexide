//! ADI Ultrasonic Range Finder
//!
//! The Ultrasonic Range Finder is a rangfinding device which uses ultrasonic sound to measure the
//! distance between the sensor and the object the sound is being reflected back from.
//!
//! # Hardware Overview
//!
//! The Ultrasonic Rangefinder uses sound pulses to measure distance, in a similar way to
//! how bats or submarines find their way around. By emitting an 40KHz ultrasonic pulse for 250mS
//! and timing how long it takes to hear an echo, the Ultrasonic Rangefinder can accurately
//! estimate how far away an object in front of it is.
//!
//! The equation used by the Ultrasonic Range Finder's to calculate its distance reading is
//! `d = t * 171.5` where "d" represents the distance between the sensor and the object found, "t"
//! represents the time it took for the sound wave to return to the sensor, and 171.5 is half the
//! speed of sound in `m/s`.
//!
//! # Effective Range
//!
//! The usable range of the Range Finder is between 1.5” (3.0cm) and 115” (300cm). When the sensor
//! attempts to measure an object at less than 1.5”, the sound echos back too quickly for the
//! sensor to detect and much beyond 115” the intensity of the sound is too weak to detect.
//!
//! Since the Ultrasonic Rangefinder relies on sound waves, surfaces that absorb or deflect sound
//! (such as cushioned surfaces or sharp angles) will limit the operating range of the sensor.
//!
//! # Wiring
//!
//! The sensor has two 3-Wire Cables. There is a black, red, and orange “Output” cable which
//! pulses power to a 40KHz speaker; and a black, red, and yellow “Input” cable which sends a
//! signal back from its high frequency microphone receiver.
//!
//! When wiring the Ultrasonic Rangefinder to the, both wires must be plugged into adjacent ADI
//! ports. For the sensor to work properly, the “INPUT” wire must be in an odd-numbered slot
//! (A, C, E, G), and the “OUTPUT” wire must be in the higher slot next to the input wire.

use snafu::Snafu;
use vex_sdk::vexDeviceAdiValueGet;

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// Range Finder
///
/// Requires two ports - one for pinging, and one for listening for the response.
/// This output port ("ping") must be indexed directly below the input ("echo") port.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiRangeFinder {
    output_port: AdiPort,
    input_port: AdiPort,
}

impl AdiRangeFinder {
    /// Create a new rangefinder sensor from an output and input [`AdiPort`].
    ///
    /// # Errors
    ///
    /// - If the top and bottom ports originate from different [`AdiExpander`](crate::smart::expander::AdiExpander)s,
    ///   returns [`RangeFinderError::ExpanderPortMismatch`].
    /// - If the output port is not odd (A, C, E, G), returns [`RangeFinderError::BadInputPort`].
    /// - If the input port is not the next after the top port, returns [`RangeFinderError::BadOutputPort`].
    pub fn new(ports: (AdiPort, AdiPort)) -> Result<Self, RangeFinderError> {
        let output_port = ports.0;
        let input_port = ports.1;

        // Port error handling - two-wire devices are a little weird with this sort of thing.
        if output_port.expander_index() != input_port.expander_index() {
            // Output and input must be plugged into the same ADI expander.
            return Err(RangeFinderError::ExpanderPortMismatch);
        } else if output_port.index() % 2 == 0 {
            // Output must be on an odd indexed port (A, C, E, G).
            return Err(RangeFinderError::BadOutputPort);
        } else if input_port.index() != (output_port.index() - 1) {
            // Input must be directly next to output on the higher port index.
            return Err(RangeFinderError::BadInputPort);
        }

        output_port.configure(AdiDeviceType::RangeFinder);

        Ok(Self {
            output_port,
            input_port,
        })
    }

    /// Returns the distance reading of the rangefinder sensor in centimeters.
    ///
    /// Round and/or fluffy objects can cause inaccurate values to be returned.
    ///
    /// # Errors
    ///
    /// - A [`RangeFinderError::NoReading`] error is returned if the rangefinder cannot find a valid reading.
    /// - A [`RangeFinderError::Port`] error is returned if the ADI device could not be accessed.
    pub fn distance(&self) -> Result<u16, RangeFinderError> {
        self.output_port.validate_expander()?;

        match unsafe {
            vexDeviceAdiValueGet(self.output_port.device_handle(), self.output_port.index())
        } {
            -1 => Err(RangeFinderError::NoReading),
            val => Ok(val as u16),
        }
    }
}

impl AdiDevice for AdiRangeFinder {
    type PortNumberOutput = (u8, u8);

    fn port_number(&self) -> Self::PortNumberOutput {
        (self.output_port.number(), self.input_port.number())
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.output_port.expander_number()
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

    /// The port number of the output wire must be odd (A, C, E, G).
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
