//! This crate provides a working entrypoint for the VEX V5 Brain.
//! In order for your code to be started you must provide a main function with the correct signature. eg.
//! ```rust
//! #[no_mangle]
//! extern "C" fn main() { ... }
//! entry!();
//! ```

#![no_std]
#![feature(asm_experimental_arch)]

use core::{arch::asm, ptr::addr_of_mut};

pub use vex_startup_macro::main;

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
    /// A bitfield of program options that change the behavior of some jumptable functions.
    pub options: u32,
}

extern "C" {
    fn main();
}

/// Sets up the user stack, zeroes the BSS section, and calls the user code.
/// This function is designed to be used as an entrypoint for programs on the VEX V5 Brain.
pub unsafe fn program_entry() {
    unsafe {
        asm!(
            "
            // Set the stack pointer for program setup
            ldr sp, =__kernel_stack_start
            "
        );
    }

    // Clear the BSS section
    let mut bss_start = unsafe { addr_of_mut!(__bss_start) };
    let bss_end = unsafe { addr_of_mut!(__bss_end) };
    while bss_start < bss_end {
        unsafe {
            core::ptr::write_volatile(bss_start, 0);
            bss_start = bss_start.offset(1);
        }
    }

    // vexPrivateApiDisable
    // (unsafe { *(0x37fc020 as *const extern "C" fn(u32)) })(COLD_HEADER.options);

    unsafe {
        asm!(
            "
            // Load the user stack
            ldr sp, =__user_stack_start
            "
        );
        // Call the user code
        main()
    }

    loop {}
}
