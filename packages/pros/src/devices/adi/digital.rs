//! Digital input and output ADI devices

use pros_sys::PROS_ERR;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicLevel {
    High,
    Low,
}

impl LogicLevel {
    pub fn is_high(&self) -> bool {
        match self {
            Self::High => true,
            Self::Low => false,
        }
    }

    pub fn is_low(&self) -> bool {
        !self.is_high()
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
    pub const fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Gets the current logic level of a digital input pin.
    pub fn level(&self) -> Result<LogicLevel, AdiError> {
        let value = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_digital_read(self.port.internal_expander_index(), self.port.index())
        }) != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }
}

impl AdiDevice for AdiDigitalIn {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalIn
    }
}

/// Generic digital output ADI device.
#[derive(Debug, Eq, PartialEq)]
/// Generic digital output ADI device.
pub struct AdiDigitalOut {
    port: AdiPort,
}

impl AdiDigitalOut {
    /// Create a digital output from an [`AdiPort`].
    pub const fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Sets the digital logic level (high or low) of a pin.
    pub fn set_level(&mut self, level: LogicLevel) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_digital_write(
                self.port.internal_expander_index(),
                self.port.index(),
                level.is_high(),
            )
        });

        Ok(())
    }

    pub fn set_high(&mut self) -> Result<(), AdiError> {
        self.set_level(LogicLevel::High)
    }

    pub fn set_low(&mut self) -> Result<(), AdiError> {
        self.set_level(LogicLevel::Low)
    }
}

impl AdiDevice for AdiDigitalOut {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalOut
    }
}
