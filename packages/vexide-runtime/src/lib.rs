//! Low level core functionality for [`vexide`](https://crates.io/crates/vexide).
//! The core crate is used in all other crates in the vexide ecosystem.
//!
//! Included in this crate:
//! - Competition state handling: [`competition`]
//! - Critical-section implementation: [`critical_section`]
//! - Serial terminal printing: [`io`]
//! - Synchronization primitives: [`sync`]
//! - Program control: [`program`]

#![feature(never_type, asm_experimental_arch)]

pub mod competition;
pub mod sync;
pub mod task;
pub mod time;
pub mod banner;

mod executor;

use core::future::Future;
use executor::EXECUTOR;

pub use task::spawn;

/// Blocks the current task untill a return value can be extracted from the provided future.
///
/// Does not poll all futures to completion.
pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    EXECUTOR.block_on(spawn(future))
}
