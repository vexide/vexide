//! Asynchronous tasks.

use core::future::Future;

pub use async_task::{FallibleTask, Task};

use crate::rt::executor::EXECUTOR;

/// Spawns a new async task that can be controlled with the returned task handle.
pub fn spawn<T>(future: impl Future<Output = T> + 'static) -> Task<T> {
    EXECUTOR.spawn(future)
}
