//! Low level core functionality for [`vexide`](https://crates.io/crates/vexide).
//! The core crate is used in all other crates in the vexide ecosystem.
//!
//! Included in this crate:
//! - Global allocator: [`pros_alloc`]
//! - Errno handling: [`error`]
//! - Serial terminal printing: [`io`]
//! - No-std [`Instant`](time::Instant)s: [`time`]
//! - Synchronization primitives: [`sync`]
//! - FreeRTOS task management: [`task`]

#![no_std]
#![feature(error_in_core)]
#![cfg_attr(feature = "critical-section", feature(asm_experimental_arch))]

pub mod allocator;
pub mod competition;
#[cfg(feature = "critical-section")]
pub mod critical_section;
pub mod io;
pub mod sync;
pub mod time;

/// Exits the program using vexSystemExitRequest.
/// This function will not instantly exit the program,
/// but will instead wait 3ms to force the serial buffer to flush.
pub fn exit() -> ! {
    unsafe {
        // Force the serial buffer to flush
        let exit_time = time::Instant::now();
        while exit_time.elapsed().as_millis() < 3 {
            vex_sdk::vexTasksRun();
        }
        // Exit the program
        // Everything after this point is unreachable.
        vex_sdk::vexSystemExitRequest();
    }
    // unreachable.
    loop {
        core::hint::spin_loop();
    }
}
