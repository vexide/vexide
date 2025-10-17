//! ADI Servo
//!
//! This module provides an interface for controlling the legacy 3-Wire Servo.
//!
//! # Hardware Overview
//!
//! Servos are similar in both appearance and function to [`AdiMotor`](super::motor::AdiMotor)s, with the
//! caveat that they are designed to hold a specific *angle* rather than a continuous
//! *speed*. In other words:
//!
//! - Motors are designed for continuous rotation, providing variable speed and
//!   direction of rotation.
//! - Servos are designed for precise angular positioning, typically rotating to and
//!   holding a specific angle within a limited range of motion.
//!
//! Servos, similar to motors, are PWM controlled. They use a standard
//! [servo control](https://en.wikipedia.org/wiki/Servo_control) signal. A PWM input of
//! 1ms - 2ms will give full reverse to full forward, while 1.5ms is neutral.
//!
//! # Operating Range
//!
//! The VEX legacy servo has an operating range of 100 degrees:
//! - Minimum: -50 degrees (represented by [`AdiServo::MIN_POSITION`])
//! - Maximum: 50 degrees (represented by [`AdiServo::MAX_POSITION`])
//!
//! Its neutral state is at 0° rotation (the middle of its operating range).

use vex_sdk::vexDeviceAdiValueSet;

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};
use crate::math::Position;

/// Legacy Servo
#[derive(Debug, Eq, PartialEq)]
pub struct AdiServo {
    port: AdiPort,
}

impl AdiServo {
    /// Maximum controllable position of the servo.
    pub const MAX_POSITION: Position = Position::from_degrees(50.0);

    /// Minimum controllable position of the servo.
    pub const MIN_POSITION: Position = Position::from_degrees(-50.0);

    /// Create a servo from an [`AdiPort`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut servo = AdiServo::new(peripherals.adi_a);
    ///     _ = servo.set_target(Position::from_degrees(25.0));
    /// }
    /// ```
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
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut servo = AdiServo::new(peripherals.adi_a);
    ///     _ = servo.set_target(Position::from_degrees(25.0));
    /// }
    /// ```
    pub fn set_target(&mut self, position: Position) -> Result<(), PortError> {
        self.set_raw_target(
            ((position.as_degrees() / Self::MAX_POSITION.as_degrees()).clamp(-1.0, 1.0) * 127.0)
                as i8,
        )
    }

    /// Sets the servo's raw position using a raw 8-bit PWM input from [-127, 127]. This is functionally equivalent
    /// to [`Self::set_target`] with the exception that it accepts an unscaled integer rather than a [`Position`].
    ///
    /// # Range
    ///
    /// VEX servos have an operating range of 100° spanning from [`AdiServo::MIN_POSITION`] (-127) to
    /// [`AdiServo::MAX_POSITION`] (127).
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut servo = AdiServo::new(peripherals.adi_a);
    ///     // Set the servo to the center position
    ///     _ = servo.set_raw_target(0);
    /// }
    /// ```
    pub fn set_raw_target(&mut self, pwm: i8) -> Result<(), PortError> {
        self.port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(self.port.device_handle(), self.port.index(), i32::from(pwm));
        }

        Ok(())
    }
}

impl AdiDevice<1> for AdiServo {
    fn port_numbers(&self) -> [u8; 1] {
        [self.port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Servo
    }
}
