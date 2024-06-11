#![no_main]
#![no_std]

use core::time::Duration;

use vexide::prelude::*;
use vexide_core::sync::OnceLock;
use vexide_devices::controller::ButtonBinding;

static CONTROLLER: OnceLock<Controller> = OnceLock::new();

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let controller = peripherals.primary_controller;

    CONTROLLER.get_or_init(|| controller).await;

    spawn(async {
        CONTROLLER
            .get()
            .unwrap()
            .button_a
            .build_binding()
            .on_press(|| {
                println!("Button A pressed!");
            })
            .on_release(|| {
                println!("Button A released!");
            })
            .while_pressed(|| {
                println!("Button A held!");
            })
            .while_released(|| {
                println!("Button A not held!");
            })
            .build()
            .await;
    })
    .detach();

    loop {
        sleep(Duration::from_secs(365)).await;
    }
}
