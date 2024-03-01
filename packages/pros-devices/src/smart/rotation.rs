//! Rotation sensor device.
//!
//! Rotation sensors operate on the same [`Position`] type as motors to measure rotation.

use pros_core::{bail_on, error::PortError};
use pros_sys::PROS_ERR;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::position::Position;

/// A physical rotation sensor plugged into a port.
#[derive(Debug, Eq, PartialEq)]
pub struct RotationSensor {
    port: SmartPort,
    /// Whether or not the sensor direction is reversed.
    pub reversed: bool,
}

impl RotationSensor {
    /// Creates a new rotation sensor on the given port.
    /// Whether or not the sensor should be reversed on creation can be specified.
    pub fn new(port: SmartPort, reversed: bool) -> Result<Self, PortError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::rotation_reset_position(port.index()));
            if reversed {
                bail_on!(
                    PROS_ERR,
                    pros_sys::rotation_set_reversed(port.index(), true)
                );
            }
        }

        Ok(Self { port, reversed })
    }

    /// Sets the position to zero.
    pub fn zero(&mut self) -> Result<(), PortError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::rotation_reset_position(self.port.index())
            );
        }
        Ok(())
    }

    /// Sets the position.
    pub fn set_position(&mut self, position: Position) -> Result<(), PortError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::rotation_set_position(
                    self.port.index(),
                    (position.into_counts() * 100) as _
                )
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
                pros_sys::rotation_set_reversed(self.port.index(), reversed)
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
                bail_on!(PROS_ERR, pros_sys::rotation_get_angle(self.port.index())) as f64 / 100.0,
            )
        })
    }
}

impl SmartDevice for RotationSensor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Rotation
    }
}
