use pros_sys::PROS_ERR;

use super::{AdiDevice, AdiDeviceType, AdiDigitalIn, AdiError, AdiPort, digital::LogicLevel};
use crate::error::bail_on;

/// Generic digital input ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiSwitch {
    port: AdiPort,
}

impl AdiSwitch {
    /// Create a digital input from an ADI port.
    pub const fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Gets the current logic level of a digital switch.
    pub fn level(&self) -> Result<LogicLevel, AdiError> {
        let value = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_digital_read(self.port.internal_expander_index(), self.port.index())
        }) != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }

	pub fn pressed(&self) -> Result<bool, AdiError> {
		Ok(self.level()?.is_high())
	}

    pub fn pressed_again(&mut self) -> Result<bool, AdiError> {
		Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_digital_get_new_press(self.port.internal_expander_index(), self.port.index())
        }) != 0)
	}
}

impl From<AdiDigitalIn> for AdiSwitch {
    fn from(device: AdiDigitalIn) -> Self {
        Self {
            port: unsafe { AdiPort::new(device.port_index(), device.expander_port_index()) },
        }
    }
}

impl AdiDevice for AdiSwitch {
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
