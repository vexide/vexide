//! Rotation Sensor
//!
//! This module provides an interface to interact with the VEX V5 Rotation Sensor,
//! which measures the absolute position, rotation count, and angular velocity of a
//! rotating shaft.
//!
//! # Hardware Overview
//!
//! The sensor provides absolute rotational position tracking from 0° to 360° with 0.088° accuracy. The
//! sensor is compromised of two magnets which utilize the [Hall Effect] to indicate angular position. A
//! chip inside the rotation sensor (a Cortex M0+) then keeps track of the total rotations of the sensor
//! to determine total position travelled.
//!
//! Position is reported by VEXos in centidegrees before being converted to an instance of [`Position`].
//!
//! The absolute angle reading is preserved across power cycles (similar to a potentiometer), while the
//! position count stores the cumulative forward and reverse revolutions relative to program start, however
//! the *position* reading will be reset if the sensor loses power. Angular velocity is measured in degrees
//! per second.
//!
//! Like all other Smart devices, VEXos will process sensor updates every 10mS.
//!
//! [Hall Effect]: https://en.wikipedia.org/wiki/Hall_effect_sensor

use core::time::Duration;

use vex_sdk::{
    vexDeviceAbsEncAngleGet, vexDeviceAbsEncDataRateSet, vexDeviceAbsEncPositionGet,
    vexDeviceAbsEncPositionSet, vexDeviceAbsEncStatusGet, vexDeviceAbsEncVelocityGet, V5_DeviceT,
};

use super::{motor::Direction, SmartDevice, SmartDeviceType, SmartPort};
use crate::{position::Position, PortError};

