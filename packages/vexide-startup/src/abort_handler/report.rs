use std::fmt::{self, Write};

#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use vex_libunwind::UnwindCursor;

use super::fault::Fault;
use crate::error_report::ErrorReport;
#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use crate::error_report::backtrace::BacktraceIter;

pub struct SerialWriter(());

impl SerialWriter {
    pub const BUFFER_SIZE: usize = 2048;
    pub const fn new() -> Self {
        Self(())
    }

    #[expect(clippy::unused_self, reason = "only used for semantics")]
    pub fn flush(&mut self) {
        unsafe {
            while (vex_sdk::vexSerialWriteFree(1) as usize) != Self::BUFFER_SIZE {
                vex_sdk::vexTasksRun();
            }
        }
    }
}

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let buf = s.as_bytes();

        for chunk in buf.chunks(Self::BUFFER_SIZE) {
            if unsafe { vex_sdk::vexSerialWriteFree(1) as usize } < chunk.len() {
                self.flush();
            }

            let count: usize =
                unsafe { vex_sdk::vexSerialWriteBuffer(1, chunk.as_ptr(), chunk.len() as u32) }
                    as _;

            if count != chunk.len() {
                break;
            }
        }

        Ok(())
    }
}

/// Prints the fault to the serial console.
pub fn report_fault(fault: &Fault) {
    let mut dialog = ErrorReport::begin();
    let mut serial = SerialWriter::new();
    serial.flush();

    let title = format_args!(
        "{} exception at 0x{:x}:",
        fault.exception, fault.program_counter
    );
    _ = writeln!(serial, "\n{title}\n{fault}\n");
    _ = writeln!(dialog, "{title}\n{fault}");

    _ = writeln!(serial, "registers at time of fault:");

    for (i, register) in fault.registers.iter().enumerate() {
        if i < 10 {
            _ = write!(serial, " ");
        }

        _ = writeln!(serial, "r{i}: 0x{register:x}");
    }
    _ = writeln!(
        serial,
        " sp: 0x{:x}\n lr: 0x{:x}\n pc: 0x{:x}\n",
        fault.stack_pointer, fault.link_register, fault.program_counter
    );

    dialog.write_registers({
        let mut arr = [0u32; 16];
        arr[..13].copy_from_slice(&fault.registers);
        arr[13] = fault.stack_pointer;
        arr[14] = fault.link_register;
        arr[15] = fault.program_counter;
        arr
    });

    #[cfg(all(target_os = "vexos", feature = "backtrace"))]
    if let Ok(cursor) = UnwindCursor::new(&unsafe { fault.unwind_context() }) {
        _ = writeln!(dialog, "stack backtrace (check terminal):");
        dialog.write_backtrace(BacktraceIter::new(cursor.clone()));

        _ = writeln!(serial, "stack backtrace:");
        for (i, frame) in BacktraceIter::new(cursor).enumerate() {
            _ = writeln!(serial, "{i:>3}: 0x{frame:x}");
        }
    }

    _ = writeln!(
        serial,
        "\nhelp: this CPU fault indicates the misuse of unsafe code."
    );
    _ = writeln!(
        serial,
        "      Use a symbolizer tool to determine the location of the crash."
    );

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    _ = writeln!(
        &mut serial,
        "      (e.g. llvm-symbolizer -e ./target/armv7a-vex-v5/{profile}/program_name 0x{:x})",
        fault.program_counter
    );

    unsafe {
        vex_sdk::vexDisplayTextSize(1, 5);
        vex_sdk::vexDisplayPrintf(
            ErrorReport::BOX_MARGIN + ErrorReport::BOX_PADDING,
            272 - ErrorReport::BOX_MARGIN - ErrorReport::BOX_PADDING - 10,
            1,
            c"help: vexide.dev/docs/abort/ - Touch screen to re-print error.".as_ptr(),
        );
    }

    serial.flush();
}
