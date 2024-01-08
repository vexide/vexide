//! A tiny async runtime.
//!
//! This runtime can be used outside of the main task, but it is reccomended to only use either
//! real FreeRTOS tasks or this async runtime.

use core::future::Future;

use async_task::Task;

pub(crate) mod executor;
pub(crate) mod reactor;

/// Runs a future in the background without having to await it
/// To get the the return value you can await a task.
pub fn spawn<T>(future: impl Future<Output = T> + 'static) -> Task<T> {
    executor::EXECUTOR.with(|e| e.spawn(future))
}

/// Blocks the current task untill a return value can be extracted from the provided future.
/// Does not poll all futures to completion.
/// If you want to complete all futures, use the [`complete_all`] function.
pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    executor::EXECUTOR.with(|e| e.block_on(spawn(future)))
}
