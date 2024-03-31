//! ADI Analog Interfaces
//!
//! # Overview
//!
//! Unlike digital ADI devices which can only report a "high" or "low" state, analog
//! ADI devices may report a wide range of values spanning 0-5 volts. These analog
//! voltages readings are then converted into a digital values using the internal
//! Analog-to-Digital Converter (ADC) in the V5 brain. The brain measures analog input
//! using 12-bit values ranging from 0 (0V) to 4095 (5V).

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use vex_sdk::vexDeviceAdiValueGet;

/// Generic analog input ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiAnalogIn {
    port: AdiPort,
}

impl AdiAnalogIn {
    /// Create a analog input from an ADI port.
    pub fn new(mut port: AdiPort) -> Result<Self, AdiError> {
        port.configure(AdiDeviceType::AnalogIn)?;

        Ok(Self { port })
    }

    /// Reads an analog input channel and returns the 12-bit value.
    ///
    /// # Sensor Compatibility
    ///
    /// The value returned is undefined if the analog pin has been switched to a different mode.
    /// The meaning of the returned value varies depending on the sensor attached.
    pub fn value(&self) -> Result<u16, AdiError> {
        self.port.validate_expander()?;

        Ok(unsafe {
            vexDeviceAdiValueGet(self.port.device_handle(), self.port.internal_index())
        } as u16)
    }

    /// Reads an analog input channel and returns the calculated voltage input (0-5V).
    ///
    /// # Precision
    ///
    /// This function has a precision of `5.0/4095.0` volts, as ADC reports 12-bit voltage data
    /// on a scale of 0-4095.
    ///
    /// # Sensor Compatibility
    ///
    /// The value returned is undefined if the analog pin has been switched to a different mode.
    /// The meaning of the returned value varies depending on the sensor attached.
    pub fn voltage(&self) -> Result<f64, AdiError> {
        Ok(self.value()? as f64 / 4095.0 * 5.0)
    }
}

impl AdiDevice for AdiAnalogIn {
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
