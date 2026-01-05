//! Access to system debug registers.

#![allow(non_upper_case_globals)]

use std::arch::asm;

use arbitrary_int::u4;
use bitbybit::{bitenum, bitfield};

/// Returns the (CRn, CRm, opc2) arguments that would be required to access the given debug register
/// via the CPU's CP14 interface.
///
/// See "C6.4.1 Using CP14 to access debug registers" in the ARMv7-A manual.
#[must_use]
pub const fn get_cp14_encoding(register: DebugRegister) -> (u32, u32, u32) {
    let regnum = register.to_raw();

    (
        (regnum >> 7) & 0b111,
        regnum & 0b1111,
        (regnum >> 4) & 0b111,
    )
}

/// Read the given debug register.
#[macro_export]
macro_rules! read_dbgreg {
    ($reg:expr) => {
        unsafe {
            const ENCODING: (u32, u32, u32) = $crate::regs::get_cp14_encoding($reg);
            let value: u32;
            core::arch::asm!(
                "mrc p14, 0, {value}, c{CRn}, c{CRm}, {opc2}",
                value = out(reg) value,
                CRn = const ENCODING.0,
                CRm = const ENCODING.1,
                opc2 = const ENCODING.2,
                options(nostack, preserves_flags),
            );
            value
        }
    };
}

/// Write the given value to the debug register.
#[macro_export]
macro_rules! write_dbgreg {
    ($reg:expr, $value:expr) => {
        {
            const ENCODING: (u32, u32, u32) = $crate::regs::get_cp14_encoding($reg);
            let value: u32 = $value;
            core::arch::asm!(
                "mcr p14, 0, {value}, c{CRn}, c{CRm}, {opc2}",
                value = in(reg) value,
                CRn = const ENCODING.0,
                CRm = const ENCODING.1,
                opc2 = const ENCODING.2,
                options(nostack, preserves_flags),
            );
        }
    };
}

/// A debug register that can be read or written to with [`write_dbgreg`] or [`read_dbgreg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugRegister(u32);

impl DebugRegister {
    pub const fn to_raw(self) -> u32 {
        self.0
    }
}

/// The raw version of [`DebugID`].
pub const DBGDIDR: DebugRegister = DebugRegister(0);

/// The DBGDIDR register.
#[bitfield(u32, debug)]
pub struct DebugID {
    #[bits(28..=31, r)]
    pub wrps: u4,
    #[bits(24..=27, r)]
    pub brps: u4,
    #[bits(16..=19, r)]
    pub version: Option<DebugVersion>,
}

impl DebugID {
    /// Read the current value.
    pub fn read() -> Self {
        Self::new_with_raw_value(read_dbgreg!(DBGDIDR))
    }
}

/// A version of the debug architecture.
#[derive(Debug, PartialEq, Eq)]
#[bitenum(u4, exhaustive = false)]
pub enum DebugVersion {
    V6 = 0b0001,
    V6_1 = 0b0010,
    V7Full = 0b0011,
    V7Minimal = 0b0100,
    V7_1 = 0b0101,
}

/// The raw version of [`DebugStatusControl`] (external view).
pub const DBGDSCRext: DebugRegister = DebugRegister(34);

/// The DBGDSCR register.
#[bitfield(u32, debug)]
pub struct DebugStatusControl {
    #[bit(16, r)]
    pub secure_pl1_invasive_debug_disabled: bool,
    #[bit(15, rw)]
    pub monitor_debug_mode: bool,
    #[bit(14, rw)]
    pub halting_debug_mode: bool,
    #[bits(2..=5, r)]
    pub method_of_entry: u4,
}

impl DebugStatusControl {
    /// Read the external view.
    pub fn read_ext() -> Self {
        Self::new_with_raw_value(read_dbgreg!(DBGDSCRext))
    }

    /// Write to the external view.
    pub unsafe fn write_ext(self) {
        unsafe {
            write_dbgreg!(DBGDSCRext, self.raw_value());
        }
    }
}

/// The SDER register.
#[bitfield(u32, debug)]
pub struct SecureDebugEnable {
    #[bit(0, rw)]
    pub secure_user_invasive_debug: bool,
    #[bit(1, rw)]
    pub secure_user_noninvasive_debug: bool,
}

impl SecureDebugEnable {
    /// Read the current value.
    pub fn read() -> Self {
        let value: u32;
        unsafe {
            asm!(
                "mrc p15, 0, {value}, c1, c1, 1",
                value = out(reg) value,
                options(nostack, preserves_flags)
            );
        }
        Self::new_with_raw_value(value)
    }

    /// Update the current value.
    pub unsafe fn write(self) {
        unsafe {
            asm!(
                "mcr p15, 0, {value}, c1, c1, 1",
                value = in(reg) self.raw_value(),
                options(nostack, preserves_flags)
            );
        }
    }
}
