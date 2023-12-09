pub mod local;
pub mod task;

pub use task::*;

use core::{future::Future, task::Poll};

use crate::async_runtime::executor::EXECUTOR;

/// Blocks the current task for the given amount of time, if you are in an async function.
/// ## you probably don't want to use this.
/// This function will block the entire task, including the async executor!
/// Instead, you should use [`sleep`].
pub fn delay(duration: core::time::Duration) {
    unsafe { pros_sys::delay(duration.as_millis() as u32) }
}

pub struct SleepFuture {
    target_millis: u32,
}
impl Future for SleepFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.target_millis < unsafe { pros_sys::millis() } {
            Poll::Ready(())
        } else {
            EXECUTOR.with(|e| {
                e.reactor
                    .borrow_mut()
                    .sleepers
                    .push(cx.waker().clone(), self.target_millis)
            });
            Poll::Pending
        }
    }
}

pub fn sleep(duration: core::time::Duration) -> SleepFuture {
    SleepFuture {
        target_millis: unsafe { pros_sys::millis() + duration.as_millis() as u32 },
    }
}

/// Returns the task the function was called from.
pub fn current() -> TaskHandle {
    unsafe {
        let task = pros_sys::task_get_current();
        TaskHandle { task }
    }
}

/// Gets the first notification in the queue.
/// If there is none, blocks until a notification is received.
/// I am unsure what happens if the thread is unblocked while waiting.
/// returns the value of the notification
pub fn get_notification() -> u32 {
    unsafe { pros_sys::task_notify_take(false, pros_sys::TIMEOUT_MAX) }
}

#[doc(hidden)]
pub fn __init_main() {
    unsafe {
        pros_sys::lcd_initialize();
    }
}
