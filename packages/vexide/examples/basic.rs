#![no_main]
#![no_std]

use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    println!("Hello, world!");
}
