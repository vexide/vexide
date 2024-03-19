//! Distance sensor device.
//!
//! Pretty much one to one with the PROS C and CPP API, except Result is used instead of ERRNO values.

use core::ffi::c_double;

use pros_core::{bail_on, error::PortError};
use pros_sys::PROS_ERR;

use super::{SmartDevice, SmartDeviceType, SmartPort};

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

    /// Returns the distance to the object the sensor detects in millimeters.
    pub fn distance(&self) -> Result<u32, PortError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::distance_get(self.port.index())
        }) as u32)
    }

    /// Returns the velocity of the object the sensor detects in m/s
    pub fn velocity(&self) -> Result<f64, PortError> {
        // All VEX Distance Sensor functions return PROS_ERR on failure even though
        // some return floating point values (not PROS_ERR_F)
        Ok(bail_on!(PROS_ERR as c_double, unsafe {
            pros_sys::distance_get_object_velocity(self.port.index())
        }))
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
    pub fn relative_size(&self) -> Result<u32, PortError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::distance_get_object_size(self.port.index())
        }) as u32)
    }

    /// Returns the confidence in the distance measurement from 0.0 to 1.0.
    pub fn distance_confidence(&self) -> Result<f64, PortError> {
        // 0 -> 63
        let confidence = bail_on!(PROS_ERR, unsafe {
            pros_sys::distance_get_confidence(self.port.index())
        }) as f64;

        Ok(confidence / 63.0)
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
