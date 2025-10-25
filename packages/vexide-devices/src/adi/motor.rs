//! ADI Motor Controller
//!
//! This module provides an interface for controlling motors over ADI using PWM (Pulse-width
//! Modulation) output. ADI motor control is typically done using a physical hardware component
//! between the brain and the motor itself such as the [Motor Controller 29] to drive the motor.
//!
//! # Hardware Overview
//!
//! The two primary motors that this module is intended to control are the legacy cortex-era [Motor
//! 393] and Motor 269 units from VEX. These are fairly standard DC motors that can be driven using
//! standard voltage control or PWM, with an integrated PTC breaker designed to prevent damage to
//! the motors in the event that they are overcurrent or stalled.
//!
//! While this module provides an API similar to that of a [Smart Motor], it is in reality simply
//! outputting an 8-bit PWM signal, which will be processed by an intermediate motor controller
//! (such as the MC29) to drive the motor using an H-bridge circuit, allowing operation in either
//! direction.
//!
//! Because these motors are no longer V5RC legal, they are not affected by competition control
//! restrictions, nor do they have any software-imposed current limitations beyond the
//! aforementioned PTC circuit.
//!
//! [Motor Controller 29]: https://www.vexrobotics.com/276-2193.html
//! [Motor 393]: https://www.vexrobotics.com/393-motors.html
//! [Smart Motor]: crate::smart::motor

use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Cortex-era Motor Controller
#[derive(Debug, Eq, PartialEq)]
pub struct AdiMotor {
    port: AdiPort,
    slew: bool,
}

impl AdiMotor {
    /// Create a new motor from an [`AdiPort`].
    ///
    /// Motors can be optionally configured to use slew rate control to prevent the internal
    /// PTC from tripping on older cortex-era 393 motors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new ADI motor on ADI port A with slew rate control enabled.
    ///     let mut motor = AdiMotor::new(peripherals.adi_a, true);
    ///
    ///     // Set the motor output to 50% power.
    ///     _ = motor.set_output(0.5);
    ///
    ///     // Get the current motor output.
    ///     let output = motor.output().unwrap();
    ///     println!("Current motor output: {}", output);
    ///
    ///     // Stop the motor.
    ///     _ = motor.stop();
    /// }
    /// ```
    #[must_use]
    pub fn new(port: AdiPort, slew: bool) -> Self {
        port.configure(match slew {
            false => AdiDeviceType::Motor,
            true => AdiDeviceType::MotorSlew,
        });

        Self { port, slew }
    }

    /// Sets the PWM output of the given motor to a floating point number in the range \[-1.0,
    /// 1.0\].
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new ADI motor on ADI port A with slew rate control enabled.
    ///     let mut motor = AdiMotor::new(peripherals.adi_a, true);
    ///
    ///     // Set the motor output to 50% power.
    ///     _ = motor.set_output(0.5);
    /// }
    /// ```
    pub fn set_output(&mut self, value: f64) -> Result<(), PortError> {
        self.set_raw_output((value * 127.0) as i8)
    }

    /// Sets the PWM output of the given motor as an integer in the range \[-127, 127\].
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new ADI motor on ADI port A with slew rate control enabled.
    ///     let mut motor = AdiMotor::new(peripherals.adi_a, true);
    ///
    ///     // Set the motor output to 100 out of 127.
    ///     motor.set_raw_output(100).unwrap();
    /// }
    /// ```
    pub fn set_raw_output(&mut self, pwm: i8) -> Result<(), PortError> {
        self.port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(self.port.device_handle(), self.port.index(), i32::from(pwm));
        }

        Ok(())
    }

    /// Returns the last set PWM output of the motor on the given port as a floating point
    /// number in the range \[-1.0, 1.0\].
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new ADI motor on ADI port A with slew rate control enabled.
    ///     let mut motor = AdiMotor::new(peripherals.adi_a, true);
    ///
    ///     // Get the current motor output.
    ///     let output = motor.output().unwrap();
    ///     println!("Current motor output: {}", output);
    /// }
    /// ```
    pub fn output(&self) -> Result<f64, PortError> {
        Ok(f64::from(self.raw_output()?) / f64::from(i8::MAX))
    }

    /// Returns the last set PWM output of the motor on the given port as an integer in the range
    /// \[-127, 127\].
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new ADI motor on ADI port A with slew rate control enabled.
    ///     let mut motor = AdiMotor::new(peripherals.adi_a, true);
    ///
    ///     // Get the current motor output.
    ///     let output = motor.raw_output().unwrap();
    ///     println!("Current motor output out of 127: {}", output);
    /// }
    /// ```
    pub fn raw_output(&self) -> Result<i8, PortError> {
        self.port.validate_expander()?;

        Ok(
            // TODO:
            // Subtracting i8::MAX from this value comes from this line in the PROS kernel:
            //
            // https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_ext_adi.c#L269
            //
            // Presumably this happens because the legacy motor device types return out of 256 (u8)
            // in the getter, but have an i8 setter. This needs hardware testing, though.
            (unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) }
                - i32::from(i8::MAX)) as i8,
        )
    }

    /// Stops the given motor by setting its output to zero.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a new ADI motor on ADI port A with slew rate control enabled.
    ///     let mut motor = AdiMotor::new(peripherals.adi_a, true);
    ///
    ///     // Stop the motor.
    ///     _ = motor.stop();
    /// }
    /// ```
    pub fn stop(&mut self) -> Result<(), PortError> {
        self.set_raw_output(0)
    }
}

impl AdiDevice<1> for AdiMotor {
    fn port_numbers(&self) -> [u8; 1] {
        [self.port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        match self.slew {
            false => AdiDeviceType::Motor,
            true => AdiDeviceType::MotorSlew,
        }
    }
}
