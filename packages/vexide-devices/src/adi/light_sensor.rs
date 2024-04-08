//! ADI Light Sensor
//!
//! The Light Sensor is a sensor which uses a photoresistor to measure the intensity of light. It is
//! one of the 3-Wire series sensors. The sensor has a single mounting holewhich will allow it to be
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
    pub fn new(mut port: AdiPort) -> Result<Self, PortError> {
        port.configure(AdiDeviceType::LightSensor)?;

        Ok(Self { port })
    }

    /// Get the brightness factor of a light source measured by
    /// the sensor.
    ///
    /// This is returned as a value ranging from [0.0, 1.0].
    pub fn brightness(&self) -> Result<f64, PortError> {
        Ok(self.raw_brightness()? as f64 / analog::ADC_MAX_VALUE as f64)
    }

    /// Get the 12-bit brightness reading of the sensor.
    ///
    /// This is a raw 12-bit value from [0, 4095] representing the voltage level from
    /// 5-0V measured by the V5 brain's ADC.
    pub fn raw_brightness(&self) -> Result<u16, PortError> {
        self.port.validate_expander()?;

        // Voltage is normally low at higher brightness, so we invert this to make more brightness = higher value.
        Ok(analog::ADC_MAX_VALUE
            - unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.internal_index()) }
                as u16)
    }
}

impl AdiDevice for AdiLightSensor {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::LightSensor
    }
}
