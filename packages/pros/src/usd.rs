//! USD api.
//!
//! The USD API provides functions for interacting with the SD card slot on the V5 Brain.

/// Checks if an SD card is installed.
pub fn usd_installed() -> bool {
    unsafe { pros_sys::misc::usd_is_installed() == 1 }
}
