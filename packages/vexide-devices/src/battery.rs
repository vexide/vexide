//! Utilities for getting information about the robot's battery.

use vex_sdk::{
    vexBatteryCapacityGet, vexBatteryCurrentGet, vexBatteryTemperatureGet, vexBatteryVoltageGet,
};

/// Get the robot's current battery capacity as a percentage out of 100.
pub fn capacity() -> f64 {
    unsafe { vexBatteryCapacityGet() }
}

/// Get the temperature of the robot's battery in degrees celsius.
pub fn temperature() -> f64 {
    unsafe { vexBatteryTemperatureGet() }
}

/// Get the electric current of the robot's battery in amps.
pub fn current() -> f64 {
    (unsafe { vexBatteryCurrentGet() } as f64) / 1000.0
}

/// Get the robot's battery voltage in volts.
pub fn voltage() -> f64 {
    (unsafe { vexBatteryVoltageGet() } as f64) / 1000.0
}
