//! Utilites for getting information about the robot's battery.

use pros_sys::{PROS_ERR, PROS_ERR_F};
use snafu::Snafu;

use crate::error::{bail_on, map_errno};

/// Get the robot's battery capacity.
pub fn capacity() -> Result<f64, BatteryError> {
    Ok(bail_on!(PROS_ERR_F, unsafe {
        pros_sys::misc::battery_get_capacity()
    }))
}

/// Get the current temperature of the robot's battery.
pub fn temperature() -> Result<f64, BatteryError> {
    Ok(bail_on!(PROS_ERR_F, unsafe {
        pros_sys::misc::battery_get_temperature()
    }))
}

/// Get the electric current of the robot's battery.
pub fn current() -> Result<i32, BatteryError> {
    Ok(bail_on!(PROS_ERR, unsafe {
        pros_sys::misc::battery_get_current()
    }))
}

/// Get the robot's battery voltage.
pub fn voltage() -> Result<i32, BatteryError> {
    Ok(bail_on!(PROS_ERR, unsafe {
        pros_sys::misc::battery_get_voltage()
    }))
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with the robot's battery.
pub enum BatteryError {
    /// Another resource is already using the battery.
    ConcurrentAccess,
}

map_errno! {
    BatteryError {
        EACCES => Self::ConcurrentAccess,
    }
}
