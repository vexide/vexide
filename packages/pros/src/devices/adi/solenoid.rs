use pros_sys::PROS_ERR;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiSolenoid {
    port: AdiPort,
    value: bool,
}

impl AdiSolenoid {
    /// Create an AdiSolenoid.
    pub fn new(port: AdiPort) -> Self {
        Self { port, value: false }
    }

    pub fn set_value(&mut self, value: bool) -> Result<i32, AdiError> {
        self.value = true;

        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_digital_write(
                self.port.internal_expander_index(),
                self.port.index(),
                value,
            )
        }))
    }

    pub fn value(&self) -> bool {
        self.value
    }

    pub fn open(&mut self) -> Result<i32, AdiError> {
        self.set_value(true)
    }

    pub fn close(&mut self) -> Result<i32, AdiError> {
        self.set_value(false)
    }

    pub fn toggle(&mut self) -> Result<i32, AdiError> {
        self.set_value(!self.value)
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
