//! Extended VEXos system time APIs.

use core::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration,
};

use vex_sdk::{vexSystemHighResTimeGet, vexSystemPowerupTimeGet, vexSystemTimeGet};

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
/// This is effectively the time since the current program was
/// started.
///
/// # Precision
///
/// The returned [`Duration`] has a precision of 1 microsecond.
#[must_use]
pub fn user_uptime() -> Duration {
    Duration::from_micros(unsafe { vexSystemHighResTimeGet() })
}

/// A timestamp recorded by the Brain's low-resolution private timer.
///
/// # Precision
///
/// This type has a precision of 1 millisecond.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LowResolutionTime {
    millis: u32,
}

impl LowResolutionTime {
    pub const EPOCH: LowResolutionTime = LowResolutionTime { millis: 0 };

    pub const fn from_millis_since_epoch(millis: u32) -> Self {
        Self { millis }
    }

    pub fn now() -> Self {
        Self {
            millis: unsafe { vexSystemTimeGet() },
        }
    }

    #[must_use]
    pub fn duration_since(&self, earlier: LowResolutionTime) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    #[must_use]
    pub const fn checked_duration_since(&self, earlier: LowResolutionTime) -> Option<Duration> {
        if earlier.millis < self.millis {
            Some(Duration::from_millis((self.millis - earlier.millis) as _))
        } else {
            None
        }
    }

    #[must_use]
    pub fn saturating_duration_since(&self, earlier: LowResolutionTime) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        Self::now() - *self
    }

    #[must_use]
    pub fn checked_add(self, rhs: Duration) -> Option<LowResolutionTime> {
        Some(Self {
            millis: self.millis.checked_add(rhs.as_micros().try_into().ok()?)?,
        })
    }

    #[must_use]
    pub fn checked_sub(self, rhs: Duration) -> Option<LowResolutionTime> {
        Some(Self {
            millis: self.millis.checked_sub(rhs.as_micros().try_into().ok()?)?,
        })
    }
}

impl Add<Duration> for LowResolutionTime {
    type Output = LowResolutionTime;

    /// # Panics
    ///
    /// This function may panic if the resulting point in time cannot be represented by the
    /// underlying data structure. See [`LowResolutionTime::checked_add`] for a version without panic.
    fn add(self, rhs: Duration) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow when adding duration to low resolution time")
    }
}

impl AddAssign<Duration> for LowResolutionTime {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for LowResolutionTime {
    type Output = LowResolutionTime;

    fn sub(self, other: Duration) -> LowResolutionTime {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for LowResolutionTime {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<LowResolutionTime> for LowResolutionTime {
    type Output = Duration;

    /// Returns the amount of time elapsed from another time to this one,
    /// or zero duration if that time is later than this one.
    fn sub(self, other: LowResolutionTime) -> Duration {
        self.duration_since(other)
    }
}

impl fmt::Debug for LowResolutionTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.millis.fmt(f)
    }
}
