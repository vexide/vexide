//! Functions for modifying the state of the current
//! user program.

use core::{convert::Infallible, fmt::Debug, time::Duration};

use vex_sdk::{vexSerialWriteFree, vexSystemExitRequest, vexTasksRun, vcodesig};
use bitflags::bitflags;

use crate::{io::{self, stdout, Write}, time::Instant};

/// A trait that can be implemented for arbitrary return types in the main function.
pub trait Termination {
    /// Run specific termination logic.
    /// Unlike in the standard library, this function does not return a status code.
    fn report(self);
}
impl Termination for () {
    fn report(self) {}
}
impl Termination for ! {
    fn report(self) {}
}
impl Termination for Infallible {
    fn report(self) {}
}
impl<T: Termination, E: Debug> Termination for Result<T, E> {
    fn report(self) {
        match self {
            Ok(t) => t.report(),
            Err(e) => {
                write!(stdout(), "Error: {e:?}").unwrap();
                exit();
            }
        }
    }
}

const FLUSH_TIMEOUT: Duration = Duration::from_millis(15);

/// Exits the program using vexSystemExitRequest.
/// This function will not instantly exit the program,
/// but will instead wait up to 15mS to force the serial buffer to flush.
pub fn exit() -> ! {
    let exit_time = Instant::now();

    unsafe {
        // Force the serial buffer to flush
        while exit_time.elapsed() < FLUSH_TIMEOUT {
            vexTasksRun();

            // If the buffer has been fully flushed, exit the loop
            if vexSerialWriteFree(io::STDIO_CHANNEL) == (crate::io::Stdout::INTERNAL_BUFFER_SIZE as i32) {
                break;
            }
        }

        // Request program exit.
        vexSystemExitRequest();

        // Loop while vexos decides what to do with our exit request.
        loop {
            vexTasksRun();
        }
    }
}
