#![no_main]
#![no_std]

use core::time::Duration;

use vexide::{
    devices::{
        controller::ControllerId,
        display::{Display, Rect},
        math::Point2,
    },
    prelude::*,
};

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    println!("Hello, world!");

    vexide::panic::set_hook(|info| {
        println!("It looks like we hit a bump.");

        // Show the panic message on the primary controller
        block_on(async {
            let mut controller_primary = unsafe { Controller::new(ControllerId::Primary) };
            let _ = controller_primary.screen.set_text("Panic!", 1, 0).await;
        });

        // Fill the screen with red to indicate a panic
        let mut display = unsafe { Display::new() };
        display.fill(
            &Rect::from_dimensions(
                Point2 { x: 0, y: 0 },
                Display::HORIZONTAL_RESOLUTION as u16,
                Display::VERTICAL_RESOLUTION as u16,
            ),
            Rgb::new(255, 0, 0),
        );

        // Call the default panic hook to print the panic message to the serial
        // console and put it on the display
        vexide::panic::default_panic_hook(info);
    });

    sleep(Duration::from_millis(1000)).await;

    panic!();
}
