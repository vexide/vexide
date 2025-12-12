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
    target::{VexideTarget, VexideTargetError},
};

/// Encoding of an ARM32 `bkpt` instruction.
const BKPT_32_INSTRUCTION: Instruction = Instruction::Arm(0xE120_0070);
/// Encoding of an Thumb `bkpt` instruction.
const BKPT_16_INSTRUCTION: Instruction = Instruction::Thumb(0xBE00);

#[derive(Debug, Snafu)]
pub enum DebuggerError {
    #[snafu(context(false))]
    Io { source: std::io::Error },
    #[snafu(context(false))]
    GdbStub {
        source: GdbStubError<VexideTargetError, std::io::Error>,
    },
}

/// Debugger state machine for vexide.
pub struct VexideDebugger<S: DebugIO> {
    /// The list of breakpoints.
    ///
    /// Breakpoint idx 0 is the fixup breakpoint, if one exists.
    breaks: [Breakpoint; 10],
    fixup_idx: Option<NonZeroUsize>,
    single_step: bool,
    stream: S,
    gdb_buffer: Option<&'static mut [u8]>,
    gdb: Option<GdbStubStateMachine<'static, VexideTarget, S>>,
}

impl<S: DebugIO> VexideDebugger<S> {
    /// Creates a new debugger.
    #[must_use]
    pub fn new(stream: S) -> Self {
        Self {
            breaks: [Breakpoint {
                is_active: false,
                instr_addr: 0,
                instr_backup: Instruction::Arm(0),
            }; _],
            fixup_idx: None,
            single_step: false,
            stream,
            gdb_buffer: Some(Box::leak(vec![0; 0x2000].into_boxed_slice())),
            gdb: None,
        }
    }

    /// Returns the index of the tracked breakpoint at the specified address.
    #[must_use]
    pub fn query_address(&self, addr: usize) -> Option<usize> {
        self.breaks
            .iter()
            .enumerate()
            .find(|(_, b)| b.is_active && b.instr_addr == addr)
            .map(|(i, _)| i)
    }

    /// Replaces the trapped instruction in-memory with the old contents, so that returning from
    /// the current exception will continue execution.
    ///
    /// Since this process involves *temporarily disabling* the requested breakpoint, it will
    /// also create a "fixup" breakpoint that isn't visible to users on the next instruction
    /// that will be executed. This is a non-persistent breakpoint which solely exists to re-enable
    /// the current breakpoint.
    pub fn prepare_for_continue(&mut self, idx: usize) {
        let bkpt = &mut self.breaks[idx];
        if !bkpt.is_active {
            return;
        }

        unsafe {
            bkpt.disable();
        }

        // Fixup handling.
        if let Some(idx) = NonZeroUsize::new(idx) {
            // A non-zero index means it's a normal, persistent breakpoint.
            //
            // We just disabled it, which is bad but necessary since we need the program to
            // continue. Let's fix this by registering an ephemeral breakpoint that gets triggered
            // right after this one with the sole purpose of re-enabling this one.

            unsafe {
                self.register_fixup(idx);
            }
        } else if let Some(fixup_idx) = self.fixup_idx.take() {
            // This is a fixup breakpoint, so it's our responsibility to re-enable whatever
            // breakpoint got invalidated.
            let invalidated_bkpt = &mut self.breaks[fixup_idx.get()];

            unsafe {
                invalidated_bkpt.enable();
            }
        }

        unsafe {
            invalidate_icache();
        }
    }

    unsafe fn register_fixup(&mut self, idx: NonZeroUsize) {
        assert!(
            !self.breaks[0].is_active,
            "Tried to create multiple fixup breakpoints (is this possible)?"
        );

        let bkpt = &mut self.breaks[idx.get()];

        // Note: this is very temporary! In reality, this will have to decode the instruction
        // and do a better job at guessing where the next instruction is. Currently, breakpoints
        // cannot be placed on jumps because then we can't guess where to put the fixup!

        let next_addr = bkpt.instr_addr + bkpt.instr_backup.size();
        let instr_backup =
            unsafe { Instruction::read(next_addr as *mut u32, bkpt.instr_backup.is_thumb()) };

        self.breaks[0] = Breakpoint {
            is_active: true,
            instr_addr: next_addr,
            instr_backup,
        };
        self.fixup_idx = Some(idx);
    }

