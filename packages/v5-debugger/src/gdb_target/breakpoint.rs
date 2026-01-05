//! Software breakpoint management.

use gdbstub::target::{
    TargetError, TargetResult, ext::breakpoints::{Breakpoints, HwBreakpoint, HwBreakpointOps, SwBreakpoint, SwBreakpointOps}
};
use gdbstub_arch::arm::ArmBreakpointKind;
use snafu::Snafu;

use crate::instruction::Instruction;

use super::{
    V5Target,
    cache::{self, CacheTarget},
};

/// A software breakpoint.
#[derive(Debug, Clone, Copy)]
pub struct Breakpoint {
    /// Indicates whether this breakpoint is considered active.
    ///
    /// This is distinct from it being enabled, which indicates that the breakpoint is actially
    /// written to system memory and is ready to interrupt program execution. After a breakpoint is
    /// triggered, it must temporarily become disabled to resume execution.
    pub is_active: bool,
    pub instr_addr: usize,
    pub instr_backup: Instruction,
}

impl Breakpoint {
    /// Encoding of an ARM32 `bkpt` instruction.
    pub const ARM_INSTR: Instruction = Instruction::Arm(0xE120_0070);
    /// Encoding of an Thumb `bkpt` instruction.
    pub const THUMB_INSTR: Instruction = Instruction::Thumb(0xBE00);

    /// Enables the breakpoint by overwriting its instruction's memory with a `bkpt` call.
    ///
    /// Note that this does not handle backing up the old instruction.
    pub unsafe fn enable(&mut self) {
        debug_assert!(self.is_active);

        // If the old instruction was Thumb, then our `bkpt` replacement needs to be Thumb too.
        let bkpt_instr = match self.instr_backup {
            Instruction::Arm(_) => Self::ARM_INSTR,
            Instruction::Thumb(_) => Self::THUMB_INSTR,
        };

        unsafe {
            bkpt_instr.write_to(self.instr_addr as *mut u32);
        }
    }

    /// Disables the breakpoint by replacing its instruction's memory with the backed up, real
    /// operation.
    pub unsafe fn disable(&mut self) {
        unsafe {
            self.instr_backup.write_to(self.instr_addr as *mut u32);
        }
    }

    /// Returns the cache target for this breakpoint's instruction.
    #[must_use]
    pub const fn cache_target(&self) -> CacheTarget {
        CacheTarget::Address(self.instr_addr)
    }
}

impl Breakpoints for V5Target {
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }

    fn support_hw_breakpoint(&mut self) -> Option<HwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for V5Target {
    fn add_sw_breakpoint(
        &mut self,
        addr: u32,
        kind: ArmBreakpointKind,
    ) -> TargetResult<bool, Self> {
        let result = unsafe {
            self.register_breakpoint(addr as usize, matches!(kind, ArmBreakpointKind::Thumb32))
        };

        Ok(result.is_ok())
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: u32,
        _kind: ArmBreakpointKind,
    ) -> TargetResult<bool, Self> {
        let changed = unsafe { self.remove_breakpoint(addr as usize) };
        Ok(changed)
    }
}

impl HwBreakpoint for V5Target {
    fn add_hw_breakpoint(
            &mut self,
            addr: u32,
            kind: ArmBreakpointKind,
        ) -> TargetResult<bool, Self> {
        Err(TargetError::Errno(0x26))
    }

    fn remove_hw_breakpoint(
            &mut self,
            addr: u32,
            kind: ArmBreakpointKind,
        ) -> TargetResult<bool, Self> {
        Err(TargetError::Errno(0x26))
    }
}

impl V5Target {
    pub unsafe fn register_breakpoint(
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
        }

        cache::sync_instruction(bkpt.cache_target());

        Ok(())
    }

    pub unsafe fn remove_breakpoint(&mut self, addr: usize) -> bool {
        let num_updated = self
            .breaks
            .iter_mut()
            .skip(1)
            .filter(|b| b.is_active && b.instr_addr == addr)
            .map(|bkpt| {
                unsafe {
                    bkpt.disable();
                }

                bkpt.is_active = false;
                cache::sync_instruction(bkpt.cache_target());
            })
            .count();

        num_updated > 0
    }
}

#[derive(Debug, Snafu)]
pub enum BreakpointError {
    /// There is already a breakpoint with this address.
    AlreadyExists,
    /// There are no free breakpoint slots.
    NoSpace,
}
