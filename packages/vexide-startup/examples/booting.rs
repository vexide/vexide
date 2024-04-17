#![no_main]
#![no_std]

extern crate alloc;
use alloc::boxed::Box;
use core::time::Duration;

use vexide_async::sleep;
use vexide_core::println;
use vexide_devices::peripherals::Peripherals;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    unsafe {
        // Write something to the screen to test if the program is running
        let test_box = Box::new(100);
        vex_sdk::vexDisplayRectFill(0, 0, *test_box, 200);
        println!("Hello, world!");

        loop {
            sleep(Duration::from_millis(100)).await;
        }
    }
}
