//! ADI Potentiometer device.

use vex_sdk::vexDeviceAdiValueGet;

use super::{analog, AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// Analog potentiometer ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiPotentiometer {
    potentiometer_type: PotentiometerType,
    port: AdiPort,
}

impl AdiPotentiometer {
    /// Create a new potentiometer from an [`AdiPort`].
    #[must_use]
    pub fn new(port: AdiPort, potentiometer_type: PotentiometerType) -> Self {
        port.configure(match potentiometer_type {
            PotentiometerType::Legacy => AdiDeviceType::Potentiometer,
            PotentiometerType::V2 => AdiDeviceType::PotentiometerV2,
        });

        Self {
            potentiometer_type,
            port,
        }
    }

    /// Get the type of ADI potentiometer device.
    pub const fn potentiometer_type(&self) -> PotentiometerType {
        self.potentiometer_type
    }

    /// Get the maximum angle measurement (in degrees) for the given [`PotentiometerType`].
    pub const fn max_angle(&self) -> f64 {
        self.potentiometer_type().max_angle()
    }

    /// Gets the current potentiometer angle in degrees.
    ///
    /// The original potentiometer rotates 250 degrees thus returning an angle between 0-250 degrees.
    /// Potentiometer V2 rotates 330 degrees thus returning an angle between 0-330 degrees.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn angle(&self) -> Result<f64, PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        Ok(
            f64::from(unsafe {
                vexDeviceAdiValueGet(self.port.device_handle(), self.port.index())
            }) * self.potentiometer_type.max_angle()
                / f64::from(analog::ADC_MAX_VALUE),
        )
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
/// The type of potentiometer device.
pub enum PotentiometerType {
    /// EDR potentiometer.
    Legacy,

    /// V2 potentiometer.
    V2,
}

impl PotentiometerType {
    /// Maxmimum angle for the older cortex-era EDR potentiometer.
    pub const LEGACY_MAX_ANGLE: f64 = 250.0;

    /// Maximum angle for the V5-era potentiometer V2.
    pub const V2_MAX_ANGLE: f64 = 330.0;

    /// Get the maximum angle measurement (in degrees) for this potentiometer type.
    #[must_use]
    pub const fn max_angle(&self) -> f64 {
        match self {
            Self::Legacy => Self::LEGACY_MAX_ANGLE,
            Self::V2 => Self::V2_MAX_ANGLE,
        }
    }
}

impl AdiDevice for AdiPotentiometer {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        match self.potentiometer_type {
            PotentiometerType::Legacy => AdiDeviceType::Potentiometer,
            PotentiometerType::V2 => AdiDeviceType::PotentiometerV2,
        }
    }
}
