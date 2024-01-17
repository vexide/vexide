use pros_sys::PROS_ERR;

use super::{AdiError, AdiPort, AdiDeviceType, AdiDevice};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiDigitalIn {
    port: AdiPort,
}

impl AdiDigitalIn {
    /// Create an AdiDigitalIn.
    pub fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Gets the current value of a digital input pin.
    pub fn new_press(&mut self) -> Result<bool, AdiError> {
        Ok(unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::ext_adi_digital_get_new_press(
                    self.port.internal_expander_index(),
                    self.port.index()
                )
            ) != 0
        })
    }

    /// Gets the current value of a digital input pin.
    pub fn value(&self) -> Result<bool, AdiError> {
        Ok(unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::ext_adi_digital_read(
                    self.port.internal_expander_index(),
                    self.port.index()
                )
            ) != 0
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

#[derive(Debug, Eq, PartialEq)]
pub struct AdiDigitalOut {
    port: AdiPort,
}

impl AdiDigitalOut {
    /// Create an AdiDigitalOut.
    pub fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Sets the digital value (1 or 0) of a pin.
    pub fn set_value(&mut self, value: bool) -> Result<i32, AdiError> {
        Ok(unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::ext_adi_digital_write(
                    self.port.internal_expander_index(),
                    self.port.index(),
                    value
                )
            )
        })
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