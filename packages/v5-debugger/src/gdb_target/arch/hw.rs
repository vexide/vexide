use snafu::Snafu;
use zynq7000::devcfg::MmioDevCfg;

use crate::{
    gdb_target::arch::access_protected_mmio, regs::{DebugID, DebugStatusControl, SecureDebugEnable}
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
}

impl HwBreakpointManager {
    /// Sets up hardware debugging.
    ///
    /// # Errors
    ///
    /// [`ConfigureDebugError::DebugLocked`] is returned if hardware debugging is disabled by the
    /// OS.
    pub fn setup(devcfg: &mut MmioDevCfg<'_>) -> Result<Self, ConfigureDebugError> {
        // Put CPU in hardware debugging mode.
        access_protected_mmio(|| {
            let lock = devcfg.read_lock();
            if lock.debug() {
                let ctrl = devcfg.read_control();
                let enabled = ctrl.invasive_debug_enable() && ctrl.secure_invasive_debug_enable();
                if !enabled {
                    return Err(ConfigureDebugError::DebugLocked);
                }
            }

            // Enable the CPU's invasive debug features.
            devcfg.modify_control(|ctrl| {
                ctrl.with_invasive_debug_enable(true)
                    .with_secure_invasive_debug_enable(true)
            });

            Ok(())
        })?;

        // Make sure we're compatible with this implementation of hardware debug.
        let debug_id = DebugID::read();

        let num_breakpoints = debug_id.brps().value() + 1;
        let num_watchpoints = debug_id.wrps().value() + 1;

        let debug_status = DebugStatusControl::read_ext();
        // When we put the CPU in debugging mode, we enabled Secure Invasive Debugging, so it should
        // now read as enabled.
        assert!(
            !debug_status.secure_pl1_invasive_debug_disabled(),
            "Secure PL1 Invasive Debug unexpectedly disabled"
        );

        // Route breakpoint/watchpoint debug events to debug exceptions. This allows us to catch
        // them at runtime as prefetch/data aborts instead of halting the processor.
        // (see Table C3-1 Processor behavior on debug events)
        unsafe {
            debug_status
                .with_halting_debug_mode(false)
                .with_monitor_debug_mode(true)
                .write_ext();
        }

        // Enable debugging in Secure PL0 processor modes.
        let secure_debug = SecureDebugEnable::read();
        unsafe {
            secure_debug.with_secure_user_invasive_debug(true).write();
        }

        Ok(Self {
            capabilities: HardwareCapabilities {
                num_breakpoints,
                num_watchpoints,
            },
        })
    }

    #[must_use]
    pub const fn capabilities(&self) -> HardwareCapabilities {
        self.capabilities
    }

    pub fn add_breakpoint(&self) {

    }
}
