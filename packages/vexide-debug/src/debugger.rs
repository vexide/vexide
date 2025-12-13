use std::num::NonZeroUsize;

use gdbstub::{common::Signal, stub::{
    GdbStubBuilder, GdbStubError, SingleThreadStopReason, state_machine::GdbStubStateMachine,
}};
use snafu::Snafu;
use vexide_startup::{
    abort_handler::fault::{Fault, Instruction},
    debugger::{BreakpointError, Debugger, invalidate_icache},
};

use crate::{
    DebugIO,
    dbg_target::{VexideTarget, VexideTargetError, breakpoint::Breakpoint},
};

#[derive(Debug, Snafu)]
pub enum DebuggerError {
    #[snafu(context(false))]
    Io { source: std::io::Error },
    #[snafu(context(false))]
    GdbStub {
        source: GdbStubError<VexideTargetError, std::io::Error>,
    },
}

/// Debugger state machine for handling remote connections.
pub struct VexideDebugger<S: DebugIO> {
    target: VexideTarget,
    stream: S,
    gdb_buffer: Option<&'static mut [u8]>,
    gdb: Option<GdbStubStateMachine<'static, VexideTarget, S>>,
}

impl<S: DebugIO> VexideDebugger<S> {
    /// Creates a new debugger.
    #[must_use]
    pub fn new(stream: S) -> Self {
        Self {
            target: VexideTarget::new(),
            stream,
            gdb_buffer: Some(Box::leak(vec![0; 0x2000].into_boxed_slice())),
            gdb: None,
        }
    }

    fn drive_state_machine<'a>(
        gdb: GdbStubStateMachine<'a, VexideTarget, S>,
        target: &mut VexideTarget,
    ) -> Result<GdbStubStateMachine<'a, VexideTarget, S>, DebuggerError> {
        match gdb {
            GdbStubStateMachine::Idle(mut gdb) => {
                if let Ok(byte) = gdb.borrow_conn().read() {
                    Ok(gdb.incoming_data(target, byte)?)
                } else {
                    Ok(gdb.into())
                }
            }
            GdbStubStateMachine::Running(gdb) => {
                let stop_reason = if target.single_step {
                    SingleThreadStopReason::DoneStep
                } else {
                    SingleThreadStopReason::Signal(Signal::SIGTRAP)
                };
                target.single_step = false;

                Ok(gdb.report_stop(target, stop_reason)?)
            }
            GdbStubStateMachine::CtrlCInterrupt(gdb) => {
                let stop_reason: Option<SingleThreadStopReason<_>> = None;
                Ok(gdb.interrupt_handled(target, stop_reason)?)
            }
            GdbStubStateMachine::Disconnected(gdb) => Ok(gdb.return_to_idle()),
        }
    }

    /// Returns the debugger's internal state.
    #[must_use]
    pub const fn target(&mut self) -> &mut VexideTarget {
        &mut self.target
    }

    /// Runs the debug console until the user indicates they want to continue program execution.
    fn run_debug_console(&mut self) {
        println!("CONSOLE");

        // If this is the first time a breakpoint has happened, then we'll set up the state machine
        // for GDB.
        let mut gdb = self.gdb.take().unwrap_or_else(|| {
            let buffer = self.gdb_buffer.take().unwrap();
            let stub = GdbStubBuilder::new(self.stream.clone())
                .with_packet_buffer(buffer)
                .build()
                .unwrap();

            stub.run_state_machine(&mut self.target).unwrap()
        });

        // Enter debugging loop until it's time to resume.

        self.target.reset_resume();
        while !self.target.resume {
            std::thread::yield_now();

            gdb = Self::drive_state_machine(gdb, &mut self.target)
                .expect("Error while processing debugger state");
        }

        self.target.resume = false;
        self.gdb = Some(gdb);
    }
}

unsafe impl<S: DebugIO> Debugger for VexideDebugger<S> {
    fn initialize(&mut self) {}

    unsafe fn register_breakpoint(
        &mut self,
        addr: usize,
        thumb: bool,
    ) -> Result<(), BreakpointError> {
        unsafe { self.target.register_breakpoint(addr, thumb) }
    }

    unsafe fn handle_exception(&mut self, fault: &mut Fault<'_>) {
        println!("BREAK");
        // Internal fixup breakpoints can skip all the normal debug loop logic once their side
        // effects are finished.
        let is_fixup = unsafe { self.target.apply_fixup(fault.ctx.program_counter) };
        if is_fixup {
            return;
        }

        // SAFETY: Since the address was able to be properly fetched, it implies it is valid for
        // reads.
        let instr = unsafe { fault.ctx.read_instr() };

        let tracked_bkpt = self.target.query_address(fault.ctx.program_counter);

        if let Some(idx) = tracked_bkpt {
            // If this is a tracked breakpoint (as opposed to an explicit `bkpt` call), then
            // we need to replace it with the real, backed-up instruction so that when we return,
            // the real code is run instead of throwing us straight back into this debug handler.
            self.target.prepare_for_continue(idx);
        }

        self.target.exception_ctx = Some(*fault.ctx);
        self.run_debug_console();

        // Normally we try to avoid an infinite loop of breakpoints by replacing tracked breakpoints
        // with their real instructions and re-running them. But if the `bkpt` *is* the real
        // instruction then we don't need to do the normal replace-and-rerun thing. Instead, we just
        // skip over it because it has been completed.
        if tracked_bkpt.is_none() {
            fault.ctx.program_counter += instr.size();
        }
    }
}
