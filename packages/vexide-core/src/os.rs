//! vexOS functions
//!
//! Module for retreiving information about vexOS

use core::fmt;

use vex_sdk::vexSystemVersion;

/// A VexOS version
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
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
pub fn system_version() -> Version {
    let version_bytes = unsafe { vexSystemVersion() }.to_be_bytes();
    Version {
        major: version_bytes[0],
        minor: version_bytes[1],
        build: version_bytes[2],
        patch: version_bytes[3],
    }
}
