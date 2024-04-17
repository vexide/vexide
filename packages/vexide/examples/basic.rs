#![no_main]
#![no_std]

use core::time::Duration;

use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    // Send messages over serial
    println!("Hello, world!");

    // Sleep to prevent the program from exiting
    loop {
        sleep(Duration::from_millis(100)).await;
    }
}
