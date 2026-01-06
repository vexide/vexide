//! Access to system debug registers.

#![allow(non_upper_case_globals)]

use std::arch::asm;

use arbitrary_int::*;
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
/// The raw version of [`DebugStatusControl`] (internal view).
pub const DBGDSCRint: DebugRegister = DebugRegister(1);
/// The raw version of [`DebugStatusControl`] (external view).
pub const DBGDSCRext: DebugRegister = DebugRegister(34);
pub const DBGDRAR: DebugRegister = DebugRegister(128);
pub const DBGDSAR: DebugRegister = DebugRegister(256);

/// The DBGDIDR register.
#[bitfield(u32, debug)]
pub struct DebugID {
    #[bits(28..=31, r)]
    wrps: u4,
    #[bits(24..=27, r)]
    brps: u4,
    #[bits(16..=19, r)]
    version: Option<DebugVersion>,
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

/// The DBGDSCR register.
#[bitfield(u32, debug)]
pub struct DebugStatusControl {
    #[bit(16, r)]
    secure_pl1_invasive_debug_disabled: bool,
    #[bit(15, rw)]
    monitor_debug_mode: bool,
    #[bit(14, rw)]
    halting_debug_mode: bool,
    #[bits(2..=5, r)]
    method_of_entry: Option<DebugEventReason>,
}

#[bitenum(u4, exhaustive = false)]
#[derive(Debug, PartialEq, Eq)]
pub enum DebugEventReason {
    HaltReq = 0b0000,
    Breakpoint = 0b0001,
    AsyncWatchpoint = 0b0010,
    BkptInstr = 0b0011,
    ExtDebugReq = 0b0100,
    VectorCatch = 0b0101,
    OSUnlock = 0b1000,
    SyncWatchpoint = 0b1010,
}

#[derive(Debug, PartialEq, Eq)]
#[bitenum(u2, exhaustive = false)]
pub enum DebugValid {
    Invalid = 0b00,
    Valid = 0b11,
}

#[bitfield(u32, debug)]
pub struct DebugROMAddress {
    #[bits(12..=31, r)]
    romaddr: u20,
    #[bits(0..=1, r)]
    valid: Option<DebugValid>,
}

impl DebugROMAddress {
    pub fn read() -> Self {
        Self::new_with_raw_value(read_dbgreg!(DBGDRAR))
    }

    #[must_use]
    pub const fn value(self) -> usize {
        (self.romaddr().value() << 12) as usize
    }
}

#[bitfield(u32, debug)]
pub struct DebugSelfAddressOffset {
    #[bits(12..=31, r)]
    selfoffset: u20,
    #[bits(0..=1, r)]
    valid: Option<DebugValid>,
}

impl DebugSelfAddressOffset {
    pub fn read() -> Self {
        Self::new_with_raw_value(read_dbgreg!(DBGDSAR))
    }

