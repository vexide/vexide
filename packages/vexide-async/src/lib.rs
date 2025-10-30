//! Tiny async runtime for `vexide`.
//!
//! This crate contains an implementation of vexide's async executor, which is driven by `smol`'s
//! [`async_task`] crate. The executor is optimized for use on VEXos-based systems and supports
//! spawning tasks and blocking on futures. It has a reactor to improve the performance of some
//! futures (such as [`Sleep`](crate::time::Sleep)).

mod executor;
mod reactor;

mod local;
pub mod task;
pub mod time;

use core::future::Future;

pub use task::spawn;

use crate::executor::EXECUTOR;

/// Synchronization primitives for async code.
///
/// vexide programs often use async [tasks](crate::task) to run multiple operations concurrently.
/// These primitives provide methods for tasks to safely communicate with each other and share data.
/// This is vexide's async equivalent to the [`std::sync` module].
///
/// [`std::sync` module]: https://doc.rust-lang.org/stable/std/sync/index.html
#[cfg(feature = "sync")]
pub mod sync {
    pub use async_lock::{
        Barrier, BarrierWaitResult, Mutex, MutexGuard, OnceCell, RwLock, RwLockReadGuard,
        RwLockWriteGuard,
    };
}

/// Blocks the current task until a return value can be extracted from the provided future.
///
/// Does not poll all futures to completion.
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    EXECUTOR.with(|ex| ex.block_on(future))
}
