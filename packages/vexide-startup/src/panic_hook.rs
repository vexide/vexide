//! Custom panic hook for vexide programs.
//!
//! This extends the default `libstd` panic handler with support for capturing
//! backtrace data and drawing the panic message to the display screen.

use std::{fmt::Write, panic::PanicHookInfo};

#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use vex_libunwind::{UnwindContext, UnwindCursor};

#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use crate::error_report::backtrace::BacktraceIter;
use crate::error_report::ErrorReport;

/// Panic hook for vexide programs.
///
/// This extends the default `libstd` panic handler with support for capturing
/// backtrace data and drawing the panic message to the display screen.
pub(crate) fn hook(info: &PanicHookInfo<'_>) {
    let mut dialog = ErrorReport::begin();

    eprintln!("{info}");
    writeln!(dialog, "{info}").unwrap();

    #[cfg(all(target_os = "vexos", feature = "backtrace"))]
    {
        _ = UnwindContext::capture(|context| {
            let cursor = UnwindCursor::new(&context)?;

            dialog
                .write_str("stack backtrace (check terminal):\n")
                .unwrap();
            dialog.write_backtrace(BacktraceIter::new(cursor.clone()));

            eprintln!("stack backtrace:");
            for (i, frame) in BacktraceIter::new(cursor).enumerate() {
                eprintln!("{i:>3}: 0x{frame:x}");
            }
            eprintln!(
                "note: Use a symbolizer to convert stack frames to human-readable function names."
            );

            Ok(())
        });
    }

    // Don't exit the program, since we want to be able to see the panic message on the screen.
    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}
