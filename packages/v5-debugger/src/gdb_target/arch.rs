use gdbstub::arch::Arch;
use gdbstub_arch::arm::{
    ArmBreakpointKind,
    reg::{ArmCoreRegs, id::ArmCoreRegId},
};

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
