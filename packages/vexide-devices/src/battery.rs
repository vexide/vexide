//! Utilities for getting information about the robot's battery.

use vex_sdk::{
    vexBatteryCapacityGet, vexBatteryCurrentGet, vexBatteryTemperatureGet, vexBatteryVoltageGet,
};

/// Returns the robot's current battery capacity from [0.0, 1.0].
///
/// A value of `0.0` indicates a completely empty battery, while a value of`1.0`
/// indicates a fully-charged battery.
pub fn capacity() -> f64 {
    (unsafe { vexBatteryCapacityGet() } as f64) / 100.0
}

/// Returns the internal temperature of the robot's battery in degrees celsius.
pub fn temperature() -> u64 {
    (unsafe { vexBatteryTemperatureGet() } as u64)
}

/// Returns the electric current of the robot's battery in amps.
pub fn current() -> f64 {
    (unsafe { vexBatteryCurrentGet() } as f64) / 1000.0
}

/// Returns the robot's battery voltage in volts.
pub fn voltage() -> f64 {
    (unsafe { vexBatteryVoltageGet() } as f64) / 1000.0
}
