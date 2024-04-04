#![no_main]
#![no_std]

use vex_sdk::vexTaskAdd;
use vexide_core::io::println;
use vexide_devices::peripherals::Peripherals;

fn background_processing() {
    loop {
        vexTasksRun();
        vexDelay(2);
    }
}

#[vexide_startup::main]
async fn main(peripherals: Peripherals) {
    unsafe {
        println!("Testing serial output.");

        vexTaskAdd(
            background_processing,
            2,
            c"vexide background processing".as_ptr(),
        );

        loop {}
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
