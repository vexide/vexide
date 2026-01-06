use gdbstub::target::ext::monitor_cmd::{ConsoleOutput, MonitorCmd};

use crate::gdb_target::V5Target;

impl MonitorCmd for V5Target {
    fn handle_monitor_cmd(
        &mut self,
        data: &[u8],
        mut out: ConsoleOutput<'_>,
    ) -> Result<(), Self::Error> {
        let cmd_str = str::from_utf8(data).unwrap_or_default();

        let mut parts = cmd_str.split(' ');
        let cmd = parts.next().unwrap_or_default();

        if cmd.starts_with("br") {
            for (i, breakpt) in self.breaks.iter().enumerate() {
                gdbstub::outputln!(out, "{i:>2}: {breakpt:x?}");
            }
        } else if cmd.starts_with("mk") {
            if let Ok(addr) = usize::from_str_radix(parts.next().unwrap_or_default(), 16) {
                let res = unsafe { self.register_breakpoint(addr, false) };

                gdbstub::outputln!(out, "{res:x?}");
            } else {
                gdbstub::outputln!(out, "Invalid syntax.");
            }
        } else if cmd.starts_with("hw") {
            gdbstub::outputln!(out, "{:#x?}", self.hw_manager);
        } else {
            gdbstub::outputln!(out, "Unknown command.\n");
            gdbstub::outputln!(out, "Commands:");
            gdbstub::outputln!(out, " - monitor breaks         (View internal breakpoints)");
            gdbstub::outputln!(out, " - monitor mkbreak <ADDR> (Create breakpoint)");
            gdbstub::outputln!(out, " - monitor hwshow         (Show hardware break status)");
        }

        Ok(())
    }
}
