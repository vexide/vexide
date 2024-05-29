//! Functions for modifying the state of the current
//! user program.

use core::{convert::Infallible, fmt::Debug, time::Duration};

use vex_sdk::{vexSerialWriteFree, vexSystemExitRequest, vexTasksRun};

use crate::{io, time::Instant};

/// A that can be implemented for arbitrary return types in the main function.
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
                io::println!("Error: {e:?}");
                exit();
            }
        }
    }
}

/// Exits the program using vexSystemExitRequest.
/// This function will not instantly exit the program,
/// but will instead wait 3ms to force the serial buffer to flush.
pub fn exit() -> ! {
    let exit_time = Instant::now();
    const FLUSH_TIMEOUT: Duration = Duration::from_millis(15);
    unsafe {
        // Force the serial buffer to flush
        while exit_time.elapsed() < FLUSH_TIMEOUT {
            // If the buffer has been fully flushed, exit the loop
            if vexSerialWriteFree(io::STDIO_CHANNEL) == (io::Stdout::INTERNAL_BUFFER_SIZE as i32) {
                break;
            }
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
