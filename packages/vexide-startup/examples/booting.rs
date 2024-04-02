#![no_main]
#![no_std]

use vexide_devices::peripherals::Peripherals;

#[vexide_startup::main]
async fn main(peripherals: Peripherals) {
    unsafe {
        // Write something to the screen to test if the program is running
        vex_sdk::vexDisplayRectFill(0, 0, 200, 200);
        loop {}
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
