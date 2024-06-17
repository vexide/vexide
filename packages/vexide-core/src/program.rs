//! Functions for modifying the state of the current
//! user program.

use core::{convert::Infallible, fmt::Debug, time::Duration};

use vex_sdk::{vexSerialWriteFree, vexSystemExitRequest, vexTasksRun};

use crate::{io, time::Instant};

/// This type represents the result of a program as a whole.
/// It should not be used in place of a [`Result`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum ExitCode {
    /// The program exited successfully.
    #[default]
    Success = 0,
    /// The program exited due to a failure.
    Failure = 1,
}

/// A trait that can be implemented to allow an arbitrary return type from the main function.
pub trait Termination {
    /// Converts the type to an [`ExitCode`], printing any failure messages over serial.
    fn report(self) -> ExitCode;
}
impl Termination for () {
    fn report(self) -> ExitCode {
        ExitCode::Success
    }
}
impl Termination for ! {
    fn report(self) -> ExitCode {
        ExitCode::Success
    }
}
impl Termination for Infallible {
    fn report(self) -> ExitCode {
        ExitCode::Success
    }
}
impl<T: Termination, E: Debug> Termination for Result<T, E> {
    fn report(self) -> ExitCode {
        match self {
            Ok(t) => t.report(),
            Err(e) => {
                io::println!("Error: {e:?}");
                ExitCode::Failure
            }
        }
    }
}

/// Exits the program, preventing execution from continuing.
/// This function may block up to 15ms before exiting to allow the serial buffer to flush.
pub fn exit() -> ! {
    let exit_time = Instant::now();
    const FLUSH_TIMEOUT: Duration = Duration::from_millis(15);
    unsafe {
        // Force the serial buffer to flush.
        while exit_time.elapsed() < FLUSH_TIMEOUT {
            // If the buffer has been fully flushed, exit the loop.
            if vexSerialWriteFree(io::STDIO_CHANNEL) == (io::Stdout::INTERNAL_BUFFER_SIZE as i32) {
                break;
            }
            vexTasksRun();
        }

        // Exit the program.
        vexSystemExitRequest();
    }

    // Wait for the system to finish stopping the program.
    loop {
        core::hint::spin_loop();
    }
}
