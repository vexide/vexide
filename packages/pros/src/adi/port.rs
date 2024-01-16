
use core::{
    ffi::c_int,
    ops::{Deref, DerefMut},
};

use crate::adi::{
    AdiError,
    AdiSlot
};

use crate::error::bail_on;

use pros_sys::{PROS_ERR, adi_port_config_e_t, E_ADI_DIGITAL_IN, E_ADI_ANALOG_OUT, E_ADI_DIGITAL_OUT, E_ADI_ANALOG_IN, E_ADI_LEGACY_ENCODER, E_ADI_LEGACY_ULTRASONIC };

pub struct AdiPort(u8);

impl AdiPort {
    /// Create an AdiPort, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn new(slot: AdiSlot) -> Option<Self> {
        let port = slot as u8;
        if c_int::from(port) < pros_sys::NUM_ADI_PORTS && c_int::from(port) > 0 {
            Some(Self(port))
        } else {
            None
        }
    }

    /// Sets the value for the given ADI port
    /// 
    /// This only works on ports configured as outputs, and the behavior will change depending on the configuration of the port.
    pub fn set_value(&mut self, value: i32) -> Result<i32, AdiError> {
        Ok(bail_on! {
            PROS_ERR,
            unsafe { pros_sys::adi_port_set_value(self.0, value) }
        })
    }

    /// Gets the current ultrasonic sensor value in centimeters.
    ///
    /// If no object was found, zero is returned. If the ultrasonic sensor was never started, the return value is PROS_ERR. Round and fluffy objects can cause inaccurate values to be returned.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_port_get_value(self.0)) })
    }

    /// Attempts to set the configuration for the given ADI port.
    pub fn try_set_config(port: u8, config: adi_port_config_e_t) -> Result<i32, AdiError> {
        if config == E_ADI_DIGITAL_IN || config == E_ADI_ANALOG_OUT || config == E_ADI_DIGITAL_OUT || config == E_ADI_ANALOG_IN || config == E_ADI_LEGACY_ENCODER || config == E_ADI_LEGACY_ULTRASONIC {
            Ok(bail_on! {
                PROS_ERR,
                unsafe { pros_sys::adi_port_set_config(port, config) }
            })
        } else {
            Err(AdiError::InvalidConfigType)
        }
    }

    /// Configures an ADI port to act as a given sensor type.
    pub fn set_config(&mut self, config: adi_port_config_e_t) -> Result<i32, AdiError> {
        Ok(bail_on! {
            PROS_ERR,
            unsafe { pros_sys::adi_port_set_config(self.0, config) }
        })
    }

    /// Returns the configuration for the given ADI port.
    pub fn config(&self) -> Result<adi_port_config_e_t, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_port_get_config(self.0)) })
    }
}

impl Deref for AdiPort {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AdiPort {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}