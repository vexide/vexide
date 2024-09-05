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

mod rt;
mod banner;

use core::future::Future;
use rt::executor::EXECUTOR;

pub use task::spawn;

/// Blocks the current task untill a return value can be extracted from the provided future.
///
/// Does not poll all futures to completion.
pub fn block_on<F: Future + 'static>(future: F) -> F::Output {
    let task = spawn(future);
    EXECUTOR.block_on(task)
}

/// Sets up the vexide async runtime.
///
/// # Safety
///
/// This function should only be called once.
pub unsafe fn init_runtime<const BANNER: bool>() {
    if BANNER {
        banner::print();
    }

    // Run vexos background processing at a regular 2ms interval.
    // This is necessary for serial and device reads to work properly.
    crate::task::spawn(async {
        loop {
            unsafe {
                vex_sdk::vexTasksRun();
            }

            // In VEXCode programs, this is ran in a tight loop with no delays, since they
            // don't need to worry about running two schedulers on top of each other, but
            // doing this in our case would cause this task to hog all the CPU time, which
            // wouldn't allow futures to be polled in the async runtime.
            crate::time::sleep(::std::time::Duration::from_millis(2)).await;
        }
    })
    .detach();
}
