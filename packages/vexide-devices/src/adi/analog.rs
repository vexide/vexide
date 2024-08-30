//! ADI Analog Interfaces
//!
//! # Overview
//!
//! Unlike digital ADI devices which can only report a "high" or "low" state, analog
//! ADI devices may report a wide range of values spanning 0-5 volts. These analog
//! voltages readings are then converted into a digital values using the internal
//! Analog-to-Digital Converter (ADC) in the V5 brain. The brain measures analog input
//! using 12-bit values ranging from 0 (0V) to 4095 (5V).

use vex_sdk::vexDeviceAdiValueGet;

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};

/// The maximum 12-bit analog value returned by the internal
/// analog-to-digital converters on the brain.
pub const ADC_MAX_VALUE: u16 = 4095;

/// Generic analog input ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiAnalogIn {
    port: AdiPort,
}

impl AdiAnalogIn {
    /// Create a analog input from an ADI port.
    pub fn new(port: AdiPort) -> Self {
        // NOTE: Don't care about whether or not the expander is available at this point, since
        // constructors need to be infalliable. We'll ensure that we're the right configuration
        // before calling any other methods.
        port.configure(AdiDeviceType::AnalogIn);

        Self { port }
    }

    /// Reads an analog input channel and returns the 12-bit value.
    ///
    /// # Sensor Compatibility
    ///
    /// The value returned is undefined if the analog pin has been switched to a different mode.
    /// The meaning of the returned value varies depending on the sensor attached.
    pub fn value(&self) -> Result<u16, PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        Ok(unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } as u16)
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
    pub fn voltage(&self) -> Result<f64, PortError> {
        Ok(self.value()? as f64 / (ADC_MAX_VALUE as f64) * 5.0)
    }
}

impl AdiDevice for AdiAnalogIn {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::AnalogIn
    }
}
