//! Startup routine and behavior in `vexide`.
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

pub mod banner;
mod code_signature;
mod patcher;

use banner::themes::BannerTheme;
pub use code_signature::{CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

/// Load address of user programs in memory.
const USER_MEMORY_START: u32 = 0x0380_0000;

// All of these symbols are defined in our linkerscript (link/v5.ld) and don't have real types or
// values, but a pointer to them points to the address of their location defined in the linkerscript.
unsafe extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;

    static mut __patcher_ram_start: u8;
    static mut __patcher_ram_end: u8;

    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

// This is the true entrypoint of vexide, containing the first instructions
// of user code executed before anything else. This is written in assembly to
// ensure that it stays the same across compilations (a requirement of the patcher),
//
// This routine loads the stack pointer to the stack region specified in our
// linker script, makes a copy of program memory for the patcher if needed, then
// branches to the Rust entrypoint (_start) created by the #[vexide::main] macro.
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
    ldr r2, =0x07A0000C @ the length of the binary is stored at 0x07A0000C
    ldr r2, [r2] @ memcpy size -> r2

    @ Do the memcpy if patch magic is present (we checked this in our `cmp` instruction).
    bleq __overwriter_aeabi_memcpy

    @ Jump to the Rust entrypoint.
    b _start
"#
);

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
        // Clear the .bss (uninitialized statics) section by filling it with zeroes.
        // This is required, since the compiler assumes it will be zeroed on first access.
        core::slice::from_raw_parts_mut(
            &raw mut __bss_start,
            (&raw mut __bss_end).offset_from(&raw mut __bss_start) as usize,
        )
        .fill(0);

        // Initialize the heap allocator using normal bounds
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
