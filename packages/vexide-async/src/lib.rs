//! Tiny async runtime for `vexide`.
//!
//! The async executor supports spawning tasks and blocking on futures.
//! It has a reactor to improve the performance of some futures.

#![no_std]

extern crate alloc;

mod executor;
mod reactor;

pub mod task;
pub mod time;

use core::future::Future;

use executor::EXECUTOR;
pub use task::spawn;

/// Blocks the current task untill a return value can be extracted from the provided future.
///
/// Does not poll all futures to completion.
pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    let task = spawn(future);
    EXECUTOR.block_on(task)
}
