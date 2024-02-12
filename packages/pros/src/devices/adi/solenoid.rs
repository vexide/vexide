//! ADI Solenoid Pneumatic Control

use pros_sys::PROS_ERR;

use super::{digital::LogicLevel, AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

/// Digital pneumatic solenoid valve.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiSolenoid {
    port: AdiPort,
    level: LogicLevel,
}

impl AdiSolenoid {
    /// Create an AdiSolenoid.
    pub const fn new(port: AdiPort) -> Self {
        Self {
            port,
            level: LogicLevel::Low,
        }
    }

    /// Sets the digital logic level of the solenoid. [`LogicLevel::Low`] will close the solenoid,
    /// and [`LogicLevel::High`] will open it.
    pub fn set_level(&mut self, level: LogicLevel) -> Result<(), AdiError> {
        self.level = level;

        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_digital_write(
                self.port.internal_expander_index(),
                self.port.index(),
                level.is_high(),
            )
        });

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
    pub fn open(&mut self) -> Result<(), AdiError> {
        self.set_level(LogicLevel::High)
    }

    /// Close the solenoid.
    ///
    /// - On single-acting solenoids (e.g. SY113-SMO-PM3-F), this will simply block air pressure
    /// through the "open" valve.
    /// - On double-acting solenoids (e.g. SYJ3120-SMO-M3-F), this will block air pressure through
    /// the "open" valve and allow air pressure into the "close" valve.
    pub fn close(&mut self) -> Result<(), AdiError> {
        self.set_level(LogicLevel::Low)
    }

    /// Toggle the solenoid's state between open and closed.
    pub fn toggle(&mut self) -> Result<(), AdiError> {
        self.set_level(!self.level)
    }
}

impl AdiDevice for AdiSolenoid {
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