    #[must_use]
    pub const fn value(self) -> isize {
        (self.selfoffset().value() << 12) as isize
    }
}

/// The SDER register.
#[bitfield(u32, debug)]
pub struct SecureDebugEnable {
    #[bit(0, rw)]
    secure_user_invasive_debug: bool,
    #[bit(1, rw)]
    secure_user_noninvasive_debug: bool,
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

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct DebugLogic {
    // Fields prefixed with an underscore cannot be accessed via MMIO on our target device.
    #[mmio(PureRead)]
    id: DebugID,
    _status_control_int: DebugStatusControl,
    _reserved0: [u32; 3],
    _data_transfer_int: u32,
    /// Available, but deprecated. Instead, read LR_abt to determine the PC of the watchpoint hit.
    _watchpoint_fault_addr: u32,
    vector_catch: u32,
    _reserved1: u32,
    event_catch: u32,
    debug_cache_ctrl: u32,
    debug_mmu_ctrl: u32,
    _reserved2: [u32; 20],
    host_to_target_data_transfer_ext: u32,
    #[mmio(Write)]
    instr_tx: u32,
    status_control_ext: DebugStatusControl,
    target_to_host_data_transfer_ext: u32,
    #[mmio(Write)]
    run_ctrl: u32,
    ext_auxiliary_ctrl: u32,
    _reserved3: [u32; 2],
    #[mmio(PureRead)]
    pc_sample: u32,
    #[mmio(PureRead)]
    context_id_sample: u32,
    _virt_id_sample: u32,
    _reserved4: [u32; 21],
    /// When matching instructions, value & 0b11 must be 0
    breakpoint_value: [u32; 16],
    breakpoint_ctrl: [BreakpointControl; 16],
    /// `value` & 0b11 must be 0.
    watchpoint_value: [u32; 16],
    watchpoint_ctrl: [WatchpointControl; 16],
    // The 2nd half of the debug logic MMIO is not accessed.
}

#[bitfield(u32, debug)]
pub struct BreakpointControl {
    // Set a breakpoint on a range of addresses by excluding some lower bits from the comparison.
    #[bits(24..=28, rw)]
    address_range_mask: u5,
    #[bits(20..=23, rw)]
    breakpoint_type: Option<BreakpointType>,
    #[bits(16..=19, rw)]
    linked_breakpoint_index: u4,
    /// Controls which security states the breakpoint filters for.
    #[bits(14..=15, rw)]
    security_state_ctrl: Option<SecurityFilter>,
    /// A bitfield that controls which addresses inside the 4-byte breakpoint value will cause a
    /// hit.
    ///
    /// Setting any number bit will enable hits on an addresses ending in that number. For instance,
    /// setting bit zero (0bxxx1) will enable hits on addresses ending with 0 (i.e. 0bxx…xx00),
    /// whereas setting bit three (0b1xxx) will enable hits on addresses ending with 3
    /// (i.e. 0bxx…xx11).
    ///
    /// For the purposes of this field, instructions are considered to take up multiple addresses
    /// at once depending on their size. It's not possible to set a breakpoint on half an
    /// instruction, so, for instance, 0b0011 is invalid in ARM and 0b0001 is invalid in both
    /// Thumb and ARM.
    #[bits(5..=8, rw)]
    byte_address_select: u4,
    #[bits(1..=2, rw)]
    privileged_mode_ctrl: PrivilegeModeFilter,
    #[bit(0, rw)]
    enabled: bool,
}

#[bitenum(u4, exhaustive = false)]
#[derive(Debug, PartialEq, Eq)]
pub enum BreakpointType {
    /// A breakpoint that triggers when the PC matches an address.
    UnlinkedInstrAddressMatch = 0b0000,
    /// The address part of a linked breakpoint that triggers on both an address match and a
    /// context-id match.
    LinkedInstrAddressMatch = 0b0001,
    /// A breakpoint that triggers on context-id (CONTEXTIDR) match.
    UnlinkedContextIDMatch = 0b0010,
    /// The context-id part of a linked breakpoint that triggers on both an address (mis)match and a
    /// context-id match.
    LinkedContextIDMatch = 0b0011,
    /// A breakpoint that triggers when the PC doesn't match an address.
    UnlinkedInstrAddressMismatch = 0b0100,
    /// The address part of a linked breakpoint that triggers on both an address mismatch and a
    /// context-id match.
    LinkedInstrAddressMismatch = 0b0101,
}

#[bitenum(u2, exhaustive = false)]
#[derive(Debug, PartialEq, Eq)]
pub enum SecurityFilter {
    All = 0b00,
    NotSecureOnly = 0b01,
    SecureOnly = 0b10,
}

#[bitenum(u2, exhaustive = true)]
#[derive(Debug, PartialEq, Eq)]
pub enum PrivilegeModeFilter {
    UserSystemSupervisorOnly = 0b00,
    Level1Only = 0b01,
    UserOnly = 0b10,
    All = 0b11,
}

#[bitfield(u32, debug)]
pub struct WatchpointControl {
    #[bit(20, rw)]
    watchpoint_type: WatchpointType,
    #[bits(16..=19, rw)]
    linked_breakpoint_index: u4,
    /// Controls which security states the watchpoint filters for.
    #[bits(14..=15, rw)]
    security_state_ctrl: Option<SecurityFilter>,
    /// See [`BreakpointControl::byte_address_select`].
    #[bits(5..=8, rw)]
    byte_address_select: u4,
    #[bits(3..=4, rw)]
    load_store_ctrl: Option<LoadStoreFilter>,
    #[bits(1..=2, rw)]
    privileged_access_ctrl: Option<PrivilegedAccessFilter>,
    #[bit(0, rw)]
    enabled: bool,
}

#[bitenum(u1, exhaustive = true)]
#[derive(Debug, PartialEq, Eq)]
pub enum WatchpointType {
    /// A watchpoint that triggers when an address is accessed as data.
    UnlinkedDataAddressMatch = 0,
    /// The address part of a watchpoint that triggers on both an address match and a context-id
    /// match. It is linked to a context-id breakpoint.
    LinkedDataAddressMatch = 1,
}

#[bitenum(u2, exhaustive = false)]
#[derive(Debug, PartialEq, Eq)]
pub enum LoadStoreFilter {
    LoadSwapOnly = 0b01,
    StoreSwapOnly = 0b10,
    All = 0b11,
}

#[bitenum(u2, exhaustive = false)]
#[derive(Debug, PartialEq, Eq)]
pub enum PrivilegedAccessFilter {
    Level1Only = 0b01,
    UserOnly = 0b10,
    All = 0b11,
}
