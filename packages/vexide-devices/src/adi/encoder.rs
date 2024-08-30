//! ADI encoder sensor.

use snafu::Snafu;
use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::{position::Position, PortError};

/// ADI Range Finders.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiEncoder {
    top_port: AdiPort,
    bottom_port: AdiPort,
}

impl AdiEncoder {
    /// Number of encoder ticks (unique sensor readings) per revolution for the encoder.
    pub const TICKS_PER_REVOLUTION: u32 = 360;

    /// Create a new encoder sensor from a top and bottom [`AdiPort`].
    pub fn new(ports: (AdiPort, AdiPort)) -> Result<Self, EncoderError> {
        let top_port = ports.0;
        let bottom_port = ports.1;

        // Port error handling - two-wire devices are a little weird with this sort of thing.
        if top_port.expander_index() != bottom_port.expander_index() {
            // Top and bottom must be plugged into the same ADI expander.
            return Err(EncoderError::ExpanderPortMismatch);
        } else if top_port.index() % 2 == 0 {
            // Top must be on an odd indexed port (A, C, E, G).
            return Err(EncoderError::BadTopPort);
        } else if bottom_port.index() != (top_port.index() + 1) {
            // Bottom must be directly next to top on the higher port index.
            return Err(EncoderError::BadBottomPort);
        }

        top_port.configure(AdiDeviceType::Encoder);

        Ok(Self {
            top_port,
            bottom_port,
        })
    }

    /// Get the distance reading of the encoder sensor in centimeters.
    ///
    /// Round and/or fluffy objects can cause inaccurate values to be returned.
    pub fn position(&self) -> Result<Position, EncoderError> {
        self.top_port.validate_expander()?;
        self.top_port.configure(self.device_type());

        Ok(Position::from_ticks(
            unsafe {
                vexDeviceAdiValueGet(self.top_port.device_handle(), self.top_port.index()) as i64
            },
            360,
        ))
    }

    /// Sets the current encoder position to the given position without moving the motor.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given position.
    pub fn set_position(&self, position: Position) -> Result<(), EncoderError> {
        self.top_port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(
                self.top_port.device_handle(),
                self.top_port.index(),
                position.as_ticks(360) as i32,
            )
        }

        Ok(())
    }

    /// Sets the current encoder position to the given position.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal
    /// to the given position.
    pub fn reset_position(&mut self) -> Result<(), EncoderError> {
        self.set_position(Position::default())
    }
}

impl AdiDevice for AdiEncoder {
    type PortNumberOutput = (u8, u8);

    fn port_number(&self) -> Self::PortNumberOutput {
        (self.top_port.number(), self.bottom_port.number())
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.top_port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Encoder
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with an encoder range finder.
pub enum EncoderError {
    /// The number of the top wire must be on an odd numbered port (A, C, E, G).
    BadTopPort,

    /// The bottom wire must be plugged in directly above the top wire.
    BadBottomPort,

    /// The specified top and bottom ports belong to different ADI expanders.
    ExpanderPortMismatch,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
