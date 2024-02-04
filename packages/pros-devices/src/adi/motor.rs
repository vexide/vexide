//! ADI motor device.

use pros_core::bail_on;
use pros_sys::PROS_ERR;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};

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

    /// Sets the PWM output of the given motor as an f32 from [-1.0, 1.0].
    pub fn set_output(&mut self, value: f32) -> Result<(), AdiError> {
        self.set_raw_output((value * 127.0) as i8)
    }

    /// Sets the PWM output of the given motor as an i8 from [-127, 127].
    pub fn set_raw_output(&mut self, value: i8) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_motor_set(
                self.port.internal_expander_index(),
                self.port.index(),
                value,
            )
        });

        Ok(())
    }

    /// Returns the last set PWM output of the motor on the given port as an f32 from [-1.0, 1.0].
    pub fn output(&self) -> Result<f32, AdiError> {
        Ok(self.raw_output()? as f32 / 127.0)
    }

    /// Returns the last set PWM output of the motor on the given port as an i8 from [-127, 127].
    pub fn raw_output(&self) -> Result<i8, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_motor_get(self.port.internal_expander_index(), self.port.index())
        }) as i8)
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
