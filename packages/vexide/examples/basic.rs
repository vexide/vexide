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

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    // Send messages over serial
    println!("Hello, world!");

    // Sleep to prevent the program from exiting
    loop {
        sleep(Duration::from_millis(100)).await;
    }
}
