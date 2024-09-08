//! Bindings to the low-level `unw_*` LLVM libunwind APIs which are an interface defined by the HP libunwind project.

#![allow(non_camel_case_types, missing_docs)]

use core::ffi::{c_char, c_int, c_void};

pub type uw_error_t = c_int;

/// Error codes.
pub mod error {
    use core::ffi::c_int;
    /// No error.
    pub const UNW_ESUCCESS: c_int = 0;
    /// Unspecified (general) error.
    pub const UNW_EUNSPEC: c_int = -6540;
    /// Out of memory.
    pub const UNW_ENOMEM: c_int = -6541;
    /// Bad register number.
    pub const UNW_EBADREG: c_int = -6542;
    /// Attempt to write read-only register.
    pub const UNW_EREADONLYREG: c_int = -6543;
    /// Stop unwinding.
    pub const UNW_ESTOPUNWIND: c_int = -6544;
    /// Invalid IP.
    pub const UNW_EINVALIDIP: c_int = -6545;
    /// Bad frame.
    pub const UNW_EBADFRAME: c_int = -6546;
    /// Unsupported operation or bad value.
    pub const UNW_EINVAL: c_int = -6547;
    /// Unwind info has unsupported version.
    pub const UNW_EBADVERSION: c_int = -6548;
    /// No unwind info found.
    pub const UNW_ENOINFO: c_int = -6549;
}

/// Architecture-specific context size
#[cfg(target_arch = "arm")]
pub const CONTEXT_SIZE: usize = 42;
/// Architecture-specific cursor size
#[cfg(target_arch = "arm")]
pub const CURSOR_SIZE: usize = 49;

