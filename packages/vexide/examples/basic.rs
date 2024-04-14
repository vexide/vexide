#![no_main]
#![no_std]

extern crate alloc;
use alloc::boxed::Box;
use core::{fmt::Write, time::Duration};

use vexide::prelude::*;
use vexide_async::sleep;
use vexide_core::{allocator, println};
use vexide_devices::{
    color::Rgb,
    peripherals::Peripherals,
    screen::{Circle, Rect},
};
use vexide_panic::panic;

#[vexide_startup::main]
async fn main(peripherals: Peripherals) {
    unsafe {
        let mut p = peripherals;
        // Write something to the screen to test if the program is running
        // let test_box = Box::new(100);
        // vex_sdk::vexDisplayRectFill(0, 0, *test_box, 200);
        println!("Hello, world!");

        p.screen.fill(&Rect::new(0, 0, 20, 20), Rgb::RED);
        p.screen.stroke(&Circle::new(25, 25, 20), Rgb::BLUE);

        writeln!(p.screen, "Hello, world.").unwrap();
    }
}
