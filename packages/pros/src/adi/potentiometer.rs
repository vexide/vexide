use pros_sys::PROS_ERR;

use crate::{
    adi::{AdiError, AdiSlot},
    error::bail_on,
};

pub struct AdiPotentiometer {
    raw: i32,
}

impl AdiPotentiometer {
    /// Create an AdiPotentiometer, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn new(port: AdiSlot) -> Result<Self, AdiError> {
        unsafe {
            Ok(Self {
                raw: pros_sys::adi_potentiometer_init(port as u8),
            })
        }
    }

    /// Gets the current potentiometer angle in tenths of a degree.
    ///
    /// The original potentiometer rotates 250 degrees
    /// thus returning an angle between 0-250 degrees.
    /// Potentiometer V2 rotates 330 degrees
    /// thus returning an angle between 0-330 degrees.
    pub fn angle(&self) -> Result<f64, AdiError> {
        Ok(unsafe {
            bail_on!(
                PROS_ERR.into(),
                pros_sys::adi_potentiometer_get_angle(self.raw)
            )
        })
    }
}
