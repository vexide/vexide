//! This crate provides a minimal startup routine for user code on the VEX V5 Brain.
//!
//! - User code begins at an assembly routine called `_boot`, which sets up the stack
//!   section before jumping to a user-provided `_start` symbol, which should be your
//!   rust entrypoint.
//!
//! - From there, the Rust entrypoint may call the [`startup`] function to finish the
//!   startup process by clearing the `.bss` section (intended for uninitialized data)
//!   and initializing vexide's heap allocator.
//!
//! This crate is NOT a crt0 implementation. No global constructors are called.

#![no_std]
#![allow(clippy::needless_doctest_main)]

use banner::themes::BannerTheme;
use bitflags::bitflags;

pub mod banner;
mod patcher;

/// Load address of user programs.
const USER_MEMORY_START: u32 = 0x0380_0000;
/// Load address of patch files.
const PATCH_MEMORY_START: u32 = 0x07A0_0000;

/// Identifies the type of binary to VEXos.
#[repr(u32)]
#[non_exhaustive]
pub enum ProgramType {
    /// User program binary.
    User = 0,
}

/// The owner (originator) of the user program
#[repr(u32)]
pub enum ProgramOwner {
    /// Program is a system binary.
    System = 0,

    /// Program originated from VEX.
    Vex = 1,

    /// Program originated from a partner developer.
    Partner = 2,
}

bitflags! {
    /// Program Flags
    ///
    /// These bitflags are part of the [`CodeSignature`] that determine some small
    /// aspects of program behavior when running under VEXos. This struct contains
    /// the flags with publicly documented behavior.
    #[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
    pub struct ProgramFlags: u32 {
        /// Inverts the background color to pure white.
        const INVERT_DEFAULT_GRAPHICS = 1 << 0;

        /// VEXos scheduler simple tasks will be killed when the program requests exit.
        const KILL_TASKS_ON_EXIT = 1 << 1;

        /// If VEXos is using the Light theme, inverts the background color to pure white.
        const THEMED_DEFAULT_GRAPHICS = 1 << 2;
    }
}

/// Program Code Signature
///
/// The first 16 bytes of a VEX user code binary contain a user code signature,
/// containing some basic metadata and startup flags about the program. This
/// signature must be at the start of the binary for booting to occur.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CodeSignature(vex_sdk::vcodesig, [u32; 4]);

impl CodeSignature {
    /// Creates a new signature given a program type, owner, and flags.
    #[must_use]
    pub const fn new(program_type: ProgramType, owner: ProgramOwner, flags: ProgramFlags) -> Self {
        Self(
            vex_sdk::vcodesig {
                magic: vex_sdk::V5_SIG_MAGIC,
                r#type: program_type as _,
                owner: owner as _,
                options: flags.bits(),
            },
            [0; 4],
        )
    }
}

unsafe extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;

    // These symbols don't have real types, so this is a little bit of a hack.
    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

// This is the true entrypoint of vexide, containing the first two
// instructions of user code executed before anything else.
//
// This function loads the stack pointer to the stack region specified
// in our linkerscript, then immediately branches to the Rust entrypoint
// created by our macro.
core::arch::global_asm!(
    r#"
.section .boot, "ax"
.global _boot

_boot:
    @ Load the user program stack.
    @
    @ This technically isn't required, as VEXos already sets up a stack for CPU1,
    @ but that stack is relatively small and we have more than enough memory
    @ available to us for this.
    ldr sp, =__stack_top

    @ Before any Rust code runs, we need to memcpy the currently running binary to
    @ 0x07C00000 if a patch file is loaded into memory. See the documentation in
    @ `patcher/mod.rs` for why we want to do this.

    @ Check for patch magic at 0x07A00000.
    mov r0, #0x07A00000
    ldr r0, [r0]
    ldr r1, =0xB1DF
    cmp r0, r1

    @ Prepare to memcpy binary to 0x07C00000
    mov r0, #0x07C00000 @ memcpy dest -> r0
    mov r1, #0x03800000 @ memcpy src -> r1
    ldr r2, =0x07A0000C @ memcpy size -> r2
    ldr r2, [r2]

    @ Do the memcpy if patch magic is present (we checked this in our `cmp` instruction).
    bleq __overwriter_aeabi_memcpy

    @ Jump to the Rust entrypoint.
    b _start
"#
);

/// Zeroes the `.bss` section
///
/// # Arguments
///
/// - `sbss`. Pointer to the start of the `.bss` section.
/// - `ebss`. Pointer to the open/non-inclusive end of the `.bss` section.
///   (The value behind this pointer will not be modified)
/// - Use `T` to indicate the alignment of the `.bss` section.
///
/// # Safety
///
/// - Must be called exactly once
/// - `mem::size_of::<T>()` must be non-zero
/// - `ebss >= sbss`
/// - `sbss` and `ebss` must be `T` aligned.
#[inline]
#[allow(clippy::similar_names)]
#[cfg(target_vendor = "vex")]
unsafe fn zero_bss<T>(mut sbss: *mut T, ebss: *mut T)
where
    T: Copy,
{
    while sbss < ebss {
        // NOTE: volatile to prevent this from being transformed into `memclr`
        unsafe {
            core::ptr::write_volatile(sbss, core::mem::zeroed());
            sbss = sbss.offset(1);
        }
    }
}

/// Startup Routine
///
/// - Sets up the heap allocator if necessary.
/// - Zeroes the `.bss` section if necessary.
/// - Prints the startup banner with a specified theme, if enabled.
///
/// # Safety
///
/// Must be called once at the start of program execution after the stack has been setup.
#[inline]
#[allow(clippy::too_many_lines)]
pub unsafe fn startup<const BANNER: bool>(theme: BannerTheme) {
    #[cfg(target_vendor = "vex")]
    unsafe {
        // Fill the `.bss` section of our program's memory with zeroes to ensure that
        // uninitialized data is allocated properly.
        zero_bss(
            core::ptr::addr_of_mut!(__bss_start),
            core::ptr::addr_of_mut!(__bss_end),
        );
    }

    // If this link address is 0x03800000, this implies we were uploaded using
    // differential uploads by cargo-v5 and may have a patch to apply.
    if unsafe { vex_sdk::vexSystemLinkAddrGet() } == USER_MEMORY_START {
        unsafe {
            patcher::patch(PATCH_MEMORY_START as *mut u32);
        }
    }

    // Initialize the heap allocator using normal bounds
    unsafe {
        vexide_core::allocator::claim(&raw mut __heap_start, &raw mut __heap_end);
    }

    // Print the banner
    if BANNER {
        banner::print(theme);
    }
}
