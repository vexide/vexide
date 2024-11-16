#![no_main]
#![no_std]

use core::time::Duration;

use vexide::{
    devices::{controller::ControllerId, geometry::Point2, screen::Rect},
    prelude::*,
};

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    println!("Hello, world!");

    vexide::panic::set_hook(|info| {
        // Show the panic message on the primary controller
        let mut controller_primary = unsafe { Controller::new(ControllerId::Primary) };
        let _ = controller_primary.screen.set_text("Panic!", 0, 0);

        // Fill the screen with red
        let mut screen = unsafe { Screen::new() };
        screen.fill(
            &Rect::from_dimensions(
                Point2 { x: 0, y: 0 },
                Screen::HORIZONTAL_RESOLUTION as u16,
                Screen::VERTICAL_RESOLUTION as u16,
            ),
            0xff0000 as u32,
        );

        vexide::panic::default_panic_hook(&info);
    });

    sleep(Duration::from_millis(1000)).await;

    panic!();
}
