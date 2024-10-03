//! Minimal example of setting up program booting without the `#[vexide::main]` attribute macro.

#![no_main]
#![no_std]

extern crate alloc;
use alloc::boxed::Box;

use vex_sdk::vexTasksRun;
use vexide_core::println;
use vexide_startup::{
    banner::themes::THEME_DEFAULT, CodeSignature, ProgramFlags, ProgramOwner, ProgramType,
};

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    unsafe {
        vexide_startup::startup::<true>(THEME_DEFAULT);

        // Write something to the screen to test if the program is running
        let test_box = Box::new(100);
        vex_sdk::vexDisplayRectFill(0, 0, *test_box, 200);
        println!("Hello, world!");
        vexTasksRun(); // Flush serial
    }

    // Exit once we're done.
    vexide_core::program::exit();
}

#[link_section = ".code_signature"]
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
