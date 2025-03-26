//! User program state.
//!
//! This module provides functionality for controlling the state of the currently running
//! user program on the Brain. At the time, that is essentially just functionality for
//! exiting the program early using the [`abort`] and [`exit`] functions.

use core::{convert::Infallible, fmt::Debug, time::Duration};

use vex_sdk::{vexSerialWriteFree, vexSystemExitRequest, vexTasksRun};

use crate::{io, time::Instant};

/// A trait that can be implemented for arbitrary return types in the `main` function.
pub trait Termination {
    /// Run specific termination logic.
    ///
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
            Err(e) => io::println!("Error: {e:?}"),
        }
    }
}

const FLUSH_TIMEOUT: Duration = Duration::from_millis(15);

/// Exits the program using `vexSystemExitRequest`.
///
/// This function will block up to 15mS to allow the serial buffer to flush, then either exit the program or
/// block for an indefinite amount of time depending on how long it takes VEXos to kill the program.
///
/// Note that because this function never returns, and that it terminates the process, no destructors on
/// the current stack will be run. If a clean shutdown is needed it is recommended to only call this function
/// at a known point where there are no more destructors left to run; or, preferably, simply return a type
/// implementing [`Termination`] (such as `Result`) from the `main` function and avoid this function altogether:
///
/// ```
/// #[vexide::main]
/// async fn main(peripherals: Peripherals) -> Result<(), MyError> {
///    // ...
///    Ok(())
/// }
/// ```
pub fn exit() -> ! {
    let exit_time = Instant::now();

    unsafe {
        // Force the serial buffer to flush
        while exit_time.elapsed() < FLUSH_TIMEOUT {
            vexTasksRun();

            // If the buffer has been fully flushed, exit the loop
            if vexSerialWriteFree(io::STDIO_CHANNEL) == (io::Stdout::INTERNAL_BUFFER_SIZE as i32) {
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

/// Exits the program using `vexSystemExitRequest` without flushing the serial buffer.
///
/// This function is virtually identical to [`exit`] with the caveat that it will not wait for
/// stdio buffers to be printed, meaning any writes to `Stdout` *MAY* not be written before the
/// program is killed by VEXos.
pub fn abort() -> ! {
    unsafe {
        // Request program exit from CPU0.
        vexSystemExitRequest();

        // Spin while CPU0 decides what to do with our exit request.
        loop {
            vexTasksRun();
        }
    }
}
