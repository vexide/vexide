#![no_main]
#![no_std]

extern crate alloc;
use alloc::boxed::Box;
use vexide_devices::peripherals::Peripherals;

#[vexide_startup::main]
async fn main(peripherals: Peripherals) {
    unsafe {
        // Write something to the screen to test if the program is running
        let test_box = Box::new(100);
        vex_sdk::vexDisplayRectFill(0, 0, *test_box, 200);

        loop {}
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
