use core::panic;

use pros_sys::PROS_ERR;

use super::{AdiError, AdiPort, AdiDevice, AdiDeviceType};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiMotor {
    port: AdiPort,
}

impl AdiMotor {
    /// Create an [`AdiMotor`].
    pub fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Sets the speed of the given motor.
    pub fn set_value(&mut self, value: i8) -> Result<i32, AdiError> {
        Ok(unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::ext_adi_motor_set(
                    self.port.internal_expander_index(),
                    self.port.index(),
                    value
                )
            )
        })
    }

    /// Returns the last set speed of the motor on the given port.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::ext_adi_motor_get(self.port.internal_expander_index(), self.port.index())
            )
        })
    }

    /// Stops the given motor.
    pub fn stop(&mut self) -> Result<i32, AdiError> {
        Ok(unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::ext_adi_motor_stop(
                    self.port.internal_expander_index(),
                    self.port.index()
                )
            )
        })
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