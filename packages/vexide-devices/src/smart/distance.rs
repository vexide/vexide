//! Distance sensor device.

use snafu::Snafu;
use vex_sdk::{
    vexDeviceDistanceConfidenceGet, vexDeviceDistanceDistanceGet, vexDeviceDistanceObjectSizeGet,
    vexDeviceDistanceObjectVelocityGet, vexDeviceDistanceStatusGet, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

/// A distance sensor plugged into a smart port.
///
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
    /// Creates a new distance sensor from a [`SmartPort`].
    #[must_use]
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

    /// Attempts to detect an object, returning `None` if no object could be found.
    ///
    /// # Errors
    ///
    /// - A [`DistanceError::Port`] error is returned if there is not a distance sensor connected to the port.
    /// - A [`DistanceError::StillInitializing`] error is returned if the distance sensor is still initializing.
    /// - A [`DistanceError::BadStatusCode`] error is returned if the distance sensor has an unknown status code.
    pub fn object(&self) -> Result<Option<DistanceObject>, DistanceError> {
        self.validate()?;

        let distance_raw = unsafe { vexDeviceDistanceDistanceGet(self.device) };

        match distance_raw {
            9999 => Ok(None), // returns 9999 if no object was found
            _ => Ok(Some(DistanceObject {
                distance: distance_raw,
                relative_size: unsafe { vexDeviceDistanceObjectSizeGet(self.device) as u32 },
                velocity: unsafe { vexDeviceDistanceObjectVelocityGet(self.device) },
                // TODO: determine if confidence reading is separate from whether or not an object is detected.
                confidence: f64::from(unsafe { vexDeviceDistanceConfidenceGet(self.device) })
                    / 63.0,
            })),
        }
    }

    /// Returns the internal status code of the distance sensor.
    ///
    /// # Errors
    ///
    /// - A [`DistanceError::Port`] error is returned if there is not a distance sensor connected to the port.
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
impl From<DistanceSensor> for SmartPort {
    fn from(device: DistanceSensor) -> Self {
        device.port
    }
}

/// Readings from a physical object detected by a Distance Sensor.
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct DistanceObject {
    /// The distance of the object from the sensor (in millimeters).
    pub distance: u32,

    /// A guess at the object's "relative size".
    ///
    /// This is a value that has a range of 0 to 400. A 18" x 30" grey card will return
    /// a value of approximately 75 in typical room lighting. If the sensor is not able to
    /// detect an object, None is returned.
    ///
    /// This sensor reading is unusual, as it is entirely unitless with the seemingly arbitrary
    /// range of 0-400 existing due to VEXCode's [`vex::sizeType`] enum having four variants. It's
    /// unknown what the sensor is *actually* measuring here either, so use this data with a grain
    /// of salt.
    ///
    /// [`vex::sizeType`]: https://api.vexcode.cloud/v5/search/sizeType/sizeType/enum
    pub relative_size: u32,

    /// Observed velocity of the object in m/s.
    pub velocity: f64,

    /// Returns the confidence in the distance measurement from 0.0 to 1.0.
    pub confidence: f64,
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using a distance sensor.
pub enum DistanceError {
    /// The sensor's status code is 0x00
    /// Need to wait for the sensor to finish initializing
    StillInitializing,

    /// The sensor has an unknown status code.
    BadStatusCode,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
