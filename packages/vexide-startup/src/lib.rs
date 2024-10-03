//! This crate provides a working entrypoint for the VEX V5 Brain.
//!
//! # Usage
//!
//! Your entrypoint function should be an async function that takes a single argument of type [`Peripherals`](vexide_devices::peripherals::Peripherals).
//! It can return any type implementing [`Termination`](vexide_core::program::Termination).
//! ```rust
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) { ... }
//! ```

#![no_std]
#![feature(asm_experimental_arch)]
#![allow(clippy::needless_doctest_main)]

use banner::themes::BannerTheme;
use bitflags::bitflags;

pub mod banner;

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
        /// Default graphics colors will be inverted.
        const INVERT_DEFAULT_GRAPHICS = 1 << 0;

        /// VEXos scheduler simple tasks will be killed when the program requests exit.
        const KILL_TASKS_ON_EXIT = 1 << 1;

        /// Default graphics colors will invert based on the selected system theme.
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

extern "C" {
    // These symbols don't have real types so this is a little bit of a hack
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
    ldr sp, =__stack_top @ Load the user stack.
    b _start             @ Jump to the Rust entrypoint.
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
unsafe fn zero_bss<T>(mut sbss: *mut T, ebss: *mut T)
where
    T: Copy,
{
    while sbss < ebss {
        // NOTE(volatile) to prevent this from being transformed into `memclr`
        unsafe {
            core::ptr::write_volatile(sbss, core::mem::zeroed());
            sbss = sbss.offset(1);
        }
    }
}

/// Startup Routine
///
/// - Sets up the heap allocator if necessary.
/// - Zeroes the `.bss`` section if necessary.
/// - Prints the startup banner with a specified theme, if enabled.
///
/// # Safety
///
/// Must be called once at the start of program execution after the stack has been setup.
#[inline]
pub unsafe fn startup<const BANNER: bool>(theme: BannerTheme) {
    #[cfg(target_arch = "arm")]
    unsafe {
        // Initialize the heap allocator
        // This cfg is mostly just to make the language server happy. All of this code is near impossible to run in the WASM sim.
        vexide_core::allocator::vexos::init_heap();

        // Fill the `.bss` section of our program's memory with zeroes to ensure that uninitialized data is allocated properly.
        zero_bss(
            core::ptr::addr_of_mut!(__bss_start),
            core::ptr::addr_of_mut!(__bss_end),
        );
    }

    // Print the banner
    if BANNER {
        banner::print(theme);
    }
}
