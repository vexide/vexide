use std::fmt::{self, Debug, Formatter};

use snafu::Snafu;
use zynq7000::devcfg::MmioDevCfg;

use crate::{
    gdb_target::arch::access_protected_mmio,
    regs::{
        DebugID, DebugLogic, DebugROMAddress, DebugSelfAddressOffset, DebugStatusControl,
        DebugValid, DebugVersion, MmioDebugLogic, SecureDebugEnable,
    },
};

#[derive(Debug, Snafu)]
pub enum ConfigureDebugError {
    /// The operating system has disabled hardware debugging.
    DebugLocked,
}

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
    /// # Errors
    ///
    /// [`ConfigureDebugError::DebugLocked`] is returned if hardware debugging is disabled by the
    /// OS.
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

        Self {
            capabilities: HardwareCapabilities {
                num_breakpoints,
                num_watchpoints,
            },
            mmio: unsafe { DebugLogic::new_mmio_at(mmio_base) },
        }
    }

    /// Disable all existing breakpoints and enable Monitor (debug exception) hardware debug mode.
    pub fn reset(&mut self) {
        let debug_status = DebugStatusControl::read_ext();
        assert!(!debug_status.secure_pl1_invasive_debug_disabled());

        // Note: before enabling halt or monitor debug mode for the first time, all breakpoints and
        // watchpoints need to be explicitly set as either enabled or disabled.

        for idx in 0..self.capabilities.num_breakpoints {
            self.mmio
                .modify_breakpoint_ctrl(idx.into(), |bkpt| bkpt.with_enable(false))
                .unwrap();

            self.mmio
                .modify_watchpoint_ctrl(idx.into(), |bkpt| bkpt.with_enable(false))
                .unwrap();
        }

        // Route breakpoint/watchpoint debug events to debug exceptions. This allows us to catch
        // them at runtime as prefetch/data aborts instead of halting the processor.
        // (see Table C3-1 Processor behavior on debug events)
        unsafe {
            debug_status
                .with_halting_debug_mode(false)
                .with_monitor_debug_mode(true)
                .write_ext();
        }
    }

    #[must_use]
    pub const fn capabilities(&self) -> HardwareCapabilities {
        self.capabilities
    }

    pub fn add_breakpoint(&self) {}
}

impl Debug for HwBreakpointManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("HwBreakpointManager")
            .field("capabilities", &self.capabilities)
            .field("mmio", &unsafe { self.mmio.ptr() })
            .finish()
    }
}
