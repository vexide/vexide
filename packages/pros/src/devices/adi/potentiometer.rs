use pros_sys::{PROS_ERR, ext_adi_potentiometer_t, adi_potentiometer_type_e_t};

use super::{AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiPotentiometer {
    raw: ext_adi_potentiometer_t,
}

impl AdiPotentiometer {
    /// Create an AdiPotentiometer, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn new(port: AdiPort, potentiometer_type: AdiPotentiometerType) -> Result<Self, AdiError> {
        unsafe {
            Ok(Self {
                raw: pros_sys::ext_adi_potentiometer_init(port.internal_expander_index(), port.index(), potentiometer_type.into()),
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
                pros_sys::ext_adi_potentiometer_get_angle(self.raw)
            )
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum AdiPotentiometerType {
    PotentiometerEdr = pros_sys::E_ADI_POT_EDR,
    PotentiometerV2 = pros_sys::E_ADI_POT_V2,
}

impl From<AdiPotentiometerType> for adi_potentiometer_type_e_t {
    fn from(value: AdiPotentiometerType) -> Self {
        value as _
    }
}