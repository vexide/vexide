//! Extended VEXos system time APIs.

use core::time::Duration;

use vex_sdk::vexSystemPowerupTimeGet;

/// Returns the duration that the brain has been powered on for.
#[must_use]
pub fn uptime() -> Duration {
    Duration::from_micros(unsafe { vexSystemPowerupTimeGet() })
}
