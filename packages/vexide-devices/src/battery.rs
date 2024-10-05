//! Utilities for getting information about the robot's battery.

use vex_sdk::{
    vexBatteryCapacityGet, vexBatteryCurrentGet, vexBatteryTemperatureGet, vexBatteryVoltageGet,
};

/// Get the robot's battery capacity.
/// TODO: Determine units
pub fn capacity() -> f64 {
    unsafe { vexBatteryCapacityGet() }
}

/// Get the current temperature of the robot's battery.
/// TODO: Determine units
pub fn temperature() -> f64 {
    unsafe { vexBatteryTemperatureGet() }
}

/// Get the electric current of the robot's battery.
/// TODO: Determine units
pub fn current() -> i32 {
    unsafe { vexBatteryCurrentGet() }
}

/// Get the robot's battery voltage.
/// TODO: Determine units
pub fn voltage() -> i32 {
    unsafe { vexBatteryVoltageGet() }
}
