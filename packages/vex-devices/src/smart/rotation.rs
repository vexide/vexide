//! Rotation sensor device.
//!
//! Rotation sensors operate on the same [`Position`] type as motors to measure rotation.

use pros_core::error::PortError;
use vex_sdk::{
    vexDeviceAbsEncAngleGet, vexDeviceAbsEncPositionGet, vexDeviceAbsEncPositionSet,
    vexDeviceAbsEncReset, vexDeviceAbsEncReverseFlagGet, vexDeviceAbsEncReverseFlagSet,
    vexDeviceAbsEncStatusGet, vexDeviceAbsEncVelocityGet,
};

use super::{motor::Direction, SmartDevice, SmartDeviceInternal, SmartDeviceType, SmartPort};
use crate::position::Position;

/// A physical rotation sensor plugged into a port.
#[derive(Debug, Eq, PartialEq)]
pub struct RotationSensor {
    port: SmartPort,
}

impl RotationSensor {
    /// Creates a new rotation sensor on the given port.
    /// Whether or not the sensor should be reversed on creation can be specified.
    pub fn new(port: SmartPort, direction: Direction) -> Result<Self, PortError> {
        let mut sensor = Self { port };

        sensor.reset()?;
        sensor.set_direction(direction)?;

        Ok(sensor)
    }

    /// Sets the position to zero.
    pub fn reset(&mut self) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceAbsEncReset(self.device_handle());
        }

        Ok(())
    }

    /// Sets the position.
    pub fn set_position(&mut self, position: Position) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceAbsEncPositionSet(self.device_handle(), position.into_degrees() as i32) }

        Ok(())
    }

    /// Sets whether or not the rotation sensor should be reversed.
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceAbsEncReverseFlagSet(self.device_handle(), direction.is_reverse()) }

        Ok(())
    }

    /// Sets whether or not the rotation sensor should be reversed.
    pub fn direction(&self) -> Result<Direction, PortError> {
        self.validate_port()?;

        Ok(
            match unsafe { vexDeviceAbsEncReverseFlagGet(self.device_handle()) } {
                false => Direction::Forward,
                true => Direction::Reverse,
            },
        )
    }

    /// Get the total number of degrees rotated by the sensor based on direction.
    pub fn position(&self) -> Result<Position, PortError> {
        self.validate_port()?;

        Ok(Position::from_degrees(
            unsafe { vexDeviceAbsEncPositionGet(self.device_handle()) } as f64 / 100.0,
        ))
    }

    /// Get the angle of rotation measured by the sensor.
    ///
    /// This value is reported from 0-360 degrees.
    pub fn angle(&self) -> Result<Position, PortError> {
        self.validate_port()?;

        Ok(Position::from_degrees(
            unsafe { vexDeviceAbsEncAngleGet(self.device_handle()) } as f64 / 100.0,
        ))
    }

    /// Get the sensor's current velocity in degrees per second
    pub fn velocity(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceAbsEncVelocityGet(self.device_handle()) as f64 / 1000.0 })
    }

    /// Returns the sensor's status code.
    pub fn status(&self) -> Result<u32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceAbsEncStatusGet(self.device_handle()) })
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
