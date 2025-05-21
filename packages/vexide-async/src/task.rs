//! Asynchronous tasks.

use core::future::Future;

use crate::local::TaskLocalStorage;

// public because it's used in Task<T> and InfallibleTask<T>
#[doc(hidden)]
pub struct TaskMetadata {
    pub(crate) tls: TaskLocalStorage,
}

/// A spawned task.
///
/// A [`Task`] can be awaited to retrieve the output of its future.
///
/// Dropping a [`Task`] cancels it, which means its future won't be polled again. To drop the
/// [`Task`] handle without canceling it, use [`detach()`][`Task::detach()`] instead. To cancel a
/// task gracefully and wait until it is fully destroyed, use the [`cancel()`][Task::cancel()]
/// method.
///
/// # Examples
///
/// ```
/// use vexide::async_runtime::spawn;
///
/// // Spawn a future onto the executor.
/// let task = spawn(async {
///     println!("Hello from a task!");
///     1 + 2
/// });
///
/// // Wait for the task's output.
/// assert_eq!(task.await, 3);
/// ```
pub type Task<T> = async_task::Task<T, TaskMetadata>;

/// A spawned task with a fallible response.
pub type FallibleTask<T> = async_task::FallibleTask<T, TaskMetadata>;

use crate::executor::EXECUTOR;

/// Spawns a new async task that can be controlled with the returned task handle.
pub fn spawn<T>(future: impl Future<Output = T> + 'static) -> Task<T> {
    EXECUTOR.spawn(future)
}
