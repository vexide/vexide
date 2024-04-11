//! Rotation sensor device.
//!
//! Rotation sensors operate on the same [`Position`] type as motors to measure rotation.

use core::time::Duration;

use vex_sdk::{
    vexDeviceAbsEncAngleGet, vexDeviceAbsEncDataRateSet, vexDeviceAbsEncPositionSet,
    vexDeviceAbsEncReset, vexDeviceAbsEncReverseFlagGet, vexDeviceAbsEncReverseFlagSet,
    vexDeviceAbsEncStatusGet, vexDeviceAbsEncVelocityGet, V5_DeviceT,
};

use super::{motor::Direction, SmartDevice, SmartDeviceType, SmartPort};
use crate::{position::Position, PortError};

/// A physical rotation sensor plugged into a port.
#[derive(Debug, Eq, PartialEq)]
pub struct RotationSensor {
    port: SmartPort,
    device: V5_DeviceT,
}

impl RotationSensor {
    /// The minimum data rate that you can set a rotation sensor to.
    pub const MIN_DATA_RATE: Duration = Duration::from_millis(5);

    /// Creates a new rotation sensor on the given port.
    /// Whether or not the sensor should be reversed on creation can be specified.
    pub fn new(port: SmartPort, direction: Direction) -> Self {
        let device = unsafe { port.device_handle() };

        // NOTE: This is safe to do without port validation, since the SDK has special handling
        // for this sensor if it is initialized when unplugged.
        //
        // FIXME: There's a known race condition with this function that can cause the position
        // reading to become incorrect if it's ran before device initialization occurs. The only
        // reasonable fix is reimplementing the reverse flag ourselves, but that's a little
        // nontrivial to do, since angle and position have different reversing behaviors and require
        // tracking some weird offsets.
        //
        // This forum post provides a better overview of the issue than I can describe here:
        // <https://www.vexforum.com/t/rotation-sensor-bug-workaround-on-vexos-1-1-0/96577/6>
        unsafe {
            vexDeviceAbsEncReverseFlagSet(device, direction.is_reverse());
        }

        Self { device, port }
    }

    /// Creates a new rotation sensor on the given port, returning a [`PortError`] if the sensor is disconnected,
    /// an incorrect device, or otherwise unavailable.
    ///
    /// Whether or not the sensor should be reversed on creation can be specified.
    pub fn try_new(port: SmartPort, direction: Direction) -> Result<Self, PortError> {
        port.validate_type(SmartDeviceType::Rotation)?;

        Ok(Self::new(port, direction))
    }

    /// Sets the position to zero.
    pub fn reset(&mut self) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceAbsEncReset(self.device);
        }

        Ok(())
    }

    /// Sets the position.
    pub fn set_position(&mut self, position: Position) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceAbsEncPositionSet(self.device, (position.into_degrees() * 1000.0) as i32)
        }

        Ok(())
    }

    /// Sets whether or not the rotation sensor should be reversed.
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceAbsEncReverseFlagSet(self.device, direction.is_reverse());
        }

        Ok(())
    }

    /// Sets the update rate of the sensor.
    ///
    /// This duration should be above [`Self::MIN_DATA_RATE`] (5 milliseconds).
    pub fn set_data_rate(&mut self, data_rate: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        let mut time_ms = data_rate.as_millis().max(Self::MIN_DATA_RATE.as_millis()) as u32;
        time_ms -= time_ms % 5; // Rate is in increments of 5ms - not sure if this is necessary, but PROS does it.

        unsafe { vexDeviceAbsEncDataRateSet(self.device, time_ms) }

        Ok(())
    }

    /// Sets whether or not the rotation sensor should be reversed.
    pub fn direction(&self) -> Result<Direction, PortError> {
        self.validate_port()?;

        Ok(
            match unsafe { vexDeviceAbsEncReverseFlagGet(self.device) } {
                false => Direction::Forward,
                true => Direction::Reverse,
            },
        )
    }

    /// Get the total number of degrees rotated by the sensor based on direction.
    pub fn position(&self) -> Result<Position, PortError> {
        self.validate_port()?;

        Ok(Position::from_degrees(
            unsafe { vexDeviceAbsEncAngleGet(self.device) } as f64 / 100.0,
        ))
    }

    /// Get the angle of rotation measured by the sensor.
    ///
    /// This value is reported from 0-360 degrees.
    pub fn angle(&self) -> Result<Position, PortError> {
        self.validate_port()?;

        Ok(Position::from_degrees(
            unsafe { vexDeviceAbsEncAngleGet(self.device) } as f64 / 100.0,
        ))
    }

    /// Get the sensor's current velocity in degrees per second
    pub fn velocity(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceAbsEncVelocityGet(self.device) as f64 / 1000.0 })
    }

    /// Returns the sensor's status code.
    pub fn status(&self) -> Result<u32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceAbsEncStatusGet(self.device) })
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
