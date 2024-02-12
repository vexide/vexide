//! ADI Solenoid Pneumatic Control

use super::{digital::LogicLevel, AdiDevice, AdiDeviceType, AdiDigitalOut, AdiError, AdiPort};

/// Digital pneumatic solenoid valve.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiSolenoid {
    digital_out: AdiDigitalOut,
    level: LogicLevel,
}

impl AdiSolenoid {
    /// Create an AdiSolenoid.
    pub fn new(port: AdiPort) -> Result<Self, AdiError> {
        Ok(Self {
            digital_out: AdiDigitalOut::new(port)?,
            level: LogicLevel::Low,
        })
    }

    /// Sets the digital logic level of the solenoid. [`LogicLevel::Low`] will close the solenoid,
    /// and [`LogicLevel::High`] will open it.
    pub fn set_level(&mut self, level: LogicLevel) -> Result<(), AdiError> {
        self.digital_out.set_level(level)?;
        self.level = level;

        Ok(())
    }

    /// Returns the current [`LogicLevel`] of the solenoid's digital output state.
    pub const fn level(&self) -> LogicLevel {
        self.level
    }

    /// Returns `true` if the solenoid is open.
    pub const fn is_open(&self) -> bool {
        self.level.is_high()
    }

    /// Returns `true` if the solenoid is closed.
    pub const fn is_closed(&self) -> bool {
        self.level.is_low()
    }

    /// Open the solenoid, allowing air pressure through the "open" valve.
    pub fn open(&mut self) -> Result<(), AdiError> {
        self.digital_out.set_level(LogicLevel::High)
    }

    /// Close the solenoid.
    ///
    /// - On single-acting solenoids (e.g. SY113-SMO-PM3-F), this will simply block air pressure
    /// through the "open" valve.
    /// - On double-acting solenoids (e.g. SYJ3120-SMO-M3-F), this will block air pressure through
    /// the "open" valve and allow air pressure into the "close" valve.
    pub fn close(&mut self) -> Result<(), AdiError> {
        self.digital_out.set_level(LogicLevel::Low)
    }

    /// Toggle the solenoid's state between open and closed.
    pub fn toggle(&mut self) -> Result<(), AdiError> {
        self.digital_out.set_level(!self.level)
    }
}

impl AdiDevice for AdiSolenoid {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.digital_out.port_index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.digital_out.expander_port_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalOut
    }
}
