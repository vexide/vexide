//! ADI Light Sensor
//!
//! The Light Sensor is a sensor which uses a photoresistor to measure the intensity of light. It is
//! one of the 3-Wire series sensors. The sensor has a single mounting hole which will allow it to be
//! attached to the robot's structure.

use vex_sdk::vexDeviceAdiValueGet;

use super::{analog, AdiDevice, AdiDeviceType, AdiPort, PortError};

/// ADI Light Sensor
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
    /// 0-%V measured by the V5 brain's ADC.
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
        self.port.configure(self.device_type());

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
