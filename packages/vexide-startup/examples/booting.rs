//! Minimal example of setting up program booting without the `#[vexide::main]` attribute macro.

#![no_main]
#![no_std]

extern crate alloc;
use alloc::boxed::Box;

use vexide_core::println;

#[no_mangle]
extern "Rust" fn main() {
    unsafe {
        // Write something to the screen to test if the program is running
        let test_box = Box::new(100);
        vex_sdk::vexDisplayRectFill(0, 0, *test_box, 200);
        println!("Hello, world!");
    }
}

#[no_mangle]
#[link_section = ".boot"]
unsafe extern "C" fn _entry() {
    unsafe { vexide_startup::program_entry::<true>() }
}

#[link_section = ".cold_magic"]
#[used] // This is needed to prevent the linker from removing this object in release builds
static COLD_HEADER: vexide_startup::ColdHeader = vexide_startup::ColdHeader::new(2, 0, 0);

#[panic_handler]
const fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
