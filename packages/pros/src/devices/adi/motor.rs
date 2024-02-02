//! ADI motor device.

use pros_sys::PROS_ERR;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
/// Cortex era motor device.
pub struct AdiMotor {
    port: AdiPort,
}

impl AdiMotor {
    /// Create a new motor from an [`AdiPort`].
    pub const fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Sets the PWM output of the given motor as an i8 from [-127, 127].
    pub fn set_value(&mut self, value: i8) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_motor_set(
                self.port.internal_expander_index(),
                self.port.index(),
                value,
            )
        });
        Ok(())
    }

    /// Returns the last set PWM output of the motor on the given port.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_motor_get(self.port.internal_expander_index(), self.port.index())
        }))
    }

    /// Stops the given motor.
    pub fn stop(&mut self) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_motor_stop(self.port.internal_expander_index(), self.port.index())
        });

        Ok(())
    }
}

impl AdiDevice for AdiMotor {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::LegacyPwm
    }
}
