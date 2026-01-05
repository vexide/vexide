use std::arch::asm;

use gdbstub::arch::Arch;
use gdbstub_arch::arm::{
    ArmBreakpointKind,
    reg::{ArmCoreRegs, id::ArmCoreRegId},
};
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

pub fn configure_debug(devcfg: &mut MmioDevCfg<'_>) {
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

    let ctrl = devcfg.read_control();
    let lock = devcfg.read_lock();

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

    println!("Devcfg Control: {ctrl:#?}");
    println!("Devcfg Lock: {lock:#?}");

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
}
