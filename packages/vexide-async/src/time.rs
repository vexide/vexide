//! Utilities for tracking time.
//!
//! This module provides a types for executing code after a set period of time.
//!
//! - [`Sleep`] is a future that does no work and completes at a specific [`Instant`] in time.
//! - [`sleep`] and [`sleep_until`] provide ways to yield control away from a future for or until a
//!   specific instant in time.

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use std::{task::Waker, time::Instant};

use crate::{executor::EXECUTOR, reactor::Sleeper};

/// A future that will complete after a certain instant is reached in time.
///
/// This type is returned by the [`sleep`] and [`sleep_until`] functions.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Sleep {
    deadline: Instant,
    registered_waker: Option<Waker>,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.deadline {
            return Poll::Ready(());
        }

        // Register a waker on the reactor to only poll this future when the deadline passes.
        //
        // We should only push to the sleeper queue if we either haven't pushed
        // (`self.registered.waker == None`) or if !w.will_wake(cx.waker()), meaning the already
        // registered waker will not wake up the same task as the current waker indicating that the
        // sleep has potentially been moved across executors.
        if self
            .registered_waker
            .as_ref()
            .map(|w| !w.will_wake(cx.waker()))
            .unwrap_or(true)
        {
            let this = self.get_mut();
            this.registered_waker = Some(cx.waker().clone());

            EXECUTOR.with(|ex| {
                ex.with_reactor(|reactor| {
                    reactor.sleepers.push(Sleeper {
                        deadline: this.deadline,
                        waker: cx.waker().clone(),
                    });
                });
            });
        }

        Poll::Pending
    }
}

/// Waits until `duration` has elapsed.
///
/// This function returns a future that will complete after the given duration, effectively yielding
/// the current task for a period of time.
///
/// Equivalent to `sleep_until(Instant::now() + duration)`.
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
///
/// use vexide::prelude::*;
///
/// #[vexide::main]
/// async fn main(_peripherals: Peripherals) {
///     println!("See you in 5 minutes.");
///     sleep(Duration::from_secs(300)).await;
///     println!("Hello again!");
/// }
/// ```
pub fn sleep(duration: Duration) -> Sleep {
    Sleep {
        deadline: Instant::now() + duration,
        registered_waker: None,
    }
}

/// Waits until `deadline` is reached.
///
/// This function returns a future that will complete once a given `Instant` in time has been
/// reached.
///
/// # Examples
///
/// ```no_run
/// use std::time::{Duration, Instant};
///
/// use vexide::prelude::*;
///
/// #[vexide::main]
/// async fn main(_peripherals: Peripherals) {
///     let now = Instant::now();
///     let deadline = now + Duration::from_secs(2); // 2 seconds in the future
///
///     println!("See you in 2 seconds.");
///     sleep_until(deadline).await;
///     println!("Hello again!");
/// }
/// ```
pub const fn sleep_until(deadline: Instant) -> Sleep {
    Sleep {
        deadline,
        registered_waker: None,
    }
}
