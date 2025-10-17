//! Distance Sensor
//!
//! This module provides an interface to interact with the VEX V5 Distance Sensor,
//! which uses a Class 1 laser to measure the distance, object size classification, and
//! relative velocity of a single object.
//!
//! # Hardware Overview
//!
//! The sensor uses a narrow-beam Class 1 laser (similar to phone proximity sensors)
//! for precise detection. It measures distances from 20mm to 2000mm with
//! varying accuracy (±15mm below 200mm, ±5% above 200mm).
//!
//! The sensor can classify detected objects by relative size, helping
//! distinguish between walls and field elements. It also measures the relative approach
//! velocity between the sensor and target.
//!
//! Due to the use of a laser, measurements are single-point and highly directional,
//! meaning that objects will only be detected when they are directly in front of the
//! sensor's field of view.
//!
//! Like all other Smart devices, VEXos will process sensor updates every 10mS.

use snafu::Snafu;
use vex_sdk::{
    vexDeviceDistanceConfidenceGet, vexDeviceDistanceDistanceGet, vexDeviceDistanceObjectSizeGet,
    vexDeviceDistanceObjectVelocityGet, vexDeviceDistanceStatusGet, V5_DeviceT,
};

use super::{PortError, SmartDevice, SmartDeviceType, SmartPort};

/// A distance sensor plugged into a Smart Port.
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
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = DistanceSensor::new(peripherals.port_1);
    /// }
    /// ```
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
    fn validate(&self) -> Result<(), DistanceObjectError> {
        match self.status()? {
            0x00 => StillInitializingSnafu.fail(),
            0x82 | 0x86 => Ok(()),
            code => BadStatusCodeSnafu { code }.fail(),
        }
    }

    /// Attempts to detect an object, returning `None` if no object could be found.
    ///
    /// # Errors
    ///
    /// - A [`DistanceObjectError::Port`] error is returned if there was not a sensor connected to the port.
    /// - A [`DistanceObjectError::StillInitializing`] error is returned if the distance sensor is still initializing.
    /// - A [`DistanceObjectError::BadStatusCode`] error is returned if the distance sensor has an unknown status code.
    ///
    /// # Examples
    ///
    /// Measure object distance and velocity:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = DistanceSensor::new(peripherals.port_1);
    ///
    ///     if let Some(object) = sensor.object().unwrap_or_default() {
    ///         println!("Object of size {}mm is moving at {}m/s", object.distance, object.velocity);
    ///     }
    /// }
    /// ```
    ///
    /// Get object distance, but only with high confidence:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = DistanceSensor::new(peripherals.port_1);
    ///
    ///     let distance = sensor.object()
    ///         .unwrap_or_default()
    ///         .and_then(|object| {
    ///             if object.confidence > 0.8 {
    ///                 Some(object.distance)
    ///             } else {
    ///                 None
    ///             }
    ///         });
    /// }
    /// ```
    pub fn object(&self) -> Result<Option<DistanceObject>, DistanceObjectError> {
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
    /// The status code of the signature can tell you if the sensor is still initializing or if it is working correctly.
    /// If the distance sensor is still initializing, the status code will be 0x00.
    /// If it is done initializing and functioning correctly, the status code will be 0x82 or 0x86.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// A simple initialization state check:
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = DistanceSensor::new(peripherals.port_1);
    ///     loop {
    ///         if let Ok(0) = sensor.status() {
    ///             println!("Sensor is still initializing");
    ///         } else {
    ///             println!("Sensor is ready");
    ///         }
    ///         sleep(Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
    ///
    /// Printing the status code in binary format:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = DistanceSensor::new(peripherals.port_1);
    ///
    ///     if let Ok(status) = sensor.status() {
    ///         println!("Status: {:b}", status);
    ///     }
    /// }
    /// ```
    pub fn status(&self) -> Result<u32, PortError> {
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

/// Errors that can occur when using a distance sensor.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum DistanceObjectError {
    /// The sensor's status code is 0x00
    /// Need to wait for the sensor to finish initializing
    StillInitializing,

    /// The sensor has an unknown status code.
    #[snafu(display("The sensor has an unknown status code (0x{code:x?})."))]
    BadStatusCode {
        /// The status code returned by the sensor.
        code: u32,
    },

    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