    fn drive_state_machine<'a>(
        &mut self,
        gdb: GdbStubStateMachine<'a, VexideTarget, S>,
        target: &mut VexideTarget,
    ) -> Result<GdbStubStateMachine<'a, VexideTarget, S>, DebuggerError> {
        match gdb {
            GdbStubStateMachine::Idle(mut gdb) => {
                let next_byte = gdb.borrow_conn().read()?;
                Ok(gdb.incoming_data(target, next_byte)?)
            }
            GdbStubStateMachine::Running(gdb) => {
                let stop_reason = if self.single_step {
                    SingleThreadStopReason::DoneStep
                } else {
                    SingleThreadStopReason::SwBreak(())
                };
                self.single_step = false;

                Ok(gdb.report_stop(target, stop_reason)?)
            }
            GdbStubStateMachine::CtrlCInterrupt(gdb) => {
                let stop_reason: Option<SingleThreadStopReason<_>> = None;
                Ok(gdb.interrupt_handled(target, stop_reason)?)
            }
            GdbStubStateMachine::Disconnected(gdb) => Ok(gdb.return_to_idle()),
        }
    }
}

unsafe impl<S: DebugIO> Debugger for VexideDebugger<S> {
    fn initialize(&mut self) {}

    unsafe fn register_breakpoint(
        &mut self,
        addr: usize,
        thumb: bool,
    ) -> Result<(), BreakpointError> {
        let mut next_inactive = None;

        // Skip the fixup breakpoint.
        for bkpt in self.breaks.iter_mut().skip(1) {
            if bkpt.is_active && bkpt.instr_addr == addr {
                return Err(BreakpointError::AlreadyExists);
            }

            if !bkpt.is_active && next_inactive.is_none() {
                next_inactive = Some(bkpt);
            }
        }

        let Some(bkpt) = next_inactive else {
            return Err(BreakpointError::NoSpace);
        };

        *bkpt = Breakpoint {
            is_active: true,
            instr_addr: addr,
            instr_backup: unsafe { Instruction::read(addr as *mut u32, thumb) },
        };

        unsafe {
            bkpt.enable();
            invalidate_icache();
        }

        Ok(())
    }

    unsafe fn handle_exception(&mut self, fault: &mut Fault<'_>) {
        // SAFETY: Since the address was able to be properly fetched, it implies it is valid for
        // reads.
        let instr = unsafe { fault.ctx.read_instr() };
        let mut is_explicit_bkpt = instr == BKPT_32_INSTRUCTION || instr == BKPT_16_INSTRUCTION;

        if let Some(idx) = self.query_address(fault.ctx.program_counter) {
            // This `bkpt` instruction is a placeholder tracked by our breakpoint manager. Let's
            // replace it and continue execution. In the future we may want to pause and
            // enter a terminal or something.

            is_explicit_bkpt = false;
            self.prepare_for_continue(idx);
        }

        let mut target = VexideTarget::new(*fault.ctx);

        // If this is the first time a breakpoint has happened, then we'll set up the state machine
        // for GDB.
        let mut state = self.gdb.take().unwrap_or_else(|| {
            let buffer = self.gdb_buffer.take().unwrap();
            let stub = GdbStubBuilder::new(self.stream.clone())
                .with_packet_buffer(buffer)
                .build()
                .unwrap();

            stub.run_state_machine(&mut target).unwrap()
        });

        // Enter debugging loop until it's time to resume.
        while !target.resume {
            state = self
                .drive_state_machine(state, &mut target)
                .expect("Error while processing debugger state");

            unsafe {
                vex_sdk::vexTasksRun();
            }
        }

        self.gdb = Some(state);

        if is_explicit_bkpt {
            // If the breakpoint was caused by an explicit `bkpt` instruction, then we don't
            // actually want to directly return to it since it'd just cause another
            // breakpoint as soon as we do. So, we need to advance by one instruction to
            // skip past it.
            fault.ctx.program_counter += instr.size();
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Breakpoint {
    is_active: bool,
    instr_addr: usize,
    instr_backup: Instruction,
}

impl Breakpoint {
    unsafe fn enable(&mut self) {
        // If the old instruction was Thumb, then our `bkpt` replacement needs to be Thumb too.
        let bkpt_instr = match self.instr_backup {
            Instruction::Arm(_) => BKPT_32_INSTRUCTION,
            Instruction::Thumb(_) => BKPT_16_INSTRUCTION,
        };

        unsafe {
            bkpt_instr.write_to(self.instr_addr as *mut u32);
        }
    }

    unsafe fn disable(&mut self) {
        unsafe {
            self.instr_backup.write_to(self.instr_addr as *mut u32);
        }
    }
}
