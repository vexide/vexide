//! Minimal example of setting up program booting without the `#[vexide::main]` attribute macro.

#![no_main]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;

use vexide_core::println;
use vexide_startup::{
    banner::themes::THEME_DEFAULT, CodeSignature, ProgramFlags, ProgramOwner, ProgramType,
};

// SAFETY: This function is unique and is being used to start the vexide runtime.
// It will be called by the _boot assembly routine after the stack has been setup.
#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {
    // Setup the heap, zero bss, apply patches, etc...
    unsafe {
        vexide_startup::startup::<true>(THEME_DEFAULT);
    }

    let test_box = Box::new(100); // On the heap to demonstrate allocation.
    unsafe {
        // Draw something to the screen to test if the program is running.
        vex_sdk::vexDisplayRectFill(0, 0, *test_box, 200);
    }

    // Print some data to the terminal.
    println!("Hello, world!");

    // Exit the program once we're done.
    vexide_core::program::exit();
}

// SAFETY: The code signature needs to be in this section so it may be found by VEXos.
#[unsafe(link_section = ".code_signature")]
#[used] // This is needed to prevent the linker from removing this object in release builds
static CODE_SIG: CodeSignature = CodeSignature::new(
    ProgramType::User,
    ProgramOwner::Partner,
    ProgramFlags::empty(),
);

#[panic_handler]
const fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
