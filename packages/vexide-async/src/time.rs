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

use vexide_core::time::Instant;

use crate::{executor::EXECUTOR, reactor::Sleeper};

/// A future that will complete after a certain instant is reached in time.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Sleep(Instant);

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> core::task::Poll<Self::Output> {
        // NOTE: This check is mostly redundant.
        // This future will only be polled twice. Once when it is first added into the run queue,
        // and a second time when our sleeper is awoken by the reactor. As such, we know ahead of
        // time that `Instant::now()` will always be greater than our deadline on the second poll.
        //
        // HOWEVER: This check may still be necessary in the event that our sleep future is already
        // past its deadlline by the first time it is polled. In that event, it's way more efficient
        // to simply return `Poll::Ready(())` than do the whole Push -> Peek -> Pop -> Wake -> Poll
        // pipeline that most sleepers go through in the executor. This may occur in sleeps with
        // very short deadlines (such as sleeping for 0 milliseconds), where it will take the executor
        // some amount of time to initially poll the future and add it into the reactor.
        if Instant::now() > self.0 {
            Poll::Ready(())
        } else {
            EXECUTOR.with_reactor(|reactor| {
                reactor.sleepers.push(Sleeper {
                    deadline: self.0,
                    waker: cx.waker().clone(),
                });
            });

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
