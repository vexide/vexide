//! vexOS functions
//!
//! Module for retreiving information about vexOS

use core::{fmt, time::Duration};

use vex_sdk::{vexSystemPowerupTimeGet, vexSystemUsbStatus, vexSystemVersion};

/// A VexOS version
#[derive(Clone, Copy, Debug)]
pub struct Version {
    /// The major version
    pub major: u8,
    /// The minor version
    pub minor: u8,
    /// The build version
    pub build: u8,
    /// The patch version
    pub patch: u8,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}-r{}",
            self.major, self.minor, self.patch, self.patch
        )
    }
}

/// Returns the current VexOS version.
#[must_use]
pub fn get_version() -> Version {
    let version_bytes = unsafe { vexSystemVersion() }.to_be_bytes();
    Version {
        major: version_bytes[0],
        minor: version_bytes[1],
        build: version_bytes[2],
        patch: version_bytes[3],
    }
}

/// Returns the duration that the brain has been turned on.
#[must_use]
pub fn get_uptime() -> Duration {
    Duration::from_micros(unsafe { vexSystemPowerupTimeGet() })
}

/// Whether or not the brain has a USB cable plugged in.
#[must_use]
pub fn has_usb() -> bool {
    (unsafe { vexSystemUsbStatus() } == 1)
}
