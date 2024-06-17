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

use vexide_core::print;

extern "C" {
    // These symbols don't have real types so this is a little bit of a hack
    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

/// The cold header is a structure that is placed at the beginning of cold memory and tells VEXos details about the program.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColdHeader {
    /// The magic number for the cold header. This should always be "XVX5".
    pub magic: [u8; 4],
    /// The program type. PROS sets this to 0.
    pub program_type: u32,
    /// The owner of the program. PROS sets this to 2.
    pub owner: u32,
    /// Padding before the options.
    pub padding: [u32; 2],
    /// A bitfield of program options that change the behavior of some jumptable functions.
    pub options: u32,
}
impl ColdHeader {
    /// Creates a new cold header with the correct magic number and options.
    pub const fn new(program_type: u32, owner: u32, options: u32) -> Self {
        Self {
            magic: *b"XVX5",
            program_type,
            owner,
            padding: [0; 2],
            options,
        }
    }
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
            ldr sp, =__stack_start
            "
        );
    }

    // Clear the BSS section
    #[cfg(target_arch = "arm")]
    unsafe {
        use core::ptr::addr_of_mut;
        let mut bss_start = addr_of_mut!(__bss_start);
        while bss_start < addr_of_mut!(__bss_end) {
            core::ptr::write_volatile(bss_start, 0);
            bss_start = bss_start.offset(1);
        }
    }
    // vexPrivateApiDisable
    // (unsafe { *(0x37fc020 as *const extern "C" fn(u32)) })(COLD_HEADER.options);

    unsafe {
        // Initialize the heap allocator
        // This cfg is mostly just to make the language server happy. All of this code is near impossible to run in the WASM sim.
        #[cfg(target_arch = "arm")]
        vexide_core::allocator::vexos::init_heap();
        // Print the banner
        if BANNER {
            print!(
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
        // This is necessary for serial and devices to work properly.
        vexide_async::task::spawn(async {
            loop {
                vex_sdk::vexTasksRun();
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
