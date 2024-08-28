//! Distance sensor device.

use snafu::Snafu;
use vex_sdk::{
    vexDeviceDistanceConfidenceGet, vexDeviceDistanceDistanceGet, vexDeviceDistanceObjectSizeGet,
    vexDeviceDistanceObjectVelocityGet, vexDeviceDistanceStatusGet, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

/// A physical distance sensor plugged into a port.
/// Distance sensors can only keep track of one object at a time.
#[derive(Debug, Eq, PartialEq)]
pub struct DistanceSensor {
    port: SmartPort,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for DistanceSensor {}
unsafe impl Sync for DistanceSensor {}

impl DistanceSensor {
    /// Create a new distance sensor from a smart port index.
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
        }
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
            0x00 => Err(DistanceError::StillInitializing),
            0x82 | 0x86 => Ok(()),
            _ => Err(DistanceError::BadStatusCode),
        }
    }

    /// Returns the distance to the object the sensor detects in millimeters or None if the
    /// distance is out of range.
    pub fn distance(&self) -> Result<Option<u32>, DistanceError> {
        self.validate()?;

        let distance_raw = unsafe { vexDeviceDistanceDistanceGet(self.device) };

        match distance_raw {
            9999 => Ok(None),
            _ => Ok(Some(distance_raw)),
        }
    }

    /// Returns the velocity of the object the sensor detects in m/s
    pub fn velocity(&self) -> Result<f64, DistanceError> {
        self.validate()?;

        Ok(unsafe { vexDeviceDistanceObjectVelocityGet(self.device) })
    }

    /// Get the current guess at relative "object size".
    ///
    /// This is a value that has a range of 0 to 400. A 18" x 30" grey card will return
    /// a value of approximately 75 in typical room lighting. If the sensor is not able to detect an object, None is returned.
    ///
    /// This sensor reading is unusual, as it is entirely unitless with the seemingly arbitrary
    /// range of 0-400 existing due to VEXCode's [`vex::sizeType`] enum having four variants. It's
    /// unknown what the sensor is *actually* measuring here either, so use this data with a grain
    /// of salt.
    ///
    /// [`vex::sizeType`]: https://api.vexcode.cloud/v5/search/sizeType/sizeType/enum
    pub fn relative_size(&self) -> Result<Option<u32>, DistanceError> {
        self.validate()?;

        let size = unsafe { vexDeviceDistanceObjectSizeGet(self.device) as i32 };
        if size >= 0 {
            Ok(Some(size as u32))
        } else {
            Ok(None)
        }
    }

    /// Returns the confidence in the distance measurement from 0.0 to 1.0.
    pub fn distance_confidence(&self) -> Result<f64, DistanceError> {
        self.validate()?;

        Ok(unsafe { vexDeviceDistanceConfidenceGet(self.device) as u32 } as f64 / 63.0)
    }

    /// Gets the status code of the distance sensor
    pub fn status(&self) -> Result<u32, DistanceError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceDistanceStatusGet(self.device) })
    }
}

impl SmartDevice for DistanceSensor {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Distance
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using a distance sensor.
pub enum DistanceError {
    /// The sensor's status code is 0x00
    /// Need to wait for the sensor to finish initializing
    StillInitializing,

    /// The sensor's status code is not 0x82 or 0x86.
    BadStatusCode,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
