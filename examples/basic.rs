#![no_main]
#![no_std]

use core::time::Duration;

use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    println!("Henlooooo");

    loop {
        sleep(Duration::from_millis(500)).await;
    }
}
