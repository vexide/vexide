//! ADI gyro device.
use pros_sys::{ext_adi_gyro_t, PROS_ERR, PROS_ERR_F};

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
/// ADI gyro device.
pub struct AdiGyro {
    raw: ext_adi_gyro_t,
    port: AdiPort,
}

impl AdiGyro {
    /// Create a new gyro from an [`AdiPort`].
    pub fn new(port: AdiPort, multiplier: f64) -> Result<Self, AdiError> {
        let raw = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_gyro_init(port.internal_expander_index(), port.index(), multiplier)
        });

        Ok(Self { raw, port })
    }

    /// Gets the current gyro angle in tenths of a degree. Unless a multiplier is applied to the gyro, the return value will be a whole number representing the number of degrees of rotation times 10.
    ///
    /// There are 360 degrees in a circle, thus the gyro will return 3600 for one whole rotation.
    pub fn value(&self) -> Result<f64, AdiError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::ext_adi_gyro_get(self.raw)
        }))
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
