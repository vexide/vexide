//! Asynchronous tasks.

use core::future::Future;

use crate::local::Tls;

/// Internal metadata used by the executor. Users should not need to access this, and this is only
/// public because it appears in the public type aliases [`Task`] and [`FallibleTask`].
pub struct TaskMetadata {
    pub(crate) tls: Tls,
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
/// Note that canceling a task actually wakes it and reschedules one last time. Then, the executor
/// can destroy the task by simply dropping its [`Runnable`][`super::Runnable`] or by invoking
/// [`run()`][`super::Runnable::run()`].
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
///
/// This type behaves like [`Task`], however it produces an `Option<T>` when
/// polled and will return `None` if the executor dropped its
/// [`Runnable`][`super::Runnable`] without being run.
///
/// This can be useful to avoid the panic produced when polling the `Task`
/// future if the executor dropped its `Runnable`.
pub type FallibleTask<T> = async_task::FallibleTask<T, TaskMetadata>;

use crate::executor::EXECUTOR;

/// Spawns a new async task that can be controlled with the returned task handle.
pub fn spawn<T>(future: impl Future<Output = T> + 'static) -> Task<T> {
    EXECUTOR.spawn(future)
}
