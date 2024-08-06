//! Tiny async runtime for `vexide`.
//!
//! The async executor supports spawning tasks and blocking on futures.
//! It has a reactor to improve the performance of some futures.

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

#[doc(hidden)]
pub fn __internal_entrypoint_task<const BANNER: bool>() {
    use no_std_io::io::Write;

    if BANNER {
        write!(
            vexide_core::io::stdout(),
            "
\x1B[1;38;5;196m=%%%%%#-  \x1B[38;5;254m-#%%%%-\x1B[1;38;5;196m  :*%%%%%+.
\x1B[38;5;208m  -#%%%%#-  \x1B[38;5;254m:%-\x1B[1;38;5;208m  -*%%%%#
\x1B[38;5;226m    *%%%%#=   -#%%%%%+
\x1B[38;5;226m      *%%%%%+#%%%%%%%#=
\x1B[38;5;34m        *%%%%%%%*-+%%%%%+
\x1B[38;5;27m          +%%%*:   .+###%#
\x1B[38;5;93m           .%:\x1B[0m
vexide startup successful!
Running user code...
"
        ).ok();
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
            crate::time::sleep(::core::time::Duration::from_millis(2)).await;
        }
    })
    .detach();
}
