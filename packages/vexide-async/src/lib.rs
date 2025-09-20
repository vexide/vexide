//! Tiny async runtime for `vexide`.
//!
//! This crate contains an implementation of vexide's async executor,
//! which is driven by `smol`'s [`async_task`] crate. The executor is
//! optimized for use on VEXos-based systems and supports spawning tasks
//! and blocking on futures. It has a reactor to improve the performance
//! of some futures (such as [`Sleep`](crate::time::Sleep)).

mod executor;
mod reactor;

mod local;
pub mod task;
pub mod time;

use core::future::Future;

use executor::EXECUTOR;
pub use task::spawn;

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
pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    let task = spawn(future);
    EXECUTOR.block_on(task)
}
