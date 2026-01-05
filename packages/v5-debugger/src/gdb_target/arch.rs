use std::arch::asm;

use gdbstub::arch::Arch;
use gdbstub_arch::arm::{
    ArmBreakpointKind,
    reg::{ArmCoreRegs, id::ArmCoreRegId},
};
use snafu::Snafu;
use zynq7000::devcfg::MmioDevCfg;

/// The ARMv7 architecture.
pub enum ArmV7 {}

impl Arch for ArmV7 {
    type Usize = u32;
    type BreakpointKind = ArmBreakpointKind;
    type RegId = ArmCoreRegId;
    type Registers = ArmCoreRegs;

    fn target_description_xml() -> Option<&'static str> {
        Some(include_str!("target.xml"))
    }
}

#[derive(Debug, Snafu)]
pub enum ConfigureDebugError {
    /// The operating system has disabled hardware breakpoints.
    DebugLocked,
}

pub fn configure_debug(devcfg: &mut MmioDevCfg<'_>) -> Result<(), ConfigureDebugError> {
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


    // Pre-setup: collect environment details
    // 1. Access debug ID
    // -> Confirm v7 debug with all cp14 registers (debug mode)
    // -> Store number of breaks and watches

    // 2. Access debug status/control
    // -> Confirm not "Secure PL1 Invasive Debug Disabled"

    // Setup
    // 1. Configure monitor mode so debug exceptions are generated
    // -> Access debug status/control, enable monitor debug MDBGen

    // 2. Enable invasive debug by setting DBGEN to HIGH (C2.3)
    // 3. Enable Secure PL0 debug by setting SDER.SUIDEN, Secure Debug Enable Register
    // 4. Enable Secure PL1 debug by setting SPIDEN to HIGH

    Ok(())
}

fn access_protected_mmio<T>(inner: impl FnOnce() -> T) -> T {
    // Disable MMU
    unsafe {
        asm!(
            "cpsid if", // Enter critical section
            "mrc p15, 0, r0, c1, c0, 0", // r0 = SCTLR
            "bic r0, r0, #0x1", // r0.M bit = 0
            "dsb", // Memory barriers before MMU manipulation
            "isb",
            "mcr p15, 0, r0, c1, c0, 0", // SCTLR = r0
            "isb", // Is this needed?
            options(nostack),
            lateout("r0") _,
        );
    }

    let ret = inner();

    // Enable MMU
    unsafe {
        asm!(
            "mrc p15, 0, r0, c1, c0, 0", // r0 = SCTLR
            "orr r0, r0, #0x1", // r0.M bit = 1
            "dsb", // Memory barriers before MMU manipulation
            "isb",
            "mcr p15, 0, r0, c1, c0, 0", // SCTLR = r0
            "isb",
            "cpsie if", // Exit critical section
            options(nostack),
            lateout("r0") _,
        );
    }

    ret
}
