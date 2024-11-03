//! ADI Motor Controller

use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// Cortex-era Motor Controller
#[derive(Debug, Eq, PartialEq)]
pub struct AdiMotor {
    port: AdiPort,
    slew: bool,
}

impl AdiMotor {
    /// Create a new motor from an [`AdiPort`].
    ///
    /// Motors can be optionally configured to use slew rate control to prevent the internal
    /// PTC from tripping on older cortex-era 393 motors.
    #[must_use]
    pub fn new(port: AdiPort, slew: bool) -> Self {
        port.configure(match slew {
            false => AdiDeviceType::Motor,
            true => AdiDeviceType::MotorSlew,
        });

        Self { port, slew }
    }

    /// Sets the PWM output of the given motor to a floating point number in the range \[-1.0, 1.0\].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn set_output(&mut self, value: f64) -> Result<(), PortError> {
        self.set_raw_output((value * 127.0) as i8)
    }

    /// Sets the PWM output of the given motor as an integer in the range \[-127, 127\].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn set_raw_output(&mut self, pwm: i8) -> Result<(), PortError> {
        self.port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(self.port.device_handle(), self.port.index(), i32::from(pwm));
        }

        Ok(())
    }

    /// Returns the last set PWM output of the motor on the given port as a floating point
    /// number in the range \[-1.0, 1.0\].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn output(&self) -> Result<f64, PortError> {
        Ok(f64::from(self.raw_output()?) / f64::from(i8::MAX))
    }

    /// Returns the last set PWM output of the motor on the given port as an integer in the range \[-127, 127\].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn raw_output(&self) -> Result<i8, PortError> {
        self.port.validate_expander()?;

        Ok(
            // TODO:
            // Subtracting i8::MAX from this value comes from this line in the PROS kernel:
            //
            // https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_ext_adi.c#L269
            //
            // Presumably this happens because the legacy motor device types return out of 256 (u8)
            // in the getter, but have an i8 setter. This needs hardware testing, though.
            (unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) }
                - i32::from(i8::MAX)) as i8,
        )
    }

    /// Stops the given motor by setting its output to zero.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
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
