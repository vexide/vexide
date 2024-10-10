//! Utilities for getting information about the robot's battery.

use vex_sdk::{
    vexBatteryCapacityGet, vexBatteryCurrentGet, vexBatteryTemperatureGet, vexBatteryVoltageGet,
};

/// Get the robot's current battery capacity as a percentage out of 100.
pub fn capacity() -> f64 {
    unsafe { vexBatteryCapacityGet() }
}

/// Get the current temperature of the robot's battery in degrees celsius.
pub fn temperature() -> f64 {
    unsafe { vexBatteryTemperatureGet() }
}

/// Get the electric current of the robot's battery.
pub fn current() -> i32 {
    unsafe { vexBatteryCurrentGet() }
}

/// Get the robot's battery voltage.
pub fn voltage() -> i32 {
    unsafe { vexBatteryVoltageGet() }
}
