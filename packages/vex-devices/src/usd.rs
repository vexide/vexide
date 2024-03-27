//! USD api.
//!
//! The USD API provides functions for interacting with the SD card slot on the V5 Brain.

use vex_sdk::vexFileDriveStatus;

/// Checks if an SD card is installed.
pub fn usd_installed() -> bool {
    unsafe { vexFileDriveStatus(0) }
}
