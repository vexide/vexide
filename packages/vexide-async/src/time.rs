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
use std::time::Instant;

use crate::{executor::EXECUTOR, reactor::Sleeper};

/// A future that will complete after a certain instant is reached in time.
///
/// This type is returned by the [`sleep`] and [`sleep_until`] functions.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Sleep {
    deadline: Instant,
    registered: bool,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> core::task::Poll<Self::Output> {
        if Instant::now() > self.deadline {
            return Poll::Ready(())
        } else if !self.registered {
            EXECUTOR.with(|ex| {
                ex.with_reactor(|reactor| {
                    reactor.sleepers.push(Sleeper {
                        deadline: self.deadline,
                        waker: cx.waker().clone(),
                    });
                });
            });

            self.registered = true;
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
    Sleep { deadline: Instant::now() + duration, registered: false }
}

/// Waits until `deadline` is reached.
///
/// This function returns a future that will complete once a given `Instant` in time has been
/// reached.
///
/// # Examples
///
/// ```
/// use std::time::{Duration, Instant};
///
/// use vexide::prelude::*;
///
/// #[vexide::main]
/// async fn main(_peripherals: Peripherals) {
///     let now = Instant::now();
///     let deadline = now + Duration::from_secs(2); // 5 minutes in the future
///
///     println!("See you in 5 minutes.");
///     sleep_until(deadline).await;
///     println!("Hello again!");
/// }
/// ```
pub const fn sleep_until(deadline: Instant) -> Sleep {
    Sleep { deadline, registered: false }
}
