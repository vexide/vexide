//! Analog input and output ADI devices.

use pros_sys::PROS_ERR;

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
/// Generic analog input ADI device.
pub struct AdiAnalogIn {
    port: AdiPort,
}

impl AdiAnalogIn {
    /// Create a analog input from an ADI port.
    pub const fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Calibrates the analog sensor on the specified channel.
    ///
    /// This method assumes that the true sensor value is
    /// not actively changing at this time and computes an average
    /// from approximately 500 samples, 1 ms apart, for a 0.5 s period of calibration.
    ///
    /// The average value thus calculated is returned and stored for later calls
    /// to the value_calibrated and value_calibrated_hr functions.
    ///
    /// These functions will return the difference between this value and the current
    /// sensor value when called.
    pub fn calibrate(&mut self) -> Result<(), AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_analog_calibrate(
                self.port.internal_expander_index(),
                self.port.index(),
            )
        });

        Ok(())
    }

    /// Reads an analog input channel and returns the 12-bit value.
    ///
    /// The value returned is undefined if the analog pin has been switched to a different mode.
    /// The meaning of the returned value varies depending on the sensor attached.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_analog_read(self.port.internal_expander_index(), self.port.index())
        }))
    }

    /// Reads the calibrated value of an analog input channel.
    ///
    /// The calibrate function must be run first on that channel.
    ///
    /// This function is inappropriate for sensor values intended for integration,
    /// as round-off error can accumulate causing drift over time.
    /// Use value_calbrated_hr instead.
    pub fn value_calibrated(&self) -> Result<i32, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_analog_read_calibrated(
                self.port.internal_expander_index(),
                self.port.index(),
            )
        }))
    }

    /// Reads the calibrated value of an analog input channel 1-8 with enhanced precision.
    ///
    /// The calibrate function must be run first.
    ///
    /// This is intended for integrated sensor values such as gyros and accelerometers
    /// to reduce drift due to round-off, and should not be used on a sensor such as a
    /// line tracker or potentiometer.
    ///
    /// The value returned actually has 16 bits of “precision”,
    /// even though the ADC only reads 12 bits,
    /// so that errors induced by the average value being
    /// between two values come out in the wash when integrated over time.
    ///
    /// Think of the value as the true value times 16.
    pub fn value_calibrated_hr(&self) -> Result<i32, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_analog_read_calibrated_HR(
                self.port.internal_expander_index(),
                self.port.index(),
            )
        }))
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

#[derive(Debug, Eq, PartialEq)]
/// Generic analog output ADI device.
pub struct AdiAnalogOut {
    port: AdiPort,
}

impl AdiAnalogOut {
    /// Create a analog output from an [`AdiPort`].
    pub const fn new(port: AdiPort) -> Self {
        Self { port }
    }

    /// Sets the output for the Analog Output from 0 (0V) to 4095 (5V).
    pub fn set_value(&mut self, value: i32) -> Result<i32, AdiError> {
        Ok(unsafe {
            bail_on! {
                PROS_ERR,
                pros_sys::ext_adi_port_set_value(self.port.internal_expander_index(), self.port.index(), value)
            }
        })
    }
}

impl AdiDevice for AdiAnalogOut {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::AnalogOut
    }
}
