//! Rotation sensor device.
//!
//! Rotation sensors operate on the same [`Position`] type as motors to measure rotation.

use pros_sys::PROS_ERR;

use crate::{
    error::{bail_on, PortError},
    position::Position,
};

/// A physical rotation sensor plugged into a port.
pub struct RotationSensor {
    port: u8,
    pub reversed: bool,
}

impl RotationSensor {
    /// Creates a new rotation sensor on the given port.
    /// Whether or not the sensor should be reversed on creation can be specified.
    pub fn new(port: u8, reversed: bool) -> Result<Self, PortError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::rotation_reset_position(port));
            if reversed {
                bail_on!(PROS_ERR, pros_sys::rotation_set_reversed(port, true));
            }
        }

        Ok(Self { port, reversed })
    }

    /// Sets the position to zero.
    pub fn zero(&mut self) -> Result<(), PortError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::rotation_reset_position(self.port));
        }
        Ok(())
    }

    /// Sets the position.
    pub fn set_position(&mut self, position: Position) -> Result<(), PortError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::rotation_set_position(self.port, (position.into_counts() * 100) as _)
            );
        }
        Ok(())
    }

    /// Sets whether or not the rotation sensor should be reversed.
    pub fn set_reversed(&mut self, reversed: bool) -> Result<(), PortError> {
        self.reversed = reversed;

        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::rotation_set_reversed(self.port, reversed)
            );
        }
        Ok(())
    }

    /// Reverses the rotation sensor.
    pub fn reverse(&mut self) -> Result<(), PortError> {
        self.set_reversed(!self.reversed)
    }

    //TODO: See if this is accurate enough or consider switching to get_position function.
    /// Gets the current position of the sensor.
    pub fn position(&self) -> Result<Position, PortError> {
        Ok(unsafe {
            Position::from_degrees(
                bail_on!(PROS_ERR, pros_sys::rotation_get_angle(self.port)) as f64 / 100.0,
            )
        })
    }
}
