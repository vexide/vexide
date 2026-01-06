use std::fmt::{self, Debug, Formatter};

use arbitrary_int::*;
use gdbstub_arch::arm::ArmBreakpointKind;
use snafu::Snafu;
use zynq7000::devcfg::MmioDevCfg;

use crate::{
    gdb_target::arch::access_protected_mmio,
    regs::{
        BreakpointType, DebugID, DebugLogic, DebugEventReason, DebugROMAddress,
        DebugSelfAddressOffset, DebugStatusControl, DebugValid, DebugVersion, MmioDebugLogic,
        PrivilegeModeFilter, SecureDebugEnable, SecurityFilter,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareCapabilities {
    pub num_breakpoints: u8,
    pub num_watchpoints: u8,
}

pub struct HwBreakpointManager {
    capabilities: HardwareCapabilities,
    mmio: MmioDebugLogic<'static>,
}

impl HwBreakpointManager {
    /// Sets up hardware debugging.
    ///
    /// # Panics
    ///
    /// A panic is triggered if:
    ///
    /// - Hardware debugging is locked by the board.
    /// - The device has no MMIO interface for debug registers.
    pub fn setup(devcfg: &mut MmioDevCfg<'_>) -> Self {
        // Enable access to the board's debug hardware.
        let enabled = access_protected_mmio(|| {
            // Code that runs before us might have disabled writes to the debug logic, so fail early
            // if it's locked OFF.
            let lock = devcfg.read_lock();
            if lock.debug() {
                let ctrl = devcfg.read_control();
                return ctrl.invasive_debug_enable() && ctrl.secure_invasive_debug_enable();
            }

            // Enable the CPU's invasive debug features.
            devcfg.modify_control(|ctrl| {
                ctrl.with_invasive_debug_enable(true)
                    .with_secure_invasive_debug_enable(true)
            });

            true
        });
        assert!(
            enabled,
            "The operating system has disabled hardware debugging."
        );

        // Enable debugging in the Secure PL0 processor mode.
        let secure_debug = SecureDebugEnable::read();
        unsafe {
            secure_debug.with_secure_user_invasive_debug(true).write();
        }

        // Look up where we will access debug MMIO from.

        let rom_base = DebugROMAddress::read();
        let self_address_offset = DebugSelfAddressOffset::read();
        assert!(
            rom_base.valid() == Ok(DebugValid::Valid)
                && self_address_offset.valid() == Ok(DebugValid::Valid),
            "This device has no debug logic MMIO"
        );

        let mmio_base = rom_base
            .value()
            .wrapping_add_signed(self_address_offset.value());

        let debug_id = DebugID::read();
        let num_breakpoints = debug_id.brps().value() + 1;
        let num_watchpoints = debug_id.wrps().value() + 1;

        let mut manager = Self {
            capabilities: HardwareCapabilities {
                num_breakpoints,
                num_watchpoints,
            },
            mmio: unsafe { DebugLogic::new_mmio_at(mmio_base) },
        };

        manager.reset();
        manager
    }

    /// Disable all existing breakpoints and enable Monitor (debug exception) hardware debug mode.
    pub fn reset(&mut self) {
        // Note: before enabling halt or monitor debug mode for the first time, all breakpoints and
        // watchpoints need to be explicitly set as either enabled or disabled.

        for idx in 0..self.capabilities.num_breakpoints {
            self.mmio
                .modify_breakpoint_ctrl(idx.into(), |bkpt| bkpt.with_enabled(false))
                .unwrap();
        }

        for idx in 0..self.capabilities.num_watchpoints {
            self.mmio
                .modify_watchpoint_ctrl(idx.into(), |wapt| wapt.with_enabled(false))
                .unwrap();
        }

        cortex_ar::asm::dsb();

        // Route breakpoint/watchpoint debug events to debug exceptions. This allows us to catch
        // them at runtime as prefetch/data aborts instead of halting the processor.
        // (see Table C3-1 Processor behavior on debug events)
        self.mmio.modify_status_control_ext(|debug_ctrl| {
            debug_ctrl
                .with_halting_debug_mode(false)
                .with_monitor_debug_mode(true)
        });

        cortex_ar::asm::dsb();
        cortex_ar::asm::isb();
    }

    #[must_use]
    pub const fn capabilities(&self) -> HardwareCapabilities {
        self.capabilities
    }

    /// Registers and activates a hardware breakpoint matching the given address.
    ///
    /// # Errors
    ///
    /// An error is returned if there are no more hardware breakpoints available.
    pub fn add_breakpoint_at(
        &mut self,
        addr: u32,
        specificity: Specificity,
        kind: &ArmBreakpointKind,
    ) -> Result<(), BreakpointError> {
        let (new_word, new_bas) = split_addr(addr, kind);

        // First, try and find an existing breakpoint at the given word to avoid making a new one.
        // (This is possible for Thumb instructions, where 2 can be in the same word.)
        let mut next_disabled_idx = None;

        for idx in 0..self.capabilities.num_breakpoints {
            let mut existing_bkpt = self.mmio.read_breakpoint_ctrl(idx as usize).unwrap();
            let existing_word = self.mmio.read_breakpoint_value(idx as usize).unwrap();

            if !existing_bkpt.enabled() && next_disabled_idx.is_none() {
                next_disabled_idx = Some(idx as usize);
            }

            if !existing_bkpt.enabled()
                || existing_bkpt.breakpoint_type() != Ok(specificity.into())
                || new_word != existing_word
            {
                continue;
            }

            // Enable matching this instruction's part of the word.
            existing_bkpt.set_byte_address_select(existing_bkpt.byte_address_select() | new_bas);

            self.mmio
                .write_breakpoint_ctrl(idx as usize, existing_bkpt)
                .unwrap();

            cortex_ar::asm::dsb();
            cortex_ar::asm::isb();

            return Ok(());
        }

        // We can't reuse any existing breakpoints, so make a new one.
        let Some(bkpt_index) = next_disabled_idx else {
            return Err(BreakpointError::NoMoreBreakpoints);
        };

        // We store address aligned to the 4-byte word containing the address because breakpoints
        // consider regions 4 bytes large and must be aligned as such.
        self.mmio
            .write_breakpoint_value(bkpt_index, new_word)
            .unwrap();

        self.mmio
            .modify_breakpoint_ctrl(bkpt_index, |bkpt| {
                bkpt.with_enabled(true)
                    .with_byte_address_select(new_bas)
                    // No mask, match exact address
                    .with_address_range_mask(u5::new(0b00000))
                    // No linked Context ID breakpoint
                    .with_linked_breakpoint_index(u4::new(0))
                    .with_breakpoint_type(specificity.into())
                    // Don't trigger inside abort mode, and "step over" IRQs
                    .with_privileged_mode_ctrl(PrivilegeModeFilter::UserSystemSupervisorOnly)
                    .with_security_state_ctrl(SecurityFilter::All)
            })
            .unwrap();

        cortex_ar::asm::dsb();
        cortex_ar::asm::isb();

        Ok(())
    }

    /// Removes all breakpoints at the given address with the given kind and type.
    ///
    /// Returns whether any changes were made.
    pub fn remove_breakpoint_at(
        &mut self,
        addr: u32,
        specificity: Specificity,
        kind: &ArmBreakpointKind,
    ) -> bool {
        let (search_word, byte_address_select) = split_addr(addr, kind);

        let mut anything_removed = false;
        for bkpt_index in 0..self.capabilities.num_breakpoints {
            let mut bkpt = self.mmio.read_breakpoint_ctrl(bkpt_index as usize).unwrap();
            if bkpt.breakpoint_type() != Ok(specificity.into()) || !bkpt.enabled() {
                continue;
            }

            let bkpt_word = self
                .mmio
                .read_breakpoint_value(bkpt_index as usize)
                .unwrap();
            if bkpt_word != search_word {
                continue;
            }

            let new_bas = bkpt.byte_address_select() & !byte_address_select;
            if new_bas != bkpt.byte_address_select() {
                anything_removed = true;
            }

            if new_bas.value() == 0 {
                // No more reason to have breakpoint, it won't match any addresses inside its span.
                bkpt.set_enabled(false);
            }

            bkpt.set_byte_address_select(new_bas);
            self.mmio
                .write_breakpoint_ctrl(bkpt_index as usize, bkpt)
                .unwrap();

            cortex_ar::asm::dsb();
            cortex_ar::asm::isb();
        }

        anything_removed
    }

    #[must_use]
    pub fn last_break_reason(&self) -> Option<DebugEventReason> {
        let status = self.mmio.read_status_control_ext();
        status.method_of_entry().ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Specificity {
    Match,
    Mismatch,
}

impl From<Specificity> for BreakpointType {
    fn from(value: Specificity) -> Self {
        match value {
            Specificity::Match => BreakpointType::UnlinkedInstrAddressMatch,
            Specificity::Mismatch => BreakpointType::UnlinkedInstrAddressMismatch,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum BreakpointError {
    NoMoreBreakpoints,
}

impl Debug for HwBreakpointManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bkpt_values = (0..self.capabilities.num_breakpoints)
            .map(|i| self.mmio.read_breakpoint_value(i as usize).unwrap())
            .collect::<Vec<_>>();

        let bkpt_ctrls = (0..self.capabilities.num_breakpoints)
            .map(|i| self.mmio.read_breakpoint_ctrl(i as usize).unwrap())
            .collect::<Vec<_>>();

        f.debug_struct("HwBreakpointManager")
            .field("capabilities", &self.capabilities)
            .field("mmio_ptr", &unsafe { self.mmio.ptr() })
            .field("bkpt_values", &bkpt_values)
            .field("bkpt_ctrls", &bkpt_ctrls)
            .finish_non_exhaustive()
    }
}

/// Splits an address into the word containing it and the byte-address-select that would match
/// the instruction's offset into the word.
fn split_addr(addr: u32, kind: &ArmBreakpointKind) -> (u32, u4) {
    let word = addr & !0b11;

    // Specify which addresses inside the 4-byte breakpoint to match. Multi-byte instructions
    // are considered to inhabit all of their addresses at once.
    let byte_address_select = match kind {
        // The instruction spans 4 bytes, so the breakpoint needs to match over its the entire
        // 4-byte value: [0-3].
        ArmBreakpointKind::Arm32 => {
            assert!(addr.is_multiple_of(4));
            u4::new(0b1111)
        }
        // 16-bit Thumb address - match either addresses ending in [0-1] or [2-3], depending on
        // which side of the word it's aligned to.
        // (Although 4-byte Thumb instructions are a thing, we can treat them the same as 2-byte
        // ones. See <Table C3-2> Effect of byte address selection on Breakpoint generation.)
        ArmBreakpointKind::Thumb16 | ArmBreakpointKind::Thumb32 => {
            assert!(addr.is_multiple_of(2));
            if addr.is_multiple_of(4) {
                u4::new(0b0011)
            } else {
                u4::new(0b1100)
            }
        }
    };

    (word, byte_address_select)
}
