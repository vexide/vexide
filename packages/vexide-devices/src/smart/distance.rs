//! Distance sensor device.

use snafu::Snafu;
use uom::si::{
    f64::{Length, Velocity},
    length::millimeter,
    velocity::meter_per_second,
};
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

    /// Attempt to detect an object, returning `None` if no object could be found.
    pub fn object(&self) -> Result<Option<DistanceObject>, DistanceError> {
        self.validate()?;

        let distance_raw = unsafe { vexDeviceDistanceDistanceGet(self.device) };

        match distance_raw {
            9999 => Ok(None), // returns 9999 if no object was found
            _ => Ok(Some(DistanceObject {
                distance: Length::new::<millimeter>(distance_raw as _),
                relative_size: unsafe { vexDeviceDistanceObjectSizeGet(self.device) as u32 },
                velocity: Velocity::new::<meter_per_second>(unsafe {
                    vexDeviceDistanceObjectVelocityGet(self.device)
                }),
                // TODO: determine if confidence reading is separate from whether or not an object is detected.
                confidence: unsafe { vexDeviceDistanceConfidenceGet(self.device) as u32 } as f64
                    / 63.0,
            })),
        }
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

/// Readings from a phyiscal object detected by a Distance Sensor.
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct DistanceObject {
    /// The distance of the object from the sensor.
    pub distance: Length,

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

    /// Observed velocity of the object.
    pub velocity: Velocity,

    /// Returns the confidence in the distance measurement from 0.0 to 1.0.
    pub confidence: f64,
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
