//! Startup routine and behavior in `vexide`.
//!
//! - User code begins at an assembly routine called `_boot`, which sets up the stack
//!   section before jumping to a user-provided `_start` symbol, which should be your
//!   rust entrypoint. This routine can be found in `boot.S`.
//!
//! - From there, the Rust entrypoint may call the [`startup`] function to finish the
//!   startup process by clearing the `.bss` section (intended for uninitialized data)
//!   and initializing vexide's heap allocator.
//!
//! This crate is NOT a crt0 implementation. No global constructors are called.

#![no_std]
#![allow(clippy::needless_doctest_main)]

pub mod banner;
mod code_signature;
mod patcher;

use banner::themes::BannerTheme;
pub use code_signature::{CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

/// Load address of user programs in memory.
const USER_MEMORY_START: u32 = 0x0380_0000;

// Linkerscript Symbols
//
// All of these external symbols are defined in our linkerscript (link/v5.ld) and don't have real types
// or values, but a pointer to them points to the address of their location defined in the linkerscript.
unsafe extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;

    static mut __patcher_ram_start: u8;
    static mut __patcher_ram_end: u8;

    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

// Include the first-stage assembly entrypoint. This routine contains the first
// instructions executed by the user processor when the program runs.
core::arch::global_asm!(include_str!("./boot.S"));

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
pub unsafe fn startup<const BANNER: bool>(theme: BannerTheme) {
    #[cfg(target_vendor = "vex")]
    unsafe {
        // Clear the .bss (uninitialized statics) section by filling it with zeroes.
        // This is required, since the compiler assumes it will be zeroed on first access.
        core::ptr::write_bytes(
            &raw mut __bss_start,
            0,
            (&raw mut __bss_end).offset_from(&raw mut __bss_start) as usize,
        );

        // Initialize the heap allocator in our heap region defined in the linkerscript
        #[cfg(feature = "allocator")]
        vexide_core::allocator::claim(&raw mut __heap_start, &raw mut __heap_end);

        // If this link address is 0x03800000, this implies we were uploaded using
        // differential uploads by cargo-v5 and may have a patch to apply.
        if vex_sdk::vexSystemLinkAddrGet() == USER_MEMORY_START {
            patcher::patch();
        }

        // Reclaim 6mb memory region occupied by patches and program copies as heap space.
        #[cfg(feature = "allocator")]
        vexide_core::allocator::claim(&raw mut __patcher_ram_start, &raw mut __patcher_ram_end);
    }

    // Print the banner
    if BANNER {
        banner::print(theme);
    }
}
