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
    pub fn new(mut port: AdiPort) -> Result<Self, PortError> {
        port.configure(AdiDeviceType::PwmOut)?;

        Ok(Self { port })
    }

    /// Sets the PWM output width.
    ///
    /// This value is sent over 16ms periods with pulse widths ranging from roughly
    /// 0.94mS to 2.03mS.
    pub fn set_output(&mut self, value: u8) -> Result<(), PortError> {
        self.port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(
                self.port.device_handle(),
                self.port.internal_index(),
                value as i32,
            );
        }

        Ok(())
    }
}

impl AdiDevice for AdiPwmOut {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::PwmOut
    }
}
