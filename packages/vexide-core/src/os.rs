//! VEXos-related functionality.
//!
//! This module provides utilities for for interacting and retrieving
//! information from VEXos.

use core::fmt;

use vex_sdk::vexSystemVersion;

/// A VEXos version
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    /// The major version
    pub major: u8,
    /// The minor version
    pub minor: u8,
    /// The build version
    pub build: u8,
    /// The beta version
    pub beta: u8,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}-b{}",
            self.major, self.minor, self.build, self.beta
        )
    }
}

/// Returns the current VEXos version.
#[must_use]
pub fn system_version() -> Version {
    let version_bytes = unsafe { vexSystemVersion() }.to_be_bytes();
    Version {
        major: version_bytes[0],
        minor: version_bytes[1],
        build: version_bytes[2],
        beta: version_bytes[3],
    }
}
