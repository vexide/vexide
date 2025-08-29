//! Temporal quantification.
//!
//! This module provides an implementation of [`Instant`] built on the VEXos high-resolution timer.

use core::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration,
};

use vex_sdk::vexSystemPowerupTimeGet;

/// Returns the duration that the brain has been turned on.
#[must_use]
pub fn uptime() -> Duration {
    Duration::from_micros(unsafe { vexSystemPowerupTimeGet() })
}
