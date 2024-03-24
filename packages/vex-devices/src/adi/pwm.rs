//! ADI Pulse-width modulation (PWM).

use pros_core::bail_on;
use pros_sys::PROS_ERR;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};

/// Generic PWM output ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiPwmOut {
    port: AdiPort,
}

impl AdiPwmOut {
    /// Create a pwm output from an [`AdiPort`].
    pub fn new(port: AdiPort) -> Result<Self, AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_port_set_config(
                port.internal_expander_index(),
                port.index(),
                pros_sys::E_ADI_ANALOG_OUT,
            )
        });

        Ok(Self { port })
    }

    /// Sets the PWM output width.
    ///
    /// This value is sent over 16ms periods with pulse widths ranging from roughly
    /// 0.94mS to 2.03mS.
    pub fn set_output(&mut self, value: u8) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_port_set_value(
                self.port.internal_expander_index(),
                self.port.index(),
                value as i32,
            )
        });

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
