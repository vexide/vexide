//! Asynchronous multitasking.
//!
//! This module provides utilities for working with asynchronous tasks.
//!
//! A *task* is a light weight, non-blocking unit of execution. Tasks allow you to cooperatively
//! perform work in the background without blocking other code from running.
//!
//! - Tasks are **light weight**. Because tasks are scheduled and managed by vexide, creating new
//!   tasks or switching between tasks does not require a context switch and has fairly low
//!   overhead. Creating, running, and destroying large numbers of tasks is relatively cheap in
//!   comparison to traditional threads.
//!
//! - Tasks are scheduled **cooperatively**. Most operating system threads implement *preemptive
//!   multitasking*. This is a scheduling technique where the operating system allows each thread to
//!   run for a period of time, and then forcibly preempts it, temporarily pausing that thread and
//!   switching to another. Tasks, on the other hand, implement *cooperative multitasking*. In
//!   cooperative multitasking, a task will run until it voluntarily yields using an `await` point,
//!   giving control back to the vexide runtime's scheduler. When a task yields by `await`ing
//!   something, the vexide runtime switches to executing a different task.
//!
//! - Tasks are **non-blocking**. Typically, when an OS thread performs I/O or must synchronize with
//!   another thread, it *blocks*, allowing the OS to schedule another thread. When a task cannot
//!   continue executing, it should yield instead, allowing the vexide runtime to schedule another
//!   task in its place. Tasks should generally not perform operations that could block the CPU for
//!   a long period of time without an `await` point, as this would prevent other tasks from
//!   executing as well. This includes situations involving long-running "tight loops" (`loop {}`)
//!   without `await` points.
//!
//! # Spawning Tasks
//!
//! Perhaps the most important function in this module is [`spawn`]. This function can be thought of
//! as an async equivalent to the standard library’s [`thread::spawn`](std::thread::spawn). It takes
//! an `async` block or other [future](std::future), and creates a new task that runs it
//! concurrently in the background:
//!
//! ```
//! # #[vexide::main]
//! # async fn main(_peripherals: vexide::peripherals::Peripherals) {
//! use vexide::task;
//!
//! task::spawn(async {
//!     // perform some work here...
//! });
//! # }
//! ```
//!
//! After a task is spawned, you are given a [`Task`] struct, representing a running (or previously
//! running) task. The [`Task`] struct is itself a future which may be used to await the output of
//! the spawned task. For example:
//!
//! ```
//! # #[vexide::main]
//! # async fn main(_peripherals: vexide::peripherals::Peripherals) {
//! use vexide::task;
//!
//! let task = task::spawn(async {
//!     // ...
//!     "hello world!"
//! });
//!
//! // ...
//!
//! // Await the result of the spawned task.
//! let result = task.await;
//! assert_eq!(result, "hello world!");
//! # }
//! ```
//!
//! # Cancellation
//!
//! When a [`Task`] is dropped, it will stop being polled by vexide's runtime. This means that a
//! task is cancelled when it leaves the scope it was spawned in.
//!
//! ```no_run
//! # #[vexide::main]
//! # async fn main(_peripherals: vexide::peripherals::Peripherals) {
//! use std::time::Duration;
//!
//! use vexide::{task, time::sleep};
//!
//! {
//!     // This task will never run, since it immediately falls out of scope after it's spawned.
//!     let task = task::spawn(async {
//!         loop {
//!             println!("Hiiiii :3");
//!             sleep(Duration::from_secs(1)).await;
//!         }
//!     });
//! }
//! # }
//! ```
//!
//! If a task must outlive the scope it was spawned in, you can [`detach`] it. This lets the task
//! run in the background beyond its current scope. When we `detach` a task, we lose its [`Task`]
//! handle and therefore have no way to `await` its output. As a result, detached tasks may run
//! forever with no way of being stopped.
//!
//! [`detach`]: Task::detach
//!
//! ```no_run
//! # #[vexide::main]
//! # async fn main(_peripherals: vexide::peripherals::Peripherals) {
//! use std::time::Duration;
//!
//! use vexide::{task, time::sleep};
//!
//! {
//!     let task = task::spawn(async {
//!         loop {
//!             println!("Hiiiii :3");
//!             sleep(Duration::from_secs(1)).await;
//!         }
//!     });
//!
//!     // Run it forever, even after it leaves scope.
//!     task.detach();
//! }
//! # }
//! ```
//!
//! # Sharing State Between Tasks
//!
//! When running multiple tasks at once, it's often useful to share some data between them.
//!
//! To do this, we need multiple owners of the same piece of data, which is something that Rust's
//! borrow checker forbids. An easy way around this is to combine an [`Rc`] with a [`RefCell`],
//! which gives us both interior mutability and multiple owners. By wrapping our shared state in
//! `Rc<RefCell<T>>`, we can clone a smart pointer to it across as many tasks as we want.
//!
//! [`Rc`]: std::rc::Rc
//! [`RefCell`]: std::cell::RefCell
//!
//! ```no_run
//! # #[vexide::main]
//! # async fn main(_peripherals: vexide::peripherals::Peripherals) {
//! use std::{cell::RefCell, rc::Rc, time::Duration};
//!
//! use vexide::{task, time::sleep};
//!
//! let counter = Rc::new(RefCell::new(1));
//!
//! // task_1 increments `counter` every second.
//! let task_1 = task::spawn({
//!     let counter = counter.clone();
//!
//!     async move {
//!         loop {
//!             *counter.borrow_mut() += 1;
//!             sleep(Duration::from_secs(1)).await;
//!         }
//!     }
//! });
//!
//! // task_2 prints `counter` every two seconds.
//! let task_2 = task::spawn(async move {
//!     loop {
//!         println!("Counter: {}", *counter.borrow());
//!
//!         sleep(Duration::from_secs(2)).await;
//!     }
//! });
//! # }
//! ```
//!
//! More complex use-cases may require you to hold ownership of shared state *across*
//! `await`-points. In these cases, a simple `Rc<RefCell<T>>` will not suffice, since another
//! running task may claim ownership of the data, which would cause the program to panic. Doing this
//! effectively requires the use of a *synchronization primitive* like a
//! [`Mutex`](crate::sync::Mutex) or [`RwLock`](crate::sync::RwLock) to manage safe access to shared
//! state across multiple running tasks.
//!
//! For more information on how to do this, see vexide's [`sync`](crate::sync) module.

use std::{future::Future, rc::Rc};

pub use crate::local::{LocalKey, task_local};
use crate::{executor::EXECUTOR, local::TaskLocalStorage};

// public because it's used in Task<T> and InfallibleTask<T>
#[doc(hidden)]
#[derive(Debug)]
pub struct TaskMetadata {
    pub(crate) tls: Rc<TaskLocalStorage>,
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
/// use vexide::prelude::*;
///
/// #[vexide::main]
/// async fn main(_peripherals: Peripherals) {
///     // Spawn a future onto the executor.
///     let task = vexide::task::spawn(async {
///         println!("Hello from a task!");
///         1 + 2
///     });
///
///     // Wait for the task's output.
///     assert_eq!(task.await, 3);
/// }
/// ```
pub type Task<T> = async_task::Task<T, TaskMetadata>;

/// A spawned task with a fallible response.
pub type FallibleTask<T> = async_task::FallibleTask<T, TaskMetadata>;

/// Spawns a new async task that can be controlled with the returned task handle.
pub fn spawn<T>(future: impl Future<Output = T> + 'static) -> Task<T> {
    EXECUTOR.with(|ex| ex.spawn(future))
}
