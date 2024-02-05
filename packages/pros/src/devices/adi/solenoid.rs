use pros_sys::PROS_ERR;

use super::{digital::LogicLevel, AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiSolenoid {
    port: AdiPort,
    level: LogicLevel,
}

impl AdiSolenoid {
    /// Create an AdiSolenoid.
    pub fn new(port: AdiPort) -> Self {
        Self { port, level: LogicLevel::Low }
    }

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

    pub fn level(&self) -> LogicLevel {
        self.level
    }

    pub fn open(&mut self) -> Result<(), AdiError> {
        self.set_level(LogicLevel::High)
    }

    pub fn close(&mut self) -> Result<(), AdiError> {
        self.set_level(LogicLevel::Low)
    }

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
