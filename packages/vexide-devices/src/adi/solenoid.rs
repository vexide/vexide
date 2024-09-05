//! ADI Solenoid Pneumatic Control

use vex_sdk::vexDeviceAdiValueSet;

use super::{digital::LogicLevel, AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// Digital pneumatic solenoid valve.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiSolenoid {
    port: AdiPort,
    level: LogicLevel,
}

impl AdiSolenoid {
    /// Create an AdiSolenoid.
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::DigitalOut);

        Self {
            port,
            level: LogicLevel::Low,
        }
    }

    /// Sets the digital logic level of the solenoid. [`LogicLevel::Low`] will close the solenoid,
    /// and [`LogicLevel::High`] will open it.
    pub fn set_level(&mut self, level: LogicLevel) -> Result<(), PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        self.level = level;

        unsafe {
            vexDeviceAdiValueSet(
                self.port.device_handle(),
                self.port.index(),
                level.is_high() as i32,
            );
        }

        Ok(())
    }

    /// Returns the current [`LogicLevel`] of the solenoid's digital output state.
    pub const fn level(&self) -> LogicLevel {
        self.level
    }

    /// Returns `true` if the solenoid is open.
    pub const fn is_open(&self) -> LogicLevel {
        self.level
    }

    /// Returns `true` if the solenoid is closed.
    pub const fn is_closed(&self) -> LogicLevel {
        self.level
    }

    /// Open the solenoid, allowing air pressure through the "open" valve.
    pub fn open(&mut self) -> Result<(), PortError> {
        self.set_level(LogicLevel::High)
    }

    /// Close the solenoid.
    ///
    /// - On single-acting solenoids (e.g. SY113-SMO-PM3-F), this will simply block air pressure
    /// through the "open" valve.
    /// - On double-acting solenoids (e.g. SYJ3120-SMO-M3-F), this will block air pressure through
    /// the "open" valve and allow air pressure into the "close" valve.
    pub fn close(&mut self) -> Result<(), PortError> {
        self.set_level(LogicLevel::Low)
    }

    /// Toggle the solenoid's state between open and closed.
    pub fn toggle(&mut self) -> Result<(), PortError> {
        self.set_level(!self.level)
    }
}

impl AdiDevice for AdiSolenoid {
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
