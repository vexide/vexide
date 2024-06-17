#![no_main]
#![no_std]

use vexide::prelude::*;

#[vexide::main(banner = false)]
async fn main(_peripherals: Peripherals) {
    println!("This is the program's only output.");
}
