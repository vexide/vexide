use std::arch::asm;

use gdbstub::arch::Arch;
use gdbstub_arch::arm::{
    ArmBreakpointKind,
    reg::{ArmCoreRegs, id::ArmCoreRegId},
};
use snafu::Snafu;
use zynq7000::devcfg::MmioDevCfg;

use crate::regs::{DebugID, DebugStatusControl, SecureDebugEnable};

pub mod hw;

/// The ARMv7 architecture.
pub enum ArmV7 {}

impl Arch for ArmV7 {
    type Usize = u32;
    type BreakpointKind = ArmBreakpointKind;
    type RegId = ArmCoreRegId;
    type Registers = ArmCoreRegs;

    fn target_description_xml() -> Option<&'static str> {
        Some(include_str!("arch/target.xml"))
    }
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