/// A rotation sensor plugged into a Smart Port.
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
    ///
    /// Whether or not the sensor should be reversed on creation can be specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    /// }
    /// ```
    #[must_use]
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

    /// Reset's the sensor's position reading to zero.
    ///
    /// # Errors
    ///
    /// An error is returned if a rotation sensor is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     println!("Before reset: {:?}", sensor.position());
    ///
    ///     _ = sensor.reset_position();
    ///
    ///     println!("After reset: {:?}", sensor.position());
    /// }
    /// ```
    pub fn reset_position(&mut self) -> Result<(), PortError> {
        // NOTE: We don't use vexDeviceAbsEncReset, since that doesn't actually
        // zero position. It sets position to whatever the angle value is.
        self.set_position(Position::default())
    }

    /// Sets the sensor's position reading.
    ///
    /// # Errors
    ///
    /// An error is returned if a rotation sensor is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     // Set position to 15 degrees.
    ///     _ = sensor.set_position(Position::from_degrees(15.0));
    /// }
    /// ```
    pub fn set_position(&mut self, mut position: Position) -> Result<(), PortError> {
        self.validate_port()?;

        if self.direction == Direction::Reverse {
            position = -position;
        }

        unsafe {
            self.direction_offset = Position::default();
            self.raw_direction_offset = Position::default();

            vexDeviceAbsEncPositionSet(self.device, position.as_ticks(36000) as i32);
        }

        Ok(())
    }

    /// Sets the sensor to operate in a given [`Direction`].
    ///
    /// This determines which way the sensor considers to be “forwards”. You can use the marking on the top of the
    /// motor as a reference:
    ///
    /// - When [`Direction::Forward`] is specified, positive velocity/voltage values will cause the motor to rotate
    ///   **with the arrow on the top**. Position will increase as the motor rotates **with the arrow**.
    /// - When [`Direction::Reverse`] is specified, positive velocity/voltage values will cause the motor to rotate
    ///   **against the arrow on the top**. Position will increase as the motor rotates **against the arrow**.
    ///
    /// # Errors
    ///
    /// - An error is returned if an rotation sensor is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Set the sensor's direction to [`Direction::Reverse`].
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     // Reverse the sensor
    ///     _ = sensor.set_direction(Direction::Reverse);
    /// }
    /// ```
    ///
    /// Reverse the sensor's direction (set to opposite of the previous direction):
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     // Reverse the sensor
    ///     _ = sensor.set_direction(!sensor.direction());
    /// }
    /// ```
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
        if new_direction != self.direction() {
            self.direction_offset = self.position()?;
            self.raw_direction_offset = Position::from_ticks(
                i64::from(unsafe { vexDeviceAbsEncPositionGet(self.device) }),
                Self::TICKS_PER_REVOLUTION,
            );
            self.direction = new_direction;
        }

        Ok(())
    }

    /// Sets the internal computation speed of the rotation sensor.
    ///
    /// This method does NOT change the rate at which user code can read data off the sensor, as the brain will only talk to
    /// the device every 10mS regardless of how fast data is being sent or computed. See [`RotationSensor::UPDATE_INTERVAL`].
    ///
    /// This duration should be above [`Self::MIN_DATA_INTERVAL`] (5 milliseconds).
    ///
    /// # Errors
    ///
    /// An error is returned if an rotation sensor is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     // Set to minimum interval.
    ///     _ = sensor.set_data_interval(RotationSensor::MIN_DATA_INTERVAL);
    /// }
    /// ```
    pub fn set_computation_interval(&mut self, interval: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        let mut time_ms = interval
            .as_millis()
            .max(Self::MIN_DATA_INTERVAL.as_millis()) as u32;
        time_ms -= time_ms % 5; // Rate is in increments of 5ms - not sure if this is necessary, but PROS does it.

        unsafe { vexDeviceAbsEncDataRateSet(self.device, time_ms) }

        Ok(())
    }

    /// Returns the [`Direction`] of this sensor.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     println!(
    ///         "Sensor's direction is {}",
    ///         match sensor.direction() {
    ///             Direction::Forward => "forward",
    ///             Direction::Reverse => "reverse",
    ///         }
    ///     );
    /// }
    /// ```
    #[must_use]
    pub const fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns the total number of degrees rotated by the sensor based on direction.
    ///
    /// # Errors
    ///
    /// An error is returned if an rotation sensor is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     if let Ok(position) = sensor.position() {
    ///         println!("Position in degrees: {}°", position.as_degrees());
    ///         println!("Position in radians: {}°", position.as_radians());
    ///         println!("Position in raw ticks (centidegrees): {}°", position.as_ticks(RotationSensor::TICKS_PER_REVOLUTION));
    ///         println!("Number of revolutions spun: {}°", position.as_revolutions());
    ///     }
    /// }
    /// ```
    pub fn position(&self) -> Result<Position, PortError> {
        self.validate_port()?;

        let mut delta_position = Position::from_ticks(
            i64::from(unsafe { vexDeviceAbsEncPositionGet(self.device) }),
            Self::TICKS_PER_REVOLUTION,
        ) - self.raw_direction_offset;

        if self.direction == Direction::Reverse {
            delta_position = -delta_position;
        }

        Ok(self.direction_offset + delta_position)
    }

    /// Returns the angle of rotation measured by the sensor.
    ///
    /// This value is reported from 0-360 degrees.
    ///
    /// # Errors
    ///
    /// An error is returned if an rotation sensor is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     if let Ok(angle) = sensor.angle() {
    ///         println!("Angle in degrees: {}°", angle.as_degrees());
    ///         println!("Angle in radians: {}°", angle.as_radians());
    ///         println!("Angle in raw ticks (centidegrees): {}°", angle.as_ticks(RotationSensor::TICKS_PER_REVOLUTION));
    ///     }
    /// }
    /// ```
    pub fn angle(&self) -> Result<Position, PortError> {
        self.validate_port()?;

        let mut raw_angle = unsafe { vexDeviceAbsEncAngleGet(self.device) };

        if self.direction == Direction::Reverse {
            raw_angle = (Self::TICKS_PER_REVOLUTION as i32) - raw_angle;
        }

        Ok(Position::from_ticks(
            i64::from(raw_angle),
            Self::TICKS_PER_REVOLUTION,
        ))
    }

    /// Returns the sensor's current velocity in degrees per second.
    ///
    /// # Errors
    ///
    /// An error is returned if an rotation sensor is not currently connected to the Smart Port.
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     if let Some(velocity) = sensor.velocity() {
    ///         println!(
    ///             "Velocity in RPM {}",
    ///             velocity / 6.0, // 1rpm = 6dps
    ///         );
    ///     }
    /// }
    /// ```
    pub fn velocity(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        let mut raw_velocity = unsafe { vexDeviceAbsEncVelocityGet(self.device) };

        if self.direction == Direction::Reverse {
            raw_velocity *= -1;
        }

        Ok(f64::from(raw_velocity) / 100.0)
    }

    /// Returns the sensor's internal status code.
    ///
    /// # Errors
    ///
    /// An error is returned if an rotation sensor is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = RotationSensor::new(peripherals.port_1, Direction::Forward);
    ///
    ///     if let Ok(status) = sensor.status() {
    ///         println!("Status: {:b}", status);
    ///     }
    /// }
    /// ```
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
impl From<RotationSensor> for SmartPort {
    fn from(device: RotationSensor) -> Self {
        device.port
    }
}
