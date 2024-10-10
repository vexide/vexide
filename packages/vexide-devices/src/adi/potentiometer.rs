//! ADI Potentiometer device.

use core::{f64::consts::TAU, marker::PhantomData};

use uom::si::f64::Angle;
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
    pub fn new(port: AdiPort, potentiometer_type: PotentiometerType) -> Self {
        port.configure(match potentiometer_type {
            PotentiometerType::Legacy => AdiDeviceType::Potentiometer,
            PotentiometerType::V2 => AdiDeviceType::PotentiometerV2,
        });

        Self {
            port,
            potentiometer_type,
        }
    }

    /// Get the type of ADI potentiometer device.
    pub fn potentiometer_type(&self) -> Result<PotentiometerType, PortError> {
        // Configuration check not necessary since we don't fetch from the SDK.
        self.port.validate_expander()?;

        Ok(self.potentiometer_type)
    }

    /// Get the maximum angle measurement for the given [`PotentiometerType`].
    pub fn max_angle(&self) -> Result<Angle, PortError> {
        Ok(self.potentiometer_type()?.max_angle())
    }

    /// Gets the current potentiometer angle in degrees.
    ///
    /// The original potentiometer rotates 250 degrees
    /// thus returning an angle between 0-250 degrees.
    /// Potentiometer V2 rotates 330 degrees
    /// thus returning an angle between 0-330 degrees.
    pub fn angle(&self) -> Result<Angle, PortError> {
        self.port.validate_expander()?;
        self.port.configure(self.device_type());

        Ok(
            unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } as f64
                * self.potentiometer_type.max_angle()
                / analog::ADC_MAX_VALUE as f64,
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
    pub const LEGACY_MAX_ANGLE: Angle = Angle {
        dimension: PhantomData,
        units: PhantomData,
        value: 250.0 * TAU / 360.0, // we have to convert degrees to radians manually because we're in a const context
    };

    /// Maximum angle for the V5-era potentiometer V2.
    pub const V2_MAX_ANGLE: Angle = Angle {
        dimension: PhantomData,
        units: PhantomData,
        value: 330.0 * TAU / 360.0,
    };

    /// Get the maximum angle measurement (in degrees) for this potentiometer type.
    pub const fn max_angle(&self) -> Angle {
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
