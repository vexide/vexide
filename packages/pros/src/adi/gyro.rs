use crate::adi::{
    AdiError,
    AdiSlot
};

use core::ffi::c_double;

use pros_sys::PROS_ERR;

use crate::error::bail_on;

pub struct AdiGyro {
    port: u8,
    reference: i32
}

impl AdiGyro {
    /// Create an AdiGyro without checking if it is valid.
    ///
    /// # Safety
    ///
    /// The port must be above 0 and below [`pros_sys::NUM_ADI_PORTS`].
    pub fn new_unchecked(port: AdiSlot, multiplier: c_double) -> Self {
        unsafe {
            Self {
                port: port as u8,
                reference: pros_sys::adi_gyro_init(port as u8, multiplier)
            }
        }
    }
    
    /// Create an AdiGyro, panicking if the port is invalid.
    pub unsafe fn new_raw(port: AdiSlot, multiplier: c_double) -> Self {
        Self::new(port, multiplier).unwrap()
    }

    /// Create an AdiGyro, returning err `AdiError::InvalidPort` if the port is invalid.
    pub unsafe fn new(port: AdiSlot, multiplier: c_double) -> Result<Self, AdiError> {
        if {port as u8} < 1 || {port as u8} > {pros_sys::NUM_ADI_PORTS as u8} {
            return Err(AdiError::InvalidPort);
        }
        Ok(Self {
            port: port as u8,
            reference: pros_sys::adi_gyro_init(port as u8, multiplier)
        })
    }

    /// Gets the current gyro angle in tenths of a degree. Unless a multiplier is applied to the gyro, the return value will be a whole number representing the number of degrees of rotation times 10.
    ///
    /// There are 360 degrees in a circle, thus the gyro will return 3600 for one whole rotation.
    pub fn value(&self) -> Result<f64, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_get(self.reference)) })
    }

    /// Gets the current gyro angle in tenths of a degree. Unless a multiplier is applied to the gyro, the return value will be a whole number representing the number of degrees of rotation times 10.
    pub fn reset(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR.into(), pros_sys::adi_gyro_reset(self.reference)) })
    }
}