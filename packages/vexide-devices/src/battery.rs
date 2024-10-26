//! Utilities for getting information about the robot's battery.

use vex_sdk::{
    vexBatteryCapacityGet, vexBatteryCurrentGet, vexBatteryTemperatureGet, vexBatteryVoltageGet,
};

/// Returns the robot's current battery capacity from [0.0, 1.0].
///
/// A value of `0.0` indicates a completely empty battery, while a value of`1.0`
/// indicates a fully-charged battery.
#[must_use]
pub fn capacity() -> f64 {
    (unsafe { vexBatteryCapacityGet() } as f64) / 100.0
}

/// Returns the internal temperature of the robot's battery in degrees Celsius.
#[must_use]
pub fn temperature() -> u64 {
    (unsafe { vexBatteryTemperatureGet() } as u64)
}

/// Returns the electric current of the robot's battery in amps.
#[must_use]
pub fn current() -> f64 {
    f64::from(unsafe { vexBatteryCurrentGet() }) / 1000.0
}

/// Returns the robot's battery voltage in volts.
#[must_use]
pub fn voltage() -> f64 {
    f64::from(unsafe { vexBatteryVoltageGet() }) / 1000.0
}
