use std::num::NonZeroUsize;

use gdbstub::stub::{
    GdbStubBuilder, GdbStubError, SingleThreadStopReason, state_machine::GdbStubStateMachine,
};
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
                if gdb.borrow_conn().peek()?.is_none() {
                    return Ok(gdb.into());
                }

                let byte = gdb.borrow_conn().read()?;
                Ok(gdb.incoming_data(target, byte)?)
            }
            GdbStubStateMachine::Running(gdb) => {
                let stop_reason = if target.single_step {
                    SingleThreadStopReason::DoneStep
                } else {
                    SingleThreadStopReason::SwBreak(())
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
        // SAFETY: Since the address was able to be properly fetched, it implies it is valid for
        // reads.
        let instr = unsafe { fault.ctx.read_instr() };
        let mut is_explicit_bkpt =
            instr == Breakpoint::ARM_INSTR || instr == Breakpoint::THUMB_INSTR;

        if let Some(idx) = self.target.query_address(fault.ctx.program_counter) {
            // This `bkpt` instruction is a placeholder tracked by our breakpoint manager. Let's
            // replace it and continue execution. In the future we may want to pause and
            // enter a terminal or something.

            is_explicit_bkpt = false;
            self.target.prepare_for_continue(idx);
        }

        self.target.exception_ctx = Some(*fault.ctx);

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
        while !self.target.resume {
            gdb = Self::drive_state_machine(gdb, &mut self.target)
                .expect("Error while processing debugger state");

            unsafe {
                vex_sdk::vexTasksRun();
            }
        }

        self.gdb = Some(gdb);

        if is_explicit_bkpt {
            // If the breakpoint was caused by an explicit `bkpt` instruction, then we don't
            // actually want to directly return to it since it'd just cause another
            // breakpoint as soon as we do. So, we need to advance by one instruction to
            // skip past it.
            fault.ctx.program_counter += instr.size();
        }
    }
}
