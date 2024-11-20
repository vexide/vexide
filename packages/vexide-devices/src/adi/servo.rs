//! ADI Servo

use vex_sdk::vexDeviceAdiValueSet;

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::{position::Position, PortError};

/// Legacy Servo
#[derive(Debug, Eq, PartialEq)]
pub struct AdiServo {
    port: AdiPort,
}

impl AdiServo {
    /// Maximum controllable position of the servo.
    pub const MAX_POSITION: Position = Position::from_ticks(
        ((50.0 / 360.0) * (Position::INTERNAL_TPR as f64)) as i64,
        Position::INTERNAL_TPR,
    );

    /// Minimum controllable position of the servo.
    pub const MIN_POSITION: Position = Position::from_ticks(
        ((-50.0 / 360.0) * (Position::INTERNAL_TPR as f64)) as i64,
        Position::INTERNAL_TPR,
    );

    /// Create a servo from an [`AdiPort`].
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::Servo);

        Self { port }
    }

    /// Sets the servo's position target.
    ///
    /// # Range
    ///
    /// VEX servos have an operating range of 100° spanning from [`AdiServo::MIN_POSITION`] (-50°) to
    /// [`AdiServo::MAX_POSITION`] (50°). Values outside of this range will be saturated at their
    /// respective min or max value.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn set_target(&mut self, position: Position) -> Result<(), PortError> {
        self.port.validate_expander()?;

        let degrees = position.as_degrees();
        unsafe {
            vexDeviceAdiValueSet(
                self.port.device_handle(),
                self.port.index(),
                ((degrees / Self::MAX_POSITION.as_degrees()).clamp(-1.0, 1.0) * 127.0) as i32,
            );
        }

        Ok(())
    }
}

impl AdiDevice for AdiServo {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Servo
    }
}
