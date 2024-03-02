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
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::distance_get(self.port.index())) as u32 })
    }

    /// returns the velocity of the object the sensor detects in m/s
    pub fn object_velocity(&self) -> Result<f64, PortError> {
        // all VEX Distance Sensor functions return PROS_ERR on failure even though
        // some return floating point values (not PROS_ERR_F)
        Ok(unsafe {
            bail_on!(
                PROS_ERR as c_double,
                pros_sys::distance_get_object_velocity(self.port.index())
            )
        })
    }

    /// Returns the confidence in the distance measurement from 0% to 100%.
    pub fn distance_confidence(&self) -> Result<f32, PortError> {
        // 0 -> 63
        let confidence = unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::distance_get_confidence(self.port.index())
            )
        } as f32;
        Ok(confidence * 100.0 / 63.0)
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
