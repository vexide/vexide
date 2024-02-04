//! ADI gyro device.

use core::time::Duration;

use pros_core::bail_on;
use pros_sys::{ext_adi_gyro_t, PROS_ERR, PROS_ERR_F};

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};

/// ADI gyro device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiGyro {
    raw: ext_adi_gyro_t,
    port: AdiPort,
}

impl AdiGyro {
    /// The time it takes to calibrate an [`AdiGyro`].
    ///
    /// The theoretical calibration time is 1024ms, but in practice this seemed to be the
    /// actual time that it takes.
    pub const CALIBRATION_TIME: Duration = Duration::from_millis(1300);

    /// Create a new gyro from an [`AdiPort`].
    ///
    /// If the given port has not previously been configured as a gyro, then this
    /// function blocks for a 1300ms calibration period.
    pub fn new(port: AdiPort, multiplier: f64) -> Result<Self, AdiError> {
        let raw = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_gyro_init(port.internal_expander_index(), port.index(), multiplier)
        });

        Ok(Self { raw, port })
    }

    /// Gets the yaw angle of the gyroscope in degrees.
    ///
    /// Unless a multiplier is applied to the gyro, the return value will be a whole
    /// number representing the number of degrees of rotation.
    pub fn angle(&self) -> Result<f64, AdiError> {
        Ok(bail_on!(PROS_ERR_F, unsafe { pros_sys::ext_adi_gyro_get(self.raw) }) / 10.0)
    }

    /// Reset the current gyro angle to zero degrees.
    pub fn zero(&mut self) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe { pros_sys::ext_adi_gyro_reset(self.raw) });
        Ok(())
    }
}

impl AdiDevice for AdiGyro {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::LegacyGyro
    }
}
