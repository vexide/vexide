use pros_sys::{adi_ultrasonic_t, PROS_ERR};

use crate::{
    adi::{AdiError, AdiSlot},
    error::bail_on,
};

pub struct AdiUltrasonic {
    raw: adi_ultrasonic_t,
}

impl AdiUltrasonic {
    /// Create an AdiUltrasonic, returning err `AdiError::InvalidPort` if the port is invalid.
    pub unsafe fn new(ports: (AdiSlot, AdiSlot)) -> Result<Self, AdiError> {
        let port_top = ports.0.index();
        let port_bottom = ports.1.index();
        Ok(Self {
            raw: pros_sys::adi_ultrasonic_init(port_top, port_bottom),
        })
    }

    /// Gets the current ultrasonic sensor value in centimeters.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_get(self.raw)) })
    }

    /// Shut down the ultrasonic sensor.
    ///
    /// # Notices
    ///
    /// This is not officially a function in the PROS API, however it is in the kernel.
    pub fn shutdown(&mut self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_shutdown(self.raw)) })
    }
}
