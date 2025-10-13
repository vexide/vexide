//! Extended VEXos system time APIs.

use core::time::Duration;

use vex_sdk::{vexSystemHighResTimeGet, vexSystemPowerupTimeGet, vexSystemTimeGet};

/// Returns the duration that the brain has been powered on for.
#[must_use]
pub fn system_uptime() -> Duration {
    Duration::from_micros(unsafe { vexSystemPowerupTimeGet() })
}

/// Returns the duration that the user processor has been running for.
#[must_use]
pub fn user_uptime() -> Duration {
    Duration::from_micros(unsafe { vexSystemHighResTimeGet() })
}
