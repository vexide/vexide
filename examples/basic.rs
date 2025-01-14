#![no_main]
#![no_std]

use core::time::Duration;

use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    println!("helo :33");

    loop {
        sleep(Duration::from_millis(500)).await;
    }
}
