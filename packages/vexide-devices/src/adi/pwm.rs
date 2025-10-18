//! ADI Pulse-width Modulation (PWM)
//!
//! This module provides an interface for generating 8-bit PWM signals through ADI ports.
//!
//! # Hardware Overview
//!
//! Pulse-width modulation (PWM) is a digital signaling technique that creates a variable
//! width high pulse over a fixed period, allowing you to communicate analog data over
//! digital signals by measuring the length of the pulse (how long it was high compared to
//! how long it was low).
//!
//! PWM signals consist of two components:
//! - ON time (pulse width): When the signal is high ([`LogicLevel::High`]/3.3V)
//! - OFF time: When the signal is low ([`LogicLevel::Low`]/0V)
//!
//! The ratio between ON time and OFF time (the "duty cycle") is used to encode
//! information for commands to devices:
//!
//! ```text
//!             |<-->| pulse width (0.94-2.03mS)
//! 3.3V  ┐     ┌────┐     ┌──┐       ┌──────┐
//! 0V    └─────┘    └─────┘  └───────┘      └────
//!             |<-------->| period (16mS)
//! ```
//!
//! [`LogicLevel::Low`]: super::digital::LogicLevel
//! [`LogicLevel::High`]: super::digital::LogicLevel

use vex_sdk::vexDeviceAdiValueSet;

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Generic PWM Output over ADI
#[derive(Debug, Eq, PartialEq)]
pub struct AdiPwmOut {
    port: AdiPort,
}

impl AdiPwmOut {
    /// Create a PWM output from an [`AdiPort`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut pwm = AdiPwmOut::new(peripherals.adi_a);
    ///     _ = pwm.set_output(128); // Set PWM to 50% duty cycle
    /// }
    /// ```
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::PwmOut);

        Self { port }
    }

    /// Sets the PWM output width.
    ///
    /// This value is sent over 16ms periods with pulse widths ranging from roughly
    /// 0.94mS to 2.03mS.
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
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut pwm = AdiPwmOut::new(peripherals.adi_a);
    ///     _ = pwm.set_output(128); // Set PWM to 50% duty cycle
    /// }
    /// ```
    pub fn set_output(&mut self, value: u8) -> Result<(), PortError> {
        self.port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(
                self.port.device_handle(),
                self.port.index(),
                i32::from(value),
            );
        }

        Ok(())
    }
}

impl AdiDevice<1> for AdiPwmOut {
    fn port_numbers(&self) -> [u8; 1] {
        [self.port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::PwmOut
    }
}
