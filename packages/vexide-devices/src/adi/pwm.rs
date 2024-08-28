//! ADI Pulse-width modulation (PWM).

use vex_sdk::vexDeviceAdiValueSet;

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Generic PWM output ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiPwmOut {
    port: AdiPort,
}

impl AdiPwmOut {
    /// Create a pwm output from an [`AdiPort`].
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::PwmOut);

        Self { port }
    }

    /// Sets the PWM output width.
    ///
    /// This value is sent over 16ms periods with pulse widths ranging from roughly
    /// 0.94mS to 2.03mS.
    pub fn set_output(&mut self, value: u8) -> Result<(), PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        unsafe {
            vexDeviceAdiValueSet(self.port.device_handle(), self.port.index(), value as i32);
        }

        Ok(())
    }
}

impl AdiDevice for AdiPwmOut {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::PwmOut
    }
}
