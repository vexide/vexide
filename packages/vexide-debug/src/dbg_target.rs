use std::{fmt::Display, num::NonZeroUsize};

use gdbstub::{
    arch::Arch,
    common::{Signal, Tid},
    target::{
        Target, TargetError, TargetResult,
        ext::{
            base::{
                BaseOps,
                single_register_access::{SingleRegisterAccess, SingleRegisterAccessOps},
                singlethread::{SingleThreadBase, SingleThreadResume, SingleThreadResumeOps},
            },
            breakpoints::{Breakpoints, BreakpointsOps, SwBreakpoint, SwBreakpointOps},
            monitor_cmd::{ConsoleOutput, MonitorCmd, MonitorCmdOps},
        },
    },
};
use gdbstub_arch::arm::{
    ArmBreakpointKind,
    reg::{ArmCoreRegs, id::ArmCoreRegId},
};
use snafu::Snafu;
use vexide_startup::{
    abort_handler::fault::{ExceptionContext, Instruction, ProgramStatus},
    debugger::invalidate_icache,
};

use crate::{DebugIO, VexideDebugger, arch::ARMv7, dbg_target::breakpoint::Breakpoint};

pub mod breakpoint;
mod memory;

#[derive(Debug, Snafu)]
pub enum VexideTargetError {}

/// Debugger state storage.
pub struct VexideTarget {
    pub exception_ctx: Option<ExceptionContext>,
    pub resume: bool,

    /// The list of breakpoints.
    ///
    /// Breakpoint idx 0 is the fixup breakpoint, if one exists.
    pub breaks: [Breakpoint; 10],
    pub fixup_idx: Option<NonZeroUsize>,
    pub single_step: bool,
}

impl Default for VexideTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl VexideTarget {
    pub const fn new() -> Self {
        Self {
            exception_ctx: None,
            resume: false,
            breaks: [Breakpoint {
                is_active: false,
                instr_addr: 0,
                instr_backup: Instruction::Arm(0),
            }; _],
            fixup_idx: None,
            single_step: false,
        }
    }

    /// Returns the index of the tracked breakpoint at the specified address.
    #[must_use]
    pub fn query_address(&self, addr: usize) -> Option<usize> {
        self.breaks
            .iter()
            .enumerate()
            .skip(1)
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
            // breakpoint got invalidated, then get out of the way.

            bkpt.is_active = false;
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
}

impl Target for VexideTarget {
    type Arch = ARMv7;
    type Error = VexideTargetError;

    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }

    fn support_monitor_cmd(&mut self) -> Option<MonitorCmdOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for VexideTarget {
    fn read_registers(&mut self, regs: &mut <ARMv7 as Arch>::Registers) -> TargetResult<(), Self> {
        if let Some(ctx) = &mut self.exception_ctx {
            *regs = ArmCoreRegs {
                r: ctx.registers,
                cpsr: ctx.spsr.0,
                lr: ctx.link_register as u32,
                pc: ctx.program_counter as u32,
                sp: ctx.stack_pointer as u32,
            };
        } else {
            return Err(TargetError::NonFatal);
        }

        Ok(())
    }

    fn write_registers(&mut self, regs: &<ARMv7 as Arch>::Registers) -> TargetResult<(), Self> {
        if let Some(ctx) = &mut self.exception_ctx {
            *ctx = ExceptionContext {
                registers: regs.r,
                spsr: ProgramStatus(regs.cpsr),
                link_register: regs.lr as usize,
                program_counter: regs.pc as usize,
                stack_pointer: regs.sp as usize,
                ..*ctx
            };
        } else {
            return Err(TargetError::NonFatal);
        }

        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u32, data: &mut [u8]) -> TargetResult<usize, Self> {
        let bytes_read = unsafe { memory::read(start_addr as usize, data)? };

        Ok(bytes_read)
    }

    fn write_addrs(&mut self, start_addr: u32, data: &[u8]) -> TargetResult<(), Self> {
        unsafe {
            memory::write(start_addr as usize, data)?;
        }

        Ok(())
    }

    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }

    fn support_single_register_access(&mut self) -> Option<SingleRegisterAccessOps<'_, (), Self>> {
        Some(self)
    }
}

impl SingleThreadResume for VexideTarget {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.resume = true;
        Ok(())
    }
}

impl SingleRegisterAccess<()> for VexideTarget {
    fn read_register(
        &mut self,
        _tid: (),
        reg_id: ArmCoreRegId,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        if let Some(ctx) = &mut self.exception_ctx {
            let reg = match reg_id {
                ArmCoreRegId::Gpr(rid) => ctx.registers.get(rid as usize).copied(),
                ArmCoreRegId::Sp => Some(ctx.stack_pointer as u32),
                ArmCoreRegId::Lr => Some(ctx.link_register as u32),
                ArmCoreRegId::Pc => Some(ctx.program_counter as u32),
                ArmCoreRegId::Cpsr => Some(ctx.spsr.0),
                _ => None,
            };

            if let Some(reg) = reg {
                let bytes = reg.to_ne_bytes();
                buf.copy_from_slice(&bytes);
                Ok(bytes.len())
            } else {
                Ok(0)
            }
        } else {
            Err(TargetError::NonFatal)
        }
    }

    fn write_register(
        &mut self,
        _tid: (),
        reg_id: ArmCoreRegId,
        val: &[u8],
    ) -> TargetResult<(), Self> {
        if let Some(ctx) = &mut self.exception_ctx
            && let Ok(bytes) = val.try_into()
        {
            let val = u32::from_ne_bytes(bytes);

            match reg_id {
                ArmCoreRegId::Gpr(rid) => {
                    let Some(storage) = ctx.registers.get_mut(rid as usize) else {
                        return Err(TargetError::NonFatal);
                    };

                    *storage = val;
                }
                ArmCoreRegId::Sp => ctx.stack_pointer = val as usize,
                ArmCoreRegId::Lr => ctx.link_register = val as usize,
                ArmCoreRegId::Pc => ctx.program_counter = val as usize,
                ArmCoreRegId::Cpsr => ctx.spsr = ProgramStatus(val),
                _ => return Err(TargetError::NonFatal),
            }

            Ok(())
        } else {
            Err(TargetError::NonFatal)
        }
    }
}

impl MonitorCmd for VexideTarget {
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
        } else {
            gdbstub::outputln!(out, "Unknown command.\n");
            gdbstub::outputln!(out, "Commands:");
            gdbstub::outputln!(out, " - monitor breaks         (View internal breakpoints)");
            gdbstub::outputln!(out, " - monitor mkbreak <ADDR> (Create breakpoint)");
        }

        Ok(())
    }
}
