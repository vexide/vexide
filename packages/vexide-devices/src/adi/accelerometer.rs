//! ADI Accelerometer device.

use vex_sdk::vexDeviceAdiValueGet;

use super::{analog, AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// A single axis connection on the 3-axis analog accelerometer.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiAccelerometer {
    sensitivity: Sensitivity,
    port: AdiPort,
}

impl AdiAccelerometer {
    /// Create a new accelerometer from an [`AdiPort`].
    pub fn new(port: AdiPort, sensitivity: Sensitivity) -> Self {
        port.configure(AdiDeviceType::Accelerometer);

        Self { port, sensitivity }
    }

    /// Get the type of ADI accelerometer device.
    pub fn sensitivity(&self) -> Result<Sensitivity, PortError> {
        // Configuration check not required here since we don't access the SDK.
        self.port.validate_expander()?;

        Ok(self.sensitivity)
    }

    /// Get the maximum acceleration measurement supported by the current [`Sensitivity`] jumper
    /// configuration.
    pub fn max_acceleration(&self) -> Result<f64, PortError> {
        Ok(self.sensitivity()?.max_acceleration())
    }

    /// Gets the current accleration measaurement for this axis in G.
    pub fn acceleration(&self) -> Result<f64, PortError> {
        Ok(
            // Convert 0-4095 to 0-1, then scale to max accel.
            self.raw_acceleration()? as f64 / analog::ADC_MAX_VALUE as f64
                * self.sensitivity.max_acceleration(),
        )
    }

    /// Returns the raw acceleration reading from [0, 4096]. This represents is an ADC-converted
    /// analog input from 0-5V.
    ///
    /// For example, when on high sensitivity a value of `4096` would represent a reading of 6g
    /// ([`Sensitivity::HIGH_MAX_ACCELERATION`]). When on low acceleration, this same value
    /// would instead represent a 2g reading ([`Sensitivity::LOW_MAX_ACCELERATION`]).
    pub fn raw_acceleration(&self) -> Result<u16, PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        Ok(unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } as u16)
    }
}

/// The jumper state of the accelerometer.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Sensitivity {
    /// 0-2g sensitivity
    Low,

    /// 0-6g sensitivity
    High,
}

impl Sensitivity {
    /// Maxmimum acceleration measurement when in low sensitivity mode.
    pub const LOW_MAX_ACCELERATION: f64 = 2.0;

    /// Maxmimum acceleration measurement when in high sensitivity mode.
    pub const HIGH_MAX_ACCELERATION: f64 = 6.0;

    /// Get the maximum acceleration measurement (in G) for this sensitivity.
    pub const fn max_acceleration(&self) -> f64 {
        match self {
            Self::Low => Self::LOW_MAX_ACCELERATION,
            Self::High => Self::HIGH_MAX_ACCELERATION,
        }
    }
}

impl AdiDevice for AdiAccelerometer {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Accelerometer
    }
}
