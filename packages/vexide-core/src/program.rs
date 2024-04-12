//! User Program Module
//!
//! This module contains functions for accessing/modifying the state of the current
//! user program.

pub use vex_sdk::{vexSystemExitRequest, vexTasksRun, vexSerialWriteFree};

pub use crate::time::Instant;
pub use crate::io;

/// Exits the program using vexSystemExitRequest.
/// This function will not instantly exit the program,
/// but will instead wait 3ms to force the serial buffer to flush.
pub fn exit() -> ! {
    unsafe {
        // Force the serial buffer to flush
        while vexSerialWriteFree(io::STDIO_CHANNEL) != (io::Stdout::INTERNAL_BUFFER_SIZE as i32) {
            vexTasksRun();
        }
        // Exit the program
        // Everything after this point is unreachable.
        vexSystemExitRequest();
    }

    // unreachable.
    loop {
        core::hint::spin_loop();
    }
}
