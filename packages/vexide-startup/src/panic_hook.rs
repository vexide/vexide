//! Custom panic hook for vexide programs.
//!
//! This extends the default `libstd` panic handler with support for capturing
//! backtrace data and drawing the panic message to the display screen.

use std::{fmt::Write, panic::PanicHookInfo};

use vexide_core::backtrace::Backtrace;

/// Panic hook for vexide programs.
///
/// This extends the default `libstd` panic handler with support for capturing
/// backtrace data and drawing the panic message to the display screen.
pub(crate) fn hook(info: &PanicHookInfo<'_>) {
    let mut dialog = ErrorReport::begin();

    eprintln!("{info}");
    _ = writeln!(dialog, "{info}");

    let backtrace = Backtrace::capture();
    _ = dialog.write_str("stack backtrace (check terminal):\n");
    dialog.write_backtrace(backtrace.iter());
    eprint!("{backtrace}");

    eprintln!(
        "note: Use a symbolizer to convert stack frames to human-readable function names."
    );

    // Don't exit the program, since we want to be able to see the panic message on the screen.
    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}
