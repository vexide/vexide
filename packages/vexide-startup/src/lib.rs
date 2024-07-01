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

use bitflags::bitflags;

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

extern "Rust" {
    fn main();
}

/// Sets up the user stack, zeroes the BSS section, and calls the user code.
/// This function is designed to be used as an entrypoint for programs on the VEX V5 Brain.
///
/// # Const Parameters
///
/// - `BANNER`: Enables the vexide startup banner, which prints the vexide logo ASCII art and a startup message.
///
/// # Safety
///
/// This function MUST only be called once and should only be called at the very start of program initialization.
/// Calling this function more than one time will seriously mess up both your stack and your heap.
pub unsafe fn program_entry<const BANNER: bool>() {
    #[cfg(target_arch = "arm")]
    unsafe {
        use core::arch::asm;
        asm!(
            "
            // Load the user stack
            ldr sp, =__stack_top
            "
        );
    }

    // Clear the BSS section (used for storing uninitialized data).
    //
    // VEXos doesn't do this for us on program start, so it's necessary here.
    #[cfg(target_arch = "arm")]
    unsafe {
        use core::ptr::addr_of_mut;
        let mut bss_start = addr_of_mut!(__bss_start);
        while bss_start < addr_of_mut!(__bss_end) {
            core::ptr::write_volatile(bss_start, 0);
            bss_start = bss_start.offset(1);
        }
    }

    unsafe {
        // Initialize the heap allocator
        // This cfg is mostly just to make the language server happy. All of this code is near impossible to run in the WASM sim.
        #[cfg(target_arch = "arm")]
        vexide_core::allocator::vexos::init_heap();
        // Print the banner
        if BANNER {
            vexide_core::io::print!(
                "
\x1B[1;38;5;196m=%%%%%#-  \x1B[38;5;254m-#%%%%-\x1B[1;38;5;196m  :*%%%%%+.
\x1B[38;5;208m  -#%%%%#-  \x1B[38;5;254m:%-\x1B[1;38;5;208m  -*%%%%#
\x1B[38;5;226m    *%%%%#=   -#%%%%%+
\x1B[38;5;226m      *%%%%%+#%%%%%%%#=
\x1B[38;5;34m        *%%%%%%%*-+%%%%%+
\x1B[38;5;27m          +%%%*:   .+###%#
\x1B[38;5;93m           .%:\x1B[0m
vexide startup successful!
Running user code...
"
            );
        }
        // Run vexos background processing at a regular 2ms interval.
        // This is necessary for serial and device reads to work properly.
        vexide_async::task::spawn(async {
            loop {
                vex_sdk::vexTasksRun();

                // In VEXCode programs, this is ran in a tight loop with no delays, since they
                // don't need to worry about running two schedulers on top of each other, but
                // doing this in our case would cause this task to hog all the CPU time, which
                // wouldn't allow futures to be polled in the async runtime.
                vexide_async::time::sleep(::core::time::Duration::from_millis(2)).await;
            }
        })
        .detach();
        // Call the user code
        main();
        // Exit the program
        vexide_core::program::exit();
    }
}
