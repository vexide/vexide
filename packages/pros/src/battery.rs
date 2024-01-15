//! Utilites for getting information about the robot's battery.

/// Get the robot's battery capacity.
pub fn get_capacity() -> f64 {
    unsafe { pros_sys::misc::battery_get_capacity() }
}

/// Get the electric current of the robot's battery.
pub fn get_current() -> i32 {
    unsafe { pros_sys::misc::battery_get_current() }
}

/// Get the current temperature of the robot's battery.
pub fn get_temperature() -> f64 {
    unsafe { pros_sys::misc::battery_get_temperature() }
}

/// Get the robot's battery voltage.
pub fn get_voltage() -> i32 {
    unsafe { pros_sys::misc::battery_get_voltage() }
}
