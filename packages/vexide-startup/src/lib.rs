//! This crate provides a working entrypoint for the VEX V5 Brain.
//! In order for your code to be started you must provide a main function with the correct signature. eg.
//! ```rust
//! #[no_mangle]
//! extern "C" fn main() { ... }
//! entry!();
//! ```

#![no_std]
#![feature(asm_experimental_arch)]
#![allow(clippy::needless_doctest_main)]

use core::{arch::asm, hint, ptr::addr_of_mut};

pub use vexide_startup_macro::main;

extern "C" {
    // These symbols don't have real types so this is a little bit of a hack
    static mut __bss_start: u32;
    static mut __bss_end: u32;
}

#[repr(C, packed)]
/// The cold header is a structure that is placed at the beginning of cold memory and tells VexOS details abuot the program.
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
/// # Safety
///
/// This function MUST only be called once and should only be called at the very start of program initialization.
/// Calling this function more than one time will seriously mess up both your stack and your heap.
pub unsafe fn program_entry() {
    unsafe {
        asm!(
            "
            // Load the user stack
            ldr sp, =__user_stack_start
            "
        );
    }

    // Clear the BSS section
    unsafe {
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
        // Call the user code
        main();
        // Exit the program
        vex_sdk::vexSystemExitRequest();
    }

    // Technically unreachable, but the compiler doesn't know that
    loop {
        hint::spin_loop();
    }
}
