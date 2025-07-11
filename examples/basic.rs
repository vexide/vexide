#![no_main]
#![no_std]

use core::{arch::asm, ptr::dangling_mut};

use vex_sdk::vexSystemSWInterrupt;
use vexide::prelude::*;

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    unsafe {
        vexide::vectors::install_vector_table();
    }

    println!("Hello, world!");

    unsafe {
        let p = dangling_mut::<u32>();
        core::ptr::write_volatile(p, 123);
    }
}
