use pros_sys::PROS_ERR;

use crate::error::{bail_on, PortError};

pub struct DistanceSensor {
    port: u8,
}

impl DistanceSensor {
    pub fn new(port: u8) -> Result<Self, PortError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::distance_get(port));
        }

        Ok(Self { port })
    }

    /// Returns the distance to the object the sensor detects in millimeters.
    pub fn distance(&self) -> u32 {
        unsafe { pros_sys::distance_get(self.port) as u32 }
    }

    /// returns the velocity of the object the sensor detects in m/s
    pub fn object_velocity(&self) -> f32 {
        unsafe { pros_sys::distance_get_object_velocity(self.port) as f32 }
    }

    /// Returns the confidence in the distance measurement from 0% to 100%.
    pub fn distance_confidence(&self) -> f32 {
        unsafe { pros_sys::distance_get_confidence(self.port) as f32 * 100.0 / 63.0 }
    }
}
