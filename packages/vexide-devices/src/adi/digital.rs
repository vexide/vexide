//! ADI Digital I/O
//!
//! ADI ports on the V5 brain are capable of sending and receiving digital signals
//! with external devices. Digital signals represent binary information using voltage levels
//! (called [logic levels](`LogicLevel`)) - they can only be in one of two states at any time.
//! Unlike analog signals which can take on any voltage within a range, digital signals are
//! either fully "on" (high) or fully "off" (low), making them ideal for simple sensors and
//! actuators such as buttons, switches and solenoids.
//!
//! # Hardware Description
//!
//! The ADI (Analog/Digital Interface) ports can be configured as either digital inputs or
//! outputs. When configured as inputs, they detect voltage levels to determine a logical high
//! (3.3V or above) or low (below 3.3V) state. When configured as outputs, they drive the pin
//! to either 3.3V (high) or 0V (low). These ports can be used with both direct brain connections
//! and through an ADI expander module.

use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Logic level of a digital pin.
///
/// On digital devices, logic levels represent the two possible voltage signals that define
/// the state of a port. This value is either [`High`](LogicLevel::High) or [`Low`](LogicLevel::Low)
/// depending on the intended state of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicLevel {
    /// A high digital signal.
    ///
    /// ADI ports operate on 3.3V logic, so this value indicates a voltage of 3.3V or above.
    High,

    /// A low digital signal.
    ///
    /// ADI ports operate on 3.3V logic, so this value indicates a voltage below 3.3V.
    Low,
}

impl LogicLevel {
    /// Returns `true` if the level is [`High`](LogicLevel::High).
    #[must_use]
    pub const fn is_high(&self) -> bool {
        match self {
            Self::High => true,
            Self::Low => false,
        }
    }

    /// Returns `true` if the level is [`Low`](LogicLevel::Low).
    #[must_use]
    pub const fn is_low(&self) -> bool {
        match self {
            Self::High => false,
            Self::Low => true,
        }
    }
}

impl core::ops::Not for LogicLevel {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Low => Self::High,
            Self::High => Self::Low,
        }
    }
}

/// Generic Digital Input over ADI
///
/// Represents an ADI port configured to recieve digital input. The pin can be read to
/// determine its current [logic level](`LogicLevel`) (above or below 3.3V).
#[derive(Debug, Eq, PartialEq)]
pub struct AdiDigitalIn {
    port: AdiPort,
}

impl AdiDigitalIn {
    /// Create a digital input from an ADI port.
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::DigitalIn);

        Self { port }
    }

    /// Returns the current logic level of a digital input pin.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn level(&self) -> Result<LogicLevel, PortError> {
        self.port.validate_expander()?;

        let value =
            unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }

    /// Returns `true` if the digital input's logic level is [`LogicLevel::High`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn is_high(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_high())
    }

    /// Returns `true` if the digital input's logic level is [`LogicLevel::Low`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn is_low(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_high())
    }
}

impl AdiDevice for AdiDigitalIn {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalIn
    }
}

/// Generic digital output over ADI.
///
/// Represents an ADI port configured to send digital signals to a device. This can be
/// used for toggling solenoids or other external devices that might need a digital signal
/// from the brain.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiDigitalOut {
    port: AdiPort,
}

impl AdiDigitalOut {
    /// Create a digital output from an [`AdiPort`].
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::DigitalOut);

        Self { port }
    }

    /// Create a digital output from an [`AdiPort`] with an initial logic level.
    ///
    /// # Example
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let digital_out = AdiDigitalOut::with_initial_level(peripherals.adi_a, LogicLevel::High).expect("failed to initialize digital output");
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn with_initial_level(port: AdiPort, initial_level: LogicLevel) -> Result<Self, PortError> {
        port.configure(AdiDeviceType::DigitalOut);

        let mut adi_digital_out = Self { port };
        adi_digital_out.set_level(initial_level)?;
        Ok(adi_digital_out)
    }

    /// Sets the digital logic level (high or low) of a pin.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn set_level(&mut self, level: LogicLevel) -> Result<(), PortError> {
        self.port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(
                self.port.device_handle(),
                self.port.index(),
                i32::from(level.is_high()),
            );
        }

        Ok(())
    }

    /// Returns the current set logic level of a digital output pin.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn level(&self) -> Result<LogicLevel, PortError> {
        self.port.validate_expander()?;

        let value =
            unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }

    /// Returns `true` if the port's logic level is set to [`LogicLevel::High`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn is_high(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_high())
    }

    /// Returns `true` if the port's logic level is set to [`LogicLevel::Low`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn is_low(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_high())
    }

    /// Set the digital logic level to [`LogicLevel::High`]. Analogous to
    /// [`Self::set_level(LogicLevel::High)`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn set_high(&mut self) -> Result<(), PortError> {
        self.set_level(LogicLevel::High)
    }

    /// Set the digital logic level to [`LogicLevel::Low`]. Analogous to
    /// [`Self::set_level(LogicLevel::Low)`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn set_low(&mut self) -> Result<(), PortError> {
        self.set_level(LogicLevel::Low)
    }

    /// Sets the digital logic level to the inverse of its previous state.
    ///
    /// - If the port was previously set to [`LogicLevel::Low`], then the level will be set to [`LogicLevel::High`].
    /// - If the port was previously set to [`LogicLevel::High`], then the level will be set to [`LogicLevel::Low`].
    ///
    /// This is analagous to `self.set_level(!self.level()?)?` and is useful for toggling devices like solenoids.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn toggle(&mut self) -> Result<(), PortError> {
        self.set_level(!self.level()?)
    }
}

impl AdiDevice for AdiDigitalOut {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalOut
    }
}
