use crate::adi::{
    AdiError,
    AdiSlot
};

use pros_sys::{
    PROS_ERR,
    adi_ultrasonic_t, ext_adi
};

use crate::error::bail_on;

type ext_adi_u8_tuple_t = (u8, u8);
type ext_adi_port_tuple_t = (AdiSlot, AdiSlot);

pub struct AdiUltrasonic {
    tup: ext_adi_u8_tuple_t,
    reference: adi_ultrasonic_t,
}

impl AdiUltrasonic {
    /// Create an AdiUltrasonic, returning err `AdiError::InvalidPort` if the port is invalid.
    pub unsafe fn new(tup: ext_adi_port_tuple_t) -> Result<Self, AdiError> {
        let port_top = tup.0 as u8;
        let port_bottom = tup.1 as u8;
        if port_top < 1 || port_top > {pros_sys::NUM_ADI_PORTS as u8} {
            return Err(AdiError::InvalidPort);
        }
        if port_bottom < 1 || port_bottom > {pros_sys::NUM_ADI_PORTS as u8} {
            return Err(AdiError::InvalidPort);
        }
        Ok(Self {
            tup: (port_top, port_bottom),
            reference: pros_sys::adi_ultrasonic_init(port_top, port_bottom),
        })
    }

    /// Gets the current ultrasonic sensor value in centimeters.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_get(self.reference)) })
    }

    /// Shut down the ultrasonic sensor.
    ///
    /// # Notices
    /// 
    /// This is not officially a function in the PROS API, however it is in the kernel.
    pub fn shutdown(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_ultrasonic_shutdown(self.reference)) })
    }
}