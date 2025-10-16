//! Extended VEXos system time APIs.

use core::time::Duration;

use vex_sdk::{vexSystemHighResTimeGet, vexSystemPowerupTimeGet};

/// Returns the duration that the brain has been powered on.
///
/// # Precision
///
/// The returned [`Duration`] has a precision of 1 microsecond.
#[must_use]
pub fn system_uptime() -> Duration {
    Duration::from_micros(unsafe { vexSystemPowerupTimeGet() })
}

/// Returns the duration that the brain's user processor has been
/// running.
///
/// This is effectively the amount of time that the current user
/// program has been running.
///
/// # Precision
///
/// The returned [`Duration`] has a precision of 1 microsecond.
#[must_use]
pub fn user_uptime() -> Duration {
    Duration::from_micros(unsafe { vexSystemHighResTimeGet() })
}
