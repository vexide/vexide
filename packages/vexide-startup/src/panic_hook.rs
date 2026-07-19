//! Custom panic hook for vexide programs.
//!
//! This extends the default `libstd` panic handler with support for capturing backtrace data and
//! drawing the panic message to the display screen.

use std::{fmt::Write, panic::PanicHookInfo};

use vexide_core::backtrace::BacktraceIter;

use crate::error_report::ErrorReport;

/// Panic hook for vexide programs.
///
/// This extends the default `libstd` panic handler with support for capturing backtrace data and
/// drawing the panic message to the display screen.
pub(crate) fn hook(info: &PanicHookInfo<'_>) {
    let mut dialog = ErrorReport::begin();

    eprintln!("{info}");
    writeln!(dialog, "{info}").unwrap();

    if cfg!(target_os = "vexos") {
        let mut i = 0;
        eprintln!("stack backtrace:");
        _ = writeln!(dialog, "stack backtrace (check terminal):");

        BacktraceIter::capture(|backtrace| {
            for addr in backtrace {
                dialog.write_backtrace(i, addr as u32);
                eprintln!("{i:>3}: 0x{:x}", addr as u32);
                i += 1;
            }
        });

        dialog.finish_backtrace(i);

        eprintln!(
            "note: Use a symbolizer to convert stack frames to human-readable function names."
        );
    }

    // Don't exit the program, since we want to be able to see the panic message on the screen.
    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}
