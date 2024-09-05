//! Digital input and output ADI devices

use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Represents the logic level of a digital pin.
///
/// On digital devices, logic levels represent the two possible voltage signals that define
/// the state of a pin. This value is either [`High`](LogicLevel::High) or [`Low`](LogicLevel::Low), depending on the intended
/// state of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicLevel {
    /// A high digital signal.
    ///
    /// ADI ports operate on 3.3V logic, so this value indicates a voltage of 3.3V or above.
    High,

    /// The low digital signal.
    ///
    /// ADI ports operate on 3.3V logic, so this value indicates a voltage below 3.3V.
    Low,
}

impl LogicLevel {
    /// Returns `true` if the level is [`High`](LogicLevel::High).
    pub const fn is_high(&self) -> bool {
        match self {
            Self::High => true,
            Self::Low => false,
        }
    }

    /// Returns `true` if the level is [`Low`](LogicLevel::Low).
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

/// Generic digital input ADI device.
#[derive(Debug, Eq, PartialEq)]
/// Generic digital input ADI device.
pub struct AdiDigitalIn {
    port: AdiPort,
}

impl AdiDigitalIn {
    /// Create a digital input from an ADI port.
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::DigitalIn);

        Self { port }
    }

    /// Gets the current logic level of a digital input pin.
    pub fn level(&self) -> Result<LogicLevel, PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        let value =
            unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }

    /// Returns `true` if the digital input's logic level level is [`LogicLevel::High`].
    pub fn is_high(&self) -> Result<bool, PortError> {
        Ok(self.level()?.is_high())
    }

    /// Returns `true` if the digital input's logic level level is [`LogicLevel::Low`].
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

/// Generic digital output ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiDigitalOut {
    port: AdiPort,
}

impl AdiDigitalOut {
    /// Create a digital output from an [`AdiPort`].
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::DigitalOut);

        Self { port }
    }

    /// Sets the digital logic level (high or low) of a pin.
    pub fn set_level(&mut self, level: LogicLevel) -> Result<(), PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        unsafe {
            vexDeviceAdiValueSet(
                self.port.device_handle(),
                self.port.index(),
                level.is_high() as i32,
            );
        }

        Ok(())
    }

    /// Set the digital logic level to [`LogicLevel::High`]. Analagous to
    /// [`Self::set_level(LogicLevel::High)`].
    pub fn set_high(&mut self) -> Result<(), PortError> {
        self.set_level(LogicLevel::High)
    }

    /// Set the digital logic level to [`LogicLevel::Low`]. Analagous to
    /// [`Self::set_level(LogicLevel::Low)`].
    pub fn set_low(&mut self) -> Result<(), PortError> {
        self.set_level(LogicLevel::Low)
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
