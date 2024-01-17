use pros_sys::PROS_ERR;

use crate::{
    adi::{AdiError, AdiSlot},
    error::bail_on,
};

pub struct AdiGyro {
    raw: pros_sys::adi_gyro_t,
}

impl AdiGyro {
    /// Create an AdiGyro, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn new(port: AdiSlot, multiplier: f64) -> Result<Self, AdiError> {
        Ok(Self {
            raw: unsafe {
                bail_on!(
                    PROS_ERR.into(),
                    pros_sys::adi_gyro_init(port.index(), multiplier)
                )
            },
        })
    }

    /// Gets the current gyro angle in tenths of a degree. Unless a multiplier is applied to the gyro, the return value will be a whole number representing the number of degrees of rotation times 10.
    ///
    /// There are 360 degrees in a circle, thus the gyro will return 3600 for one whole rotation.
    pub fn value(&self) -> Result<f64, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_get(self.raw)) })
    }

    /// Gets the current gyro angle in tenths of a degree. Unless a multiplier is applied to the gyro, the return value will be a whole number representing the number of degrees of rotation times 10.
    pub fn zero(&mut self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_reset(self.raw)) })
    }
}
