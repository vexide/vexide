//! ADI Potentiometer
//!
//! This module provides an interface for interacting with VEX's ADI potentiometers.
//!
//! # Hardware Overview
//!
//! Potentiometers are analog sensors that measure angular position. They function as
//! variable resistors that change their resistance based on the angular position of
//! their shaft.
//!
//! VEX offers two variants:
//!
//! - Legacy (EDR) Potentiometer: Provides measurements across a 250-degree range.
//! - V2 Potentiometer: Provides measurements across a 330-degree range.
//!
//! Both variants connect to the ADI ports and provide analog signals that are converted
//! to measurements of a shaft's angle.
//!
//! # Comparison to [`AdiEncoder`](super::encoder::AdiEncoder)
//!
//! Potentiometers are fundamentally *analog* sensors. They directly output a measurement
//! of their electrical resistance to the ADI port. The more a shaft rotates along a
//! conductive material inside of them, the higher the reported angle.
//!
//! With this in mind, this means that potentiometers are capable of measuring absolute
//! position at *all times*, even after they have lost power. Encoders on the other hand
//! can only track *changes in position* as a digital signal, meaning that any changes in
//! rotation under an encoder can only be recorded while the encoder is plugged in and
//! being read.
//!
//! # Comparison to [`RotationSensor`](crate::smart::rotation::RotationSensor)
//!
//! Rotation sensors operate similarly to a potentiometer, in that they know their absolute
//! angle at all times (even when being powered off). This is achieved through a hall-effect
//! sensor rather than a conductive material, however. Rotation sensors can also measure their
//! position along with their angle, similar to how an encoder can. They also have a full range
//! of motion and can track angle/position in a full 360-degree range. Potentiometers use ADI
//! ports while Rotation Sensors use Smart ports.

use vex_sdk::vexDeviceAdiValueGet;

use super::{analog, AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// Potentiometer
#[derive(Debug, Eq, PartialEq)]
pub struct AdiPotentiometer {
    potentiometer_type: PotentiometerType,
    port: AdiPort,
}

impl AdiPotentiometer {
    /// Create a new potentiometer from an [`AdiPort`].
    ///
    /// # Example
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let potentiometer = AdiPotentiometer::new(peripherals.adi_a, PotentiometerType::V2);
    ///     loop {
    ///         let angle = potentiometer.angle().expect("Failed to read potentiometer angle");
    ///         println!("Potentiometer Angle: {}", angle);
    ///         sleep(Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
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

    /// Returns the type of ADI potentiometer device.
    ///
    /// This is either the legacy EDR potentiometer or the V5-era potentiometer V2.
    #[must_use]
    pub const fn potentiometer_type(&self) -> PotentiometerType {
        self.potentiometer_type
    }

    /// Returns the maximum angle measurement (in degrees) for the given [`PotentiometerType`].
    #[must_use]
    pub const fn max_angle(&self) -> f64 {
        self.potentiometer_type().max_angle()
    }

    /// Returns the current potentiometer angle in degrees.
    ///
    /// The original potentiometer rotates 250 degrees thus returning an angle between 0-250 degrees.
    /// Potentiometer V2 rotates 330 degrees thus returning an angle between 0-330 degrees.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    ///
    /// # Example
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let potentiometer = AdiPotentiometer::new(peripherals.adi_a, PotentiometerType::V2);
    ///     loop {
    ///         let angle = potentiometer.angle().expect("Failed to read potentiometer angle");
    ///         println!("Potentiometer Angle: {}", angle);
    ///         sleep(Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
    pub fn angle(&self) -> Result<f64, PortError> {
        self.port.validate_expander()?;

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
    /// Maximum angle for the older cortex-era EDR potentiometer.
    pub const LEGACY_MAX_ANGLE: f64 = 250.0;

    /// Maximum angle for the V5-era potentiometer V2.
    pub const V2_MAX_ANGLE: f64 = 333.0;

    /// Returns the maximum angle measurement (in degrees) for this potentiometer type.
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
