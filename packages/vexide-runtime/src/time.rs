//! Utilities for tracking time.
//!
//! This module provides a types for executing code after a set period of time.
//!
//! * [`Sleep`] is a future that does no work and completes at a specific [`Instant`]
//!   in time.
//!
//! * [`sleep`] and [`sleep_until`] provide ways to yield control away from a future
//!   for or until a specific instant in time.

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use std::time::Instant;

use crate::executor::EXECUTOR;

/// A future that will complete after a certain instant is reached in time.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Sleep(Instant);

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> core::task::Poll<Self::Output> {
        if Instant::now() > self.0 {
            Poll::Ready(())
        } else {
            EXECUTOR.with_reactor(|reactor| reactor.sleepers.push(cx.waker().clone(), self.0));

            Poll::Pending
        }
    }
}

/// Returns a future that will complete after the given duration.
pub fn sleep(duration: Duration) -> Sleep {
    Sleep(Instant::now() + duration)
}

/// Returns a future that waits until a deadline is reached.
pub const fn sleep_until(deadline: Instant) -> Sleep {
    Sleep(deadline)
}
