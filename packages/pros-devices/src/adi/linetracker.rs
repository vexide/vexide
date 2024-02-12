//! ADI Line Tracker
//!
//! Line trackers read the difference between a black line and a white surface. They can
//! be used to follow a marked path on the ground.
//!
//! # Overview
//!
//! A line tracker consists of an analog infrared light sensor and an infrared LED.
//! It works by illuminating a surface with infrared light; the sensor then picks up
//! the reflected infrared radiation and, based on its intensity, determines the
//! reflectivity of the surface in question. White surfaces will reflect more light
//! than dark surfaces, resulting in their appearing brighter to the sensor. This
//! allows the sensor to detect a dark line on a white background, or a white line on
//! a dark background.
//!
//! # Hardware
//!
//! The Line Tracking Sensor is an analog sensor, and it internally measures values in the
//! range of 0 to 4095 from 0-5V. Darker objects reflect less light, and are indicated by
//! higher numbers. Lighter objects reflect more light, and are indicated by lower numbers.
//!
//! For best results when using the Line Tracking Sensors, it is best to mount the sensors
//! between 1/8 and 1/4 of an inch away from the surface it is measuring. It is also important
//! to keep lighting in the room consistent, so sensors' readings remain accurate.

use pros_core::bail_on;
use pros_sys::PROS_ERR;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};

/// Analog line tracker device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiLineTracker {
    port: AdiPort,
}

impl AdiLineTracker {
    /// Create a line tracker on an ADI port.
    pub fn new(port: AdiPort) -> Result<Self, AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_port_set_config(
                port.internal_expander_index(),
                port.index(),
                pros_sys::E_ADI_ANALOG_IN,
            )
        });

        Ok(Self { port })
    }

    /// Get the reflectivity factor measured by the sensor.
    ///
    /// This is returned as a value ranging from [0.0, 1.0].
    pub fn reflectivity(&self) -> Result<f64, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_analog_read(self.port.internal_expander_index(), self.port.index())
        }) as f64
            / 4095.0)
    }

    /// Get the raw reflectivity factor of the sensor.
    ///
    /// This is a raw 12-bit value from [0, 4095] representing the voltage level from
    /// 0-5V measured by the V5 brain's ADC.
    pub fn raw_reflectivity(&self) -> Result<u16, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_analog_read(self.port.internal_expander_index(), self.port.index())
        }) as u16)
    }
}

impl AdiDevice for AdiLineTracker {
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
