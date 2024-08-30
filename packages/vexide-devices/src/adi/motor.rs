//! ADI motor device.

use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

#[derive(Debug, Eq, PartialEq)]
/// Cortex era motor device.
pub struct AdiMotor {
    port: AdiPort,
    slew: bool,
}

impl AdiMotor {
    /// Create a new motor from an [`AdiPort`].
    ///
    /// Motors can be optionally configured to use slew rate control to prevent the internal
    /// PTC from tripping on older cortex-era 393 motors.
    pub fn new(port: AdiPort, slew: bool) -> Self {
        port.configure(match slew {
            false => AdiDeviceType::Motor,
            true => AdiDeviceType::MotorSlew,
        });

        Self { port, slew }
    }

    /// Sets the PWM output of the given motor as an f64 from [-1.0, 1.0].
    pub fn set_output(&mut self, value: f64) -> Result<(), PortError> {
        self.set_raw_output((value * 127.0) as i8)
    }

    /// Sets the PWM output of the given motor as an i8 from [-127, 127].
    pub fn set_raw_output(&mut self, pwm: i8) -> Result<(), PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        unsafe {
            vexDeviceAdiValueSet(self.port.device_handle(), self.port.index(), pwm as i32);
        }

        Ok(())
    }

    /// Returns the last set PWM output of the motor on the given port as an f32 from [-1.0, 1.0].
    pub fn output(&self) -> Result<f64, PortError> {
        Ok(self.raw_output()? as f64 / i8::MAX as f64)
    }

    /// Returns the last set PWM output of the motor on the given port as an i8 from [-127, 127].
    pub fn raw_output(&self) -> Result<i8, PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        Ok(
            // TODO:
            // Subtracting i8::MAX from this value comes from this line in the PROS kernel:
            //
            // https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_ext_adi.c#L269
            //
            // Presumably this happens because the legacy motor device types return out of 256 (u8)
            // in the getter, but have an i8 setter. This needs hardware testing, though.
            (unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) }
                - i8::MAX as i32) as i8,
        )
    }

    /// Stops the given motor.
    pub fn stop(&mut self) -> Result<(), PortError> {
        self.set_raw_output(0)
    }
}

impl AdiDevice for AdiMotor {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        match self.slew {
            false => AdiDeviceType::Motor,
            true => AdiDeviceType::MotorSlew,
        }
    }
}
