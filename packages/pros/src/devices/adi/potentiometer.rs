//! ADI Potentiometer device.

use pros_sys::{adi_potentiometer_type_e_t, ext_adi_potentiometer_t, PROS_ERR, PROS_ERR_F};

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
/// Analog potentiometer ADI device.
pub struct AdiPotentiometer {
    potentiometer_type: AdiPotentiometerType,
    raw: ext_adi_potentiometer_t,
    port: AdiPort,
}

impl AdiPotentiometer {
    /// Create a new potentiometer from an [`AdiPort`].
    pub fn new(port: AdiPort, potentiometer_type: AdiPotentiometerType) -> Result<Self, AdiError> {
        let raw = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_potentiometer_init(
                port.internal_expander_index(),
                port.index(),
                potentiometer_type.into(),
            )
        });

        Ok(Self {
            potentiometer_type,
            raw,
            port,
        })
    }

    /// Get the type of ADI potentiometer device.
    pub const fn potentiometer_type(&self) -> AdiPotentiometerType {
        self.potentiometer_type
    }

    /// Gets the current potentiometer angle in tenths of a degree.
    ///
    /// The original potentiometer rotates 250 degrees
    /// thus returning an angle between 0-250 degrees.
    /// Potentiometer V2 rotates 330 degrees
    /// thus returning an angle between 0-330 degrees.
    pub fn angle(&self) -> Result<f64, AdiError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::ext_adi_potentiometer_get_angle(self.raw)
        }))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
/// The type of potentiometer device.
pub enum AdiPotentiometerType {
    /// EDR potentiometer.
    PotentiometerEdr = pros_sys::E_ADI_POT_EDR,
    /// V2 potentiometer.
    PotentiometerV2 = pros_sys::E_ADI_POT_V2,
}

impl From<AdiPotentiometerType> for adi_potentiometer_type_e_t {
    fn from(value: AdiPotentiometerType) -> Self {
        value as _
    }
}

impl AdiDevice for AdiPotentiometer {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::AnalogIn
    }
}
