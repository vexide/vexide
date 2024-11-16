//! ADI Light Sensor
//!
//! The Light Sensor measures the intensity of visible light with a photoresistor.
//!
//! # Hardware Overview
//!
//! Using a Cadmium Sulfoselenide photoconductive photocell (CdS), the light sensor
//! is able to adjust its resistance based on the amount of visible light shining on it.
//!
//! The light sensor only measures light in the visible spectrum. It cannot detect
//! infrared or ultraviolet sources.
//!
//! # Effective Range
//!
//! Effective range is dependent on both the intensity of the source and the surrounding
//! enviornment. Darker ambient surroundings with a brighter source will result in a
//! greater effective range.
//!
//! That being said, the sensor generally has a usable range of up to 6 feet, meaning it
//! can distinguish a light source from the surrounding ambient light at up to six feet
//! away. Measurements farther than this might cause the sensor to return inconclusive
//! results or blend into the ambient light.

use vex_sdk::vexDeviceAdiValueGet;

use super::{analog, AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Light Sensor
#[derive(Debug, Eq, PartialEq)]
pub struct AdiLightSensor {
    port: AdiPort,
}

impl AdiLightSensor {
    /// Create a light sensor from an ADI port.
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::LightSensor);

        Self { port }
    }

    /// Returns the brightness factor measured by the sensor. Higher numbers mean
    /// a brighter light source.
    ///
    /// This is returned as a value ranging from [0.0, 1.0].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn brightness(&self) -> Result<f64, PortError> {
        Ok(f64::from(analog::ADC_MAX_VALUE - self.raw_brightness()?)
            / f64::from(analog::ADC_MAX_VALUE))
    }

    /// Returns the 12-bit brightness reading of the sensor.
    ///
    /// This is a raw 12-bit value from [0, 4095] representing the voltage level from
    /// 0-%V measured by the V5 Brain's ADC.
    ///
    /// A low number (less voltage) represents a **brighter** light source.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn raw_brightness(&self) -> Result<u16, PortError> {
        self.port.validate_expander()?;

        Ok(unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } as u16)
    }
}

impl AdiDevice for AdiLightSensor {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::LightSensor
    }
}
