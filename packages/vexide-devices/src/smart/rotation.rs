//! Rotation sensor device.
//!
//! Rotation sensors operate on the same [`Position`] type as motors to measure rotation.

use core::time::Duration;

use vex_sdk::{
    vexDeviceAbsEncAngleGet, vexDeviceAbsEncDataRateSet, vexDeviceAbsEncPositionGet,
    vexDeviceAbsEncPositionSet, vexDeviceAbsEncStatusGet, vexDeviceAbsEncVelocityGet, V5_DeviceT,
};

use super::{motor::Direction, SmartDevice, SmartDeviceType, SmartPort};
use crate::{position::Position, PortError};

/// A physical rotation sensor plugged into a port.
#[derive(Debug, PartialEq)]
pub struct RotationSensor {
    /// Smart Port
    port: SmartPort,

    /// Handle to the internal SDK device instance.
    device: V5_DeviceT,

    /// Current direction state of the sensor.
    direction: Direction,

    /// The position data recorded by [`Self::position`] at the time the sensor is reversed.
    direction_offset: Position,

    /// The raw position data recorded by the SDK at the time the sensor is reversed.
    raw_direction_offset: Position,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for RotationSensor {}
unsafe impl Sync for RotationSensor {}

impl RotationSensor {
    /// The minimum data rate that you can set a rotation sensor to.
    pub const MIN_DATA_INTERVAL: Duration = Duration::from_millis(5);

    /// The amount of unique sensor readings per one revolution of the sensor.
    pub const TICKS_PER_REVOLUTION: u32 = 36000;

    /// Creates a new rotation sensor on the given port.
    /// Whether or not the sensor should be reversed on creation can be specified.
    pub fn new(port: SmartPort, direction: Direction) -> Self {
        let device = unsafe { port.device_handle() };

        Self {
            device,
            port,
            direction,
            direction_offset: Position::default(),
            raw_direction_offset: Position::default(),
        }
    }

    /// Sets the position to zero.
    pub fn reset_position(&mut self) -> Result<(), PortError> {
        // NOTE: We don't use vexDeviceAbsEncReset, since that doesn't actually
        // zero position. It sets position to whatever the angle value is.
        self.set_position(Position::default())
    }

    /// Sets the position.
    pub fn set_position(&mut self, mut position: Position) -> Result<(), PortError> {
        self.validate_port()?;

        if self.direction == Direction::Reverse {
            position = -position;
        }

        unsafe {
            self.direction_offset = Position::default();
            self.raw_direction_offset = Position::default();

            vexDeviceAbsEncPositionSet(self.device, position.as_ticks(36000) as i32)
        }

        Ok(())
    }

    /// Sets whether or not the rotation sensor should be reversed.
    pub fn set_direction(&mut self, new_direction: Direction) -> Result<(), PortError> {
        // You're probably wondering why I don't use [`vexDeviceAbsEncReverseFlagSet`] here. So about that...
        //
        // This sensor is a little unique in that it stores two separate values - "position" and "angle". Angle is the literal
        // angle of rotation of the sensor from 0-36000 centidegrees. Position is how *many* centidegrees the sensor was rotated
        // by. Position is completely unbounded. Both of these values are treated separately in the SDK, and have different
        // behaviors when dealing with the reverse flag. When the sensor is reversed, angle is transformed to become 36000 - angle
        // (converted clockwise -> counterclockwise essentially), while position actually doesn't change at all.
        //
        // Rather than simply negating position when reversed, the SDK keeps the current position before reversing and just
        // reverses the direction of future measurements. So if I were to rotate the sensor by 90 degrees, reverse the
        // direction, then rotate it another 90 degrees it would now be at a net 0 degree rotation value.
        //
        // Now, here's where this all falls apart. There's a known race condition in [`vexDeviceAbsEncReverseFlagSet`], where
        // if the reverse flag is set before the device reports its first *position* value, the starting position will be at
        // 36000 rather than 0. This is because the SDK has code for ensuring that "angle" and "position" are set the same. If
        // we set the reverse flag before position has been initially set, then rather than starting with a position of 0, we
        // start with a position of 36000 (the default angle after being reversed). So rather than dealing with polling and
        // status codes and potentially blocking the current thread until this device is initialized, I just recreated this
        // behavior on our end without ever touching the status code.
        //
        // For more information: <https://www.vexforum.com/t/rotation-sensor-bug-workaround-on-vexos-1-1-0/96577/2>
        if new_direction != self.direction()? {
            self.direction_offset = self.position()?;
            self.raw_direction_offset = Position::from_ticks(
                unsafe { vexDeviceAbsEncPositionGet(self.device) } as i64,
                Self::TICKS_PER_REVOLUTION,
            );
            self.direction = new_direction;
        }

        Ok(())
    }

    /// Sets the update rate of the sensor.
    ///
    /// This duration should be above [`Self::MIN_DATA_RATE`] (5 milliseconds).
    pub fn set_data_rate(&mut self, data_rate: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        let mut time_ms = data_rate
            .as_millis()
            .max(Self::MIN_DATA_INTERVAL.as_millis()) as u32;
        time_ms -= time_ms % 5; // Rate is in increments of 5ms - not sure if this is necessary, but PROS does it.

        unsafe { vexDeviceAbsEncDataRateSet(self.device, time_ms) }

        Ok(())
    }

    /// Sets whether or not the rotation sensor should be reversed.
    pub fn direction(&self) -> Result<Direction, PortError> {
        self.validate_port()?;

        Ok(self.direction)
    }

    /// Get the total number of degrees rotated by the sensor based on direction.
    pub fn position(&self) -> Result<Position, PortError> {
        self.validate_port()?;

        let mut delta_position = Position::from_ticks(
            unsafe { vexDeviceAbsEncPositionGet(self.device) } as i64,
            Self::TICKS_PER_REVOLUTION,
        ) - self.raw_direction_offset;

        if self.direction == Direction::Reverse {
            delta_position = -delta_position;
        }

        Ok(self.direction_offset + delta_position)
    }

    /// Get the angle of rotation measured by the sensor.
    ///
    /// This value is reported from 0-360 degrees.
    pub fn angle(&self) -> Result<Position, PortError> {
        self.validate_port()?;

        let mut raw_angle = unsafe { vexDeviceAbsEncAngleGet(self.device) };

        if self.direction == Direction::Reverse {
            raw_angle = (Self::TICKS_PER_REVOLUTION as i32) - raw_angle;
        }

        Ok(Position::from_ticks(
            raw_angle as i64,
            Self::TICKS_PER_REVOLUTION,
        ))
    }

    /// Get the sensor's current velocity in degrees per second
    pub fn velocity(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        let mut raw_velocity = unsafe { vexDeviceAbsEncVelocityGet(self.device) };

        if self.direction == Direction::Reverse {
            raw_velocity *= -1;
        }

        Ok(raw_velocity as f64 / 100.0)
    }

    /// Returns the sensor's status code.
    pub fn status(&self) -> Result<u32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceAbsEncStatusGet(self.device) })
    }
}

impl SmartDevice for RotationSensor {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Rotation
    }
}
