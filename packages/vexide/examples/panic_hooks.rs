#![no_main]
#![no_std]

use core::time::Duration;

use vexide::prelude::*;
use vexide_devices::controller::ControllerId;
use vexide_panic::set_panic_hook;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    println!("Hello, world!");

    set_panic_hook(|_| {
        let mut controller_primary = unsafe { Controller::new(ControllerId::Primary) };

        let _ = controller_primary.screen.set_text("Panic!", 0, 0);
    });

    sleep(Duration::from_millis(1000)).await;

    panic!();
}
