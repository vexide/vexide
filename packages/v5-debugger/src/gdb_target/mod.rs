#![allow(clippy::missing_safety_doc)]

use std::convert::Infallible;

use gdbstub::{
    arch::Arch,
    target::{
        Target, TargetError, TargetResult,
        ext::{
            base::{
                BaseOps,
                single_register_access::SingleRegisterAccessOps,
                singlethread::{SingleThreadBase, SingleThreadResumeOps},
            },
            breakpoints::BreakpointsOps,
            monitor_cmd::MonitorCmdOps,
        },
    },
};
use gdbstub_arch::arm::reg::ArmCoreRegs;
use zynq7000::devcfg::{DevCfg, MmioDevCfg};

use crate::{
    cache,
    exception::{DebugEventContext, ProgramStatus},
    gdb_target::{
        arch::{ArmV7, hw::HwBreakpointManager},
        breakpoint::Breakpoint,
    },
    instruction::Instruction,
};

pub mod arch;
pub mod breakpoint;
pub mod monitor;
pub mod resume;
pub mod single_register_access;

/// Debugger state storage.
pub struct V5Target {
    pub exception_ctx: Option<DebugEventContext>,
    pub resume: bool,
    pub single_step: bool,

    /// The list of breakpoints.
    ///
    /// Breakpoint idx 0 is the fixup breakpoint, if one exists.
    pub breaks: [Breakpoint; 10],
    pub fixup_idx: usize,
    pub hw_manager: HwBreakpointManager,
}

impl Default for V5Target {
    fn default() -> Self {
        Self::new()
    }
}

impl V5Target {
    #[must_use]
    pub fn new() -> Self {
        let mut devcfg = unsafe { DevCfg::new_mmio_fixed() };

        Self {
            exception_ctx: None,
            resume: false,
            breaks: [Breakpoint {
                is_active: false,
                instr_addr: 0,
                instr_backup: Instruction::Arm(0),
            }; _],
            fixup_idx: 0,
            single_step: false,
            hw_manager: HwBreakpointManager::setup(&mut devcfg),
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
    /// also create an internal "fixup" breakpoint to re-enable the given breakpoint. (See
    /// [`Self::register_fixup`])
    pub fn prepare_for_continue(&mut self, idx: usize) {
        assert!(idx != 0);

        let bkpt = &mut self.breaks[idx];
        if !bkpt.is_active {
            return;
        }

        // Disabling the current breakpoint allows us to continue execution without immediately
        // triggering it again.
        unsafe {
            bkpt.disable();
        }

        cache::sync_instruction(bkpt.cache_target());

        // This is supposed to be a persistent breakpoint, so we have to re-enable it at some
        // point in the future. To enable this behavior, guess what the next instruction will
        // be and put an internal breakpoint on it.
        unsafe {
            self.register_fixup(idx);
        }
    }

    /// Applies any fixup operation that this breakpoint is responsible for.
    ///
    /// Returns whether a fixup breakpoint was inhabiting the given address.
    pub unsafe fn apply_fixup(&mut self, addr: usize) -> bool {
        let fixup = &mut self.breaks[0];

        // Ensure this is an active fixup.
        if !fixup.is_active || fixup.instr_addr != addr {
            return false;
        }

        // This is a fixup breakpoint, so it's our responsibility to re-enable whatever
        // breakpoint got invalidated, then get out of the way.

        debug_assert!(self.fixup_idx != 0);

        fixup.is_active = false;
        unsafe {
            fixup.disable();
        }

        let invalidated_bkpt = &mut self.breaks[self.fixup_idx];
        unsafe {
            invalidated_bkpt.enable();
        }

        true
    }

    /// Clears the resume flag.
    pub const fn reset_resume(&mut self) {
        self.resume = false;
        self.single_step = false;
    }

    /// Marks the debugger as ready to resume.
    pub const fn resume(&mut self) {
        self.resume = true;
    }

    /// Marks the debugger as ready to resume for a single-step.
    pub const fn step(&mut self) {
        self.resume = true;
        self.single_step = true;
    }

    /// Creates a fixup breakpoint responsible for enabling the given breakpoint.
    ///
    /// This function places a new breakpoint on the next instruction that will be evaluated after
    /// the given breakpoint returns. The new fixup breakpoint will not enter debug mode like
    /// standard persistent breakpoints, and will instead only enable the given breakpoint and
    /// return.
    ///
    /// This functionality is used to support persistent breakpoints, since returning from a
    /// breakpoint requires you to temporarily disable it (otherwise it would immediately trigger
    /// again).
    ///
    /// # Safety
    ///
    /// Fixup breakpoints must not be registered for breakpoints on branching instructions. This
    /// requirement may change in the future.
    ///
    /// # Panics
    ///
    /// A panic will be emitted if a fixup breakpoint already exists, or if the given breakpoint
    /// is not active.
    unsafe fn register_fixup(&mut self, idx: usize) {
        assert!(!self.breaks[0].is_active, "Tried to create multiple fixups");

        let bkpt = &mut self.breaks[idx];
        assert!(
            bkpt.is_active,
            "Can't create a fixup for an inactive breakpoint"
        );

        println!("MKFIX");

        // Note: this is very temporary! In reality, this will have to decode the instruction
        // and do a better job at guessing where the next instruction is. Currently, breakpoints
        // cannot be placed on jumps because then we can't guess where to put the fixup!

        let next_addr = bkpt.instr_addr + bkpt.instr_backup.size();
        let instr_backup =
            unsafe { Instruction::read(next_addr as *mut u32, bkpt.instr_backup.is_thumb()) };

        let mut fixup = Breakpoint {
            is_active: true,
            instr_addr: next_addr,
            instr_backup,
        };

        self.breaks[0] = fixup;
        self.fixup_idx = idx;

        unsafe {
            fixup.enable();
        }

        cache::sync_instruction(fixup.cache_target());
    }
}

impl Target for V5Target {
    type Arch = ArmV7;
    type Error = Infallible;

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

impl SingleThreadBase for V5Target {
    fn read_registers(&mut self, regs: &mut <ArmV7 as Arch>::Registers) -> TargetResult<(), Self> {
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

    fn write_registers(&mut self, regs: &<ArmV7 as Arch>::Registers) -> TargetResult<(), Self> {
        if let Some(ctx) = &mut self.exception_ctx {
            *ctx = DebugEventContext {
                _pad: 0,
                registers: regs.r,
                spsr: ProgramStatus(regs.cpsr),
                link_register: regs.lr as usize,
                program_counter: regs.pc as usize,
                stack_pointer: regs.sp as usize,
                // ..*ctx
            };
        } else {
            return Err(TargetError::NonFatal);
        }

        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u32, data: &mut [u8]) -> TargetResult<usize, Self> {
        // TODO: check MMU table to ensure these pages are readable.
        unsafe {
            core::ptr::copy(start_addr as *const u8, data.as_mut_ptr(), data.len());
        }

        Ok(data.len())
    }

    fn write_addrs(&mut self, start_addr: u32, data: &[u8]) -> TargetResult<(), Self> {
        unsafe {
            core::ptr::copy(data.as_ptr(), start_addr as *mut u8, data.len());
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
