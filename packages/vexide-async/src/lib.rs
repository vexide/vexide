//! Tiny async runtime for `vexide`.
//! The async executor supports spawning tasks and blocking on futures.
//! It has a reactor to improve the performance of some futures.

#![no_std]

extern crate alloc;

use core::{future::Future, task::Poll};

use async_task::Task;
use executor::EXECUTOR;

mod executor;
mod reactor;

/// Runs a future in the background without having to await it
/// To get the the return value you can await a task.
pub fn spawn<T>(future: impl Future<Output = T> + 'static) -> Task<T> {
    EXECUTOR.spawn(future)
}

/// Blocks the current task untill a return value can be extracted from the provided future.
/// Does not poll all futures to completion.
pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    let task = spawn(future);
    EXECUTOR.block_on(task)
}

/// A future that will complete after the given duration.
/// Sleep futures that are closer to completion are prioritized to improve accuracy.
#[derive(Debug)]
pub struct SleepFuture {
    target_millis: u32,
}
impl Future for SleepFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.target_millis < unsafe { (vex_sdk::vexSystemHighResTimeGet() / 1000) as _ } {
            Poll::Ready(())
        } else {
            EXECUTOR.with_reactor(|reactor| {
                reactor
                    .sleepers
                    .push(cx.waker().clone(), self.target_millis)
            });
            Poll::Pending
        }
    }
}

/// Returns a future that will complete after the given duration.
pub fn sleep(duration: core::time::Duration) -> SleepFuture {
    SleepFuture {
        target_millis: unsafe {
            (vex_sdk::vexSystemHighResTimeGet() / 1000) as u32 + duration.as_millis() as u32
        },
    }
}
