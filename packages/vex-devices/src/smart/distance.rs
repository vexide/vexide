//! Distance sensor device.
//!
//! Pretty much one to one with the PROS C and CPP API, except Result is used instead of ERRNO values.

use pros_core::error::PortError;
use snafu::Snafu;
use vex_sdk::{
    vexDeviceDistanceConfidenceGet, vexDeviceDistanceDistanceGet, vexDeviceDistanceObjectSizeGet,
    vexDeviceDistanceObjectVelocityGet, vexDeviceDistanceStatusGet,
};

use super::{SmartDevice, SmartDeviceInternal, SmartDeviceType, SmartPort};

/// A physical distance sensor plugged into a port.
/// Distance sensors can only keep track of one object at a time.
#[derive(Debug, Eq, PartialEq)]
pub struct DistanceSensor {
    port: SmartPort,
}

impl DistanceSensor {
    /// Create a new distance sensor from a smart port index.
    pub const fn new(port: SmartPort) -> Self {
        Self { port }
    }

    /// Validates that the sensor is currently connected to its port, and that its status code
    /// is either 0x82 or 0x86.
    ///
    /// It's unknown what these status codes indicate (likely related to port status), but PROS
    /// performs this check in their API, so we will too.
    ///
    /// <https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_distance.c#L20>
    fn validate(&self) -> Result<(), DistanceError> {
        match self.status()? {
            0x82 | 0x86 => Ok(()),
            _ => Err(DistanceError::BadStatusCode),
        }
    }

    /// Returns the distance to the object the sensor detects in millimeters.
    pub fn distance(&self) -> Result<u32, DistanceError> {
        self.validate()?;

        Ok(unsafe { vexDeviceDistanceDistanceGet(self.device_handle()) })
    }

    /// Returns the velocity of the object the sensor detects in m/s
    pub fn velocity(&self) -> Result<f64, DistanceError> {
        self.validate()?;

        Ok(unsafe { vexDeviceDistanceObjectVelocityGet(self.device_handle()) })
    }

    /// Get the current guess at relative "object size".
    ///
    /// This is a value that has a range of 0 to 400. A 18" x 30" grey card will return
    /// a value of approximately 75 in typical room lighting.
    ///
    /// This sensor reading is unusual, as it is entirely unitless with the seemingly arbitrary
    /// range of 0-400 existing due to VEXCode's [`vex::sizeType`] enum having four variants. It's
    /// unknown what the sensor is *actually* measuring here either, so use this data with a grain
    /// of salt.
    ///
    /// [`vex::sizeType`]: https://api.vexcode.cloud/v5/search/sizeType/sizeType/enum
    pub fn relative_size(&self) -> Result<u32, DistanceError> {
        self.validate()?;

        Ok(unsafe { vexDeviceDistanceObjectSizeGet(self.device_handle()) as u32 })
    }

    /// Returns the confidence in the distance measurement from 0.0 to 1.0.
    pub fn distance_confidence(&self) -> Result<f64, DistanceError> {
        self.validate()?;

        Ok(unsafe { vexDeviceDistanceConfidenceGet(self.device_handle()) as u32 } as f64 / 63.0)
    }

    /// Gets the status code of the distance sensor
    pub fn status(&self) -> Result<u32, DistanceError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceDistanceStatusGet(self.device_handle()) })
    }
}

impl SmartDevice for DistanceSensor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Distance
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using a distance sensor.
pub enum DistanceError {
    /// The sensor's status code is not 0x82 or 0x86.
    BadStatusCode,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
