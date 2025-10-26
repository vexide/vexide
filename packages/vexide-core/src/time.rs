//! Extended VEXos system time APIs.

use core::{
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

/// Returns the duration that the brain's user processor has been running.
///
/// This is effectively the time since the current program was started.
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
/// This type is not in sync with [`Instant`], which instead uses the brain's global high-resolution
/// timer.
///
/// [`Instant`]: https://doc.rust-lang.org/stable/std/time/struct.Instant.html
///
/// # Precision
///
/// This type has a precision of 1 millisecond.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LowResolutionTime {
    millis: u32,
}

impl LowResolutionTime {
    /// An anchor in time which represents the start of the clock.
    ///
    /// In practice, the epoch represents the start of the Brain's user processor, meaning the start
    /// of the current user program.
    pub const EPOCH: LowResolutionTime = LowResolutionTime { millis: 0 };

    /// Returns a low-resolution timestamp corresponding to "now".
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::time::LowResolutionTime;
    ///
    /// let now = LowResolutionTime::now();
    /// ```
    #[must_use]
    pub fn now() -> Self {
        Self {
            millis: unsafe { vexSystemTimeGet() },
        }
    }

    /// Creates a new timestamp at the provided number of milliseconds since
    /// [`LowResolutionTime::EPOCH`].
    ///
    /// # Use this sparingly.
    ///
    /// This method generally only exists for compatibility with FFI and system APIs. The only clock
    /// measurement that should be provided to `millis` should be measurements derived from the CPU1
    /// private timer (e.g. [`vexSystemTimeGet`]) to ensure that clock drift is not a factor.
    ///
    /// When possible, prefer using [`LowResolutionTime::now`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::time::LowResolutionTime;
    ///
    /// // Equivalent to `LowResolutionTime::now()`.
    /// let now = LowResolutionTime::from_millis_since_epoch(unsafe { vex_sdk::vexSystemTimeGet() });
    /// ```
    #[must_use]
    pub const fn from_millis_since_epoch(millis: u32) -> Self {
        Self { millis }
    }

    /// Returns the amount of time elapsed from another timestamp to this one, or zero duration if
    /// that timestamp is later than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::{prelude::*, time::LowResolutionTime};
    ///
    /// #[vexide::main]
    /// async fn main(_peripherals: Peripherals) {
    ///     let now = LowResolutionTime::now();
    ///     sleep(Duration::new(1, 0)).await;
    ///
    ///     let new_now = LowResolutionTime::now();
    ///     println!("{:?}", new_now.duration_since(now));
    ///     println!("{:?}", now.duration_since(new_now)); // 0ns
    /// }
    /// ```
    #[must_use]
    pub fn duration_since(&self, earlier: LowResolutionTime) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    /// Returns the amount of time elapsed from another timestamp to this one, or None if that
    /// timestamp is later than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::{prelude::*, time::LowResolutionTime};
    ///
    /// #[vexide::main]
    /// async fn main(_peripherals: Peripherals) {
    ///     let now = LowResolutionTime::now();
    ///     sleep(Duration::new(1, 0)).await;
    ///
    ///     let new_now = LowResolutionTime::now();
    ///     println!("{:?}", new_now.checked_duration_since(now));
    ///     println!("{:?}", now.checked_duration_since(new_now)); // None
    /// }
    /// ```
    #[must_use]
    pub const fn checked_duration_since(&self, earlier: LowResolutionTime) -> Option<Duration> {
        if earlier.millis < self.millis {
            Some(Duration::from_millis((self.millis - earlier.millis) as _))
        } else {
            None
        }
    }

    /// Returns the amount of time elapsed from another timestamp to this one, or zero duration if
    /// that timestamp is later than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::{prelude::*, time::LowResolutionTime};
    ///
    /// #[vexide::main]
    /// async fn main(_peripherals: Peripherals) {
    ///     let now = LowResolutionTime::now();
    ///     sleep(Duration::new(1, 0)).await;
    ///     let new_now = LowResolutionTime::now();
    ///     println!("{:?}", new_now.saturating_duration_since(now));
    ///     println!("{:?}", now.saturating_duration_since(new_now)); // 0ns
    /// }
    /// ```
    #[must_use]
    pub fn saturating_duration_since(&self, earlier: LowResolutionTime) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    /// Returns the amount of time elapsed since this timestamp.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::{prelude::*, time::LowResolutionTime};
    ///
    /// #[vexide::main]
    /// async fn main(_peripherals: Peripherals) {
    ///     let start = LowResolutionTime::now();
    ///     let three_secs = Duration::from_secs(3);
    ///     sleep(three_secs).await;
    ///     assert!(start.elapsed() >= three_secs);
    /// }
    /// ```
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        Self::now() - *self
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `LowResolutionTime` (which means it's inside the bounds of the underlying data structure),
    /// `None` otherwise.
    #[must_use]
    pub fn checked_add(self, rhs: Duration) -> Option<LowResolutionTime> {
        Some(Self {
            millis: self.millis.checked_add(rhs.as_millis().try_into().ok()?)?,
        })
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `LowResolutionTime` (which means it's inside the bounds of the underlying data structure),
    /// `None` otherwise.
    #[must_use]
    pub fn checked_sub(self, rhs: Duration) -> Option<LowResolutionTime> {
        Some(Self {
            millis: self.millis.checked_sub(rhs.as_millis().try_into().ok()?)?,
        })
    }
}

impl Add<Duration> for LowResolutionTime {
    type Output = LowResolutionTime;

    /// # Panics
    ///
    /// This function may panic if the resulting point in time cannot be represented by the
    /// underlying data structure. See [`LowResolutionTime::checked_add`] for a version without
    /// panic.
    fn add(self, rhs: Duration) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow when adding duration to timestamp")
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
            .expect("overflow when subtracting duration from timestamp")
    }
}

impl SubAssign<Duration> for LowResolutionTime {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<LowResolutionTime> for LowResolutionTime {
    type Output = Duration;

    /// Returns the amount of time elapsed from another time to this one, or zero duration if that
    /// time is later than this one.
    fn sub(self, other: LowResolutionTime) -> Duration {
        self.duration_since(other)
    }
}