/// The step was successful.
pub const UNW_STEP_SUCCESS: c_int = 1;
/// There are no more stack frames.
pub const UNW_STEP_END: c_int = 0;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct unw_context_t {
    _data: [u64; CONTEXT_SIZE],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct unw_cursor_t {
    _data: [u64; CURSOR_SIZE],
}

pub type unw_addr_space_t = *mut c_void;

pub type unw_regnum_t = c_int;
pub type unw_word_t = usize;
pub type unw_fpreg_t = u64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct unw_proc_info_t {
    /// Start address of function
    pub start_ip: unw_word_t,
    /// Address after end of function
    pub end_ip: unw_word_t,
    /// Address of language specific data area or zero if not used
    pub lsda: unw_word_t,
    /// Personality routine, or zero if not used
    pub handler: unw_word_t,
    /// Not used
    pub gp: unw_word_t,
    /// Not used
    pub flags: unw_word_t,
    /// Compact unwind encoding, or zero if none
    pub format: u32,
    /// Size of DWARF unwind info, or zero if none
    pub unwind_info_size: u32,
    /// Address of DWARF unwind info, or zero
    pub unwind_info: unw_word_t,
    /// mach_header of mach-o image containing func
    pub extra: unw_word_t,
}

#[link(name = "unwind")]
extern "C" {
    pub fn unw_getcontext(ctx: *mut unw_context_t) -> c_int;

    pub fn unw_init_local(cur: *mut unw_cursor_t, ctx: *mut unw_context_t) -> c_int;

    pub fn unw_step(cur: *mut unw_cursor_t) -> c_int;

    pub fn unw_get_reg(cur: *mut unw_cursor_t, reg: unw_regnum_t, val: *mut unw_word_t) -> c_int;

    pub fn unw_get_fpreg(cur: *mut unw_cursor_t, reg: unw_regnum_t, val: *mut unw_fpreg_t)
        -> c_int;

    pub fn unw_set_reg(cur: *mut unw_cursor_t, reg: unw_regnum_t, val: unw_word_t) -> c_int;

    pub fn unw_set_fpreg(cur: *mut unw_cursor_t, reg: unw_regnum_t, val: unw_fpreg_t) -> c_int;

    pub fn unw_resume(cur: *mut unw_cursor_t) -> c_int;

    #[cfg(target_arch = "arm")]
    // Save VFP registers in FSTMX format (instead of FSTMD).
    pub fn unw_save_vfp_as_X(cur: *mut unw_cursor_t);

    pub fn unw_regname(cur: *mut unw_cursor_t, reg: unw_regnum_t) -> *const c_char;

    pub fn unw_get_proc_info(cur: *mut unw_cursor_t, info: *mut unw_proc_info_t) -> c_int;

    pub fn unw_is_fpreg(cur: *mut unw_cursor_t, reg: unw_regnum_t) -> c_int;

    pub fn unw_is_signal_frame(cur: *mut unw_cursor_t) -> c_int;

    pub fn unw_get_proc_name(
        cur: *mut unw_cursor_t,
        buf: *mut c_char,
        len: usize,
        offp: *mut unw_word_t,
    ) -> c_int;

    pub static mut unw_local_addr_space: unw_addr_space_t;
}

/// Instruction pointer
pub const UNW_REG_IP: unw_regnum_t = -1;
/// Stack pointer
pub const UNW_REG_SP: unw_regnum_t = -2;

// 32-bit ARM registers. Numbers match DWARF for ARM spec #3.1 Table 1.
// Naming scheme uses recommendations given in Note 4 for VFP-v2 and VFP-v3.
// In this scheme, even though the 64-bit floating point registers D0-D31
// overlap physically with the 32-bit floating pointer registers S0-S31,
// they are given a non-overlapping range of register numbers.
//
// Commented out ranges are not preserved during unwinding.
pub const UNW_ARM_R0: unw_regnum_t = 0;
pub const UNW_ARM_R1: unw_regnum_t = 1;
pub const UNW_ARM_R2: unw_regnum_t = 2;
pub const UNW_ARM_R3: unw_regnum_t = 3;
pub const UNW_ARM_R4: unw_regnum_t = 4;
pub const UNW_ARM_R5: unw_regnum_t = 5;
pub const UNW_ARM_R6: unw_regnum_t = 6;
pub const UNW_ARM_R7: unw_regnum_t = 7;
pub const UNW_ARM_R8: unw_regnum_t = 8;
pub const UNW_ARM_R9: unw_regnum_t = 9;
pub const UNW_ARM_R10: unw_regnum_t = 10;
pub const UNW_ARM_R11: unw_regnum_t = 11;
pub const UNW_ARM_R12: unw_regnum_t = 12;
pub const UNW_ARM_SP: unw_regnum_t = 13; // Logical alias for UNW_REG_SP
pub const UNW_ARM_R13: unw_regnum_t = 13;
pub const UNW_ARM_LR: unw_regnum_t = 14;
pub const UNW_ARM_R14: unw_regnum_t = 14;
pub const UNW_ARM_IP: unw_regnum_t = 15; // Logical alias for UNW_REG_IP
pub const UNW_ARM_R15: unw_regnum_t = 15;
// 16-63 -- OBSOLETE. Used in VFP1 to represent both S0-S31 and D0-D31.
pub const UNW_ARM_S0: unw_regnum_t = 64;
pub const UNW_ARM_S1: unw_regnum_t = 65;
pub const UNW_ARM_S2: unw_regnum_t = 66;
pub const UNW_ARM_S3: unw_regnum_t = 67;
pub const UNW_ARM_S4: unw_regnum_t = 68;
pub const UNW_ARM_S5: unw_regnum_t = 69;
pub const UNW_ARM_S6: unw_regnum_t = 70;
pub const UNW_ARM_S7: unw_regnum_t = 71;
pub const UNW_ARM_S8: unw_regnum_t = 72;
pub const UNW_ARM_S9: unw_regnum_t = 73;
pub const UNW_ARM_S10: unw_regnum_t = 74;
pub const UNW_ARM_S11: unw_regnum_t = 75;
pub const UNW_ARM_S12: unw_regnum_t = 76;
pub const UNW_ARM_S13: unw_regnum_t = 77;
pub const UNW_ARM_S14: unw_regnum_t = 78;
pub const UNW_ARM_S15: unw_regnum_t = 79;
pub const UNW_ARM_S16: unw_regnum_t = 80;
pub const UNW_ARM_S17: unw_regnum_t = 81;
pub const UNW_ARM_S18: unw_regnum_t = 82;
pub const UNW_ARM_S19: unw_regnum_t = 83;
pub const UNW_ARM_S20: unw_regnum_t = 84;
pub const UNW_ARM_S21: unw_regnum_t = 85;
pub const UNW_ARM_S22: unw_regnum_t = 86;
pub const UNW_ARM_S23: unw_regnum_t = 87;
pub const UNW_ARM_S24: unw_regnum_t = 88;
pub const UNW_ARM_S25: unw_regnum_t = 89;
pub const UNW_ARM_S26: unw_regnum_t = 90;
pub const UNW_ARM_S27: unw_regnum_t = 91;
pub const UNW_ARM_S28: unw_regnum_t = 92;
pub const UNW_ARM_S29: unw_regnum_t = 93;
pub const UNW_ARM_S30: unw_regnum_t = 94;
pub const UNW_ARM_S31: unw_regnum_t = 95;
//  96-103 -- OBSOLETE. F0-F7. Used by the FPA system. Superseded by VFP.
// 104-111 -- wCGR0-wCGR7, ACC0-ACC7 (Intel wireless MMX)
pub const UNW_ARM_WR0: unw_regnum_t = 112;
pub const UNW_ARM_WR1: unw_regnum_t = 113;
pub const UNW_ARM_WR2: unw_regnum_t = 114;
pub const UNW_ARM_WR3: unw_regnum_t = 115;
pub const UNW_ARM_WR4: unw_regnum_t = 116;
pub const UNW_ARM_WR5: unw_regnum_t = 117;
pub const UNW_ARM_WR6: unw_regnum_t = 118;
pub const UNW_ARM_WR7: unw_regnum_t = 119;
pub const UNW_ARM_WR8: unw_regnum_t = 120;
pub const UNW_ARM_WR9: unw_regnum_t = 121;
pub const UNW_ARM_WR10: unw_regnum_t = 122;
pub const UNW_ARM_WR11: unw_regnum_t = 123;
pub const UNW_ARM_WR12: unw_regnum_t = 124;
pub const UNW_ARM_WR13: unw_regnum_t = 125;
pub const UNW_ARM_WR14: unw_regnum_t = 126;
pub const UNW_ARM_WR15: unw_regnum_t = 127;
// 128-133 -- SPSR, SPSR_{FIQ|IRQ|ABT|UND|SVC}
// 134-143 -- Reserved
// 144-150 -- R8_USR-R14_USR
// 151-157 -- R8_FIQ-R14_FIQ
// 158-159 -- R13_IRQ-R14_IRQ
// 160-161 -- R13_ABT-R14_ABT
// 162-163 -- R13_UND-R14_UND
// 164-165 -- R13_SVC-R14_SVC
// 166-191 -- Reserved
pub const UNW_ARM_WC0: unw_regnum_t = 192;
pub const UNW_ARM_WC1: unw_regnum_t = 193;
pub const UNW_ARM_WC2: unw_regnum_t = 194;
pub const UNW_ARM_WC3: unw_regnum_t = 195;
// 196-199 -- wC4-wC7 (Intel wireless MMX control)
// 200-255 -- Reserved
pub const UNW_ARM_D0: unw_regnum_t = 256;
pub const UNW_ARM_D1: unw_regnum_t = 257;
pub const UNW_ARM_D2: unw_regnum_t = 258;
pub const UNW_ARM_D3: unw_regnum_t = 259;
pub const UNW_ARM_D4: unw_regnum_t = 260;
pub const UNW_ARM_D5: unw_regnum_t = 261;
pub const UNW_ARM_D6: unw_regnum_t = 262;
pub const UNW_ARM_D7: unw_regnum_t = 263;
pub const UNW_ARM_D8: unw_regnum_t = 264;
pub const UNW_ARM_D9: unw_regnum_t = 265;
pub const UNW_ARM_D10: unw_regnum_t = 266;
pub const UNW_ARM_D11: unw_regnum_t = 267;
pub const UNW_ARM_D12: unw_regnum_t = 268;
pub const UNW_ARM_D13: unw_regnum_t = 269;
pub const UNW_ARM_D14: unw_regnum_t = 270;
pub const UNW_ARM_D15: unw_regnum_t = 271;
pub const UNW_ARM_D16: unw_regnum_t = 272;
pub const UNW_ARM_D17: unw_regnum_t = 273;
pub const UNW_ARM_D18: unw_regnum_t = 274;
pub const UNW_ARM_D19: unw_regnum_t = 275;
pub const UNW_ARM_D20: unw_regnum_t = 276;
pub const UNW_ARM_D21: unw_regnum_t = 277;
pub const UNW_ARM_D22: unw_regnum_t = 278;
pub const UNW_ARM_D23: unw_regnum_t = 279;
pub const UNW_ARM_D24: unw_regnum_t = 280;
pub const UNW_ARM_D25: unw_regnum_t = 281;
pub const UNW_ARM_D26: unw_regnum_t = 282;
pub const UNW_ARM_D27: unw_regnum_t = 283;
pub const UNW_ARM_D28: unw_regnum_t = 284;
pub const UNW_ARM_D29: unw_regnum_t = 285;
pub const UNW_ARM_D30: unw_regnum_t = 286;
pub const UNW_ARM_D31: unw_regnum_t = 287;
// 288-319 -- Reserved for VFP/Neon
// 320-8191 -- Reserved
// 8192-16383 -- Unspecified vendor co-processor register.
