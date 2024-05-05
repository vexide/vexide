//! Asynchronous tasks.

use core::future::Future;

pub use async_task::{FallibleTask, Task};

use crate::executor::EXECUTOR;

/// Runs a future in the background without having to await it.
///
/// To get the the return value you can await a task.
pub fn spawn<T>(future: impl Future<Output = T> + 'static) -> Task<T> {
    EXECUTOR.spawn(future)
}
